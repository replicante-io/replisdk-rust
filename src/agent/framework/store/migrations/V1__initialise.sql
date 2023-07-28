-- Store action records to persist the current state of actions.
-- Based on ActionRecord from src/agent/models.rs
CREATE TABLE IF NOT EXISTS actions(
  id TEXT PRIMARY KEY NOT NULL,
  args TEXT NOT NULL,
  created_time INTEGER NOT NULL,
  finished_time INTEGER DEFAULT NULL,
  kind TEXT NOT NULL,
  metadata TEXT NOT NULL,
  scheduled_time INTEGER NOT NULL,
  state_error TEXT DEFAULT NULL,
  state_payload TEXT DEFAULT NULL,
  state_phase TEXT NOT NULL
);
CREATE INDEX actions_queue ON actions(scheduled_time, finished_time);
CREATE INDEX actions_ttl ON actions(finished_time);
