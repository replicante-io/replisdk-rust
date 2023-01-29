//! Data structures for Platform related entities.
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

/// Declarative definition of a cluster and its nodes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDefinition {
    /// Additional attributes attached to all nodes in the cluster.
    ///
    /// These attributes can be used by the Platform to customise nodes in the cluster.
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,

    /// ID of the cluster to add the node to.
    pub cluster_id: String,

    /// The store software to provision on the node.
    pub store: String,

    /// The version of the store software to provision on the node.
    pub store_version: String,

    /// Map of node group configurations.
    ///
    /// A cluster can be composed of differently configured nodes.
    pub nodes: HashMap<String, ClusterDefinitionNodeGroup>,
}

/// Declarative definition of a cluster's node group.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDefinitionNodeGroup {
    /// Additional attributes for nodes in this group.
    ///
    /// Extends `cluster.attributes` or override values with the same key.
    #[serde(default)]
    pub attributes: HashMap<String, serde_json::Value>,

    /// Number of desired nodes for this group.
    pub desired_count: u32,

    /// Platform specific class of node to provision (such as instance type).
    ///
    /// If a platform does not support node types this can be anything.
    pub node_class: String,

    /// The version of the store software for nodes in this group
    /// 
    /// Overrides the `cluster.store_version` value for individual node groups.
    #[serde(default)]
    pub store_version: Option<String>
}

/// Information about a cluster and all existing nodes within.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClusterDiscovery {
    /// ID of the cluster.
    pub cluster_id: String,

    /// List of all the nodes in the cluster.
    pub nodes: Vec<ClusterDiscoveryNode>,
}

/// API Response schema for a Platform node provision action.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClusterDiscoveryResponse {
    /// List of clusters on the platform.
    pub clusters: Vec<ClusterDiscovery>,
}

/// Information about an individual cluster node.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClusterDiscoveryNode {
    /// Address to connect to the node's agent service.
    ///
    /// Usually this is an HTTP(S) URL but agents can support additional transport protocols.
    /// The exact format of the address therefore depends on the supported protocol.
    ///
    /// Additional connection parameters may be required to connect to the agent,
    /// such as TLS certificates for agents using the HTTPS transport protocol.
    /// It is the responsibility of the client to correctly identify the transport protocol
    /// and provide all required connection parameters.
    ///
    /// It is possible for the agent address to change over the lifetime of a node.
    pub agent_address: String,

    /// Platform defined ID on the node.
    ///
    /// A node ID MUST:
    ///
    /// * Be unique across the cluster.
    /// * Never change for the same underling node.
    ///
    /// For example, a good node ID is the instance ID reported by a cloud provider.
    pub node_id: String,
}

/// API Request schema for a Platform node deprovision action.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeDeprovisionRequest {
    /// ID of the cluster the node to deprovision is part of.
    pub cluster_id: String,

    /// Platform defined ID on the node to deprovision.
    pub node_id: String,
}

/// API Request schema for a Platform node provision action.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NodeProvisionRequest {
    /// Declarative definition of the whole cluster to provision node(s) in.
    pub cluster: ClusterDefinition,

    /// Details of the node(s) provisioning request.
    pub provision: NodeProvisionRequestDetails,
}

/// Details of the node(s) to provisions in a NodeProvision action.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct NodeProvisionRequestDetails {
    /// ID of the node group to provision.
    pub node_group_id: String,
}

/// API Response schema for a Platform node provision action.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct NodeProvisionResponse {
    /// Number of nodes being provisioned due to this request.
    pub count: u32,

    /// If available, the Platform can return a list of node IDs being provisioned.
    ///
    /// The Platform MUST return an ID for each provisioning node or no IDs at all.
    ///
    /// The IDs returned are not required to be the IDs of the nodes ultimately provisioned
    /// but discrepancies should be limited.
    /// If the Platform can't provide fairly reliable node IDs it should NOT return any.
    ///
    /// For example the Platform may begin node provisioning and return IDs.
    /// On provisioning failure the Platform MAY terminated the failed node and
    /// retry provisioning under a new ID.
    #[serde(default)]
    pub node_ids: Option<Vec<String>>,
}
