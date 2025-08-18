use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, Row};
use crate::monitor::AppInfo;

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

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(err) => write!(f, "Database error: {}", err),
            StorageError::SessionNotFound => write!(f, "Session not found"),
        }
    }
    
}

impl std::error::Error for StorageError {}

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

    pub async fn start_session(&mut self, app_info: &AppInfo) -> Result<i64, StorageError> {
        if self.current_session_id.is_some() {
            self.end_session().await?;
        }

        let result = sqlx::query(
            r#"
            INSERT INTO activity_sessions (app_name, bundle_id, window_title, process_id, start_time)
            VALUES (?, ?, ?, ?, ?)
            "#
            )
            .bind(&app_info.app_name)
            .bind(&app_info.bundle_id)
            .bind(&app_info.window_title)
            .bind(&app_info.process_id)
            .bind(Utc::now().to_rfc3339())
            .execute(&self.pool)
            .await?;

        let session_id = result.last_insert_rowid();
        self.current_session_id = Some(session_id);

        Ok(session_id)
    }

    pub async fn end_session(&mut self) -> Result<(), StorageError> {
        let session_id = self.current_session_id.ok_or(StorageError::SessionNotFound)?;

        let now = Utc::now();

        let row = sqlx::query("SELECT start_time FROM activity_sessions WHERE id = ?")
          .bind(session_id)
          .fetch_one(&self.pool)
          .await?; 
        let start_time_str: String = row.get("start_time");
        let start_time = DateTime::parse_from_rfc3339(&start_time_str)
          .map_err(|e| StorageError::DatabaseError(sqlx::Error::Decode(Box::new(e))))?
          .with_timezone(&Utc);

        let duration_seconds = (now - start_time).num_seconds();

        sqlx::query(
          r#"
          UPDATE activity_sessions
          SET end_time = ?, duration_seconds = ?
          WHERE id = ?
          "#
        )
        .bind(now.to_rfc3339())
        .bind(duration_seconds)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        self.current_session_id = None;

        Ok(())
        
    }

}
