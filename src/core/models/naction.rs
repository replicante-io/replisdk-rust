//! Models to describe Node Actions.
//!
//! Node actions are performed on nodes along side the store process.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::agent::models::ActionExecutionPhase;
use crate::agent::models::ActionExecutionState;

/// Information about an Node Action execution (definition and state).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NAction {
    // ID attributes.
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Cluster Node ID for the action.
    pub node_id: String,

    /// Cluster unique ID of the action record.
    pub action_id: Uuid,

    // Record attributes.
    /// Arguments passed to the action when it was created.
    #[serde(default)]
    pub args: Json,

    /// Time the action was first created.
    ///
    /// An action execution may be created in systems other then Agents (such as Core).
    /// In such cases the `created_time` is the time the action execution was created in the
    /// system other then the Agent (such as Core) and is passed to Agents.
    #[serde(with = "time::serde::rfc3339")]
    pub created_time: OffsetDateTime,

    /// Time the action entered a final state, for finished actions only.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub finished_time: Option<OffsetDateTime>,

    /// Identifier of the action implementation to execute.
    pub kind: String,

    /// Unstructured metadata attached to the action when it was created.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// Time the agent recorded the action execution in its own store.
    #[serde(with = "time::serde::rfc3339::option")]
    pub scheduled_time: Option<OffsetDateTime>,

    /// Current state of an Agent Action execution.
    pub state: NActionState,
}

impl NAction {
    /// Mark the action as finished and sets the finish timestamp to now.
    pub fn finish(&mut self, phase: NActionPhase) {
        self.state.phase = phase;
        self.finished_time = Some(OffsetDateTime::now_utc());
    }

    /// Update the [`NAction`] state and apply side effects if needed.
    ///
    /// For final states (`Cancelled`, `Done`, ...) this is equivalent to
    /// [`NAction::finish`].
    pub fn phase_to(&mut self, phase: NActionPhase) {
        if phase.is_final() {
            self.finish(phase);
            return;
        }
        self.state.phase = phase;
    }
}

/// Phases of the node action execution process.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NActionPhase {
    /// The action was interrupted or never executed.
    #[serde(rename = "CANCELLED")]
    Cancelled,

    /// The action execution completed successfully.
    #[serde(rename = "DONE")]
    Done,

    /// The action execution resulted in an error and won't progress any further.
    #[serde(rename = "FAILED")]
    Failed,

    /// The action execution record was deleted by the agent before a sync with a final state.
    #[serde(rename = "LOST")]
    Lost,

    /// The action execution has not begun but it is ready to do so.
    #[serde(rename = "NEW")]
    New,

    /// The Control Plane is waiting for a user to approve the action before executing it.
    #[serde(rename = "PENDING_APPROVE")]
    PendingApprove,

    /// The Control Plane knows about the action and will start it in due course.
    #[serde(rename = "PENDING_SCHEDULE")]
    PendingSchedule,

    /// The action execution is in progress and the system is moving towards a final state.
    #[serde(rename = "RUNNING")]
    Running,
}

impl std::fmt::Display for NActionPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Done => write!(f, "DONE"),
            Self::Failed => write!(f, "FAILED"),
            Self::Lost => write!(f, "LOST"),
            Self::New => write!(f, "NEW"),
            Self::PendingApprove => write!(f, "PENDING_APPROVE"),
            Self::PendingSchedule => write!(f, "PENDING_SCHEDULE"),
            Self::Running => write!(f, "RUNNING"),
        }
    }
}

impl NActionPhase {
    /// Check if the action is in a final state (done, failed, ...).
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            Self::Cancelled | Self::Done | Self::Failed | Self::Lost
        )
    }

    /// Check if the action is running or sent to the agent to run.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl From<ActionExecutionPhase> for NActionPhase {
    fn from(value: ActionExecutionPhase) -> Self {
        match value {
            ActionExecutionPhase::Done => Self::Done,
            ActionExecutionPhase::Failed => Self::Failed,
            ActionExecutionPhase::New => Self::New,
            ActionExecutionPhase::Running => Self::Running,
        }
    }
}

/// Identifier attributes for a [`NAction`].
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct NActionRef {
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Node ID the action targets.
    pub node_id: String,

    /// Cluster unique ID of the action record.
    pub action_id: Uuid,
}

/// State of a Node Action execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NActionState {
    /// Loosely structured information for any error encountered during action execution.
    #[serde(default)]
    pub error: Option<Json>,

    /// Scratch pad for action implementations to keep track of how they are processing.
    #[serde(default)]
    pub payload: Option<Json>,

    /// Current phase of the action execution process.
    pub phase: NActionPhase,
}

impl From<ActionExecutionState> for NActionState {
    fn from(value: ActionExecutionState) -> Self {
        NActionState {
            error: value.error,
            payload: value.payload,
            phase: value.phase.into(),
        }
    }
}
