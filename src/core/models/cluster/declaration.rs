//! RepliCore cluster declaration models.
use serde::Deserialize;
use serde::Serialize;

use super::ClusterDeclarationExpand;
use super::ClusterDeclarationInit;
use super::ClusterDefinition;
use crate::core::models::action::ActionApproval;

/// Grace periods for cluster convergence operations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterConvergenceGraces {
    /// Grace period between cluster expand attempts, in minutes.
    #[serde(default = "ClusterConvergenceGraces::default_expand")]
    pub expand: u64,

    /// Grace period between cluster initialisation attempts, in minutes.
    #[serde(default = "ClusterConvergenceGraces::default_init")]
    pub init: u64,

    /// Grace period between scale up actions, in minutes.
    #[serde(default = "ClusterConvergenceGraces::default_scale_up")]
    pub scale_up: u64,
}

impl ClusterConvergenceGraces {
    fn default_expand() -> u64 {
        5
    }

    fn default_init() -> u64 {
        5
    }

    fn default_scale_up() -> u64 {
        5
    }
}

impl Default for ClusterConvergenceGraces {
    fn default() -> Self {
        ClusterConvergenceGraces {
            expand: Self::default_expand(),
            init: Self::default_init(),
            scale_up: Self::default_scale_up(),
        }
    }
}

/// Declaration of what the cluster should look like.
///
/// This object groups the definition (what exactly the cluster looks like)
/// along side the attributes needed to manage that definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDeclaration {
    /// Deactivate cluster convergence operations without changing individual options.
    #[serde(default = "ClusterDeclaration::default_active")]
    pub active: bool,

    /// Approval state of actions created during cluster convergence.
    #[serde(default)]
    pub approval: ActionApproval,

    /// Definition of what we want the cluster to be.
    ///
    /// Cluster convergence is disabled if a definition is not set.
    #[serde(default)]
    pub definition: Option<ClusterDefinition>,

    /// Configure cluster expansion when nodes are added to the cluster.
    #[serde(default)]
    pub expand: ClusterDeclarationExpand,

    /// Grace periods for cluster convergence operations.
    #[serde(default)]
    pub graces: ClusterConvergenceGraces,

    /// Configure cluster initiation for brand new clusters.
    #[serde(default)]
    pub initialise: ClusterDeclarationInit,
}

impl Default for ClusterDeclaration {
    fn default() -> Self {
        Self {
            active: Self::default_active(),
            approval: Default::default(),
            definition: None,
            expand: Default::default(),
            graces: Default::default(),
            initialise: Default::default(),
        }
    }
}

impl ClusterDeclaration {
    fn default_active() -> bool {
        true
    }
}
