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
const ACTION_PERSIST_SQL: &str = r#"
    INSERT INTO actions (
        args, created_time, finished_time, id, kind, metadata, 
        scheduled_time, state_error, state_payload, state_phase
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
                    let args: String = row.get("args")?;
                    let created_time: String = row.get("created_time")?;
                    let finished_time: Option<String> = row.get("finished_time")?;
                    let id: String = row.get("id")?;
                    let kind: String = row.get("kind")?;
                    let metadata: String = row.get("metadata")?;
                    let scheduled_time: String = row.get("scheduled_time")?;
                    let state_error: Option<String> = row.get("state_error")?;
                    let state_payload: Option<String> = row.get("state_payload")?;
                    let state_phase: String = row.get("state_phase")?;
                    Some((
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
                    ))
                }
            };
            Ok(row)
        })
        .await
        .context(StatementError::QueryFailed)?;

    // Return if there was action for the given ID.
    let row = match row {
        None => return Ok(None),
        Some(row) => row,
    };

    // Decode the store record into an ActionExecution model.
    let args = encoding::decode_serde(&row.0)?;
    let created_time = encoding::decode_time(&row.1)?;
    let finished_time = encoding::decode_time_option(&row.2)?;
    let id = uuid::Uuid::parse_str(&row.3)?;
    let metadata = encoding::decode_serde(&row.5)?;
    let scheduled_time = encoding::decode_time(&row.6)?;
    let state_error = encoding::decode_serde_option(&row.7)?;
    let state_payload = encoding::decode_serde_option(&row.8)?;
    let state_phase = encoding::decode_serde(&row.9)?;
    let action = ActionExecution {
        args,
        created_time,
        finished_time,
        id,
        kind: row.4,
        metadata,
        scheduled_time,
        state: ActionExecutionState {
            error: state_error,
            payload: state_payload,
            phase: state_phase,
        },
    };
    Ok(Some(action))
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

/// Insert or update an [`ActionExecution`] record.
pub async fn persist(store: &Connection, action: ActionExecution) -> Result<()> {
    // Serialise special types into stings for the DB.
    let args = encoding::encode_serde(&action.args)?;
    let created_time = encoding::encode_time(action.created_time)?;
    let finished_time = encoding::encode_time_option(action.finished_time)?;
    let metadata = encoding::encode_serde(&action.metadata)?;
    let scheduled_time = encoding::encode_time(action.scheduled_time)?;
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

    //#[tokio::test]
    //async fn get_action_not_found() {
    //}

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
