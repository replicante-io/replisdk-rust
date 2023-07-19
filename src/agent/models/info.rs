//! Replicante Agent node information models.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Number;

/// Information about an Agent version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentVersion {
    /// The git commit hash of the agent code that is running.
    pub checkout: String,

    /// [Semantic Version](https://semver.org/) string of the agent.
    pub number: String,

    /// Additional indicator of changes not reflected in the checkout string.
    ///
    /// The aim of this field is to determine whether the checkout string
    /// can be used to run an exact copy of the agent process.
    pub taint: String,
}

/// Typed value of a Node attribute.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AttributeValue {
    /// Represents a boolean attribute value.
    Boolean(bool),

    /// Represents an attribute without a value.
    #[default]
    Null,

    /// Represents a numeric attribute, based on JSON number representation.
    Number(Number),

    /// Represents a string attribute.
    String(String),
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        AttributeValue::Boolean(value)
    }
}

impl From<Number> for AttributeValue {
    fn from(value: Number) -> Self {
        AttributeValue::Number(value)
    }
}

impl<'a> From<&'a str> for AttributeValue {
    fn from(value: &'a str) -> Self {
        AttributeValue::String(value.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        AttributeValue::String(value)
    }
}

/// Map of Node attribute identifies to values.
pub type AttributesMap = BTreeMap<String, AttributeValue>;

/// Information about a Store Node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Version information for the agent.
    pub agent_version: AgentVersion,

    /// Additional attributes based on information available even without the store process.
    #[serde(default)]
    pub attributes: AttributesMap,

    /// Unique identifier of the node, as reported by the Platform provider the node is running on.
    pub node_id: String,

    /// The current status of the node.
    pub node_status: NodeStatus,

    /// Identifier of the store software running on the node.
    pub store_id: String,

    /// Version information for the store software.
    pub store_version: StoreVersion,
}

/// Overall state of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// The agent is unable to connect to the node.
    Unavailable,

    /// The node is running but it is not part of any cluster.
    NotInCluster,

    /// The node is in the process of joining a cluster.
    JoiningCluster,

    /// The node is in the process of leaving a cluster.
    LeavingCluster,

    /// The agent has confirmed the node has experienced an issue and is unhealthy.
    Unhealthy,

    /// The agent can connect to the node and has not noticed any failures.
    Healthy,

    /// The agent was unable to determine the sate of the node (and provides a reason).
    Unknown(String),
}

/// Information about a shard managed by a node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Shard {
    /// Current offset committed to permanent storage for the shard.
    pub commit_offset: ShardCommitOffset,

    /// Lag between this shard commit offset and its matching primary commit offset.
    pub lag: Option<ShardCommitOffset>,

    /// The role of the node with regards to shard management.
    pub role: ShardRole,

    /// Identifier of the specific data shard.
    #[serde(rename = "id")]
    pub shard_id: String,
}

/// Current offset committed to permanent storage for the shard.
///
/// This type is also used to report commit lag between to shards.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShardCommitOffset {
    /// Unit the commit offset value is presented as.
    pub unit: ShardCommitOffsetUnit,

    /// The commit offset value itself.
    pub value: i64,
}

impl ShardCommitOffset {
    /// Create a [`ShardCommitOffset`] from the given value in milliseconds.
    pub fn milliseconds(value: i64) -> ShardCommitOffset {
        ShardCommitOffset {
            unit: ShardCommitOffsetUnit::Milliseconds,
            value,
        }
    }

    /// Create a [`ShardCommitOffset`] from the given value in seconds.
    pub fn seconds(value: i64) -> ShardCommitOffset {
        ShardCommitOffset {
            unit: ShardCommitOffsetUnit::Seconds,
            value,
        }
    }

    /// Create a [`ShardCommitOffset`] from the given value and custom unit.
    pub fn unit<S>(value: i64, unit: S) -> ShardCommitOffset
    where
        S: Into<String>,
    {
        ShardCommitOffset {
            unit: ShardCommitOffsetUnit::Unit(unit.into()),
            value,
        }
    }
}

/// Unit the commit offset value is presented as.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShardCommitOffsetUnit {
    /// The commit offset is presented as seconds since a fixed starting time.
    ///
    /// The starting time may be cluster specific (such as the cluster initialisation event)
    /// or unrelated to the cluster (such as the UNIX epoch).
    #[serde(rename = "milliseconds")]
    Milliseconds,

    /// The commit offset is presented as seconds since a fixed starting time.
    ///
    /// The starting time may be cluster specific (such as the cluster initialisation event)
    /// or unrelated to the cluster (such as the UNIX epoch).
    #[serde(rename = "seconds")]
    Seconds,

    /// The commit offset is presented in an custom unit.
    #[serde(rename = "unit")]
    Unit(String),
}

/// The role a given node plays in managing a given shard located on it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ShardRole {
    /// The node is responsible for both reads and writes on the shard.
    Primary,

    /// The node is responsible for replicating data for the shard and may perform reads.
    Secondary,

    /// The node is currently re-syncing the shards data from another node.
    Recovering,

    /// The node is responsible for the shard in some undefined way.
    ///
    /// This role is primarily intended as a way to report shard state information
    /// without specifying expectations of what the node can do with the data.
    ///
    /// For example, Replicante Core assumes no operations can be safely performed
    /// on shards in this state and may request operator intervention to "recover".
    Other(String),
}

/// Information about [`Shard`]s managed by a node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShardsInfo {
    /// All shards managed by the node.
    pub shards: Vec<Shard>,
}

/// Additional node information only available when connected to the store.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoreExtras {
    /// Store determined cluster identifier.
    pub cluster_id: String,

    /// Additional attributes based on information available only from the store process.
    #[serde(default)]
    pub attributes: AttributesMap,
}

/// Information about a Node's Store version.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoreVersion {
    /// The VCS commit identifier of the store code that is running.
    #[serde(default)]
    pub checkout: Option<String>,

    /// [Semantic Version](https://semver.org/) string of the store.
    pub number: String,

    /// Store specific additional version information.
    #[serde(default)]
    pub extra: Option<String>,
}
