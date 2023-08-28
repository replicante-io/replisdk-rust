//! Agent SDK metrics related to the agent store.
use once_cell::sync::Lazy;
use prometheus::Counter;
use prometheus::CounterVec;
use prometheus::HistogramOpts;
use prometheus::HistogramTimer;
use prometheus::HistogramVec;
use prometheus::Opts;

/// Duration (in seconds) of an agent store operation.
pub static OPS_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        HistogramOpts::new(
            "repliagent_store_ops_duration",
            "Duration (in seconds) of an action execution loop",
        )
        .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["op"],
    )
    .expect("failed to initialise OPS_DURATION histogram")
});

/// Number of agent store operations that resulted in error.
pub static OPS_ERR: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new(
            "repliagent_store_ops_error",
            "Number of agent store operations that resulted in error",
        ),
        &["op"],
    )
    .expect("failed to initialise OPS_ERR counter")
});

/// Observe the execution of an agent store operation.
///
/// ## Returns
///
/// - A started timer to observe the duration of the operation.
/// - A [`Counter`] to increment in case of error.
#[inline]
pub fn observe_op(op: &str) -> (Counter, HistogramTimer) {
    let err_count = OPS_ERR.with_label_values(&[op]);
    let timer = OPS_DURATION.with_label_values(&[op]).start_timer();
    (err_count, timer)
}
