//! Action implementations for the `agent.replicante.io/test.*` group.
use anyhow::Result;

use crate::agent::framework::actions::ActionHandler;
use crate::agent::framework::actions::ActionHandlerChanges as Changes;
use crate::agent::framework::actions::ActionMetadata;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionPhase;
use crate::context::Context;

const KIND_PREFIX: &str = "agent.replicante.io/test";

/// Fail action execution as soon as invoked.
#[derive(Debug)]
pub struct Fail;

impl Fail {
    /// Registration metadata for the `agent.replicante.io/test.fail` action.
    pub fn metadata() -> ActionMetadata {
        ActionMetadata::build_internal(format!("{}.fail", KIND_PREFIX), Fail).finish()
    }
}

#[async_trait::async_trait]
impl ActionHandler for Fail {
    async fn invoke(&self, _: &Context, _: &ActionExecution) -> Result<Changes> {
        anyhow::bail!(anyhow::anyhow!("Test action failed as expected"))
    }
}

/// Increment a counter each invocation loop until a total is reached.
///
/// The total to reach can be specified with `target` key in the [`ActionExecution::args`].
/// A default of 10 is used if the argument is missing or invalid.
///
/// When the target total is reached the action is completed.
#[derive(Debug)]
pub struct Loop;

impl Loop {
    /// Registration metadata for the `agent.replicante.io/test.loop` action.
    pub fn metadata() -> ActionMetadata {
        ActionMetadata::build_internal(format!("{}.loop", KIND_PREFIX), Loop).finish()
    }
}

#[async_trait::async_trait]
impl ActionHandler for Loop {
    async fn invoke(&self, _: &Context, action: &ActionExecution) -> Result<Changes> {
        // Get target from the payload, or fallback to args, or fallback to default.
        let target = action
            .state
            .payload
            .as_ref()
            .and_then(|payload| payload.as_object())
            .or_else(|| action.args.as_object())
            .and_then(|object| object.get("target"))
            .and_then(|target| target.as_u64())
            .unwrap_or(10);
        // Get current count for payload, or fallback to start.
        let count = action
            .state
            .payload
            .as_ref()
            .and_then(|payload| payload.as_object())
            .and_then(|payload| payload.get("count"))
            .and_then(|count| count.as_u64())
            .unwrap_or(0);
        let count = count + 1;
        let phase = if count >= target {
            ActionExecutionPhase::Done
        } else {
            ActionExecutionPhase::Running
        };
        let changes = Changes::to(phase).payload(serde_json::json!({
            "count": count,
            "target": target,
        }));
        Ok(changes)
    }
}

/// Complete action execution as soon as invoked.
#[derive(Debug)]
pub struct Success;

impl Success {
    /// Registration metadata for the `agent.replicante.io/test.success` action.
    pub fn metadata() -> ActionMetadata {
        ActionMetadata::build_internal(format!("{}.success", KIND_PREFIX), Success).finish()
    }
}

#[async_trait::async_trait]
impl ActionHandler for Success {
    async fn invoke(&self, _: &Context, _: &ActionExecution) -> Result<Changes> {
        Ok(Changes::to(ActionExecutionPhase::Done))
    }
}

/// Collection of actions metadata for the `agent.replicnate.io/test.*` group.
///
/// Register actions during agent initialisation with
///
/// ```ignore
/// Agent::build()
///     .register_actions(crate::agent::framework::actions::wellknown::test::all())
/// ```
pub fn all() -> impl IntoIterator<Item = ActionMetadata> {
    [Fail::metadata(), Loop::metadata(), Success::metadata()]
}

#[cfg(test)]
mod tests {
    use crate::agent::framework::actions::ActionHandlerChangeValue;
    use crate::agent::framework::store::fixtures;
    use crate::agent::models::ActionExecutionPhase;
    use crate::context::Context;

    #[tokio::test]
    async fn fail() {
        let action = fixtures::action(uuid::Uuid::new_v4());
        let context = Context::fixture();
        let meta = super::Fail::metadata();
        let changes = meta.handler.invoke(&context, &action).await;
        assert!(changes.is_err());
    }

    #[tokio::test]
    async fn loop_continue() {
        let mut action = fixtures::action(uuid::Uuid::new_v4());
        action.state.payload = Some(serde_json::json!({"target": 5, "count": 2}));
        let context = Context::fixture();
        let meta = super::Loop::metadata();
        let changes = meta.handler.invoke(&context, &action).await.unwrap();

        assert_eq!(changes.phase, ActionExecutionPhase::Running);
        let payload = match changes.payload {
            ActionHandlerChangeValue::Update(payload) => payload,
            other => panic!("expected payload changes but found {:?}", other),
        };
        let count = payload.as_object().unwrap().get("count").unwrap();
        assert_eq!(count, 3);
        let target = payload.as_object().unwrap().get("target").unwrap();
        assert_eq!(target, 5);
    }

    #[tokio::test]
    async fn loop_done() {
        let mut action = fixtures::action(uuid::Uuid::new_v4());
        action.state.payload = Some(serde_json::json!({"target": 3, "count": 2}));
        let context = Context::fixture();
        let meta = super::Loop::metadata();
        let changes = meta.handler.invoke(&context, &action).await.unwrap();

        assert_eq!(changes.phase, ActionExecutionPhase::Done);
        let payload = match changes.payload {
            ActionHandlerChangeValue::Update(payload) => payload,
            other => panic!("expected payload changes but found {:?}", other),
        };
        let count = payload.as_object().unwrap().get("count").unwrap();
        assert_eq!(count, 3);
        let target = payload.as_object().unwrap().get("target").unwrap();
        assert_eq!(target, 3);
    }

    #[tokio::test]
    async fn loop_new_no_arg() {
        let action = fixtures::action(uuid::Uuid::new_v4());
        let context = Context::fixture();
        let meta = super::Loop::metadata();
        let changes = meta.handler.invoke(&context, &action).await.unwrap();

        assert_eq!(changes.phase, ActionExecutionPhase::Running);
        let payload = match changes.payload {
            ActionHandlerChangeValue::Update(payload) => payload,
            other => panic!("expected payload changes but found {:?}", other),
        };
        let count = payload.as_object().unwrap().get("count").unwrap();
        assert_eq!(count, 1);
        let target = payload.as_object().unwrap().get("target").unwrap();
        assert_eq!(target, 10);
    }

    #[tokio::test]
    async fn loop_new_with_arg() {
        let mut action = fixtures::action(uuid::Uuid::new_v4());
        action.args = serde_json::json!({"target": 16});
        let context = Context::fixture();
        let meta = super::Loop::metadata();
        let changes = meta.handler.invoke(&context, &action).await.unwrap();

        assert_eq!(changes.phase, ActionExecutionPhase::Running);
        let payload = match changes.payload {
            ActionHandlerChangeValue::Update(payload) => payload,
            other => panic!("expected payload changes but found {:?}", other),
        };
        let count = payload.as_object().unwrap().get("count").unwrap();
        assert_eq!(count, 1);
        let target = payload.as_object().unwrap().get("target").unwrap();
        assert_eq!(target, 16);
    }

    #[tokio::test]
    async fn success() {
        let action = fixtures::action(uuid::Uuid::new_v4());
        let context = Context::fixture();
        let meta = super::Success::metadata();
        let changes = meta.handler.invoke(&context, &action).await.unwrap();
        assert_eq!(changes.phase, ActionExecutionPhase::Done);
    }
}
