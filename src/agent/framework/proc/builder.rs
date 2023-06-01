//! Builder for the entire Agent process.
use actix_web::FromRequest;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agent::framework::AgentConf;
use crate::agent::framework::AgentOptions;
use crate::agent::framework::NodeInfo;
use crate::agent::framework::NodeInfoFactory;
use crate::agent::framework::NodeInfoFactoryArgs;
use crate::runtime::actix_web::AppConfigurer;
use crate::runtime::shutdown::ShutdownManager;
use crate::runtime::shutdown::ShutdownManagerBuilder;
use crate::runtime::telemetry::initialise as telemetry_init;
use crate::runtime::telemetry::TelemetryOptions;

use super::super::info;

/// Configure a process to run a Replicante Agent.
///
/// # Factories
///
/// The [`Agent`] builder aims to handle as much of the process initialisation
/// and general features handling as possible.
///
/// To tailor the generic agent process to a specific store the builder
/// requires some logic to be provided.
/// This logic is provided by way of factories so that initialisation logic specific
/// to implementations can happen at the correct stage of the overall process initialisation.
///
/// For example the [`Agent`] builder handles set up of the process loggers.
/// Agent implementations may want to (and should) generate log data as part of
/// their initialisation steps to help troubleshoot or simply understand what is going on.
/// If the agent implementation had to initialise specific logic outside of the [`Agent`]
/// builder then this logic would not have access to the process logger.
///
/// Factories allow implementations to provide the [`Agent`] builder both custom logic
/// as well as custom initialisation steps which can then be given access to additional resources.
///
/// ## Call Order
///
/// To ensure agent initialisation can be implemented correctly the builder
/// guarantees the following order of invocation for factories:
///
/// 1. The [`NodeInfoFactory`] is called.
pub struct Agent<C, IF>
where
    // Type parameter for custom a agent configuration container.
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
    // Type parameter for the agent specification info extractor factory.
    // IF == InfoFactory
    IF: NodeInfoFactory<Conf = C>,
    IF::NodeInfo: NodeInfo,
    IF::Context: FromRequest,
{
    node_info: Option<IF>,
    app: AppConfigurer,
    conf: Option<AgentConf<C>>,
    options: Option<AgentOptions>,
    shutdown: ShutdownManagerBuilder<()>,
    telemetry_options: Option<TelemetryOptions>,
}

impl<C, IF> Agent<C, IF>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
    IF: NodeInfoFactory<Conf = C>,
    IF::NodeInfo: NodeInfo,
    IF::Context: FromRequest,
{
    /// Build an agent while reusing shared logic from the Replicante SDK.
    pub fn build() -> Self {
        let shutdown = ShutdownManager::builder().watch_signal_with_default();
        Agent {
            node_info: None,
            app: AppConfigurer::default(),
            conf: None,
            options: None,
            shutdown,
            telemetry_options: None,
        }
    }

    /// Set the agent configuration to use.
    pub fn configure(mut self, conf: AgentConf<C>) -> Self {
        self.conf = Some(conf);
        self
    }

    /// Set the implementation for the node information gathering to use.
    pub fn node_info(mut self, factory: IF) -> Self {
        self.node_info = Some(factory);
        self
    }

    /// Set the agent programmatic options.
    pub fn options(mut self, options: AgentOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Finalise agent setup and run it.
    ///
    /// # Panics
    ///
    /// This method panics if required elements are not defined:
    ///
    /// - The agent MUST be configured with a call to [`Agent::configure`].
    /// - The agent MUST be given [`AgentOptions`] with a call to [`Agent::options`].
    /// - The agent MUST be given [`NodeInfo`] with a call to [`Agent::node_info`].
    /// - The agent MUST be given [`TelemetryOptions`] with a call to [`Agent::telemetry_options`].
    pub async fn run(self) -> Result<()> {
        // Validate the agent build.
        let conf = self
            .conf
            .expect("must configure(...) the agent before it can run");
        let options = self
            .options
            .expect("must set options(...) for the agent before it can run");
        let node_info = self
            .node_info
            .expect("must set node_info(...) for the agent before it can run");
        let telemetry_options = self
            .telemetry_options
            .expect("must provide telemetry_options(...) to the agent before it can run");

        // Initialise the process.
        let telemetry = telemetry_init(conf.telemetry.clone(), telemetry_options).await?;
        let shutdown = self
            .shutdown
            .logger(telemetry.logger.clone())
            .watch_signal_with_default();
        slog::info!(telemetry.logger, "Process telemetry initialised");

        // Initialise info gathering.
        slog::debug!(telemetry.logger, "Initialising node information gatherer.");
        let node_info = node_info
            .factory(NodeInfoFactoryArgs {
                conf: &conf,
                telemetry: &telemetry,
            })
            .await?;

        // Set up predefined agent endpoints.
        slog::debug!(telemetry.logger, "Configuring agent API endpoints");
        let mut app = self.app;
        let logger = telemetry.logger.clone();
        app.with_config(move |conf| {
            let info = node_info.clone();
            let info = info::into_actix_service(info, logger.clone());
            let info = actix_web::web::scope("/api/unstable").service(info);
            conf.service(info);
        });

        // Configure and start the HTTP Server.
        let server = conf
            .http
            .opinionated(app)
            .metrics(options.requests_metrics_prefix, telemetry.metrics.clone())
            .run(Some(&telemetry.logger))?;
        let shutdown = shutdown.watch_actix(server, ());

        // Complete shutdown setup and run the agent until an exit condition.
        let exit = shutdown.build();
        exit.wait().await
    }

    /// Set the [`TelemetryOptions`] for the agent process to use.
    pub fn telemetry_options(mut self, options: TelemetryOptions) -> Self {
        self.telemetry_options = Some(options);
        self
    }
}

/* *** Agent process builder ***
let (tokio, conf) = config::load()?;
let runtime = tokio::Runtime::from_conf(tokio)?;
runtime.block_on(async || {
    Agent::build()...
})

Once agent is all done can look at a proc macro to write?

#[replisdk::agent::main(conf::load)]
async fn main(conf: AgentConf<C>) -> Result<E> {
    Agent::build()...
}
*/
