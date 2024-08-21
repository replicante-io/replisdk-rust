//! RepliCore cluster definition objects.
use serde::Deserialize;
use serde::Serialize;

const DEFAULT_INTERVAL: i64 = 60;

use super::action::ActionApproval;
use super::node::NodeSearch;
use super::platform::PlatformRef;

pub use crate::platform::models::ClusterDefinition;
pub use crate::platform::models::ClusterDiscoveryNode;

/// Declaration of what the cluster should look like.
///
/// This object groups the definition (what exactly the cluster looks like)
/// along side the attributes needed to manage that definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterDeclaration {
    /// Convenience option to deactivate cluster convergence even when a definition is set.
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

    /// Grace period between scale up actions, in minutes.
    #[serde(default = "ClusterDeclaration::default_grace_up")]
    pub grace_up: u64,

    /// Configure cluster initiation for brand new clusters.
    #[serde(default)]
    pub initialise: ClusterSpecInit,
}

impl Default for ClusterDeclaration {
    fn default() -> Self {
        Self {
            active: Self::default_active(),
            approval: ActionApproval::default(),
            definition: None,
            grace_up: Self::default_grace_up(),
            initialise: ClusterSpecInit::default(),
        }
    }
}

impl ClusterDeclaration {
    fn default_active() -> bool {
        true
    }

    fn default_grace_up() -> u64 {
        5
    }
}

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

/// Specification describing a cluster desired state and how to manage it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterSpec {
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Activate/deactivate orchestrating the cluster.
    #[serde(default = "ClusterSpec::default_active")]
    pub active: bool,

    /// Declaration of what the cluster should look like.
    ///
    /// The declaration allows users to state the properties of the cluster and
    /// enables RepliCore to periodically check reality against this expectation.
    #[serde(default)]
    pub declaration: ClusterDeclaration,

    /// Interval, in seconds, between orchestration runs.
    #[serde(default = "ClusterSpec::default_interval")]
    pub interval: i64,

    /// Reference to the Platform expected to manage the cluster.
    ///
    /// This is required when a `declaration.definition` is set as it indicates
    /// the platform used to manage nodes in the cluster.
    #[serde(default)]
    pub platform: Option<PlatformRef>,
}

impl ClusterSpec {
    /// Return synthetic cluster specification for clusters first seen by platform discovery.
    pub fn synthetic<S1, S2>(namespace: S1, cluster_id: S2) -> ClusterSpec
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let cluster_id = cluster_id.into();
        let ns_id = namespace.into();
        ClusterSpec {
            ns_id,
            cluster_id,
            active: ClusterSpec::default_active(),
            declaration: ClusterDeclaration::default(),
            interval: ClusterSpec::default_interval(),
            platform: None,
        }
    }
}

impl ClusterSpec {
    fn default_active() -> bool {
        true
    }

    fn default_interval() -> i64 {
        DEFAULT_INTERVAL
    }
}

/// Configure cluster initiation for brand new clusters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterSpecInit {
    /// Node action arguments passed to the cluster initialise action.
    #[serde(default)]
    pub action_args: serde_json::Value,

    /// Grace period between cluster initialisation attempts, in minutes.
    #[serde(default = "ClusterSpecInit::default_grace")]
    pub grace: u64,

    /// The initialisation mode appropriate to the software the cluster runs.
    ///
    /// By default cluster initialisation is not managed.
    #[serde(default)]
    pub mode: ClusterSpecInitMode,

    /// Search definition for the node(s) to drive cluster initialisation through.
    ///
    /// By default the first node, sorted by `node_id` is used.
    #[serde(default)]
    pub search: Option<NodeSearch>,
}

impl ClusterSpecInit {
    fn default_grace() -> u64 {
        5
    }
}

impl Default for ClusterSpecInit {
    fn default() -> Self {
        Self {
            action_args: Default::default(),
            grace: ClusterSpecInit::default_grace(),
            mode: Default::default(),
            search: None,
        }
    }
}

/// Supported initialisation modes appropriate to the software the cluster runs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterSpecInitMode {
    /// Clusters are initialised automatically or otherwise outside of Replicante Core.
    #[default]
    #[serde(alias = "not-managed", alias = "not_managed")]
    NotManaged,

    /// A node is used to initialise a one-node cluster, additional nodes can later join this one.
    #[serde(alias = "single-node", alias = "single_node")]
    SingleNode,
}
