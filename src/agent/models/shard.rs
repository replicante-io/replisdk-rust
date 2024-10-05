//! Replicante Agent shard information models.
use serde::Deserialize;
use serde::Serialize;

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
