//! Models to describe cluster nodes.
use serde::Deserialize;
use serde::Serialize;

use crate::agent::models::AgentVersion;
use crate::agent::models::AttributesMap;
use crate::agent::models::NodeAddresses;
use crate::agent::models::NodeStatus as AgentNodeStatus;
use crate::agent::models::ShardCommitOffset;
use crate::agent::models::ShardRole;
use crate::agent::models::StoreVersion;

mod attribute;
mod search;

#[cfg(test)]
mod tests;

pub use self::attribute::AttributeMatcher;
pub use self::attribute::AttributeMatcherComplex;
pub use self::attribute::AttributeMatcherOp;
pub use self::attribute::AttributeValueRef;
pub use self::search::NodeSearch;
pub use self::search::NodeSearchMatches;

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

impl Node {
    /// Lookup a node attribute by name.
    ///
    /// The lookup process supports:
    ///
    /// - `Node` fields (such as `ns_id` and `node_status`).
    /// - `NodeDetails` fields (such as `agent_version` and `store_id`).
    /// - Dynamic node attributes returned by the agent.
    ///
    /// This approach provides a simple and consistent interface to node properties for
    /// node search matching as well as templating logic.
    ///
    /// ## `None` or `Null`
    ///
    /// This method returned `None` if the attribute is not attached to the node at all.
    ///
    /// An attribute can be set on the node but have no current value, in this cases
    /// [`AttributeValueRef::Null`] is returned.
    ///
    /// The distinction allows callers and nodes to better define what causes missing data
    /// and how to handle each case (of a node reporting missing data or a node not able to report
    /// on a specific kind of data).
    ///
    /// ## Special Attributes
    ///
    /// - Attribute `ns_id` maps to [`Node::ns_id`].
    /// - Attribute `cluster_id` maps to [`Node::cluster_id`].
    /// - Attribute `node_id` maps to [`Node::node_id`].
    /// - Attribute `node_status` maps to [`Node::node_status`] (all uppercase).
    /// - Attribute `address.client` maps to [`NodeAddresses::client`].
    /// - Attribute `address.member` maps to [`NodeAddresses::member`].
    /// - Attribute `agent_version` maps to [`AgentVersion::number`].
    /// - Attribute `agent_version.checkout` maps to [`AgentVersion::checkout`].
    /// - Attribute `agent_version.number` maps to [`AgentVersion::number`].
    /// - Attribute `agent_version.taint` maps to [`AgentVersion::taint`].
    /// - Attribute `store_id` maps to [`NodeDetails::store_id`].
    /// - Attribute `store_version` maps to [`StoreVersion::number`].
    /// - Attribute `store_version.checkout` maps to [`StoreVersion::checkout`].
    /// - Attribute `store_version.number` maps to [`StoreVersion::number`].
    /// - Attribute `store_version.extra` maps to [`StoreVersion::extra`].
    /// - Attribute `address.${name}` lookups `name` from [`NodeAddresses::other`].
    pub fn attribute<S>(&self, attribute: S) -> Option<AttributeValueRef>
    where
        S: AsRef<str>,
    {
        let attribute = attribute.as_ref();
        match attribute {
            "ns_id" => Some(AttributeValueRef::String(&self.ns_id)),
            "cluster_id" => Some(AttributeValueRef::String(&self.cluster_id)),
            "node_id" => Some(AttributeValueRef::String(&self.node_id)),
            "node_status" => Some(AttributeValueRef::String(self.node_status.as_ref())),
            _ => self
                .details
                .as_ref()
                .and_then(|details| details.attribute(attribute)),
        }
    }
}

/// Information about a node that was reachable from core.
///
/// When core syncs information about a node it may not be able to connect to it.
/// In these cases we still need to track knowledge of the node with a [`Node`] object
/// but are unable to provide any [`NodeDetails`] for it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeDetails {
    /// Addresses used by other systems to connect to the node.
    pub address: NodeAddresses,

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

impl NodeDetails {
    /// Continue [`Node::attribute`] logic for `NodeDetails` fields and attributes.
    fn attribute(&self, attribute: &str) -> Option<AttributeValueRef> {
        match attribute {
            "address.client" => self
                .address
                .client
                .as_ref()
                .map(|address| AttributeValueRef::String(address)),
            "address.member" => self
                .address
                .member
                .as_ref()
                .map(|address| AttributeValueRef::String(address)),
            "agent_version" => Some(AttributeValueRef::String(&self.agent_version.number)),
            "agent_version.checkout" => {
                Some(AttributeValueRef::String(&self.agent_version.checkout))
            }
            "agent_version.number" => Some(AttributeValueRef::String(&self.agent_version.number)),
            "agent_version.taint" => Some(AttributeValueRef::String(&self.agent_version.taint)),
            "store_id" => Some(AttributeValueRef::String(&self.store_id)),
            "store_version" => Some(AttributeValueRef::String(&self.store_version.number)),
            "store_version.checkout" => self
                .store_version
                .checkout
                .as_ref()
                .map(|checkout| AttributeValueRef::String(checkout)),
            "store_version.number" => Some(AttributeValueRef::String(&self.store_version.number)),
            "store_version.extra" => self
                .store_version
                .extra
                .as_ref()
                .map(|extra| AttributeValueRef::String(extra)),
            attribute if attribute.starts_with("address.") => {
                let start = "address.".len();
                let name = &attribute[start..];
                self.address
                    .other
                    .get(name)
                    .map(|address| AttributeValueRef::String(address))
            }
            attribute => self.attributes.get(attribute).map(|value| value.into()),
        }
    }
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

impl AsRef<str> for NodeStatus {
    fn as_ref(&self) -> &str {
        match self {
            NodeStatus::Unreachable => "UNREACHABLE",
            NodeStatus::Incomplete => "INCOMPLETE",
            NodeStatus::Unavailable => "UNAVAILABLE",
            NodeStatus::NotInCluster => "NOT_IN_CLUSTER",
            NodeStatus::JoiningCluster => "JOINING_CLUSTER",
            NodeStatus::LeavingCluster => "LEAVING_CLUSTER",
            NodeStatus::Unhealthy => "UNHEALTHY",
            NodeStatus::Healthy => "HEALTHY",
            NodeStatus::Unknown(status) => status,
        }
    }
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
