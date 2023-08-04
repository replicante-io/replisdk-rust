//! Background task to perform [`Store`] cleaning tasks.
use std::future::Future;
use std::time::Duration;

use anyhow::Result;

use super::Store;
use crate::agent::framework::store::manage;
use crate::agent::framework::DefaultContext;
use crate::agent::framework::Injector;
use crate::utils::error::slog::ErrorAttributes;

const EXECUTE_DELAY: Duration = Duration::from_secs(10);

/// Background task to periodically clean the agent store.
///
/// Cleaning the store is done to prevent historic data from growing unbound and cause
/// performance issues.
///
/// The following cleaning tasks are performed:
///
/// - Finished actions are removed after the configured amount of time.
pub struct StoreClean {
    context: DefaultContext,
    store: Store,
}

impl StoreClean {
    /// Loop performing store cleaning duties until process shutdown.
    pub async fn task<S>(self, shutdown: S) -> Result<()>
    where
        S: Future<Output = ()>,
    {
        tokio::pin!(shutdown);
        slog::debug!(self.context.logger, "Starting background store cleaner");

        loop {
            // Execute a cleaning loop.
            if let Err(error) = self.task_loop().await {
                slog::error!(
                    self.context.logger,
                    "Store clean loop encountered an error";
                    ErrorAttributes::from(&error)
                );
                // TODO(metrics): count errors.
            }

            // Sleep until the next cycle or shutdown.
            tokio::select! {
                _ = tokio::time::sleep(EXECUTE_DELAY) => {},
                _ = &mut shutdown => {
                    slog::debug!(self.context.logger, "Gracefully shutting down store cleaner");
                    return Ok(());
                }
            }
        }
    }

    /// Initialise a [`StoreClean`] with dependencies from the given [`Injector`].
    pub fn with_injector(injector: &Injector) -> StoreClean {
        let context = DefaultContext {
            logger: injector
                .logger
                .new(slog::o!("component" => "store-cleaner")),
        };
        StoreClean {
            context,
            store: injector.store.clone(),
        }
    }
}

impl StoreClean {
    /// Perform a round of cleaning duties.
    async fn task_loop(&self) -> Result<()> {
        // TODO: make age configurable.
        let age = Duration::from_secs(14 * 24 * 3600);
        let expire = time::OffsetDateTime::now_utc() - age;
        let expire = manage::CleanActions::since(expire);
        self.store.manage(&self.context, expire).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use time::OffsetDateTime;

    use super::DefaultContext;
    use super::Injector;
    use super::StoreClean;
    use crate::agent::framework::store::fixtures;
    use crate::agent::models::ActionExecutionPhase;

    struct Fixtures {
        context: DefaultContext,
        injector: Injector,
    }

    impl Fixtures {
        async fn default() -> Fixtures {
            let context = DefaultContext::fixture();
            let injector = Injector::fixture().await;
            Fixtures { context, injector }
        }

        async fn add_action<T>(&self, phase: ActionExecutionPhase, finished: T)
        where
            T: Into<Option<OffsetDateTime>>,
        {
            let mut action = fixtures::action(uuid::Uuid::new_v4());
            action.finished_time = finished.into();
            action.state.phase = phase;
            self.injector
                .store
                .persist(&self.context, action)
                .await
                .unwrap();
        }

        async fn count_actions(&self) -> i32 {
            self.injector
                .store
                .store
                .call(|connection| {
                    let mut statement = connection.prepare("SELECT COUNT(*) FROM actions;")?;
                    let count: i32 = statement.query_row([], |row| row.get(0))?;
                    Ok(count)
                })
                .await
                .expect("sql actions count failed")
        }
    }

    #[tokio::test]
    async fn clean_actions() {
        let fixtures = Fixtures::default().await;
        let recent = OffsetDateTime::now_utc() - Duration::from_secs(30);
        let old = OffsetDateTime::now_utc() - Duration::from_secs(15 * 24 * 60 * 60);
        fixtures
            .add_action(ActionExecutionPhase::Running, None)
            .await;
        fixtures
            .add_action(ActionExecutionPhase::Done, recent)
            .await;
        fixtures.add_action(ActionExecutionPhase::Done, old).await;
        fixtures.add_action(ActionExecutionPhase::Failed, old).await;

        let cleaner = StoreClean::with_injector(&fixtures.injector);
        cleaner.task_loop().await.unwrap();

        let actions = fixtures.count_actions().await;
        assert_eq!(actions, 2);
    }

    #[tokio::test]
    async fn clean_actions_nothing_to_do() {
        let fixtures = Fixtures::default().await;
        let recent = OffsetDateTime::now_utc() - Duration::from_secs(30);
        fixtures.add_action(ActionExecutionPhase::New, None).await;
        fixtures
            .add_action(ActionExecutionPhase::Running, None)
            .await;
        fixtures
            .add_action(ActionExecutionPhase::Running, None)
            .await;
        fixtures
            .add_action(ActionExecutionPhase::Done, recent)
            .await;
        fixtures
            .add_action(ActionExecutionPhase::Failed, recent)
            .await;

        let cleaner = StoreClean::with_injector(&fixtures.injector);
        cleaner.task_loop().await.unwrap();

        let actions = fixtures.count_actions().await;
        assert_eq!(actions, 5);
    }
}
