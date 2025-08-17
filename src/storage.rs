use chrono::{DateTime, Utc};
use sqlx::{SqlitePool};

#[derive(Debug, Clone)]
pub struct ActivitySession {
    pub id: Option<i64>,
    pub application_name: String,
    pub bundle_id: String,
    pub window_title:String,
    pub process_id: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>, 
}

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(sqlx::Error),
    SessionNotFound,
}

impl From<sqlx::Error> for StorageError {
    fn from(err : sqlx::Error) -> Self {
        StorageError::DatabaseError(err)
    }
}

pub struct ActivityLogger {
    pool: SqlitePool,
    current_session_id: Option<i64>
}

impl ActivityLogger {
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = SqlitePool::connect(database_url).await?;
        Self::create_tables(&pool).await?;

        Ok(Self { pool, current_session_id: None })
    }

    async fn create_tables(pool: &SqlitePool) -> Result<(), StorageError> {
        sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activity_sessions (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              app_name TEXT NOT NULL,
              bundle_id TEXT NOT NULL,
              window_title TEXT,
              process_id INTEGER,
              start_time TEXT NOT NULL,
              end_time TEXT,
              duration_seconds INTEGER,
              created_at TEXT DEFAULT CURRENT_TIMESTAMP
          )"#
        ).execute(pool).await?;
        Ok(())
    }
}
