//! Data structures for Platform related entities.
use serde::Deserialize;
use serde::Serialize;

/// Information about a cluster and all existing nodes within.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClusterDiscovery {
    /// ID of the cluster.
    pub cluster_id: String,

    /// List of all the nodes in the cluster.
    pub nodes: Vec<ClusterDiscoveryNode>,
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
