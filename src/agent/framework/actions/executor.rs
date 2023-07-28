//! Execute running and queued actions, progressing them until a final state.
use std::future::Future;
use std::time::Duration;

use anyhow::Error;
use anyhow::Result;

use crate::agent::framework::actions::ActionHandlerChangeValue;
use crate::agent::framework::actions::ActionsRegistry;
use crate::agent::framework::store::query::ActionNextToExecute;
use crate::agent::framework::store::Store;
use crate::agent::framework::DefaultContext;
use crate::agent::framework::Injector;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionPhase;
use crate::utils::error::slog::ErrorAttributes;

const EXECUTE_DELAY: Duration = Duration::from_secs(10);

/// Background worker to execute agent actions.
pub struct ActionsExecutor {
    context: DefaultContext,
    registry: ActionsRegistry,
    store: Store,
}

impl ActionsExecutor {
    /// Loop executing agent until the process is shut down.
    pub async fn task<S>(self, shutdown: S) -> Result<()>
    where
        S: Future<Output = ()>,
    {
        tokio::pin!(shutdown);
        slog::debug!(self.context.logger, "Starting actions execution");

        loop {
            // Look for next action to execute and invoke its handler.
            let action = self
                .store
                .query(&self.context, ActionNextToExecute {})
                .await;
            if let Err(error) = self.task_loop(action).await {
                slog::error!(
                    self.context.logger,
                    "Actions execution loop did not complete successfully";
                    ErrorAttributes::from(&error)
                );
                // TODO(metrics): count errors.
            }

            // Sleep until the next cycle or shutdown.
            tokio::select! {
                _ = tokio::time::sleep(EXECUTE_DELAY) => {},
                _ = &mut shutdown => {
                    slog::debug!(self.context.logger, "Gracefully shutting down actions executor");
                    return Ok(());
                }
            }
        }
    }

    /// Initialise an [`ActionsExecutor`] with dependencies from the given [`Injector`].
    pub fn with_injector(injector: &Injector) -> Self {
        let context = DefaultContext {
            logger: injector
                .logger
                .new(slog::o!("component" => "actions-executor")),
        };
        ActionsExecutor {
            context,
            registry: injector.actions.clone(),
            store: injector.store.clone(),
        }
    }
}

impl ActionsExecutor {
    /// Handle execution logic of any running or queued actions.
    async fn task_loop(&self, action: Result<Option<ActionExecution>>) -> Result<()> {
        let action = match action? {
            None => return Ok(()),
            Some(action) => action,
        };

        // Lookup the action handler and invoke it.
        let metadata = match self.registry.lookup(&action.kind) {
            Err(error) => return self.fail_action(action, error).await,
            Ok(metadata) => metadata,
        };
        let mut changes = match metadata.handler.invoke(&self.context, &action).await {
            Err(error) => return self.fail_action(action, error).await,
            Ok(changes) => changes,
        };

        // If the action was new and invocation did not fail ensure it is now running.
        if changes.phase == ActionExecutionPhase::New {
            changes.phase = ActionExecutionPhase::Running;
        }
        let changes = changes;

        // Update the ActionExecution record based on the invocation result.
        let mut action = action;
        let mut save = false;
        if changes.phase != action.state.phase {
            action.state.phase = changes.phase;
            save = true;
        }
        match changes.error {
            ActionHandlerChangeValue::Remove if action.state.error.is_some() => {
                action.state.error = None;
                save = true;
            }
            ActionHandlerChangeValue::Update(error) => {
                let error = Some(error);
                if action.state.error != error {
                    action.state.error = error;
                    save = true;
                }
            }
            _ => (),
        }
        match changes.payload {
            ActionHandlerChangeValue::Remove if action.state.payload.is_some() => {
                action.state.payload = None;
                save = true;
            }
            ActionHandlerChangeValue::Update(payload) => {
                let payload = Some(payload);
                if action.state.payload != payload {
                    action.state.payload = payload;
                    save = true;
                }
            }
            _ => (),
        }

        if !save {
            return Ok(());
        }
        self.store.persist(&self.context, action).await
    }

    /// Fail the action due to an error during handling or invocation.
    async fn fail_action(&self, mut action: ActionExecution, error: Error) -> Result<()> {
        action.state.error = Some(crate::utils::error::into_json(error));
        action.finish(ActionExecutionPhase::Failed);
        self.store.persist(&self.context, action).await
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::ActionsExecutor;
    use super::DefaultContext;
    use crate::agent::framework::actions::ActionHandler;
    use crate::agent::framework::actions::ActionHandlerChanges as Changes;
    use crate::agent::framework::actions::ActionMetadata;
    use crate::agent::framework::actions::ActionsRegistry;
    use crate::agent::framework::store::fixtures;
    use crate::agent::framework::store::query::Action;
    use crate::agent::framework::Injector;
    use crate::agent::models::ActionExecution;
    use crate::agent::models::ActionExecutionPhase;

    const ACTION_KIND_DONE: &str = "agent.replicante.io/test.done";
    const ACTION_KIND_FAIL: &str = "agent.replicante.io/test.fail";
    const ACTION_KIND_NO_CHANGE: &str = "agent.replicante.io/test.no.change";
    const ACTION_KIND_RESET: &str = "agent.replicante.io/test.reset";
    const ACTION_KIND_UPDATE: &str = "agent.replicante.io/test.update";

    pub struct DoneAction;
    #[async_trait::async_trait]
    impl ActionHandler for DoneAction {
        async fn invoke(&self, _: &DefaultContext, _: &ActionExecution) -> Result<Changes> {
            Ok(Changes::to(ActionExecutionPhase::Done))
        }
    }

    pub struct FailAction;
    #[async_trait::async_trait]
    impl ActionHandler for FailAction {
        async fn invoke(&self, _: &DefaultContext, _: &ActionExecution) -> Result<Changes> {
            anyhow::bail!(anyhow::anyhow!("test action to always fail"));
        }
    }

    pub struct LoopAction;
    #[async_trait::async_trait]
    impl ActionHandler for LoopAction {
        async fn invoke(&self, _: &DefaultContext, action: &ActionExecution) -> Result<Changes> {
            Ok(Changes::to(action.state.phase))
        }
    }

    pub struct ResetAction;
    #[async_trait::async_trait]
    impl ActionHandler for ResetAction {
        async fn invoke(&self, _: &DefaultContext, _: &ActionExecution) -> Result<Changes> {
            let changes = Changes::to(ActionExecutionPhase::Done)
                .error(None)
                .payload(None);
            Ok(changes)
        }
    }

    pub struct UpdateAction;
    #[async_trait::async_trait]
    impl ActionHandler for UpdateAction {
        async fn invoke(&self, _: &DefaultContext, _: &ActionExecution) -> Result<Changes> {
            let changes = Changes::to(ActionExecutionPhase::Done)
                .error(serde_json::json!({ "changed": true }))
                .payload(serde_json::json!({ "result": 42 }));
            Ok(changes)
        }
    }

    struct Fixtures {
        action: ActionExecution,
        context: DefaultContext,
        executor: ActionsExecutor,
        injector: Injector,
    }

    impl Fixtures {
        /// Initialise fixtures for [`ActionsExecutor`] tests.
        async fn default() -> Fixtures {
            Fixtures::with_action_config(|mut action| {
                action.kind = fixtures::ACTION_KIND.to_string();
                action
            })
            .await
        }

        async fn with_action_config<F>(configure_action: F) -> Fixtures
        where
            F: FnOnce(ActionExecution) -> ActionExecution,
        {
            let id = uuid::Uuid::new_v4();
            let action = configure_action(fixtures::action(id));
            let context = DefaultContext::fixture();
            let mut injector = Injector::fixture().await;

            injector
                .store
                .persist(&context, action.clone())
                .await
                .unwrap();

            let actions = ActionsRegistry::build()
                .register(ActionMetadata::build_internal(ACTION_KIND_DONE, DoneAction).finish())
                .register(ActionMetadata::build_internal(ACTION_KIND_FAIL, FailAction).finish())
                .register(
                    ActionMetadata::build_internal(ACTION_KIND_NO_CHANGE, LoopAction).finish(),
                )
                .register(ActionMetadata::build_internal(ACTION_KIND_RESET, ResetAction).finish())
                .register(ActionMetadata::build_internal(ACTION_KIND_UPDATE, UpdateAction).finish())
                .finish();
            injector.actions = actions;

            let executor = ActionsExecutor::with_injector(&injector);
            Fixtures {
                action,
                context,
                executor,
                injector,
            }
        }

        /// Get the action from the store to assert on changes.
        async fn action_from_store(&self) -> Option<ActionExecution> {
            let query = Action::new(self.action.id);
            self.injector
                .store
                .query(&self.context, query)
                .await
                .unwrap()
        }
    }

    #[tokio::test]
    async fn invoke_error() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_FAIL.to_string();
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Failed);
        let error = action.state.error.expect("structured error details");
        assert_eq!(
            error,
            serde_json::json!({
                "error_msg": "test action to always fail",
            })
        );
    }

    #[tokio::test]
    async fn invoke_no_changes() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_NO_CHANGE.to_string();
            action.state.error = Some(serde_json::json!({ "error": false }));
            action.state.payload = Some(serde_json::json!({ "payload": true }));
            action.state.phase = ActionExecutionPhase::Running;
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Running);
        assert_eq!(
            action.state.error,
            Some(serde_json::json!({ "error": false }))
        );
        assert_eq!(
            action.state.payload,
            Some(serde_json::json!({ "payload": true }))
        );
    }

    #[tokio::test]
    async fn invoke_no_changes_new() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_NO_CHANGE.to_string();
            action.state.error = Some(serde_json::json!({ "error": false }));
            action.state.payload = Some(serde_json::json!({ "payload": true }));
            action.state.phase = ActionExecutionPhase::New;
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Running);
        assert_eq!(
            action.state.error,
            Some(serde_json::json!({ "error": false })),
        );
        assert_eq!(
            action.state.payload,
            Some(serde_json::json!({ "payload": true })),
        );
    }

    #[tokio::test]
    async fn invoke_update_phase() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_DONE.to_string();
            action.state.error = Some(serde_json::json!({ "error": false }));
            action.state.payload = Some(serde_json::json!({ "payload": true }));
            action.state.phase = ActionExecutionPhase::New;
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Done);
        assert_eq!(
            action.state.error,
            Some(serde_json::json!({ "error": false }))
        );
        assert_eq!(
            action.state.payload,
            Some(serde_json::json!({ "payload": true }))
        );
    }

    #[tokio::test]
    async fn invoke_update_state() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_UPDATE.to_string();
            action.state.error = Some(serde_json::json!({ "error": false }));
            action.state.payload = Some(serde_json::json!({ "payload": true }));
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Done);
        assert_eq!(
            action.state.error,
            Some(serde_json::json!({ "changed": true }))
        );
        assert_eq!(
            action.state.payload,
            Some(serde_json::json!({ "result": 42 }))
        );
    }

    #[tokio::test]
    async fn invoke_reset_state() {
        let fixtures = Fixtures::with_action_config(|mut action| {
            action.kind = ACTION_KIND_RESET.to_string();
            action.state.error = Some(serde_json::json!({ "error": false }));
            action.state.payload = Some(serde_json::json!({ "payload": true }));
            action
        })
        .await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Done);
        assert_eq!(action.state.error, None);
        assert_eq!(action.state.payload, None);
    }

    #[tokio::test]
    async fn metadata_lookup_failed() {
        let fixtures = Fixtures::default().await;
        let action = Ok(Some(fixtures.action.clone()));
        fixtures.executor.task_loop(action).await.unwrap();

        let action = fixtures.action_from_store().await.unwrap();
        assert_eq!(action.state.phase, ActionExecutionPhase::Failed);
        let error = action.state.error.expect("structured error details");
        assert_eq!(
            error,
            serde_json::json!({
                "error_msg": "metadata for action agent.replicante.io/test not found",
            })
        );
    }

    #[tokio::test]
    async fn skip_on_no_action() {
        let fixtures = Fixtures::default().await;
        let action = Ok(None);
        fixtures.executor.task_loop(action).await.unwrap();
    }
}
