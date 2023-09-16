//! Metadata factories for wellknown actions relating to store clusters.
use crate::agent::framework::actions::ActionHandler;
use crate::agent::framework::actions::ActionMetadata;

/// Agent Action identifier (kind) for the agent to add a node to the cluster.
pub const KIND_CLUSTER_ADD: &str = "agent.replicante.io/cluster.add";

/// Agent Action identifier (kind) for the agent to initialise a new cluster on the node.
pub const KIND_CLUSTER_INIT: &str = "agent.replicante.io/cluster.init";

/// Define an agent action to add a node to the cluster.
pub fn add<H>(handler: H) -> ActionMetadata
where
    H: ActionHandler + 'static,
{
    ActionMetadata::build_internal(KIND_CLUSTER_ADD, handler).finish()
}

/// Define an agent action to initialise a new cluster on the node.
pub fn init<H>(handler: H) -> ActionMetadata
where
    H: ActionHandler + 'static,
{
    ActionMetadata::build_internal(KIND_CLUSTER_INIT, handler).finish()
}
