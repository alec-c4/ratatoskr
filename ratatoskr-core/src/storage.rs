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
}
