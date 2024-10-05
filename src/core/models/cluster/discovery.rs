//! RepliCore cluster discovery models.
use serde::Deserialize;
use serde::Serialize;

use super::ClusterDiscoveryNode;

/// Record of a cluster and all of its nodes as discovered from the Platform the cluster runs on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDiscovery {
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// List of all the nodes in the cluster.
    pub nodes: Vec<ClusterDiscoveryNode>,
}
