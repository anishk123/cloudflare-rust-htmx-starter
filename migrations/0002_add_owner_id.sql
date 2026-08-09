PRAGMA foreign_keys = ON;

-- Every note belongs to exactly one owner. Existing rows are marked as
-- unowned ('') and become invisible to every user; reassign or delete them
-- during rollout.
ALTER TABLE notes ADD COLUMN owner_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_notes_owner_updated ON notes(owner_id, updated_at_ms DESC);

-- The idempotency boundary is per-owner: an operation_id from one user must
-- never satisfy a replay from another.
ALTER TABLE processed_operations ADD COLUMN owner_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS idx_processed_ops_owner ON processed_operations(owner_id, operation_id);
