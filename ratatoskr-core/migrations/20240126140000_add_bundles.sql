CREATE TABLE IF NOT EXISTS bundles (
    did TEXT PRIMARY KEY,
    bundle_data BLOB NOT NULL,
    updated_at INTEGER NOT NULL
);
