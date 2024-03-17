//! RepliCore cluster definition objects.
use serde::Deserialize;
use serde::Serialize;

const DEFAULT_INTERVAL: i64 = 60;

use crate::core::models::platform::PlatformRef;

pub use crate::platform::models::ClusterDefinition;
pub use crate::platform::models::ClusterDiscoveryNode;

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
    pub declaration: Option<ClusterDefinition>,

    /// Interval, in seconds, between orchestration runs.
    #[serde(default = "ClusterSpec::default_interval")]
    pub interval: i64,

    /// Reference to the Platform expected to manage the cluster.
    ///
    /// This is required when a `declaration` is set as it references the platform
    /// used to manage nodes in the cluster.
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
            declaration: None,
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
