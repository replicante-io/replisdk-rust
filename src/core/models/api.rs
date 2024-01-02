//! RepliCore definitions for data in API endpoints.
use serde::Deserialize;
use serde::Serialize;

use super::namespace::NamespaceStatus;

/// Definition of entries returned when listing namespaces.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamespaceEntry {
    /// Identifier of the namespace (also known as the namespace name).
    pub id: String,

    /// Lifecycle status of the namespace.
    pub status: NamespaceStatus,
}

/// Response for the namespace list API endpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NamespaceList {
    /// List of namespaces on the control plane.
    pub items: Vec<NamespaceEntry>,
}

/// Definition of entries returned when listing platforms in a namespace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformEntry {
    /// Indicates if the Platform is active or not.
    pub active: bool,

    /// Namespaced identifier of the Platform.
    pub name: String,
}

/// Response for the platform list API endpoint.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformList {
    /// List of platform found on the control plane.
    pub items: Vec<PlatformEntry>,
}
