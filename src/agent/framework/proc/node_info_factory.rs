//! Factory trait to initialise the selected [`NodeInfo`] implementation
//! at the right stage of process initialisation.
use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agent::framework::AgentConf;
use crate::agent::framework::NodeInfo;
use crate::runtime::telemetry::Telemetry;

/// Factory for [`NodeInfo`] implementations.
#[async_trait::async_trait]
pub trait NodeInfoFactory {
    /// Agent configuration specific to the implementation.
    type Conf: Clone + std::fmt::Debug + Serialize + DeserializeOwned;

    /// Implementation of the [`NodeInfo`] interface returned by this factory.
    type NodeInfo: NodeInfo;

    /// Initialise the [`NodeInfo`] gathering implementation.
    async fn factory<'a>(
        &self,
        args: NodeInfoFactoryArgs<'a, Self::Conf>,
    ) -> Result<Self::NodeInfo>;
}

/// Arguments provided to the [`NodeInfo`] initialisation method [`NodeInfoFactory::factory`].
#[derive(Clone)]
pub struct NodeInfoFactoryArgs<'a, C>
where
    C: Clone + std::fmt::Debug + Serialize + DeserializeOwned,
{
    /// Full configuration for the agent process.
    pub conf: &'a AgentConf<C>,

    /// Configured telemetry resources for the agent process.
    pub telemetry: &'a Telemetry,
}
