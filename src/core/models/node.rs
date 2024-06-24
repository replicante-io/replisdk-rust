//! Models to describe store nodes.
use serde::Deserialize;
use serde::Serialize;

use crate::agent::models::AgentVersion;
use crate::agent::models::AttributesMap;
use crate::agent::models::NodeStatus as AgentNodeStatus;
use crate::agent::models::ShardCommitOffset;
use crate::agent::models::ShardRole;
use crate::agent::models::StoreVersion;

/// Information about a Store's node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    // ID attributes.
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Unique identifier of the node, as reported by the Platform provider the node is running on.
    pub node_id: String,

    // Record attributes.
    /// Information about a node that was reachable.
    pub details: Option<NodeDetails>,

    /// The current status of the node.
    pub node_status: NodeStatus,
}

/// Information about a node that was reachable from core.
///
/// When core syncs information about a node it may not be able to connect to it.
/// In these cases we still need to track knowledge of the node with a [`Node`] object
/// but are unable to provide any [`NodeDetails`] for it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeDetails {
    /// Version information for the agent.
    pub agent_version: AgentVersion,

    /// Additional attributes based on information available even without the store process.
    #[serde(default)]
    pub attributes: AttributesMap,

    /// Identifier of the store software running on the node.
    pub store_id: String,

    /// Version information for the store software.
    pub store_version: StoreVersion,
}

/// Overall state of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Core is unable to connect to the agent.
    #[serde(rename = "UNREACHABLE")]
    Unreachable,

    /// Core is unable to sync all essential node information from the agent.
    #[serde(rename = "INCOMPLETE")]
    Incomplete,

    /// The agent is unable to connect to the node.
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,

    /// The node is running but it is not part of any cluster.
    #[serde(rename = "NOT_IN_CLUSTER")]
    NotInCluster,

    /// The node is in the process of joining a cluster.
    #[serde(rename = "JOINING_CLUSTER")]
    JoiningCluster,

    /// The node is in the process of leaving a cluster.
    #[serde(rename = "LEAVING_CLUSTER")]
    LeavingCluster,

    /// The agent has confirmed the node has experienced an issue and is unhealthy.
    #[serde(rename = "UNHEALTHY")]
    Unhealthy,

    /// The agent can connect to the node and has not noticed any failures.
    #[serde(rename = "HEALTHY")]
    Healthy,

    /// The agent was unable to determine the sate of the node (and provides a reason).
    #[serde(rename = "UNKNOWN")]
    Unknown(String),
}

impl From<AgentNodeStatus> for NodeStatus {
    fn from(value: AgentNodeStatus) -> Self {
        match value {
            AgentNodeStatus::Unavailable => Self::Unavailable,
            AgentNodeStatus::NotInCluster => Self::NotInCluster,
            AgentNodeStatus::JoiningCluster => Self::JoiningCluster,
            AgentNodeStatus::LeavingCluster => Self::LeavingCluster,
            AgentNodeStatus::Unhealthy => Self::Unhealthy,
            AgentNodeStatus::Healthy => Self::Healthy,
            AgentNodeStatus::Unknown(data) => Self::Unknown(data),
        }
    }
}

/// Information about a shard located on a node in the cluster.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Shard {
    // ID attributes.
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Unique identifier of the node, as reported by the Platform provider the node is running on.
    pub node_id: String,

    /// Identifier of the specific data shard.
    pub shard_id: String,

    // Record attributes.
    /// Current offset committed to permanent storage for the shard.
    pub commit_offset: ShardCommitOffset,

    /// True when the shard was successfully fetched by the latest node sync.   
    pub fresh: bool,

    /// Lag between this shard commit offset and its matching primary commit offset.
    pub lag: Option<ShardCommitOffset>,

    /// The role of the node with regards to shard management.
    pub role: ShardRole,
}

impl Shard {
    /// Compare self with another [`Shard`] excluding commit offset fields.
    pub fn same(&self, other: &Shard) -> bool {
        self.ns_id == other.ns_id
            && self.cluster_id == other.cluster_id
            && self.node_id == other.node_id
            && self.shard_id == other.shard_id
            && self.fresh == other.fresh
            && self.role == other.role
    }
}

/// Additional node information only available when connected to the store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoreExtras {
    // ID attributes.
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Unique identifier of the node, as reported by the Platform provider the node is running on.
    pub node_id: String,

    // Record attributes.
    /// Additional attributes based on information available only from the store process.
    #[serde(default)]
    pub attributes: AttributesMap,

    /// True when the store extras were successfully fetched by the latest node sync.
    pub fresh: bool,
}
