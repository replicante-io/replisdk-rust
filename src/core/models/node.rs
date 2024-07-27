//! Models to describe store nodes.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

use crate::agent::models::AgentVersion;
use crate::agent::models::AttributeValue;
use crate::agent::models::AttributesMap;
use crate::agent::models::NodeStatus as AgentNodeStatus;
use crate::agent::models::ShardCommitOffset;
use crate::agent::models::ShardRole;
use crate::agent::models::StoreVersion;

/// Reference to a typed value of a [`Node`] attribute.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum AttributeValueRef<'a> {
    /// Represents a boolean attribute value.
    Boolean(bool),

    /// Represents an attribute without a value.
    #[default]
    Null,

    /// Represents a numeric attribute, based on JSON number representation.
    Number(&'a Number),

    /// Represents a string attribute.
    String(&'a str),
}

impl<'a> From<&'a AttributeValue> for AttributeValueRef<'a> {
    fn from(value: &'a AttributeValue) -> Self {
        match value {
            AttributeValue::Boolean(value) => AttributeValueRef::Boolean(*value),
            AttributeValue::Null => AttributeValueRef::Null,
            AttributeValue::Number(value) => AttributeValueRef::Number(value),
            AttributeValue::String(value) => AttributeValueRef::String(value),
        }
    }
}

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
    /// - Attribute `agent_version` maps to [`AgentVersion::number`].
    /// - Attribute `agent_version.checkout` maps to [`AgentVersion::checkout`].
    /// - Attribute `agent_version.number` maps to [`AgentVersion::number`].
    /// - Attribute `agent_version.taint` maps to [`AgentVersion::taint`].
    /// - Attribute `store_id` maps to [`NodeDetails::store_id`].
    /// - Attribute `store_version` maps to [`StoreVersion::number`].
    /// - Attribute `store_version.checkout` maps to [`StoreVersion::checkout`].
    /// - Attribute `store_version.number` maps to [`StoreVersion::number`].
    /// - Attribute `store_version.extra` maps to [`StoreVersion::extra`].
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
            attribute => self.attributes.get(attribute).map(|value| value.into()),
        }
    }
}

/// Description of how to select a set of cluster nodes to target by control logic.
///
/// Many cluster management tasks target one or more nodes within the cluster itself.
/// Examples of this are:
///
/// - Act on cluster membership changes.
/// - Backup source selection.
///
/// Node searches enable configuration of how to select the nodes these operations should
/// target and in which ordering these should be targeted.
///
/// ## [`Node`]s order
///
/// A key concept is that [`Node`]s do not have a natural order:
/// different situations and uses call for a different orders, often user specified.
///
/// For these reasons the [`Node`] comparison implementation logic must make some choices.
/// While these choices are largely opinion-based it is more important for these rules
/// to be clearly set and consistently applied.
///
/// ### Attribute presence
///
/// The first choice to make is comparing attributes only set on one node:
/// value existence takes precedence over missing attributes.
///
/// - If the attribute is missing from both nodes the two are equal.
/// - If only one node has the value that node is "smaller" (ordered first).
///
/// ### Type order
///
/// When both nodes have the sorting attribute set there is no guarantee it is set to the same
/// value type. When this happens we need to define a "sorting order" across types.
///
/// The chosen order is as follows: `Number` < `String` < `bool` < `null`.
/// And within number types the order is: `i64` < `u64` < `f64`.
///
/// ### Handling `NaN`s
///
/// Finally there is one more choice to be made: how do we order `f64::NaN`?
/// By very definition `NaN` can't be ordered or compared, even to itself, but we need
/// a total order for our nodes so they can be selected/processed correctly.
///
/// But our use case does not require an absolute, fixed sort order and instead simply
/// aims for users to tell us which nodes to pick/process "first".
/// There is no need for (mathematical) correctness, just consistent results.
/// Additionally `NaN` as a [`Node`] attribute feels like a rare and unexpected event.
///
/// For these reasons the [`Node`] sorting logic assumes that `NaN == NaN` even if this is wrong!
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeSearch {
    /// Select nodes that match the given attributes.
    ///
    /// - Nodes are selected if all listed attributes are set and the node value matches.
    /// - The empty set matches all nodes (default).
    #[serde(default)]
    pub matches: NodeSearchMatches,

    /// Limit the number of selected nodes to the first N, or unlimited if `None`.
    #[serde(default)]
    pub max_results: Option<usize>,

    /// Order nodes one or more by attribute.
    ///
    /// The vector lists the sorting attributes by priority.
    /// An optional field prefix indicates ascending (default) or descending order:
    ///
    /// - `+` for ascending.
    /// - `-` for descending.
    ///
    /// ## Default
    ///
    /// By default nodes are sorted ascending by Node ID.
    #[serde(default = "NodeSearch::default_sort_by")]
    pub sort_by: Vec<String>,
}

impl NodeSearch {
    fn default_sort_by() -> Vec<String> {
        vec![String::from("node_id")]
    }
}

impl Default for NodeSearch {
    fn default() -> Self {
        NodeSearch {
            matches: Default::default(),
            max_results: None,
            sort_by: NodeSearch::default_sort_by(),
        }
    }
}

/// Type alias for collection of attribute values to match for node searches.
pub type NodeSearchMatches = HashMap<String, AttributeValue>;

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

#[cfg(test)]
mod tests {
    use super::AttributeValueRef;
    use super::Node;
    use super::NodeDetails;
    use super::NodeStatus;

    #[rstest::rstest]
    #[case("ns_id", Some("test-ns"))]
    #[case("cluster_id", Some("test-cluster"))]
    #[case("node_id", Some("test-node"))]
    #[case("node_status", Some("INCOMPLETE"))]
    #[case("agent_version", None)]
    fn node_attribute_lookup(#[case] attribute: &str, #[case] expected: Option<&str>) {
        let node = Node {
            ns_id: "test-ns".into(),
            cluster_id: "test-cluster".into(),
            node_id: "test-node".into(),
            details: None,
            node_status: NodeStatus::Incomplete,
        };
        let actual = node.attribute(attribute);
        let actual = actual.map(|actual| match actual {
            AttributeValueRef::String(actual) => actual,
            _ => panic!("test requires a string attribute"),
        });
        assert_eq!(actual, expected);
    }

    #[rstest::rstest]
    #[case("agent_version", Some("1.2.3"))]
    #[case("agent_version.checkout", Some("agent-sha"))]
    #[case("agent_version.number", Some("1.2.3"))]
    #[case("agent_version.taint", Some("test"))]
    #[case("store_id", Some("test-store"))]
    #[case("store_version", Some("4.5.6"))]
    #[case("store_version.checkout", None)]
    #[case("store_version.number", Some("4.5.6"))]
    #[case("store_version.extra", Some("mocked"))]
    #[case("test.attribute", Some("value"))]
    #[case("missing-attribute", None)]
    fn node_attribute_lookup_details(#[case] attribute: &str, #[case] expected: Option<&str>) {
        let details = NodeDetails {
            agent_version: super::AgentVersion {
                checkout: "agent-sha".into(),
                number: "1.2.3".into(),
                taint: "test".into(),
            },
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert("test.attribute".into(), "value".into());
                map
            },
            store_id: "test-store".into(),
            store_version: super::StoreVersion {
                checkout: None,
                number: "4.5.6".into(),
                extra: Some("mocked".into()),
            },
        };
        let node = Node {
            ns_id: "test-ns".into(),
            cluster_id: "test-cluster".into(),
            node_id: "test-node".into(),
            details: Some(details),
            node_status: NodeStatus::Unhealthy,
        };
        let actual = node.attribute(attribute);
        let actual = actual.map(|actual| match actual {
            AttributeValueRef::String(actual) => actual,
            _ => panic!("test requires a string attribute"),
        });
        assert_eq!(actual, expected);
    }
}
