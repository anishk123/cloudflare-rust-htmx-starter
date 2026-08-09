PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 120),
  body TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','published','archived')),
  version INTEGER NOT NULL DEFAULT 1,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  summary TEXT
);
CREATE INDEX IF NOT EXISTS idx_notes_updated_at ON notes(updated_at_ms DESC);

CREATE TABLE IF NOT EXISTS processed_operations (
  operation_id TEXT PRIMARY KEY,
  entity_id TEXT NOT NULL,
  processed_at_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS processed_jobs (
  job_id TEXT PRIMARY KEY,
  processed_at_ms INTEGER NOT NULL
);
