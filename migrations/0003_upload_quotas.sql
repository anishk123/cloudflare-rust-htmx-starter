PRAGMA foreign_keys = ON;

-- Running total of stored attachment bytes per owner. Enforced by the app
-- Worker before each R2 write; monotonic until object deletion exists.
CREATE TABLE IF NOT EXISTS upload_usage (
  owner_id TEXT PRIMARY KEY,
  bytes_used INTEGER NOT NULL DEFAULT 0,
  updated_at_ms INTEGER NOT NULL
);
