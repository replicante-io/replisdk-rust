//! RepliCore definitions for both node and orchestrator actions.
use serde::Deserialize;
use serde::Serialize;

use super::oaction::OActionState;

/// Automatically grant or explicitly require approval before actions are executed.
///
/// The main value of this attribute is in conjunctions with automated action submission
/// such as the declarative clusters feature.
/// In such cases users can request explicit approval to review automated actions
/// or to have greater control of when the system will execute these automated actions.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ActionApproval {
    /// The action can be scheduled (and begin execution) as soon as possible.
    #[default]
    #[serde(rename = "granted")]
    Granted,

    /// The action must be reviewed and manually approved before it can execute.
    #[serde(rename = "required")]
    Required,
}

impl std::fmt::Display for ActionApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionApproval::Granted => write!(f, "granted"),
            ActionApproval::Required => write!(f, "required"),
        }
    }
}

impl From<ActionApproval> for OActionState {
    fn from(value: ActionApproval) -> Self {
        match value {
            ActionApproval::Granted => OActionState::PendingSchedule,
            ActionApproval::Required => OActionState::PendingApprove,
        }
    }
}
