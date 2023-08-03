//! Agent [`ActionExecution`] handling definitions.
use anyhow::Result;

use crate::agent::framework::DefaultContext;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionPhase;

/// Action logic to progress an [`ActionExecution`] record.
#[async_trait::async_trait]
pub trait ActionHandler: std::fmt::Debug + Send + Sync {
    /// Execute action specific logic to move an [`ActionExecution`] towards a final state.
    ///
    /// [`ActionExecution`] records track the current recorded state of an action
    /// execution and are provided to the invoke method to determine what the next steps are.
    ///
    /// When the [`ActionExecutionState::phase`] is `NEW` the execution has never executed yet.
    /// In this case the [`ActionExecution`] is updated to the `RUNNING` phase if the method
    /// returns no change details, otherwise the details are used to update the action.
    ///
    /// ## Errors
    ///
    /// If the invocation fails for any reason it can return an error to indicate so.
    /// On error the [`ActionExecution`] is updated to the final `FAILED` state.
    /// The error information is stored in the [`ActionExecutionState::error`] for user review.
    ///
    /// Retry of failed actions is NOT automatically handled so transient failures need
    /// to be handled by the implementation if needed.
    ///
    /// [`ActionExecutionState::error`]: crate::agent::models::ActionExecutionState::error
    /// [`ActionExecutionState::phase`]: crate::agent::models::ActionExecutionState::phase
    async fn invoke(
        &self,
        context: &DefaultContext,
        action: &ActionExecution,
    ) -> Result<ActionHandlerChanges>;
}

/// Changes to an [`ActionExecution`] record as a result of its [`ActionHandler`] invocation.
pub struct ActionHandlerChanges {
    /// Optionally change the action error data.
    pub(in crate::agent::framework) error: ActionHandlerChangeValue,

    /// Optionally change the action payload data.
    pub(in crate::agent::framework) payload: ActionHandlerChangeValue,

    /// Change the [`ActionExecution`] phase.
    pub(in crate::agent::framework) phase: ActionExecutionPhase,
}

impl ActionHandlerChanges {
    /// Update or reset the action error data.
    pub fn error<E>(mut self, error: E) -> Self
    where
        E: Into<Option<serde_json::Value>>,
    {
        self.error = match error.into() {
            Some(error) => ActionHandlerChangeValue::Update(error),
            None => ActionHandlerChangeValue::Remove,
        };
        self
    }

    /// Update or reset the action payload data.
    pub fn payload<P>(mut self, payload: P) -> Self
    where
        P: Into<Option<serde_json::Value>>,
    {
        self.payload = match payload.into() {
            Some(payload) => ActionHandlerChangeValue::Update(payload),
            None => ActionHandlerChangeValue::Remove,
        };
        self
    }

    /// Update the action phase as a result of this invocation.
    pub fn to(phase: ActionExecutionPhase) -> ActionHandlerChanges {
        ActionHandlerChanges {
            error: Default::default(),
            payload: Default::default(),
            phase,
        }
    }
}

/// Describes how state data should be changed after an [`ActionHandler::invoke`] call.
#[derive(Debug, Default)]
pub enum ActionHandlerChangeValue {
    /// Remove the current state data.
    Remove,

    /// No changes should be made to state data.
    #[default]
    Unchanged,

    /// Update the state data to the given value.
    Update(serde_json::Value),
}
