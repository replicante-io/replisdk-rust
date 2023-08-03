//! Builder for the entire Agent process.
use std::time::Duration;

use actix_web::FromRequest;
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agent::framework::actions::ActionMetadata;
use crate::agent::framework::actions::ActionsExecutor;
use crate::agent::framework::actions::ActionsRegistry;
use crate::agent::framework::actions::ActionsRegistryBuilder;
use crate::agent::framework::actions::ActionsService;
use crate::agent::framework::info;
use crate::agent::framework::store::Store;
use crate::agent::framework::AgentConf;
use crate::agent::framework::AgentOptions;
use crate::agent::framework::Injector;
use crate::agent::framework::NodeInfo;
use crate::agent::framework::NodeInfoFactory;
use crate::agent::framework::NodeInfoFactoryArgs;
use crate::runtime::actix_web::AppConfigurer;
use crate::runtime::shutdown::ShutdownManager;
use crate::runtime::shutdown::ShutdownManagerBuilder;
use crate::runtime::telemetry::initialise as telemetry_init;
use crate::runtime::telemetry::TelemetryOptions;

use super::init::InitialiseHookVec;
use super::InitialiseHook;
use super::InitialiseHookArgs;

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
/// 1. Initialisation functions registered with [`Agent::initialise_with`]
///    in the order they are registered.
/// 2. The [`NodeInfoFactory`] is called.
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
    actions: ActionsRegistryBuilder,
    app: AppConfigurer,
    conf: Option<AgentConf<C>>,
    initialisers: InitialiseHookVec<C>,
    node_info: Option<IF>,
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
            actions: ActionsRegistry::build(),
            app: AppConfigurer::default(),
            conf: None,
            initialisers: Default::default(),
            node_info: None,
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

    /// Register metadata for handling of [`ActionExecution`] records.
    ///
    /// [`ActionExecution`]: crate::agent::models::ActionExecution
    pub fn register_action(mut self, action: ActionMetadata) -> Self {
        self.actions = self.actions.register(action);
        self
    }

    /// Register metadata for handling of [`ActionExecution`] records.
    ///
    /// [`ActionExecution`]: crate::agent::models::ActionExecution
    pub fn register_actions<I>(mut self, actions: I) -> Self
    where
        I: IntoIterator<Item = ActionMetadata>,
    {
        for action in actions {
            self.actions = self.actions.register(action);
        }
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
            .graceful_shutdown_timeout(Duration::from_secs(conf.runtime.shutdown_grace_sec));
        slog::info!(telemetry.logger, "Process telemetry initialised");

        // Initialise agent globals.
        let store = Store::initialise(&telemetry.logger, &conf.store_path).await?;
        let injector = Injector {
            actions: self.actions.finish(),
            logger: telemetry.logger.clone(),
            store,
        };
        Injector::initialise(injector.clone());

        // Run custom agent initialisation hooks.
        slog::debug!(telemetry.logger, "Running agent initialisation hooks");
        let initialiser_args = InitialiseHookArgs {
            conf: &conf,
            telemetry: &telemetry,
        };
        for initialiser in self.initialisers {
            initialiser.initialise(&initialiser_args).await?;
        }

        // Initialise info gathering.
        slog::debug!(telemetry.logger, "Initialising node information gatherer");
        let node_info = node_info
            .factory(NodeInfoFactoryArgs {
                conf: &conf,
                telemetry: &telemetry,
            })
            .await?;

        // Set up predefined agent endpoints.
        slog::debug!(telemetry.logger, "Configuring agent API endpoints");
        let mut app = self.app;
        let api_logger = telemetry.logger.new(slog::o!("component" => "api"));
        let app_injector = injector.clone();
        app.with_config(move |conf| {
            let actions = ActionsService::with_injector(&app_injector);
            let info = node_info.clone();
            let info = info::into_actix_service(info, api_logger.clone());
            let info = actix_web::web::scope("/api/unstable")
                .service(info)
                .service(actions);
            conf.service(info);
        });

        // Configure and start the HTTP Server.
        let server = conf
            .http
            .opinionated(app)
            .metrics(options.requests_metrics_prefix, telemetry.metrics.clone())
            .run(Some(&telemetry.logger))?;
        let shutdown = shutdown.watch_actix(server, ());

        // Spawn actions execution background tasks.
        let executor = ActionsExecutor::with_injector(&injector);
        let executor = executor.task(shutdown.shutdown_notification());
        let shutdown = shutdown.watch_tokio(tokio::spawn(executor));

        // Complete shutdown setup and run the agent until an exit condition.
        let exit = shutdown.build();
        exit.wait().await
    }

    /// Set the [`TelemetryOptions`] for the agent process to use.
    pub fn telemetry_options(mut self, options: TelemetryOptions) -> Self {
        self.telemetry_options = Some(options);
        self
    }

    /// Add a [`InitialiseHook`] to the list of process initialisers.
    pub fn initialise_with<I>(mut self, initialiser: I) -> Self
    where
        I: InitialiseHook<Conf = C> + 'static,
    {
        self.initialisers.push(Box::new(initialiser));
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
