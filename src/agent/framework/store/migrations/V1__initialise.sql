-- Store action records to persist the current state of actions.
-- Based on ActionRecord from src/agent/models.rs
CREATE TABLE IF NOT EXISTS actions(
  id TEXT PRIMARY KEY NOT NULL
  -- TODO: Define table once model exists
);
--CREATE INDEX actions_created_ts ON actions(created_ts);
--CREATE INDEX actions_finished_ts ON actions(finished_ts);
--CREATE INDEX actions_state ON actions(state);
