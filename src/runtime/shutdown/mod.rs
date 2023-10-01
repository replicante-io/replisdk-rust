//! Tools to manage process shutdown on error or at user's request.
use std::future::Future;
use std::time::Duration;

use anyhow::Result;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use slog::Logger;
use tokio::sync::watch;
use tokio::task::JoinHandle;

#[cfg(test)]
mod tests;

/// Short-hand for tokio task handles that can return an [`anyhow::Result`].
type WatchTask<T> = JoinHandle<Result<T>>;

/// Short-hand for the output of a [`Future`] waiting for a [`WatchTask`] to exit.
type WatchTaskOutput<T> = Option<<WatchTask<T> as Future>::Output>;

/// Default time to wait for graceful shutdown to complete.
pub const DEFAULT_SHUTDOWN_GRACE_TIMEOUT: u64 = 2 * 60;

/// Exit code for abrupt exit caused by user signal during graceful shutdown.
const FORCE_SHUTDOWN_EXIT_CODE: i32 = 42;

/// Errors waiting for exit or during the shutdown sequence.
#[derive(Debug, thiserror::Error)]
pub enum ShutdownError {
    #[cfg(feature = "runtime-shutdown_actix")]
    /// Actix-web HttpServer stopped with an error.
    #[error("actix-web HttpServer stopped with an error")]
    ActixServer,

    /// Unable to wait for exit signal from the OS.
    #[error("unable to wait for exit signal from the OS")]
    SignalError,

    /// Returned when a tokio task was cancelled.
    #[error("tokio task was cancelled")]
    TokioTaskCancelled,

    /// Returned when a tokio task failed to join.
    #[error("tokio task failed to join")]
    TokioTaskError,

    /// Returned when a tokio task exited with a panic.
    #[error("tokio task exited with a panic")]
    TokioTaskPanic,
}

/// Manage process shutdown on error or user request, with support for clean-up chances.
///
/// # Process shutdown
///
/// The [`ShutdownManager`] instances await for the first exit condition
/// and begins a clean shutdown sequence.
///
/// Supported exit conditions are:
///
/// * Watching [`tokio::task`s]: begin exit when any registered task exists.
/// * Process signals (from users): begin exit when the process receives an exit signal from the OS.
///
/// The clean shutdown sequence works as follows:
///
/// 1. A shutdown notification is sent to all interested parties
///    (see [`ShutdownManagerBuilder::shutdown_notification`]).
/// 2. The [`ShutdownManager`] instance awaits for all registered [`tokio::task`s] to complete
///    or for a configurable timeout to expire, whichever comes first.
/// 3. All [`tokio::task`s] that have not completed yet are cancelled.
/// 4. The shutdown sequence returns, with the original error if a task triggered shutdown.
///
/// ## Watching [`tokio::task`s]
///
/// The [`ShutdownManager`] can watch [`tokio::task`s] for exit, either error or success.
/// This is done by registering [`JoinHandle`s](tokio::task::JoinHandle).
///
/// Tasks are used instead of generic features to simplify implementation of the shutdown logic:
/// [`tokio::task`s] run even when their future is not polled.
/// This enables them to react to the shutdown signal without [`ShutdownManager`]
/// having to drive them and they can be [cancelled](tokio::task::JoinHandle::abort).
///
/// ## Process signals
///
/// The [`ShutdownManager`] automatically awaits for user signals from the OS using
/// [`tokio::signal`] features.
///
/// When the shutdown signal is received once the above mentioned shutdown sequence begins.
/// If a second signal is sent to the process while shutdown is in progress the process is
/// terminated abruptly [`std::process::exit`].
///
/// Process signals are only used as an exit condition if an exit signal value is defined with
/// [`ShutdownManagerBuilder::watch_signal`] or
/// [`ShutdownManagerBuilder::watch_signal_with_default`].
///
/// ## Receiving the shutdown signal
///
/// During process initialisation components can register interest into shutdown notifications.
/// These are sent when the shutdown sequence begins and can be used to stop working and clean up
/// when the process needs to exit.
///
/// # Example
///
/// ```ignore
/// let exit = ShutdownManager::<()>::builder()
///     .watch_signal_with_default()
///     .watch_tokio(tokio::spawn(async {
///         let mut count = 0;
///         loop {
///             println!("Task 1: {}", count);
///             count += 1;
///             tokio::time::sleep(std::time::Duration::from_secs(1)).await;
///         }
///     }))
///     .watch_tokio(tokio::spawn(async {
///         let mut count = 0;
///         loop {
///             println!("Task 2: {}", count);
///             count += 1;
///             tokio::time::sleep(std::time::Duration::from_secs(3)).await;
///         }
///     }))
///     .build();
/// exit.wait().await.unwrap();
/// ```
///
/// [`tokio::task`s]: tokio::task
pub struct ShutdownManager<T> {
    exit_logger: Option<Logger>,
    grace_timeout: Duration,
    shutdown_notification_sender: watch::Sender<bool>,
    signal_exit_value: Option<Result<T>>,
    tasks: FuturesUnordered<WatchTask<T>>,
}

impl<T> ShutdownManager<T> {
    /// Begin building a [`ShutdownManager`] watching for signals and no [`tokio::task`s].
    ///
    /// [`tokio::task`s]: tokio::task
    pub fn builder() -> ShutdownManagerBuilder<T> {
        let (sender, receiver) = watch::channel(false);
        ShutdownManagerBuilder {
            exit_logger: None,
            grace_duration: Duration::from_secs(DEFAULT_SHUTDOWN_GRACE_TIMEOUT),
            shutdown_notification_receiver: receiver,
            shutdown_notification_sender: sender,
            signal_exit_value: None,
            tasks: Vec::new(),
        }
    }

    /// Asynchronously wait for graceful shutdown signal and handle that process.
    ///
    /// Graceful exit conditions and shutdown sequence are documented in [`ShutdownManager`].
    pub async fn wait(mut self) -> Result<T> {
        // Wait for the first exit condition that triggers.
        let exit_on_tokio_task = ShutdownManager::exit_condition_tokio_task(
            self.tasks.next(),
            self.exit_logger.as_ref(),
        );
        let exit_on_signal = ShutdownManager::exit_condition_signal(
            self.signal_exit_value,
            self.exit_logger.as_ref(),
        );
        let exit = tokio::select! {
            exit = exit_on_tokio_task => exit,
            exit = exit_on_signal => exit,
        };

        // Notify any interested parties about the graceful shutdown.
        let _ = self.shutdown_notification_sender.send(true);
        drop(self.shutdown_notification_sender);

        // Give tasks a chance to complete graceful shutdown.
        // This ends when the first condition below is met:
        // - All tasks have exited.
        // - Further user signals (this causes an abrupt exit and does not return here).
        // - The shutdown timeout has elapsed.
        let await_all_tokio = async {
            while let Some(task) = self.tasks.next().await {
                let logger = match &self.exit_logger {
                    None => continue,
                    Some(logger) => logger,
                };
                match task {
                    // Ignore non-error cases.
                    Ok(Ok(_)) => (),
                    Err(error) if error.is_cancelled() => (),

                    // Log errors to aid debugging and troubleshooting.
                    Err(join_error) => {
                        let is_panic = join_error.is_panic();
                        slog::error!(
                            logger, "Unable to join Tokio task while shutting down";
                            "error" => ?join_error,
                            "is_panic" => is_panic,
                        );
                    }
                    Ok(Err(task_error)) => {
                        slog::error!(
                            logger, "Tokio task returned an error while shutting down";
                            // TODO(anyhow-log-utils): Attach error as structured KV.
                            "error" => %task_error,
                        );
                    }
                }
            }
        };
        let exit_on_more_signals = async {
            let _ = tokio::signal::ctrl_c().await;
            std::process::exit(FORCE_SHUTDOWN_EXIT_CODE);
        };
        let grace_timeout = tokio::time::sleep(self.grace_timeout);
        tokio::select! {
            _ = await_all_tokio => (),
            _ = exit_on_more_signals => (),
            _ = grace_timeout => (),
        };

        // Ensure all tasks that have not completed still are cancelled.
        for task in self.tasks {
            task.abort();
        }

        // Return the value/error that triggered shutdown.
        if let Some(logger) = self.exit_logger {
            slog::info!(logger, "Graceful shutdown completed");
        }
        exit
    }

    /// Watch for exit signals from the OS.
    ///
    /// When an exit signal is received this future will return a value
    /// and trigger a clean shutdown sequence.
    /// The exit signal is platform dependent and defined as [`tokio::signal::ctrl_c`].
    ///
    /// If no exit value is set for the [`ShutdownManager`] instance this future never resolves.
    ///
    /// As exit conditions manipulate different [`ShutdownManager`] fields we decompose the
    /// structure in [`ShutdownManager::wait`] and only take the needed fields for this condition.
    async fn exit_condition_signal(
        exit_value: Option<Result<T>>,
        logger: Option<&Logger>,
    ) -> Result<T> {
        // If signal handling is not desired wait forever.
        if exit_value.is_none() {
            std::future::pending::<()>().await;
        }

        // Wait for the first signal to trigger shutdown.
        let signal = tokio::signal::ctrl_c().await;
        if let Some(logger) = logger {
            slog::info!(logger, "Received exit signal: beginning graceful shutdown");
        }
        if let Err(error) = signal {
            let error = anyhow::anyhow!(error);
            let error = error.context(ShutdownError::SignalError);
            return Err(error);
        }
        exit_value.expect("signal exit value function must be set to get here")
    }

    /// Watch for any tokio tasks to exit.
    ///
    /// This future resolves as soon as any of the registered tokio tasks ends regardless
    /// of success or failure of it.
    ///
    /// If no tokio task is registered this future never resolves.
    ///
    /// As exit conditions manipulate different [`ShutdownManager`] fields we decompose the
    /// structure in [`ShutdownManager::wait`] and only take the needed fields for this condition.
    async fn exit_condition_tokio_task(
        task: impl Future<Output = WatchTaskOutput<T>>,
        logger: Option<&Logger>,
    ) -> Result<T> {
        let task = task.await;
        if task.is_none() {
            std::future::pending::<()>().await;
        }

        let first_exit = task.expect("tokio tasks set must have at least one task to get here");
        if let Some(logger) = logger {
            slog::info!(
                logger,
                "Watched tokio task exited: beginning graceful shutdown",
            );
        }
        match first_exit {
            Err(error) if error.is_cancelled() => {
                Err(anyhow::anyhow!(ShutdownError::TokioTaskCancelled))
            }
            Err(error) if error.is_panic() => {
                let error = anyhow::anyhow!(error);
                Err(error.context(ShutdownError::TokioTaskPanic))
            }
            Err(error) => {
                let error = anyhow::anyhow!(error);
                Err(error.context(ShutdownError::TokioTaskError))
            }
            Ok(result) => result,
        }
    }
}

/// Build [`ShutdownManager`] instances.
pub struct ShutdownManagerBuilder<T> {
    exit_logger: Option<Logger>,
    grace_duration: Duration,
    shutdown_notification_receiver: watch::Receiver<bool>,
    shutdown_notification_sender: watch::Sender<bool>,
    signal_exit_value: Option<Result<T>>,
    tasks: Vec<WatchTask<T>>,
}

impl<T> ShutdownManagerBuilder<T> {
    /// Complete configuration of the [`ShutdownManager`] instance.
    ///
    /// # Panic
    ///
    /// This method panics if no exit condition is watched for.
    /// Make sure to call at least one of:
    ///
    /// * [`ShutdownManagerBuilder::watch_signal`]
    /// * [`ShutdownManagerBuilder::watch_signal_with_default`]
    /// * [`ShutdownManagerBuilder::watch_tokio`]
    pub fn build(self) -> ShutdownManager<T> {
        if self.tasks.is_empty() && self.signal_exit_value.is_none() {
            panic!("ShutdownManager needs at least one exit condition to watch for");
        }

        let tasks = self.tasks.into_iter().collect();
        ShutdownManager {
            exit_logger: self.exit_logger,
            grace_timeout: self.grace_duration,
            shutdown_notification_sender: self.shutdown_notification_sender,
            signal_exit_value: self.signal_exit_value,
            tasks,
        }
    }

    /// Set the maximum amount of time to wait for graceful shutdown to complete.
    pub fn graceful_shutdown_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.grace_duration = timeout;
        self
    }

    /// Set the logger used to inform of shutdown events and issues.
    pub fn logger(&mut self, logger: Logger) -> &mut Self {
        self.exit_logger = Some(logger);
        self
    }

    /// Return a future that resolves to notify graceful shutdown was requested.
    pub fn shutdown_notification(&self) -> impl Future<Output = ()> {
        let mut receiver = self.shutdown_notification_receiver.clone();
        async move {
            // If a signal was sent before we started waiting on it notify immediately.
            if *receiver.borrow() {
                return;
            }

            // Otherwise wait for a change or the sender side closing.
            let _ = receiver.changed().await;
        }
    }

    /// Watch [`tokio::signal::ctrl_c`] for exit, returning the given value.
    pub fn watch_signal(&mut self, exit_value: Result<T>) -> &mut Self {
        self.signal_exit_value = Some(exit_value);
        self
    }

    /// Watch a [`tokio::task::JoinHandle`] for exit.
    pub fn watch_tokio(&mut self, task: JoinHandle<Result<T>>) -> &mut Self {
        self.tasks.push(task);
        self
    }
}

#[cfg(feature = "runtime-shutdown_actix")]
impl<T: Send + 'static> ShutdownManagerBuilder<T> {
    /// Watch [`actix_web::dev::Server`] for exit, returning the given value.
    pub fn watch_actix(&mut self, server: actix_web::dev::Server, value: T) -> &mut Self {
        let notification = self.shutdown_notification();
        self.watch_tokio(tokio::spawn(async {
            let handle = server.handle();
            tokio::select! {
                reason = server => if let Err(error) = reason {
                    let error = anyhow::anyhow!(error).context(ShutdownError::ActixServer);
                    anyhow::bail!(error);
                },
                _ = notification => handle.stop(true).await,
            };
            Ok(value)
        }))
    }
}

impl<T: Default> ShutdownManagerBuilder<T> {
    /// Watch [`tokio::signal::ctrl_c`] for exit, returning the default value of `T`.
    pub fn watch_signal_with_default(&mut self) -> &mut Self {
        self.signal_exit_value = Some(Ok(T::default()));
        self
    }
}
