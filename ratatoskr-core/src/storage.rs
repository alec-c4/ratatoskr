use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use thiserror::Error;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("Crypto error: {0}")]
    Crypto(String),
}

pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
    // ... existing init ...
    pub async fn init<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db_url = format!("sqlite://{}", path.as_ref().to_string_lossy());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    // ... contacts methods ...

    pub async fn add_contact(&self, did: &str, alias: &str) -> Result<(), StorageError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO contacts (did, alias, added_at) VALUES (?, ?, ?) 
             ON CONFLICT(did) DO UPDATE SET alias = excluded.alias",
        )
        .bind(did)
        .bind(alias)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_contacts(&self) -> Result<Vec<(String, Option<String>)>, StorageError> {
        use sqlx::Row;
        let recs = sqlx::query("SELECT did, alias FROM contacts WHERE trust_level > 0")
            .fetch_all(&self.pool)
            .await?;

        Ok(recs
            .into_iter()
            .map(|r| (r.get("did"), r.get("alias")))
            .collect())
    }

    // ... messages methods ...

    pub async fn cleanup_expired_messages(&self) -> Result<u64, StorageError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let result = sqlx::query("DELETE FROM messages WHERE ttl IS NOT NULL AND ttl < ?")
            .bind(now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn save_message(&self, msg: &crate::models::ChatMessage) -> Result<(), StorageError> {
        let status = serde_json::to_string(&msg.status).unwrap();
        let msg_type = serde_json::to_string(&msg.msg_type).unwrap();

        sqlx::query(
            "INSERT INTO messages (id, sender_did, recipient_did, content, timestamp, status, msg_type, ttl, reply_to_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&msg.id)
        .bind(&msg.sender_did)
        .bind(&msg.recipient_did)
        .bind(&msg.content)
        .bind(msg.timestamp as i64)
        .bind(status)
        .bind(msg_type)
        .bind(msg.ttl.map(|t| t as i64))
        .bind(&msg.reply_to_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_messages(
        &self,
        contact_did: &str,
    ) -> Result<Vec<crate::models::ChatMessage>, StorageError> {
        use sqlx::Row;

        let recs = sqlx::query(
                "SELECT * FROM messages WHERE sender_did = ? OR recipient_did = ? ORDER BY timestamp ASC"
            )
            .bind(contact_did)
            .bind(contact_did)
            .fetch_all(&self.pool)
            .await?;

        Ok(recs
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let type_str: String = r.get("msg_type");

                crate::models::ChatMessage {
                    id: r.get("id"),
                    sender_did: r.get("sender_did"),
                    recipient_did: r
                        .get::<Option<String>, _>("recipient_did")
                        .unwrap_or("me".to_string()),
                    msg_type: serde_json::from_str(&type_str)
                        .unwrap_or(crate::models::MessageType::Direct),
                    status: serde_json::from_str(&status_str)
                        .unwrap_or(crate::models::MessageStatus::Done),
                    content: r.get("content"),
                    timestamp: r.get::<i64, _>("timestamp") as u64,
                    ttl: r.get::<Option<i64>, _>("ttl").map(|t| t as u64),
                    schema_id: "raw".to_string(),
                    reply_to_id: r.get("reply_to_id"),
                }
            })
            .collect())
    }

    pub async fn update_message_status(
        &self,
        id: &str,
        status: crate::models::MessageStatus,
    ) -> Result<(), StorageError> {
        let status_json = serde_json::to_string(&status).unwrap();
        sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
            .bind(status_json)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_session(
        &self,
        did: &str,
        session: &crate::ratchet::DoubleRatchetSession,
    ) -> Result<(), StorageError> {
        let data = serde_json::to_vec(session).map_err(|e| {
            StorageError::Db(sqlx::Error::Protocol(format!("Serialization error: {}", e)))
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO sessions (did, session_data, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(did) DO UPDATE SET session_data = excluded.session_data, updated_at = excluded.updated_at"
        )
        .bind(did)
        .bind(data)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_session(
        &self,
        did: &str,
    ) -> Result<Option<crate::ratchet::DoubleRatchetSession>, StorageError> {
        use sqlx::Row;
        let rec = sqlx::query("SELECT session_data FROM sessions WHERE did = ?")
            .bind(did)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = rec {
            let data: Vec<u8> = row.get("session_data");
            let session = serde_json::from_slice(&data).map_err(|e| {
                StorageError::Db(sqlx::Error::Protocol(format!(
                    "Deserialization error: {}",
                    e
                )))
            })?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    // --- PreKey Management ---

    pub async fn save_signed_prekey(&self, secret: &StaticSecret) -> Result<(), StorageError> {
        let secret_bytes = secret.to_bytes();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query("INSERT INTO my_signed_prekeys (key_data, created_at) VALUES (?, ?)")
            .bind(&secret_bytes[..])
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_signed_prekey_secret(
        &self,
        pub_key: &PublicKey,
    ) -> Result<Option<StaticSecret>, StorageError> {
        use sqlx::Row;
        // We scan and find the one matching. Efficient enough for few keys.
        let recs = sqlx::query("SELECT key_data FROM my_signed_prekeys")
            .fetch_all(&self.pool)
            .await?;

        for r in recs {
            let data: Vec<u8> = r.get("key_data");
            if data.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&data);
                let secret = StaticSecret::from(arr);
                let pk = PublicKey::from(&secret);
                if pk == *pub_key {
                    return Ok(Some(secret));
                }
            }
        }
        Ok(None)
    }

    pub async fn save_onetime_prekey(&self, secret: &StaticSecret) -> Result<(), StorageError> {
        let secret_bytes = secret.to_bytes();

        sqlx::query("INSERT INTO my_onetime_prekeys (key_data) VALUES (?)")
            .bind(&secret_bytes[..])
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_onetime_prekey_secret(
        &self,
        pub_key: &PublicKey,
    ) -> Result<Option<StaticSecret>, StorageError> {
        use sqlx::Row;
        // Scan for matching OPK
        let recs = sqlx::query("SELECT key_data FROM my_onetime_prekeys WHERE published = 0") // Assume 0 means active/valid?
            // Actually, 'consumed' or 'published' logic is app specific.
            // Let's just find it by content.
            .fetch_all(&self.pool)
            .await?;

        for r in recs {
            let data: Vec<u8> = r.get("key_data");
            if data.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&data);
                let secret = StaticSecret::from(arr);
                let pk = PublicKey::from(&secret);
                if pk == *pub_key {
                    // Mark as consumed? Or do it in a separate transaction?
                    // X3DH says OPK should be deleted after use.
                    // For now, return it. Logic layer deletes it.
                    return Ok(Some(secret));
                }
            }
        }
        Ok(None)
    }

    pub async fn delete_onetime_prekey(&self, pub_key: &PublicKey) -> Result<(), StorageError> {
        // Find ID and delete? Or delete by content blob (slower but works).
        // Since we don't store public key explicitly in DB column (we store secret blob),
        // we strictly speaking can't query by public key easily without re-deriving.
        // But we just did get_secret, so we know the secret bytes.
        // Let's implement delete by secret bytes.

        // This requires the caller to pass the secret they just retrieved.
        // Simpler: iterate, find, delete by ID.
        // Refactoring `get_onetime_prekey_secret` to return (ID, Secret) would be better.
        // But for MVP, let's just assume we delete by derived Public Key matching?
        // Actually, we can fetch all, find matching, get ID, delete by ID.

        use sqlx::Row;
        let recs = sqlx::query("SELECT id, key_data FROM my_onetime_prekeys")
            .fetch_all(&self.pool)
            .await?;

        for r in recs {
            let data: Vec<u8> = r.get("key_data");
            if data.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&data);
                let secret = StaticSecret::from(arr);
                let pk = PublicKey::from(&secret);
                if pk == *pub_key {
                    let id: i64 = r.get("id");
                    sqlx::query("DELETE FROM my_onetime_prekeys WHERE id = ?")
                        .bind(id)
                        .execute(&self.pool)
                        .await?;
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    pub async fn save_bundle(
        &self,
        did: &str,
        bundle: &crate::x3dh::PreKeyBundle,
    ) -> Result<(), StorageError> {
        let data = serde_json::to_vec(bundle).map_err(|e| {
            StorageError::Db(sqlx::Error::Protocol(format!("Serialization error: {}", e)))
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        sqlx::query(
            "INSERT INTO bundles (did, bundle_data, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(did) DO UPDATE SET bundle_data = excluded.bundle_data, updated_at = excluded.updated_at"
        )
        .bind(did)
        .bind(data)
        .bind(now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_bundle(
        &self,
        did: &str,
    ) -> Result<Option<crate::x3dh::PreKeyBundle>, StorageError> {
        use sqlx::Row;
        let rec = sqlx::query("SELECT bundle_data FROM bundles WHERE did = ?")
            .bind(did)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = rec {
            let data: Vec<u8> = row.get("bundle_data");
            let bundle = serde_json::from_slice(&data).map_err(|e| {
                StorageError::Db(sqlx::Error::Protocol(format!(
                    "Deserialization error: {}",
                    e
                )))
            })?;
            Ok(Some(bundle))
        } else {
            Ok(None)
        }
    }
}
