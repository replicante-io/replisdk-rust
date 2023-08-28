//! Collection of various utilities and code for common tasks.
#[cfg(any(feature = "utils-actix_error", feature = "utils-actix_metrics"))]
pub mod actix;
#[cfg(any(feature = "utils-error_json", feature = "utils-error_slog"))]
pub mod error;
#[cfg(feature = "utils-metrics")]
pub mod metrics;
#[cfg(feature = "utils-trace")]
pub mod trace;

/// Special marker used by anyhow to indicate the backtrace is not available.
#[cfg(any(
    feature = "utils-actix_error",
    feature = "utils-error_json",
    feature = "utils-error_slog",
))]
const BACKTRACE_DISABLED: &str = "disabled backtrace";
