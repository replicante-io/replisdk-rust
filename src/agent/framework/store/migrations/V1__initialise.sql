-- Store action records to persist the current state of actions.
-- Based on ActionRecord from src/agent/models.rs
CREATE TABLE IF NOT EXISTS actions(
  id TEXT PRIMARY KEY NOT NULL,
  args TEXT NOT NULL,
  -- Created time is not queried on so use TEXT to preserve precision.
  created_time TEXT NOT NULL,
  -- Finished time is sorted and queried on so use REAL for SQLite to operate on it correctly.
  finished_time REAL DEFAULT NULL,
  kind TEXT NOT NULL,
  metadata TEXT NOT NULL,
  -- Scheduled time is queried on so use REAL for SQLite to operate on it correctly.
  scheduled_time REAL NOT NULL,
  state_error TEXT DEFAULT NULL,
  state_payload TEXT DEFAULT NULL,
  state_phase TEXT NOT NULL
);
CREATE INDEX actions_queue ON actions(scheduled_time, finished_time);
CREATE INDEX actions_ttl ON actions(finished_time);
