//! Tests for the Agent Store module.
use anyhow::Result;
use rusqlite::Connection;

use super::Store;
use crate::agent::framework::DefaultContext;

/// Create an in-memory store for tests to use.
async fn store_factory() -> Result<Store> {
    let context = DefaultContext::fixture();
    let path = ":memory:";
    Store::initialise(&context.logger, path).await
}

#[tokio::test]
async fn initialise() {
    let store = store_factory().await.expect("store to be initialised");
    let migrations = store
        .store
        .call(fetch_migration_version)
        .await
        .expect("unable to detect migrations count");
    store.close().await.unwrap();
    assert!(migrations >= 1);
}

fn fetch_migration_version(connection: &mut Connection) -> tokio_rusqlite::Result<i32> {
    let mut statement = connection.prepare("SELECT COUNT(*) FROM refinery_schema_history;")?;
    let count = statement.query_row([], |row| row.get(0))?;
    Ok(count)
}
