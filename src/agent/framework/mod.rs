//! Abstraction layer for the Replicante ecosystem to integrate with Stores.
//!
//! Replicante Agents are processes running along side the Stores to be managed.
//! These processes act as a standard interface between the Replicante ecosystem and
//! the many possible software that match the Store definition.
//!
//! The SDK aims to:
//!
//! - Standardise user experience: we want to enable anyone to write agents but we also want
//!   users that rely on agents from different authors to find a familiar landscape.
//! - Optimise agent development: agents share requirements, features and operational needs.
//!   The SDK aim to provide as much of that as possible so the focus can be on the Store needs.
//!
//! # Implementing Agents
//!
//! The SDK provides an [`Agent`] builder object that sets up a process to run agents.
//! For the agent to be built some required elements must be set and match expectations:
//!
//! - Process setup and configuration (such as process telemetry).
//! - Interactions with the store software with trait implementations.
//! - Registering supported builtin and custom actions.
//!
//! ## Agent Configuration
//!
//! The SDK hopes to minimise the required configuration as much as possible and provide
//! sensible defaults.
//!
//! For agents a configuration container is provided with [`AgentConf`].
//! This deserializable struct collects all framework related configuration values and
//! has a "slot" for agent specific options.
//!
//! The idea behind this is that configuration files can be deserialized into `AgentConf<C>`
//! structures that collect all the information needed by both framework and agent implementation.
//! The loaded configuration is the provided to the [`Agent::configure`] method.
//!
//! Aside from the user configuration options described above the framework expects some
//! agent specific options that implementations must provide.
//! The [`Agent::run`] method lists all the options that, if missing, cause the process to fail.
//!
//! ## Node Information
//!
//! TODO: the node info trait
//!
//! ## Implementing actions
//!
//! TODO: provide well defined action impls
//!
//! TODO: provide custom action impls
//!
//! ## Examples
//!
//! TODO: link to repositories implementing agents.
//!
//! Overall an agent setup may look like the snipped below.
//!
//! ```ignore
//! Agent::build()
//!     .configure(conf)
//!     .options(...)
//!     .telemetry_options(...)
//!     .agent_info(info::AgentInfo::new(...))
//!     .watch_task(background::custom_worker_task(...))
//!     .watch_task(background::store_monitor_task(...))
//!     .register_action(actions::custom(...))
//!     .register_action(actions::cluster::init(...))
//!     .register_action(actions::cluster::join(...))
//!
//!     // Once the agent is configured we can run it forever.
//!     .run()
//!     .await
//! ```
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::runtime::actix_web::AppConfigurer;
use crate::runtime::shutdown::ShutdownManager;
use crate::runtime::shutdown::ShutdownManagerBuilder;
use crate::runtime::telemetry::initialise as telemetry_init;
use crate::runtime::telemetry::TelemetryOptions;

mod conf;

pub use self::conf::AgentConf;
pub use self::conf::AgentOptions;

/// Configure a process to run a Replicante Agent.
pub struct Agent<C>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    conf: Option<AgentConf<C>>,
    options: Option<AgentOptions>,
    shutdown: ShutdownManagerBuilder<()>,
    telemetry_options: Option<TelemetryOptions>,
}

impl<C> Agent<C>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    /// Build an agent while reusing shared logic from the Replicante SDK.
    pub fn build() -> Self {
        let shutdown = ShutdownManager::builder().watch_signal_with_default();
        Agent {
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
    /// - The agent MUST be given [`TelemetryOptions`] with a call to [`Agent::telemetry_options`].
    pub async fn run(self) -> Result<()> {
        // Validate the agent build.
        let conf = self
            .conf
            .expect("must configure(...) the agent before it can run");
        let options = self
            .options
            .expect("must set options(...) for the agent before it can run");
        let telemetry_options = self
            .telemetry_options
            .expect("must provide telemetry_options(...) to the agent before it can run");

        // Initialise the process.
        let telemetry = telemetry_init(conf.telemetry, telemetry_options).await?;
        let shutdown = self
            .shutdown
            .logger(telemetry.logger.clone())
            .watch_signal_with_default();

        // Configure and start the HTTP Server.
        let app = AppConfigurer::default();
        let server = conf
            .http
            .opinionated(app)
            .metrics(options.requests_metrics_prefix, telemetry.metrics.clone())
            .build()?;
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
