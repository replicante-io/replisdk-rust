//! RepliCore cluster specification models for cluster expansion.
use serde::Deserialize;
use serde::Serialize;

use crate::core::models::node::NodeSearch;

/// Configure cluster initiation for brand new clusters.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDeclarationExpand {
    /// Cluster expansion mode to match how the cluster software operates.
    ///
    /// By default cluster expansion assumed automatic.
    #[serde(default)]
    pub mode: ClusterDeclarationExpandMode,

    /// Search definition for the cluster node to join to/add from.
    ///
    /// By default use the first `Healthy` node, sorted by `node_id` is used.
    #[serde(default)]
    pub target_member: Option<NodeSearch>,
}

/// Cluster expansion mode to match how the cluster software operates.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterDeclarationExpandMode {
    /// An initialised node is tasked with adding the new node.
    #[serde(alias = "add")]
    Add,

    /// New nodes join the cluster automatically.
    #[default]
    #[serde(alias = "auto")]
    Auto,

    /// The new node is tasked with joining an initialised node.
    #[serde(alias = "join")]
    Join,
}

impl std::fmt::Display for ClusterDeclarationExpandMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "Add"),
            Self::Auto => write!(f, "Auto"),
            Self::Join => write!(f, "Join"),
        }
    }
}
