//! Agent SDK metrics related to actions.
use once_cell::sync::Lazy;
use prometheus::Counter;
use prometheus::Histogram;
use prometheus::HistogramOpts;

/// Number of action execution loops where an action was run.
pub static EXECUTE_LOOPS_BUSY: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "repliagent_action_loops_busy",
        "Number of action execution loops where an action was run",
    )
    .expect("failed to initialise EXECUTE_LOOPS_BUSY counter")
});

/// Duration (in seconds) of an action execution loop.
pub static EXECUTE_LOOPS_DURATION: Lazy<Histogram> = Lazy::new(|| {
    Histogram::with_opts(
        HistogramOpts::new(
            "repliagent_action_loops_duration",
            "Duration (in seconds) of an action execution loop",
        )
        .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
    )
    .expect("failed to initialise EXECUTE_LOOPS_DURATION histogram")
});

/// Number of action execution loops that ended in error.
///
/// NOTE: actions can fail with the execution loop completing successfully.
pub static EXECUTE_LOOPS_ERROR: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "repliagent_action_loops_error",
        "Number of action execution loops that ended in error",
    )
    .expect("failed to initialise EXECUTE_LOOPS_BUSY counter")
});

/// Number of actions that ended in the failed state.
pub static FAILED: Lazy<Counter> = Lazy::new(|| {
    Counter::new(
        "repliagent_action_failed",
        "Number of actions that ended in the failed state",
    )
    .expect("failed to initialise FAILED counter")
});
