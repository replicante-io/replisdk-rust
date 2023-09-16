//! Crate [`Runtime`] instances from configuration.
use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;
use tokio::runtime::Builder;
use tokio::runtime::Runtime;

/// Configuration of the tokio runtime for the process.
///
/// These options configure the handling of synchronous and asynchronous tasks.
/// These are low level code execution patters and you should be familiar with the concept of
/// asynchronous programming before making changes.
///
/// For an introduction to async in Rust you can refer to
/// <https://rust-lang.github.io/async-book/01_getting_started/02_why_async.html>.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TokioRuntimeConf {
    /// Maximum number of threads processing blocking tasks.
    ///
    /// Blocking tasks take over a thread until they complete, even during wait times such as IO.
    /// To prevent blocking tasks from preventing non-blocking tasks from executing they get
    /// a dedicated pool of threads to execute on.
    ///
    /// This option sets the maximum number of threads that can run blocking tasks.
    /// If all threads are busy, new blocking tasks will be queued until threads are available.
    #[serde(default)]
    pub sync_workers: Option<usize>,

    /// Time in second to keep blocking task threads alive waiting for more tasks.
    #[serde(default)]
    pub sync_workers_keep_alive: Option<u64>,

    /// Number of threads processing non-blocking tasks.
    ///
    /// As tasks keep a thread busy only when they can progress a small number of threads
    /// can handle a large number of non-block tasks.
    #[serde(default)]
    pub workers: Option<usize>,
}

impl TokioRuntimeConf {
    /// Create a [`Runtime`] with the given configuration.
    pub fn into_runtime(self) -> Result<Runtime, std::io::Error> {
        Runtime::try_from(self)
    }
}

impl TryFrom<TokioRuntimeConf> for Runtime {
    type Error = std::io::Error;

    fn try_from(conf: TokioRuntimeConf) -> Result<Self, Self::Error> {
        let mut runtime = Builder::new_multi_thread();
        if let Some(sync_workers) = conf.sync_workers {
            runtime.max_blocking_threads(sync_workers);
        }
        if let Some(keep_alive) = conf.sync_workers_keep_alive {
            let keep_alive = std::time::Duration::from_secs(keep_alive);
            runtime.thread_keep_alive(keep_alive);
        }
        if let Some(workers) = conf.workers {
            runtime.worker_threads(workers);
        }
        runtime.enable_all().build()
    }
}
