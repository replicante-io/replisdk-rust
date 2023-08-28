//! Set of fixtures to test Store logic.
use uuid::Uuid;

use crate::agent::framework::store::Store;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionPhase;
use crate::agent::models::ActionExecutionState;
use crate::context::Context;

pub const ACTION_KIND: &str = "agent.replicante.io/test.success";

/// Create a new action with the given UUID.
pub fn action(id: Uuid) -> ActionExecution {
    let timestamp = time::OffsetDateTime::parse(
        "2023-04-05T06:07:08Z",
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    ActionExecution {
        args: serde_json::Value::Null,
        created_time: timestamp,
        finished_time: None,
        id,
        kind: String::from(ACTION_KIND),
        metadata: Default::default(),
        scheduled_time: timestamp,
        state: ActionExecutionState {
            error: None,
            payload: None,
            phase: ActionExecutionPhase::New,
        },
    }
}

/// Create an in-memory store for tests to use.
pub async fn store() -> Store {
    let context = Context::fixture();
    let path = ":memory:";
    Store::initialise(&context.logger, path)
        .await
        .expect("store to be initialised")
}
