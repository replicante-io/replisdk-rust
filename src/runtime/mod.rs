//! Utilities to manage general features and needs of the process lifecycle.
#[cfg(feature = "runtime-shutdown")]
mod shutdown;

#[cfg(feature = "runtime-telemetry")]
pub mod telemetry;

#[cfg(feature = "runtime-shutdown")]
pub use {
    self::shutdown::ShutdownManager, self::shutdown::ShutdownManagerBuilder,
    self::shutdown::DEFAULT_SHUTDOWN_GRACE_TIMEOUT,
};
