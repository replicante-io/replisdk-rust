//! Replicante Agent action models.
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

/// Information about an Agent Action execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecution {
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

    /// Unique ID of the action execution.
    pub id: Uuid,

    /// Identifier of the action implementation to execute.
    pub kind: String,

    /// Unstructured metadata attached to the action when it was created.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// Time the agent recorded the action execution in its own store.
    #[serde(with = "time::serde::rfc3339")]
    pub scheduled_time: OffsetDateTime,

    /// Current state of an Agent Action execution.
    pub state: ActionExecutionState,
}

impl ActionExecution {
    /// Finish the action by transitioning to the given state.
    pub fn finish(&mut self, phase: ActionExecutionPhase) {
        self.state.phase = phase;
        self.finished_time = Some(time::OffsetDateTime::now_utc());
    }

    /// Update the [`ActionExecution`] phase and apply side effects.
    ///
    /// For final states (`Done` and `Failed`) this is equivalent to [`ActionExecution::finish`].
    pub fn phase_to(&mut self, phase: ActionExecutionPhase) {
        if matches!(
            phase,
            ActionExecutionPhase::Done | ActionExecutionPhase::Failed
        ) {
            self.finish(phase);
            return;
        }
        self.state.phase = phase;
    }
}

/// API response for lookups of lists of [`ActionExecution`]s records.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecutionList {
    /// Actions returned by the lookup operation.
    pub actions: Vec<ActionExecutionListItem>,
}

/// Summary information about [`ActionExecution`]s stored on an agent.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecutionListItem {
    /// Unique identifier of the action execution.
    pub id: Uuid,

    /// Identifier of the action implementation to execute.
    pub kind: String,

    /// Current phase of the action execution process.
    pub phase: ActionExecutionPhase,
}

/// Phases of the action execution process.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActionExecutionPhase {
    /// The action execution completed successfully.
    #[serde(rename = "DONE")]
    Done,

    /// The action execution resulted in an error and won't progress any further.
    #[serde(rename = "FAILED")]
    Failed,

    /// The action execution has not begun but it is ready to do so.
    #[serde(rename = "NEW")]
    New,

    /// The action execution is in progress and the system is moving towards a final state.
    #[serde(rename = "RUNNING")]
    Running,
}

/// API Request schema for an [`ActionExecution`] schedule call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecutionRequest {
    /// Arguments passed to the action execution being created.
    #[serde(default)]
    pub args: Json,

    /// Time the action execution was first created.
    ///
    /// An action execution may be created in systems other then Agents (such as Core).
    /// In such cases the `created_time` is the time the action execution was created in the
    /// system other then the Agent (such as Core) and is passed to Agents.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub created_time: Option<OffsetDateTime>,

    /// Unique ID of the action execution.
    ///
    /// An ID is generated automatically if none is provided.
    #[serde(default)]
    pub id: Option<Uuid>,

    /// Identifier of the action implementation to execute.
    pub kind: String,

    /// Unstructured metadata attached to the action when it was created.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl From<ActionExecutionRequest> for ActionExecution {
    fn from(value: ActionExecutionRequest) -> Self {
        let now = time::OffsetDateTime::now_utc();
        ActionExecution {
            args: value.args,
            created_time: value.created_time.unwrap_or(now),
            finished_time: None,
            id: value.id.unwrap_or_else(uuid::Uuid::new_v4),
            kind: value.kind,
            metadata: value.metadata,
            scheduled_time: now,
            state: ActionExecutionState {
                error: None,
                payload: None,
                phase: ActionExecutionPhase::New,
            },
        }
    }
}

/// API Response schema for an [`ActionExecution`] schedule call.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecutionResponse {
    /// Unique identifier of the action execution.
    pub id: Uuid,
}

/// State of an Agent Action execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionExecutionState {
    /// Loosely structured information for any error encountered during action execution.
    #[serde(default)]
    pub error: Option<Json>,

    /// Scratch pad for action implementations to keep track of how they are processing.
    #[serde(default)]
    pub payload: Option<Json>,

    /// Current phase of the action execution process.
    pub phase: ActionExecutionPhase,
}
