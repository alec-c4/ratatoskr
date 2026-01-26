CREATE TABLE IF NOT EXISTS sessions (
    did TEXT PRIMARY KEY,
    session_data BLOB NOT NULL,
    updated_at INTEGER NOT NULL
);
