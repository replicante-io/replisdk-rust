//! Types to support custom agent process initialisation logic.
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agent::framework::AgentConf;
use crate::runtime::telemetry::Telemetry;

/// Hook to perform custom initialisation as part of the agent start up.
#[async_trait::async_trait]
pub trait InitialiseHook {
    /// Agent configuration specific to the implementation.
    type Conf: Clone + std::fmt::Debug + Serialize + DeserializeOwned;

    /// Execute initialisation logic.
    async fn initialise<'a>(&self, args: &InitialiseHookArgs<'a, Self::Conf>) -> Result<()>;
}

/// Arguments passed to agent initialisation hooks.
pub struct InitialiseHookArgs<'a, C>
where
    C: Clone + std::fmt::Debug + Serialize + DeserializeOwned,
{
    /// Full configuration for the agent process.
    pub conf: &'a AgentConf<C>,

    /// Configured telemetry resources for the agent process.
    pub telemetry: &'a Telemetry,
}

/// List of [`InitialiseHook`]s to execute during agent initialisation.
pub type InitialiseHookVec<C> = Vec<Box<dyn InitialiseHook<Conf = C>>>;
