//! Utilities and logic for process runtime management.
mod shutdown;

pub use self::shutdown::ShutdownManager;
pub use self::shutdown::ShutdownManagerBuilder;
pub use self::shutdown::DEFAULT_SHUTDOWN_GRACE_TIMEOUT;
