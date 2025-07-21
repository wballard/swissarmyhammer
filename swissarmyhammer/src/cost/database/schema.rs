//! Database schema implementation for cost analytics

use crate::cost::database::config::{DatabaseConfig, DatabaseConfigError};
use crate::cost::{ApiCall, CostSession, CostSessionId};
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{Sqlite, Transaction};
use std::str::FromStr;
use thiserror::Error;
use tracing::{error, info, warn};

/// Database errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// Database connection error
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    /// Database migration error
    #[error("Database migration error: {message}")]
    Migration { message: String },

    /// Invalid data error
    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] DatabaseConfigError),

    /// Transaction error
    #[error("Transaction error: {message}")]
    Transaction { message: String },
}

/// Cost analytics database
///
/// Provides optional SQLite storage for cost sessions and API calls to enable
/// advanced analytics and historical reporting. The database is completely optional
/// and failures are handled gracefully without affecting core functionality.
///
/// # Examples
///
/// ```no_run
/// use swissarmyhammer::cost::database::{CostDatabase, DatabaseConfig};
/// 
/// # tokio_test::block_on(async {
/// let config = DatabaseConfig::default();
/// let database = CostDatabase::new(config).await?;
/// 
/// // Check if database is available
/// if database.is_available() {
///     println!("Database storage enabled");
/// } else {
///     println!("Database storage disabled");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub struct CostDatabase {
    /// Database connection pool (None if disabled or failed to initialize)
    pool: Option<SqlitePool>,
    /// Database configuration
    config: DatabaseConfig,
    /// Whether database initialization was attempted
    initialization_attempted: bool,
}

impl CostDatabase {
    /// Create a new cost database instance
    ///
    /// If database storage is disabled in configuration, the database will be
    /// created but connection pool will be None. Database failures are logged
    /// but do not cause errors - the instance is created in a disabled state.
    ///
    /// # Arguments
    ///
    /// * `config` - Database configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swissarmyhammer::cost::database::{CostDatabase, DatabaseConfig};
    ///
    /// # tokio_test::block_on(async {
    /// let config = DatabaseConfig {
    ///     enabled: true,
    ///     file_path: "./costs.db".into(),
    ///     ..DatabaseConfig::default()
    /// };
    /// 
    /// let database = CostDatabase::new(config).await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        // Validate configuration
        config.validate()?;

        let mut database = Self {
            pool: None,
            config: config.clone(),
            initialization_attempted: false,
        };

        // Only attempt initialization if enabled
        if config.is_enabled() {
            database.initialize().await;
        }

        Ok(database)
    }

    /// Initialize database connection and schema
    async fn initialize(&mut self) {
        self.initialization_attempted = true;

        match self.setup_database().await {
            Ok(pool) => {
                info!("Database initialized successfully");
                self.pool = Some(pool);
            }
            Err(e) => {
                error!("Failed to initialize database: {}", e);
                // Database remains disabled, but this is not a fatal error
                self.pool = None;
            }
        }
    }

    /// Set up database connection and run migrations
    async fn setup_database(&self) -> Result<SqlitePool, DatabaseError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config.file_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| DatabaseError::Transaction {
                    message: format!("Failed to create database directory: {}", e),
                })?;
            }
        }

        // Build connection options
        let connection_options = SqliteConnectOptions::from_str(
            &format!(
                "sqlite://{}",
                self.config
                    .file_path_str()
                    .ok_or_else(|| DatabaseError::Transaction {
                        message: "Invalid database file path".to_string(),
                    })?
            ),
        )?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(self.config.connection_timeout);

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(self.config.max_connections)
            .connect_with(connection_options)
            .await?;

        // Run migrations
        self.run_migrations(&pool).await?;

        Ok(pool)
    }

    /// Run database migrations
    async fn run_migrations(&self, pool: &SqlitePool) -> Result<(), DatabaseError> {
        info!("Running database migrations");

        let mut tx = pool.begin().await?;

        // Create schema version table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_migrations (
                id INTEGER PRIMARY KEY,
                version INTEGER UNIQUE NOT NULL,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&mut *tx)
        .await?;

        // Check current schema version
        let current_version: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(0);

        info!("Current schema version: {}", current_version);

        // Migration 1: Create core tables
        if current_version < 1 {
            info!("Applying migration 1: Create core tables");
            self.apply_migration_1(&mut tx).await?;
            
            sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
                .bind(1)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        info!("Database migrations completed");

        Ok(())
    }

    /// Apply migration 1: Create core cost tracking tables
    async fn apply_migration_1(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), DatabaseError> {
        // Create cost_sessions table
        sqlx::query(
            r#"
            CREATE TABLE cost_sessions (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                workflow_run_id TEXT,
                started_at DATETIME NOT NULL,
                completed_at DATETIME,
                total_cost DECIMAL(10,4),
                total_calls INTEGER,
                total_input_tokens INTEGER,
                total_output_tokens INTEGER,
                pricing_model TEXT NOT NULL,
                session_duration_ms INTEGER,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&mut **tx)
        .await?;

        // Create api_calls table
        sqlx::query(
            r#"
            CREATE TABLE api_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                call_id TEXT NOT NULL UNIQUE,
                timestamp DATETIME NOT NULL,
                completed_at DATETIME,
                endpoint TEXT NOT NULL,
                model TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                duration_ms INTEGER,
                cost DECIMAL(8,4),
                status TEXT NOT NULL,
                error_message TEXT,
                FOREIGN KEY (session_id) REFERENCES cost_sessions(id)
            )
            "#,
        )
        .execute(&mut **tx)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX idx_cost_sessions_issue_id ON cost_sessions(issue_id)")
            .execute(&mut **tx)
            .await?;

        sqlx::query("CREATE INDEX idx_cost_sessions_started_at ON cost_sessions(started_at)")
            .execute(&mut **tx)
            .await?;

        sqlx::query("CREATE INDEX idx_api_calls_session_id ON api_calls(session_id)")
            .execute(&mut **tx)
            .await?;

        sqlx::query("CREATE INDEX idx_api_calls_timestamp ON api_calls(timestamp)")
            .execute(&mut **tx)
            .await?;

        sqlx::query("CREATE INDEX idx_api_calls_call_id ON api_calls(call_id)")
            .execute(&mut **tx)
            .await?;

        Ok(())
    }

    /// Check if database storage is available
    ///
    /// Returns `true` if database is enabled and connection is established,
    /// `false` otherwise.
    pub fn is_available(&self) -> bool {
        self.pool.is_some()
    }

    /// Check if database is enabled in configuration
    pub fn is_enabled(&self) -> bool {
        self.config.is_enabled()
    }

    /// Store a cost session in the database
    ///
    /// If database is not available, this operation is silently ignored.
    /// Any database errors are logged but do not propagate.
    ///
    /// # Arguments
    ///
    /// * `session` - The cost session to store
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swissarmyhammer::cost::{CostSession, IssueId};
    /// use swissarmyhammer::cost::database::{CostDatabase, DatabaseConfig};
    ///
    /// # tokio_test::block_on(async {
    /// let config = DatabaseConfig::default();
    /// let database = CostDatabase::new(config).await?;
    ///
    /// let issue_id = IssueId::new("test-issue")?;
    /// let session = CostSession::new(issue_id);
    ///
    /// database.store_session(&session).await;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn store_session(&self, session: &CostSession) {
        if let Some(ref pool) = self.pool {
            if let Err(e) = self.store_session_impl(pool, session).await {
                error!("Failed to store cost session: {}", e);
            }
        }
    }

    /// Internal implementation for storing a session
    async fn store_session_impl(
        &self,
        pool: &SqlitePool,
        session: &CostSession,
    ) -> Result<(), DatabaseError> {
        let mut tx = pool.begin().await?;

        // Store session
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO cost_sessions (
                id, issue_id, started_at, completed_at, 
                total_calls, total_input_tokens, total_output_tokens,
                pricing_model, session_duration_ms, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session.session_id.to_string())
        .bind(session.issue_id.as_str())
        .bind(session.started_at)
        .bind(session.completed_at)
        .bind(session.api_calls.len() as i64)
        .bind(session.total_input_tokens() as i64)
        .bind(session.total_output_tokens() as i64)
        .bind("paid") // Default pricing model
        .bind(session.total_duration.map(|d| d.as_millis() as i64))
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;

        // Store API calls
        for api_call in session.api_calls.values() {
            self.store_api_call_impl(&mut tx, &session.session_id, api_call)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Store an API call in the database
    ///
    /// If database is not available, this operation is silently ignored.
    /// Any database errors are logged but do not propagate.
    pub async fn store_api_call(&self, session_id: &CostSessionId, api_call: &ApiCall) {
        if let Some(ref pool) = self.pool {
            if let Err(e) = self.store_api_call_to_pool(pool, session_id, api_call).await {
                error!("Failed to store API call: {}", e);
            }
        }
    }

    /// Store API call to pool
    async fn store_api_call_to_pool(
        &self,
        pool: &SqlitePool,
        session_id: &CostSessionId,
        api_call: &ApiCall,
    ) -> Result<(), DatabaseError> {
        let mut tx = pool.begin().await?;
        self.store_api_call_impl(&mut tx, session_id, api_call)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Internal implementation for storing an API call
    async fn store_api_call_impl(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        session_id: &CostSessionId,
        api_call: &ApiCall,
    ) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO api_calls (
                call_id, session_id, timestamp, completed_at, endpoint, model,
                input_tokens, output_tokens, duration_ms, status, error_message
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(api_call.call_id.to_string())
        .bind(session_id.to_string())
        .bind(api_call.started_at)
        .bind(api_call.completed_at)
        .bind(&api_call.endpoint)
        .bind(&api_call.model)
        .bind(api_call.input_tokens as i64)
        .bind(api_call.output_tokens as i64)
        .bind(api_call.duration.map(|d| d.as_millis() as i64))
        .bind(format!("{:?}", api_call.status))
        .bind(&api_call.error_message)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Clean up old data based on retention policy
    ///
    /// Removes sessions and API calls older than the configured retention period.
    /// If database is not available, this operation is silently ignored.
    pub async fn cleanup_old_data(&self) {
        if let Some(ref pool) = self.pool {
            if let Err(e) = self.cleanup_old_data_impl(pool).await {
                error!("Failed to cleanup old data: {}", e);
            }
        }
    }

    /// Internal implementation for cleaning up old data
    async fn cleanup_old_data_impl(&self, pool: &SqlitePool) -> Result<(), DatabaseError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        let mut tx = pool.begin().await?;

        // Delete old API calls (cascade will handle this with foreign keys, but explicit is better)
        let deleted_calls = sqlx::query(
            "DELETE FROM api_calls WHERE session_id IN (SELECT id FROM cost_sessions WHERE started_at < ?)"
        )
        .bind(cutoff_date)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Delete old sessions
        let deleted_sessions = sqlx::query("DELETE FROM cost_sessions WHERE started_at < ?")
            .bind(cutoff_date)
            .execute(&mut *tx)
            .await?
            .rows_affected();

        tx.commit().await?;

        if deleted_sessions > 0 || deleted_calls > 0 {
            info!(
                "Cleaned up {} old sessions and {} old API calls",
                deleted_sessions, deleted_calls
            );
        }

        Ok(())
    }

    /// Get database connection pool for advanced queries
    ///
    /// Returns the connection pool if database is available, None otherwise.
    /// This is primarily used by the queries module for advanced analytics.
    pub fn pool(&self) -> Option<&SqlitePool> {
        self.pool.as_ref()
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Get database file size in bytes
    ///
    /// Returns None if database is not available or file doesn't exist.
    pub fn file_size(&self) -> Option<u64> {
        if self.config.file_path.exists() {
            std::fs::metadata(&self.config.file_path).ok().map(|m| m.len())
        } else {
            None
        }
    }

    /// Check database health
    ///
    /// Performs a simple query to verify database connectivity.
    /// Returns true if database is healthy, false otherwise.
    pub async fn health_check(&self) -> bool {
        if let Some(ref pool) = self.pool {
            sqlx::query_scalar::<_, i64>("SELECT 1")
                .fetch_one(pool)
                .await
                .map(|_| true)
                .unwrap_or_else(|e| {
                    warn!("Database health check failed: {}", e);
                    false
                })
        } else {
            false
        }
    }
}

// Implement Debug manually to avoid exposing pool internals
impl std::fmt::Debug for CostDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostDatabase")
            .field("config", &self.config)
            .field("is_available", &self.is_available())
            .field("initialization_attempted", &self.initialization_attempted)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::{ApiCall, ApiCallStatus, CostSession, IssueId};
    use std::time::Duration;
    use tempfile::TempDir;

    async fn create_test_database() -> (CostDatabase, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = DatabaseConfig {
            enabled: true,
            file_path: db_path,
            connection_timeout: Duration::from_secs(5),
            max_connections: 2,
            retention_days: 30,
        };

        let database = CostDatabase::new(config).await.unwrap();
        (database, temp_dir)
    }

    #[tokio::test]
    async fn test_database_creation_disabled() {
        let config = DatabaseConfig {
            enabled: false,
            ..DatabaseConfig::default()
        };

        let database = CostDatabase::new(config).await.unwrap();
        assert!(!database.is_available());
        assert!(!database.is_enabled());
    }

    #[tokio::test]
    async fn test_database_creation_enabled() {
        let (database, _temp_dir) = create_test_database().await;

        assert!(database.is_available());
        assert!(database.is_enabled());
        assert!(database.health_check().await);
    }

    #[tokio::test]
    async fn test_store_session() {
        let (database, _temp_dir) = create_test_database().await;

        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add some API calls
        let mut api_call = ApiCall::new("https://api.test.com", "test-model").unwrap();
        api_call.complete(100, 200, ApiCallStatus::Success, None);
        session.add_api_call(api_call).unwrap();

        // Store the session
        database.store_session(&session).await;

        // Verify it was stored by querying the database directly
        if let Some(pool) = database.pool() {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cost_sessions")
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(count, 1);

            let call_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_calls")
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(call_count, 1);
        }
    }

    #[tokio::test]
    async fn test_store_api_call_separately() {
        let (database, _temp_dir) = create_test_database().await;

        let issue_id = IssueId::new("test-issue").unwrap();
        let session = CostSession::new(issue_id);

        // Store session first
        database.store_session(&session).await;

        // Store API call separately
        let mut api_call = ApiCall::new("https://api.test.com", "test-model").unwrap();
        api_call.complete(150, 250, ApiCallStatus::Success, None);

        database.store_api_call(&session.session_id, &api_call).await;

        // Verify both were stored
        if let Some(pool) = database.pool() {
            let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cost_sessions")
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(session_count, 1);

            let call_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM api_calls")
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(call_count, 1);
        }
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let (database, _temp_dir) = create_test_database().await;

        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add API call
        let mut api_call = ApiCall::new("https://api.test.com", "test-model").unwrap();
        api_call.complete(100, 200, ApiCallStatus::Success, None);
        session.add_api_call(api_call).unwrap();

        // Store session
        database.store_session(&session).await;

        // Manually update the session timestamp to be old
        if let Some(pool) = database.pool() {
            let old_date = Utc::now() - chrono::Duration::days(400);
            sqlx::query("UPDATE cost_sessions SET started_at = ? WHERE id = ?")
                .bind(old_date)
                .bind(session.session_id.to_string())
                .execute(pool)
                .await
                .unwrap();
        }

        // Run cleanup
        database.cleanup_old_data().await;

        // Verify data was cleaned up
        if let Some(pool) = database.pool() {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cost_sessions")
                .fetch_one(pool)
                .await
                .unwrap();
            assert_eq!(count, 0);
        }
    }

    #[tokio::test]
    async fn test_database_with_invalid_config() {
        let temp_dir = TempDir::new().unwrap();

        let config = DatabaseConfig {
            enabled: true,
            file_path: temp_dir.path().join("test.db"),
            connection_timeout: Duration::from_secs(0), // Invalid - too short
            max_connections: 5,
            retention_days: 30,
        };

        let result = CostDatabase::new(config).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DatabaseError::Config { .. }));
    }

    #[tokio::test]
    async fn test_file_size() {
        let (database, _temp_dir) = create_test_database().await;

        // Initially should have some size due to schema
        assert!(database.file_size().is_some());
        assert!(database.file_size().unwrap() > 0);

        // Add some data
        let issue_id = IssueId::new("test-issue").unwrap();
        let session = CostSession::new(issue_id);
        database.store_session(&session).await;

        // Size should still be available
        assert!(database.file_size().is_some());
    }

    #[tokio::test]
    async fn test_graceful_failure_handling() {
        // Test that database creation doesn't fail even with bad paths
        let config = DatabaseConfig {
            enabled: true,
            file_path: "/invalid/path/that/does/not/exist.db".into(),
            ..DatabaseConfig::default()
        };

        // This should not panic, but database will be unavailable
        let database = CostDatabase::new(config).await.unwrap();
        assert!(!database.is_available());

        // Operations should be silently ignored
        let issue_id = IssueId::new("test-issue").unwrap();
        let session = CostSession::new(issue_id);
        database.store_session(&session).await; // Should not panic

        let api_call = ApiCall::new("https://api.test.com", "model").unwrap();
        database.store_api_call(&session.session_id, &api_call).await; // Should not panic

        database.cleanup_old_data().await; // Should not panic
    }
}