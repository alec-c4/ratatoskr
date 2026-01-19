use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

pub struct Storage {
    pool: Pool<Sqlite>,
}

impl Storage {
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

    pub async fn save_message(&self, msg: &crate::models::ChatMessage) -> Result<(), StorageError> {
        let status = serde_json::to_string(&msg.status).unwrap();

        sqlx::query(
            "INSERT INTO messages (id, sender_did, content, timestamp, status) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&msg.id)
        .bind(&msg.sender_did)
        .bind(&msg.content)
        .bind(msg.timestamp as i64)
        .bind(status) // We store status as JSON string or integer, for now string is easier
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_messages(
        &self,
        contact_did: &str,
    ) -> Result<Vec<crate::models::ChatMessage>, StorageError> {
        use sqlx::Row;
        // In a real app we'd need a more complex query to handle all Message fields
        let recs =
            sqlx::query("SELECT * FROM messages WHERE sender_did = ? ORDER BY timestamp ASC")
                .bind(contact_did)
                .fetch_all(&self.pool)
                .await?;

        // Mapping logic (simplified for prototype)
        Ok(recs
            .into_iter()
            .map(|r| {
                crate::models::ChatMessage {
                    id: r.get("id"),
                    sender_did: r.get("sender_did"),
                    recipient_did: "me".to_string(), // Simplified
                    msg_type: crate::models::MessageType::Direct,
                    status: crate::models::MessageStatus::Done,
                    content: r.get("content"),
                    timestamp: r.get::<i64, _>("timestamp") as u64,
                    ttl: None,
                    schema_id: "raw".to_string(),
                }
            })
            .collect())
    }
}
