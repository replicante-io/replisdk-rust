//! Replicante Agent node information models.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use super::AttributesMap;

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

/// Information about a Store Node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Addresses used by other systems to connect to the node.
    pub address: NodeAddresses,

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

/// Addresses to reach the node with.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeAddresses {
    /// Address used by clients to connect to the cluster.
    pub client: Option<String>,

    /// Address used by other cluster members to connect to this node.
    pub member: Option<String>,

    /// Additional addresses useable to connect to this node.
    #[serde(flatten)]
    pub other: BTreeMap<String, String>,
}

/// Overall state of the node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
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
