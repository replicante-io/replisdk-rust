//! Models to describe Orchestrator Actions.
//! 
//! Orchestrator actions are performed at the Control Plane level rather then node level.
//! Control Plane in this case doesn't exclusively mean RepliCore but any action that executes
//! outside of individual nodes.
use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as Json;
use time::OffsetDateTime;
use uuid::Uuid;

/// Orchestrator action definition and state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OAction {
    // ID attributes.
    /// Namespace ID the cluster belongs to.
    pub ns_id: String,

    /// Namespace unique ID of the cluster.
    pub cluster_id: String,

    /// Cluster unique ID of the action record.
    pub action_id: Uuid,

    // Record attributes.
    /// Action-dependent arguments to execute with.
    #[serde(default)]
    pub args: Json,

    /// Timestamp of action creation.
    #[serde(with = "time::serde::rfc3339")]
    pub created_ts: OffsetDateTime,

    /// Timestamp action entered a final state (success or failure).
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub finished_ts: Option<OffsetDateTime>,

    /// Identifier of the orchestrator action logic to execute.
    pub kind: String,

    /// Additional unstructured metadata attached to the action.
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    /// Timestamp the action was started by the Control Plane.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub scheduled_ts: Option<OffsetDateTime>,

    /// State the action is currently in.
    pub state: OActionState,

    /// Action-dependent state data, if the action needs to persist state.
    #[serde(default)]
    pub state_payload: Option<Json>,

    /// Information about errors encountered when executing the action.
    #[serde(default)]
    pub state_payload_error: Option<Json>,

    /// Timeout after which the running action is failed.
    ///
    /// Overrides the default execution logic timeout.
    #[serde(default)]
    pub timeout: Option<Duration>,
}

impl OAction {
    /// Mark the action as finished and sets the finish timestamp to now.
    pub fn finish(&mut self, state: OActionState) {
        self.state = state;
        self.finished_ts = Some(OffsetDateTime::now_utc());
    }

    /// Create an empty action record with the basic required data.
    pub fn new<C, K, N>(ns_id: N, cluster_id: C, kind: K) -> OAction
    where
        C: Into<String>,
        K: Into<String>,
        N: Into<String>,
    {
        OAction {
            ns_id: ns_id.into(),
            cluster_id: cluster_id.into(),
            action_id: Uuid::new_v4(),
            args: Default::default(),
            created_ts: OffsetDateTime::now_utc(),
            finished_ts: None,
            kind: kind.into(),
            metadata: Default::default(),
            scheduled_ts: None,
            state: OActionState::PendingApprove,
            state_payload: None,
            state_payload_error: None,
            timeout: None,
        }
    }
}

/// Current state of an orchestrator action in its execution lifecycle.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum OActionState {
    /// The action was interrupted or never executed.
    #[serde(rename = "CANCELLED")]
    Cancelled,

    /// The action finished successfully.
    #[serde(rename = "DONE")]
    Done,

    /// Unable to successfully execute the action.
    #[serde(rename = "FAILED")]
    Failed,

    /// The Control Plane is waiting for a user to approve the action before executing it.
    #[serde(rename = "PENDING_APPROVE")]
    PendingApprove,

    /// The Control Plane knows about the action and will start it in due course.
    #[serde(rename = "PENDING_SCHEDULE")]
    PendingSchedule,

    /// The action is running.
    #[serde(rename = "RUNNING")]
    Running,
}

impl OActionState {
    /// Check if the action is in a final state (done, failed, ...).
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Done | Self::Failed)
    }

    /// Check if the action is running or sent to the agent to run.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

impl std::fmt::Display for OActionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Done => write!(f, "DONE"),
            Self::Failed => write!(f, "FAILED"),
            Self::PendingApprove => write!(f, "PENDING_APPROVE"),
            Self::PendingSchedule => write!(f, "PENDING_SCHEDULE"),
            Self::Running => write!(f, "RUNNING"),
        }
    }
}
