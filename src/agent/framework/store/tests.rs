//! Tests for the Agent Store module.
use rusqlite::Connection;

use super::fixtures;

#[tokio::test]
async fn initialise() {
    let store = fixtures::store().await;
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
