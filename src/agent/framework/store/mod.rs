//! The agent [`Store`] provides access to persistent information needed to function.
//!
//! Querying and updating the [`Store`] is performed using operation objects
//! which allow the generic [`Store::query`] and [`Store::persist`] methods to perform
//! specialised operations while preserving strict typing.
use anyhow::Result;
use slog::Logger;
use tokio_rusqlite::Connection;

mod encoding;
mod schema;
mod statements;

pub mod persist;
pub mod query;

#[cfg(test)]
pub(super) mod fixtures;
#[cfg(test)]
mod tests;

use self::persist::PersistOp;
use self::persist::PersistOps;
use self::persist::PersistResponses;
use self::query::QueryOp;
use self::query::QueryOps;
use self::query::QueryResponses;
use crate::agent::framework::DefaultContext;

/// Special path requesting the use of an in-memory store.
pub const MEMORY_PATH: &str = ":memory:";

/// Manage persisted data needed for Agent operations.
#[derive(Clone, Debug)]
pub struct Store {
    store: Connection,
}

impl Store {
    /// Close the connection to the store and flush all pending updates.
    pub async fn close(&self) -> Result<()> {
        self.store.clone().close().await?;
        Ok(())
    }

    /// Initialise the Agent store, including any needed schema migrations.
    ///
    /// The special [`MEMORY_PATH`] constant can be specified to create an in-memory store.
    ///
    /// NOTE:
    ///   The use of an in-memory store is only intended for tests and experimentation
    ///   as all data will be lost as soon as the process terminates.
    pub async fn initialise(logger: &Logger, path: &str) -> Result<Store> {
        // Open or create the SQLite DB.
        let store = if path == MEMORY_PATH {
            slog::warn!(
                logger,
                "Using in-memory store means data will be lost once the process terminates"
            );
            Connection::open_in_memory().await
        } else {
            Connection::open(path).await
        };
        let store = store?;

        // Run schema migrations if needed.
        store
            .call(|connection| {
                self::schema::migrations::runner()
                    .run(connection)
                    .map_err(|error| {
                        let error = Box::new(error);
                        tokio_rusqlite::Error::Other(error)
                    })
            })
            .await?;

        Ok(Store { store })
    }

    /// Query records from the agent store.
    ///
    /// The supported query operations are defined in the [`query`] module and
    /// determine the return type.
    pub async fn query<O>(&self, _context: &DefaultContext, op: O) -> Result<O::Response>
    where
        O: QueryOp,
    {
        let op = op.into();
        let response = match op {
            QueryOps::Action(id) => statements::actions::get(&self.store, id)
                .await
                .map(QueryResponses::Action),
            QueryOps::ActionsFinished => statements::actions::finished(&self.store)
                .await
                .map(QueryResponses::ActionsList),
            QueryOps::ActionsQueue => statements::actions::queue(&self.store)
                .await
                .map(QueryResponses::ActionsList),
        };
        response.map(O::Response::from)
    }

    /// Persist records to the agent store.
    ///
    /// The supported persist operations are defined in the [`persist`] module and
    /// determine the return type.
    pub async fn persist<O>(&self, _context: &DefaultContext, op: O) -> Result<O::Response>
    where
        O: PersistOp,
    {
        let op = op.into();
        let response = match op {
            PersistOps::ActionExecution(action) => {
                statements::actions::persist(&self.store, action)
                    .await
                    .map(|_| PersistResponses::Success)
            }
        };
        response.map(O::Response::from)
    }
}
