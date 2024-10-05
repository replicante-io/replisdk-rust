//! RepliCore cluster specification models for cluster initialisation.
use serde::Deserialize;
use serde::Serialize;

use crate::core::models::node::NodeSearch;

/// Configure cluster initiation for brand new clusters.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDeclarationInit {
    /// Node action arguments passed to the cluster initialise action.
    #[serde(default)]
    pub action_args: serde_json::Value,

    /// Cluster initialisation mode to match how the cluster software operates.
    ///
    /// By default cluster initialisation is assumed automatic.
    #[serde(default)]
    pub mode: ClusterDeclarationInitMode,

    /// Search definition for the node to drive cluster initialisation through.
    ///
    /// By default the first `NotInCluster` node, sorted by `node_id` is used.
    #[serde(default)]
    pub search: Option<NodeSearch>,
}

/// Cluster initialisation mode to match how the cluster software operates.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterDeclarationInitMode {
    /// Clusters are initialised automatically.
    #[default]
    #[serde(alias = "auto")]
    Auto,

    /// A node is used to initialise a one-node cluster, additional nodes can later join this one.
    #[serde(alias = "single-node", alias = "single_node")]
    SingleNode,
}

impl std::fmt::Display for ClusterDeclarationInitMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::SingleNode => write!(f, "SingleNode"),
        }
    }
}
