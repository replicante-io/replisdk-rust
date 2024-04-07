//! RepliCore definitions for data in API endpoints.
use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

use super::action::ActionApproval;
use super::namespace::NamespaceStatus;
use super::oaction::OActionState;

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

/// Response for the oaction list API endpoint.
pub type OActionList = EntriesList<OActionEntry>;

/// Response for the platform list API endpoint.
pub type PlatformList = EntriesList<PlatformEntry>;

/// Definition of entries returned when listing cluster specs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterSpecEntry {
    /// Namespace the cluster is in.
    pub ns_id: String,

    /// Namespaced identifier of the cluster.
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

/// Definition of entries returned when listing orchestrator actions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OActionEntry {
    /// Namespace the cluster is in.
    pub ns_id: String,

    /// Namespaced identifier of the cluster.
    pub cluster_id: String,

    /// Identifier of the action.
    pub action_id: Uuid,

    /// Timestamp of action creation.
    #[serde(with = "time::serde::rfc3339")]
    pub created_ts: OffsetDateTime,

    /// Timestamp action entered a final state (success or failure).
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub finished_ts: Option<OffsetDateTime>,

    /// Identifier of the orchestrator action logic to execute.
    pub kind: String,

    /// State the action is currently in.
    pub state: OActionState,
}

/// Specification of [`OAction`] for the apply interface.
///
/// The apply interface creates new actions and not all fields in [`OAction`] records
/// are needed or should be provided by users. For example:
///
/// - A new UUID can be auto-created.
/// - The action state has to be new and can't be set by users.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OActionSpec {
    /// Namespace the cluster is in.
    pub ns_id: String,

    /// Namespaced identifier of the cluster.
    pub cluster_id: String,

    /// Identifier of the action.
    pub action_id: Option<Uuid>,

    /// Action-dependent arguments to execute with.
    #[serde(default)]
    pub args: Json,

    /// Automatically grant or explicitly require approval before actions are executed.
    #[serde(default)]
    pub approval: ActionApproval,

    /// Identifier of the orchestrator action logic to execute.
    pub kind: String,

    /// Additional unstructured metadata attached to the action.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// Timeout after which the running action is failed.
    ///
    /// Overrides the default execution logic timeout.
    #[serde(default)]
    pub timeout: Option<Duration>,
}

/// Definition of entries returned when listing platforms in a namespace.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlatformEntry {
    /// Indicates if the Platform is active or not.
    pub active: bool,

    /// Namespaced identifier of the Platform.
    pub name: String,
}
