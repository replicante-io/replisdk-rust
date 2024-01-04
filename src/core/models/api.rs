//! RepliCore definitions for data in API endpoints.
use serde::Deserialize;
use serde::Serialize;

use super::namespace::NamespaceStatus;

/// Response for the platform list API endpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EntriesList<T> {
    /// List of entries found on the control plane.
    pub items: Vec<T>,
}

/// Response for the cluster spec list API endpoint.
pub type ClusterSpecList = EntriesList<ClusterSpecEntry>;

/// Response for the namespace list API endpoint.
pub type NamespaceList = EntriesList<NamespaceEntry>;

/// Response for the platform list API endpoint.
pub type PlatformList = EntriesList<PlatformEntry>;

/// Definition of entries returned when listing cluster specs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterSpecEntry {
    /// Namespaced identifier of the cluster (specification).
    pub cluster_id: String,

    /// Activate/deactivate orchestrating the cluster.
    pub active: bool,
}

/// Definition of entries returned when listing namespaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamespaceEntry {
    /// Identifier of the namespace (also known as the namespace name).
    pub id: String,

    /// Lifecycle status of the namespace.
    pub status: NamespaceStatus,
}

/// Definition of entries returned when listing platforms in a namespace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformEntry {
    /// Indicates if the Platform is active or not.
    pub active: bool,

    /// Namespaced identifier of the Platform.
    pub name: String,
}
