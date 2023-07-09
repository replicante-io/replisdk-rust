//! Overall configuration for Agents.
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::runtime::actix_web::ServerConfig;
use crate::runtime::shutdown::DEFAULT_SHUTDOWN_GRACE_TIMEOUT;
use crate::runtime::telemetry::TelemetryConfig;
use crate::runtime::tokio_conf::TokioRuntimeConf;

/// Container for the complete agent configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentConf<C>
where
    C: Clone + std::fmt::Debug + Serialize + DeserializeOwned,
{
    /// Agent configuration specific to the implementation.
    #[serde(flatten, deserialize_with = "C::deserialize")]
    pub custom: C,

    /// ActixWeb HTTP Server configuration.
    #[serde(default)]
    pub http: ServerConfig,

    /// ID of the node as defined by the platform the node runs on.
    ///
    /// For example if the node is running on a cloud instance this ID would be
    /// the cloud instance ID.
    #[serde(default)]
    pub node_id: Option<String>,

    /// Process runtime configuration.
    #[serde(default)]
    pub runtime: RuntimeConf,

    /// Path to the persistence store for the agent.
    #[serde(default = "AgentConf::<C>::default_store_path")]
    pub store_path: String,

    /// Telemetry configuration for the agent.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl<C> Default for AgentConf<C>
where
    C: Clone + std::fmt::Debug + Default + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        AgentConf {
            custom: Default::default(),
            http: Default::default(),
            node_id: None,
            runtime: Default::default(),
            store_path: AgentConf::<C>::default_store_path(),
            telemetry: Default::default(),
        }
    }
}

impl<C> AgentConf<C>
where
    C: Clone + std::fmt::Debug + Serialize + DeserializeOwned,
{
    fn default_store_path() -> String {
        "agent.db".into()
    }
}

/// Programmatic options for the agent process.
pub struct AgentOptions {
    /// Prefix for web request metrics names.
    pub requests_metrics_prefix: &'static str,
}

/// Container for the complete agent runtime configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeConf {
    /// Allowed time, in seconds, for operations to complete once process shutdown begins.
    #[serde(default = "RuntimeConf::default_shutdown_grace")]
    pub shutdown_grace_sec: u64,

    /// Tokio Runtime configuration.
    #[serde(default, flatten)]
    pub tokio: TokioRuntimeConf,
}

impl RuntimeConf {
    fn default_shutdown_grace() -> u64 {
        DEFAULT_SHUTDOWN_GRACE_TIMEOUT
    }
}

impl Default for RuntimeConf {
    fn default() -> Self {
        RuntimeConf {
            shutdown_grace_sec: Self::default_shutdown_grace(),
            tokio: Default::default(),
        }
    }
}
