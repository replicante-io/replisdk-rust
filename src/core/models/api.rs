//! RepliCore definitions for data in API endpoints.
use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

use super::action::ActionApproval;
use super::naction::NActionPhase;
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

/// Response for the naction list API endpoint.
pub type NActionList = EntriesList<NActionEntry>;

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

/// Definition of entries returned when listing node actions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NActionEntry {
    /// Namespace the cluster is in.
    pub ns_id: String,

    /// Namespaced identifier of the cluster.
    pub cluster_id: String,

    /// Node in the cluster the action is targeting.
    pub node_id: String,

    /// Identifier of the action.
    pub action_id: Uuid,

    /// Timestamp of action creation.
    #[serde(with = "time::serde::rfc3339")]
    pub created_time: OffsetDateTime,

    /// Timestamp action entered a final state (success or failure).
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub finished_time: Option<OffsetDateTime>,

    /// Identifier of the node action logic to execute.
    pub kind: String,

    /// Phase the action is currently in.
    #[serde(deserialize_with = "deserialize::naction_entry_state")]
    pub state: NActionPhase,
}

/// Specification of [`NAction`] for the apply interface.
///
/// The apply interface creates new actions and not all fields in [`NAction`] records
/// are needed or should be provided by users. For example:
///
/// - A new UUID can be auto-created.
/// - The action state has to be new and can't be set by users.
///
/// [`NAction`]: crate::core::models::naction::NAction
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NActionSpec {
    /// Namespace the cluster is in.
    pub ns_id: String,

    /// Namespaced identifier of the cluster.
    pub cluster_id: String,

    /// Identifier of the node the action is for.
    pub node_id: String,

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
///
/// [`OAction`]: crate::core::models::oaction::OAction
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

    /// Namespace of the Platform.
    pub ns_id: String,
}

mod deserialize {
    use serde::de::Deserializer;
    use serde::Deserialize;

    use super::super::naction::NActionPhase;
    use super::super::naction::NActionState;

    /// Helper type to decode node action phase for NActionEntry objects.
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum NActionPhaseOrState {
        /// Deserialize the phase directly.
        Phase(NActionPhase),

        /// Deserialize the phase for the nested state type.
        State(NActionState),
    }

    /// Decode a node action phase either directly or from a nested [`NActionState`].
    pub fn naction_entry_state<'a, D>(deserializer: D) -> Result<NActionPhase, D::Error>
    where
        D: Deserializer<'a>,
    {
        let value = NActionPhaseOrState::deserialize(deserializer)?;
        match value {
            NActionPhaseOrState::Phase(phase) => Ok(phase),
            NActionPhaseOrState::State(state) => Ok(state.phase),
        }
    }
}
