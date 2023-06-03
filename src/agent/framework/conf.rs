//! Overall configuration for Agents.
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::runtime::actix_web::ServerConfig;
use crate::runtime::telemetry::TelemetryConfig;
use crate::runtime::tokio_conf::TokioRuntimeConf;

/// Container for the complete agent configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentConf<C>
where
    C: Clone + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
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

    /// Tokio Runtime configuration.
    #[serde(default)]
    pub runtime: TokioRuntimeConf,

    /// Telemetry configuration for the agent.
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl<C> Default for AgentConf<C>
where
    C: Clone + std::fmt::Debug + Default + PartialEq + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        AgentConf {
            custom: Default::default(),
            http: Default::default(),
            node_id: None,
            telemetry: Default::default(),
            runtime: Default::default(),
        }
    }
}

/// Programmatic options for the agent process.
pub struct AgentOptions {
    /// Prefix for web request metrics names.
    pub requests_metrics_prefix: &'static str,
}
