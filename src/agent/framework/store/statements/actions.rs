//! Implementation of the actions portion of the store interface.
use anyhow::Context;
use anyhow::Result;
use tokio_rusqlite::Connection;

use super::StatementError;
use crate::agent::framework::store::encoding;
use crate::agent::models::ActionExecution;
use crate::agent::models::ActionExecutionList;
use crate::agent::models::ActionExecutionListItem;
use crate::agent::models::ActionExecutionState;

const ACTION_GET_SQL: &str = r#"
    SELECT
        args,
        created_time,
        finished_time,
        id,
        kind,
        metadata,
        scheduled_time,
        state_error,
        state_payload,
        state_phase
    FROM actions
    WHERE id=?1;
"#;
const ACTION_NEXT_SQL: &str = r#"
    SELECT
        args,
        created_time,
        finished_time,
        id,
        kind,
        metadata,
        scheduled_time,
        state_error,
        state_payload,
        state_phase,
        CASE state_phase
            WHEN '"RUNNING"' THEN 0
            WHEN '"NEW"' THEN 1
            ELSE 2
        END AS phase_priority
    FROM actions
    WHERE finished_time IS NULL
    ORDER BY phase_priority ASC, scheduled_time ASC, ROWID ASC
    LIMIT 1;
"#;
const ACTION_PERSIST_SQL: &str = r#"
    INSERT INTO actions (
        args,
        created_time,
        finished_time,
        id,
        kind,
        metadata, 
        scheduled_time,
        state_error,
        state_payload,
        state_phase
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
    ON CONFLICT(id)
    DO UPDATE SET
        args=?1,
        created_time=?2,
        finished_time=?3,
        metadata=?6,
        scheduled_time=?7,
        state_error=?8,
        state_payload=?9,
        state_phase=?10
    ;
"#;
const ACTIONS_CLEAN_FINISHED_SQL: &str = r#"
    DELETE FROM actions
    WHERE finished_time IS NOT NULL
        AND finished_time <= ?1;
"#;
const ACTIONS_FINISHED_SQL: &str = r#"
    SELECT kind, id, state_phase
    FROM actions
    WHERE finished_time IS NOT NULL
    ORDER BY scheduled_time ASC, ROWID ASC
    -- Limit results to reduce blast radius in case of bugs.
    -- There really should not be many running/pending actions on an agent.
    LIMIT 50;
"#;
const ACTIONS_QUEUE_SQL: &str = r#"
    SELECT kind, id, state_phase
    FROM actions
    WHERE finished_time IS NULL
    ORDER BY scheduled_time ASC, ROWID ASC
    -- Limit results to reduce blast radius in case of bugs.
    -- There really should not be many running/pending actions on an agent.
    LIMIT 50;
"#;

/// [`ActionExecution`] row partially decoded from SQLite.
struct ActionRow {
    args: String,
    created_time: String,
    finished_time: Option<f64>,
    id: String,
    kind: String,
    metadata: String,
    scheduled_time: f64,
    state_error: Option<String>,
    state_payload: Option<String>,
    state_phase: String,
}

impl<'a> TryFrom<&rusqlite::Row<'a>> for ActionRow {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row<'a>) -> std::result::Result<Self, Self::Error> {
        let args: String = row.get("args")?;
        let created_time: String = row.get("created_time")?;
        let finished_time: Option<f64> = row.get("finished_time")?;
        let id: String = row.get("id")?;
        let kind: String = row.get("kind")?;
        let metadata: String = row.get("metadata")?;
        let scheduled_time: f64 = row.get("scheduled_time")?;
        let state_error: Option<String> = row.get("state_error")?;
        let state_payload: Option<String> = row.get("state_payload")?;
        let state_phase: String = row.get("state_phase")?;
        Ok(Self {
            args,
            created_time,
            finished_time,
            id,
            kind,
            metadata,
            scheduled_time,
            state_error,
            state_payload,
            state_phase,
        })
    }
}

impl TryFrom<ActionRow> for ActionExecution {
    type Error = anyhow::Error;
    fn try_from(row: ActionRow) -> std::result::Result<Self, Self::Error> {
        let args = encoding::decode_serde(&row.args)?;
        let created_time = encoding::decode_time(&row.created_time)?;
        let finished_time = encoding::decode_time_option_f64(row.finished_time)?;
        let id = uuid::Uuid::parse_str(&row.id)?;
        let metadata = encoding::decode_serde(&row.metadata)?;
        let scheduled_time = encoding::decode_time_f64(row.scheduled_time)?;
        let state_error = encoding::decode_serde_option(&row.state_error)?;
        let state_payload = encoding::decode_serde_option(&row.state_payload)?;
        let state_phase = encoding::decode_serde(&row.state_phase)?;
        let action = ActionExecution {
            args,
            created_time,
            finished_time,
            id,
            kind: row.kind,
            metadata,
            scheduled_time,
            state: ActionExecutionState {
                error: state_error,
                payload: state_payload,
                phase: state_phase,
            },
        };
        Ok(action)
    }
}

/// Clean [`ActionExecution`] records for actions finished prior to to the given time.
pub async fn clean(store: &Connection, age: time::OffsetDateTime) -> Result<()> {
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    let age = encoding::encode_time_f64(age)?;
    store
        .call(move |connection| {
            let removed = connection.execute(ACTIONS_CLEAN_FINISHED_SQL, rusqlite::params![age])?;
            Ok(removed)
        })
        .await?;
    Ok(())
}

/// List [`ActionExecution`] summaries for finished actions.
pub async fn finished(store: &Connection) -> Result<ActionExecutionList> {
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    let rows = store
        .call(|connection| {
            let mut statement = connection.prepare_cached(ACTIONS_FINISHED_SQL)?;
            let mut rows = statement.query([])?;
            let mut queue = Vec::new();
            while let Some(row) = rows.next()? {
                let kind: String = row.get("kind")?;
                let id: String = row.get("id")?;
                let phase: String = row.get("state_phase")?;
                queue.push((kind, id, phase));
            }
            Ok(queue)
        })
        .await
        .context(StatementError::QueryFailed)?;

    let mut actions = Vec::new();
    for (kind, id, phase) in rows {
        let id = uuid::Uuid::parse_str(&id)?;
        let phase = encoding::decode_serde(&phase)?;
        actions.push(ActionExecutionListItem { kind, id, phase });
    }
    Ok(ActionExecutionList { actions })
}

/// Lookup an [`ActionExecution`] record by ID from the store.
pub async fn get(store: &Connection, id: uuid::Uuid) -> Result<Option<ActionExecution>> {
    // Query the store for an action record.
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    let row = store
        .call(move |connection| {
            let mut statement = connection.prepare_cached(ACTION_GET_SQL)?;
            let mut rows = statement.query([id.to_string()])?;
            let row = match rows.next()? {
                None => None,
                Some(row) => {
                    let row = ActionRow::try_from(row)?;
                    Some(row)
                }
            };
            Ok(row)
        })
        .await
        .context(StatementError::QueryFailed)?;

    // Decode the row into an action.
    match row {
        None => Ok(None),
        Some(row) => {
            let action = ActionExecution::try_from(row)?;
            Ok(Some(action))
        }
    }
}

/// List [`ActionExecution`] summaries for unfinished actions.
pub async fn queue(store: &Connection) -> Result<ActionExecutionList> {
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    let rows = store
        .call(|connection| {
            let mut statement = connection.prepare_cached(ACTIONS_QUEUE_SQL)?;
            let mut rows = statement.query([])?;
            let mut queue = Vec::new();
            while let Some(row) = rows.next()? {
                let kind: String = row.get("kind")?;
                let id: String = row.get("id")?;
                let phase: String = row.get("state_phase")?;
                queue.push((kind, id, phase));
            }
            Ok(queue)
        })
        .await
        .context(StatementError::QueryFailed)?;

    let mut actions = Vec::new();
    for (kind, id, phase) in rows {
        let id = uuid::Uuid::parse_str(&id)?;
        let phase = encoding::decode_serde(&phase)?;
        actions.push(ActionExecutionListItem { kind, id, phase });
    }
    Ok(ActionExecutionList { actions })
}

/// Check the next action to execute, if any is pending.
pub async fn next_to_execute(store: &Connection) -> Result<Option<ActionExecution>> {
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    let row = store
        .call(|connection| {
            let mut statement = connection.prepare_cached(ACTION_NEXT_SQL)?;
            let mut rows = statement.query([])?;
            match rows.next()? {
                None => Ok(None),
                Some(row) => {
                    let row = ActionRow::try_from(row)?;
                    Ok(Some(row))
                }
            }
        })
        .await
        .context(StatementError::QueryFailed)?;

    // Decode the row into an action.
    match row {
        None => Ok(None),
        Some(row) => {
            let action = ActionExecution::try_from(row)?;
            Ok(Some(action))
        }
    }
}

/// Insert or update an [`ActionExecution`] record.
pub async fn persist(store: &Connection, action: ActionExecution) -> Result<()> {
    // Serialise special types into stings for the DB.
    let args = encoding::encode_serde(&action.args)?;
    let created_time = encoding::encode_time(action.created_time)?;
    let finished_time = encoding::encode_time_option_f64(action.finished_time)?;
    let metadata = encoding::encode_serde(&action.metadata)?;
    let scheduled_time = encoding::encode_time_f64(action.scheduled_time)?;
    let state_error = encoding::encode_serde_option(&action.state.error)?;
    let state_payload = encoding::encode_serde_option(&action.state.payload)?;
    let state_phase = encoding::encode_serde(&action.state.phase)?;

    // Execute the insert statement.
    // TODO(tracing): trace DB call.
    // TODO(metrics): add DB call metrics.
    store
        .call(move |connection| {
            connection.execute(
                ACTION_PERSIST_SQL,
                rusqlite::params![
                    args,
                    created_time,
                    finished_time,
                    action.id.to_string(),
                    action.kind,
                    metadata,
                    scheduled_time,
                    state_error,
                    state_payload,
                    state_phase,
                ],
            )?;
            Ok(())
        })
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::agent::framework::store::fixtures;
    use crate::agent::models::ActionExecutionPhase;

    const ACTION_UUID_1: uuid::Uuid = uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8");
    const ACTION_UUID_2: uuid::Uuid = uuid::uuid!("cb4995fc-c62d-41ca-9e66-156f357e2df1");
    const ACTION_UUID_3: uuid::Uuid = uuid::uuid!("156dd85c-afd9-4135-afcd-9003d351e9c9");

    #[tokio::test]
    async fn get_action() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;
        let action = fixtures::action(ACTION_UUID_1);
        store.persist(&context, action.clone()).await.unwrap();

        let id = action.id;
        let query = crate::agent::framework::store::query::Action { id };
        let actual = store.query(&context, query).await.unwrap();
        assert_eq!(Some(action), actual);
    }

    #[tokio::test]
    async fn get_action_not_found() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;
        let id = ACTION_UUID_1;
        let query = crate::agent::framework::store::query::Action { id };
        let actual = store.query(&context, query).await.unwrap();
        assert_eq!(None, actual);
    }

    #[tokio::test]
    async fn query_actions_queue() {
        // Store actions to build a queue.
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;

        let action = fixtures::action(ACTION_UUID_1);
        store.persist(&context, action).await.unwrap();

        let mut action = fixtures::action(ACTION_UUID_2);
        action.state.phase = ActionExecutionPhase::Running;
        action.scheduled_time = time::OffsetDateTime::parse(
            "2023-04-05T05:00:08Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        store.persist(&context, action).await.unwrap();

        let mut action = fixtures::action(ACTION_UUID_3);
        action.finished_time = Some(action.created_time);
        action.state.phase = ActionExecutionPhase::Done;
        store.persist(&context, action).await.unwrap();

        // Query the actions queue.
        let query = super::super::super::query::ActionsQueue {};
        let queue = store.query(&context, query).await.unwrap();
        let actions = queue.actions;
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].id, ACTION_UUID_2);
        assert_eq!(actions[1].id, ACTION_UUID_1);
    }

    #[tokio::test]
    async fn next_action_new() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;

        let action = fixtures::action(ACTION_UUID_1);
        store.persist(&context, action).await.unwrap();
        let action = fixtures::action(ACTION_UUID_2);
        store.persist(&context, action).await.unwrap();
        let action = fixtures::action(ACTION_UUID_3);
        store.persist(&context, action).await.unwrap();

        let query = super::super::super::query::ActionNextToExecute {};
        let next = store.query(&context, query).await.unwrap().unwrap();
        assert_eq!(next.id, ACTION_UUID_1);
    }

    #[tokio::test]
    async fn next_action_none() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;

        let query = super::super::super::query::ActionNextToExecute {};
        let next = store.query(&context, query).await.unwrap();
        assert_eq!(next, None);
    }

    #[tokio::test]
    async fn next_action_running() {
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;

        let action = fixtures::action(ACTION_UUID_1);
        store.persist(&context, action).await.unwrap();
        let mut action = fixtures::action(ACTION_UUID_2);
        action.state.phase = ActionExecutionPhase::Running;
        action.scheduled_time = time::OffsetDateTime::parse(
            "2023-04-06T06:07:08Z",
            &time::format_description::well_known::Rfc3339,
        )
        .unwrap();
        store.persist(&context, action).await.unwrap();
        let action = fixtures::action(ACTION_UUID_3);
        store.persist(&context, action).await.unwrap();

        let query = super::super::super::query::ActionNextToExecute {};
        let next = store.query(&context, query).await.unwrap().unwrap();
        assert_eq!(next.id, ACTION_UUID_2);
    }

    #[tokio::test]
    async fn persist_action_execution() {
        // Store an action.
        let action = fixtures::action(ACTION_UUID_1);
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;
        store.persist(&context, action).await.unwrap();

        // Check it was stored.
        let count: i32 = store
            .store
            .call(|connection| {
                let mut statement = connection.prepare("SELECT COUNT(*) FROM actions;")?;
                let count = statement.query_row([], |row| row.get(0))?;
                Ok(count)
            })
            .await
            .expect("could not count actions");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn persist_action_execution_update_existing() {
        // Store an action.
        let action = fixtures::action(ACTION_UUID_1);
        let context = crate::agent::framework::DefaultContext::fixture();
        let store = fixtures::store().await;
        store.persist(&context, action.clone()).await.unwrap();

        // Update the action.
        let mut action = action;
        action
            .metadata
            .insert("test".to_string(), "value".to_string());
        action.state.phase = ActionExecutionPhase::Running;
        store.persist(&context, action).await.unwrap();

        // Check it was stored.
        let (metadata, phase) = store
            .store
            .call(|connection| {
                let mut statement =
                    connection.prepare("SELECT metadata, state_phase FROM actions WHERE id=?1;")?;
                let record = statement.query_row([ACTION_UUID_1.to_string()], |row| {
                    let metadata: String = row.get(0)?;
                    let phase: String = row.get(1)?;
                    Ok((metadata, phase))
                })?;
                Ok(record)
            })
            .await
            .expect("could not query action");
        assert_eq!(metadata, r#"{"test":"value"}"#);
        assert_eq!(phase, r#""RUNNING""#);
    }
}
