CREATE TABLE IF NOT EXISTS contacts (
    did TEXT PRIMARY KEY NOT NULL,
    alias TEXT,
    trust_level INTEGER NOT NULL DEFAULT 1, -- 0: Blocked, 1: Stranger, 2: Contact, 3: Verified
    added_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    sender_did TEXT NOT NULL,
    content BLOB NOT NULL, -- Encrypted
    timestamp INTEGER NOT NULL,
    status INTEGER NOT NULL DEFAULT 0 -- 0: Unread, 1: Read
);
