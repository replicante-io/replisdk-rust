//! Implementation of the store interface using SQLite.
pub mod actions;

/// Errors while executing SQLite statements.
#[derive(Debug, thiserror::Error)]
pub enum StatementError {
    /// Error while querying data from the store.
    #[error("error while querying data from the store")]
    QueryFailed,
}
