CREATE TABLE IF NOT EXISTS my_signed_prekeys (
    id INTEGER PRIMARY KEY,
    key_data BLOB NOT NULL, -- Serialized KeyPair or Secret
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS my_onetime_prekeys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_data BLOB NOT NULL,
    published INTEGER DEFAULT 0 -- 0=No, 1=Yes
);
