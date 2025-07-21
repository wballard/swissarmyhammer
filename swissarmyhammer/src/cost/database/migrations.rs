//! Database migrations for cost analytics schema

use sqlx::{Sqlite, Transaction};
use thiserror::Error;
use tracing::{info, warn};

/// Migration errors
#[derive(Error, Debug)]
pub enum MigrationError {
    /// Database error during migration
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Migration validation error
    #[error("Migration validation error: {message}")]
    Validation { message: String },

    /// Migration rollback error
    #[error("Migration rollback failed: {message}")]
    Rollback { message: String },
}

/// Represents a single database migration
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version number (must be unique and sequential)
    pub version: i64,
    /// Human-readable description of the migration
    pub description: String,
    /// SQL statements to apply the migration
    pub up_sql: Vec<String>,
    /// SQL statements to rollback the migration (optional)
    pub down_sql: Option<Vec<String>>,
}

impl Migration {
    /// Create a new migration
    pub fn new(
        version: i64,
        description: impl Into<String>,
        up_sql: Vec<String>,
        down_sql: Option<Vec<String>>,
    ) -> Self {
        Self {
            version,
            description: description.into(),
            up_sql,
            down_sql,
        }
    }

    /// Validate the migration
    pub fn validate(&self) -> Result<(), MigrationError> {
        if self.version <= 0 {
            return Err(MigrationError::Validation {
                message: format!("Migration version must be positive, got: {}", self.version),
            });
        }

        if self.description.trim().is_empty() {
            return Err(MigrationError::Validation {
                message: "Migration description cannot be empty".to_string(),
            });
        }

        if self.up_sql.is_empty() {
            return Err(MigrationError::Validation {
                message: "Migration must have at least one up SQL statement".to_string(),
            });
        }

        // Validate that SQL statements are not empty
        for (i, sql) in self.up_sql.iter().enumerate() {
            if sql.trim().is_empty() {
                return Err(MigrationError::Validation {
                    message: format!("Up SQL statement {} cannot be empty", i + 1),
                });
            }
        }

        // Validate down SQL if provided
        if let Some(ref down_sql) = self.down_sql {
            for (i, sql) in down_sql.iter().enumerate() {
                if sql.trim().is_empty() {
                    return Err(MigrationError::Validation {
                        message: format!("Down SQL statement {} cannot be empty", i + 1),
                    });
                }
            }
        }

        Ok(())
    }

    /// Apply the migration (run up SQL)
    pub async fn apply(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        self.validate()?;

        info!("Applying migration {}: {}", self.version, self.description);

        for (i, sql) in self.up_sql.iter().enumerate() {
            match sqlx::query(sql).execute(&mut **tx).await {
                Ok(_) => {
                    info!("  Executed statement {}/{}", i + 1, self.up_sql.len());
                }
                Err(e) => {
                    return Err(MigrationError::Database(e));
                }
            }
        }

        // Record migration in schema_migrations table
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (?)")
            .bind(self.version)
            .execute(&mut **tx)
            .await?;

        info!("Migration {} applied successfully", self.version);
        Ok(())
    }

    /// Rollback the migration (run down SQL)
    pub async fn rollback(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        let down_sql = self.down_sql.as_ref().ok_or_else(|| {
            MigrationError::Rollback {
                message: format!("Migration {} has no rollback SQL", self.version),
            }
        })?;

        warn!(
            "Rolling back migration {}: {}",
            self.version, self.description
        );

        for (i, sql) in down_sql.iter().enumerate() {
            match sqlx::query(sql).execute(&mut **tx).await {
                Ok(_) => {
                    info!("  Executed rollback statement {}/{}", i + 1, down_sql.len());
                }
                Err(e) => {
                    return Err(MigrationError::Rollback {
                        message: format!("Rollback statement failed: {}", e),
                    });
                }
            }
        }

        // Remove migration from schema_migrations table
        sqlx::query("DELETE FROM schema_migrations WHERE version = ?")
            .bind(self.version)
            .execute(&mut **tx)
            .await?;

        warn!("Migration {} rolled back successfully", self.version);
        Ok(())
    }
}

/// Migration runner for managing database schema evolution
pub struct MigrationRunner {
    /// Available migrations in version order
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    /// Create a new migration runner with the available migrations
    pub fn new() -> Self {
        Self {
            migrations: Self::get_available_migrations(),
        }
    }

    /// Get all available migrations
    fn get_available_migrations() -> Vec<Migration> {
        vec![
            Migration::new(
                1,
                "Create core cost tracking tables",
                vec![
                    // cost_sessions table
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
                    "#.to_string(),
                    // api_calls table
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
                    "#.to_string(),
                    // Indexes for cost_sessions
                    "CREATE INDEX idx_cost_sessions_issue_id ON cost_sessions(issue_id)".to_string(),
                    "CREATE INDEX idx_cost_sessions_started_at ON cost_sessions(started_at)".to_string(),
                    // Indexes for api_calls  
                    "CREATE INDEX idx_api_calls_session_id ON api_calls(session_id)".to_string(),
                    "CREATE INDEX idx_api_calls_timestamp ON api_calls(timestamp)".to_string(),
                    "CREATE INDEX idx_api_calls_call_id ON api_calls(call_id)".to_string(),
                ],
                Some(vec![
                    "DROP INDEX IF EXISTS idx_api_calls_call_id".to_string(),
                    "DROP INDEX IF EXISTS idx_api_calls_timestamp".to_string(),
                    "DROP INDEX IF EXISTS idx_api_calls_session_id".to_string(),
                    "DROP INDEX IF EXISTS idx_cost_sessions_started_at".to_string(),
                    "DROP INDEX IF EXISTS idx_cost_sessions_issue_id".to_string(),
                    "DROP TABLE IF EXISTS api_calls".to_string(),
                    "DROP TABLE IF EXISTS cost_sessions".to_string(),
                ]),
            ),
        ]
    }

    /// Get the target migration version (highest available version)
    pub fn target_version(&self) -> i64 {
        self.migrations
            .iter()
            .map(|m| m.version)
            .max()
            .unwrap_or(0)
    }

    /// Get migrations to apply to reach target version from current version
    pub fn get_pending_migrations(&self, current_version: i64) -> Vec<&Migration> {
        self.migrations
            .iter()
            .filter(|m| m.version > current_version)
            .collect()
    }

    /// Get migrations to rollback from current version to target version
    pub fn get_rollback_migrations(&self, current_version: i64, target_version: i64) -> Vec<&Migration> {
        if target_version >= current_version {
            return vec![];
        }

        self.migrations
            .iter()
            .filter(|m| m.version > target_version && m.version <= current_version)
            .rev() // Rollback in reverse order
            .collect()
    }

    /// Apply all pending migrations
    pub async fn migrate_up(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        current_version: i64,
    ) -> Result<i64, MigrationError> {
        let pending = self.get_pending_migrations(current_version);

        if pending.is_empty() {
            info!("No pending migrations");
            return Ok(current_version);
        }

        info!(
            "Applying {} pending migrations from version {} to {}",
            pending.len(),
            current_version,
            self.target_version()
        );

        let mut applied_version = current_version;
        for migration in pending {
            migration.apply(tx).await?;
            applied_version = migration.version;
        }

        info!(
            "Successfully applied all migrations, now at version {}",
            applied_version
        );

        Ok(applied_version)
    }

    /// Rollback migrations to target version
    pub async fn migrate_down(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        current_version: i64,
        target_version: i64,
    ) -> Result<i64, MigrationError> {
        let rollback_migrations = self.get_rollback_migrations(current_version, target_version);

        if rollback_migrations.is_empty() {
            info!("No migrations to rollback");
            return Ok(current_version);
        }

        info!(
            "Rolling back {} migrations from version {} to {}",
            rollback_migrations.len(),
            current_version,
            target_version
        );

        let mut current_version = current_version;
        for migration in rollback_migrations {
            migration.rollback(tx).await?;
            current_version = migration.version - 1;
        }

        info!(
            "Successfully rolled back migrations, now at version {}",
            current_version.max(0)
        );

        Ok(current_version.max(0))
    }

    /// Validate all migrations
    pub fn validate_migrations(&self) -> Result<(), MigrationError> {
        // Check for duplicate versions
        let mut versions = std::collections::HashSet::new();
        for migration in &self.migrations {
            if !versions.insert(migration.version) {
                return Err(MigrationError::Validation {
                    message: format!("Duplicate migration version: {}", migration.version),
                });
            }
        }

        // Validate each migration
        for migration in &self.migrations {
            migration.validate()?;
        }

        // Check for version gaps (should be sequential)
        let mut sorted_versions: Vec<_> = versions.into_iter().collect();
        sorted_versions.sort();

        for i in 0..sorted_versions.len() {
            let expected = i as i64 + 1;
            if sorted_versions[i] != expected {
                return Err(MigrationError::Validation {
                    message: format!(
                        "Migration version gap detected: expected {}, found {}",
                        expected, sorted_versions[i]
                    ),
                });
            }
        }

        Ok(())
    }

    /// Get migration by version
    pub fn get_migration(&self, version: i64) -> Option<&Migration> {
        self.migrations.iter().find(|m| m.version == version)
    }

    /// Get all migrations
    pub fn get_migrations(&self) -> &[Migration] {
        &self.migrations
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creation() {
        let migration = Migration::new(
            1,
            "Test migration",
            vec!["CREATE TABLE test (id INTEGER)".to_string()],
            Some(vec!["DROP TABLE test".to_string()]),
        );

        assert_eq!(migration.version, 1);
        assert_eq!(migration.description, "Test migration");
        assert_eq!(migration.up_sql.len(), 1);
        assert!(migration.down_sql.is_some());
        assert!(migration.validate().is_ok());
    }

    #[test]
    fn test_migration_validation() {
        // Test invalid version
        let invalid_version = Migration::new(
            0,
            "Invalid version",
            vec!["CREATE TABLE test (id INTEGER)".to_string()],
            None,
        );
        assert!(invalid_version.validate().is_err());

        // Test empty description
        let empty_desc = Migration::new(
            1,
            "",
            vec!["CREATE TABLE test (id INTEGER)".to_string()],
            None,
        );
        assert!(empty_desc.validate().is_err());

        // Test empty up SQL
        let empty_up = Migration::new(1, "Test", vec![], None);
        assert!(empty_up.validate().is_err());

        // Test empty SQL statement
        let empty_statement = Migration::new(1, "Test", vec!["".to_string()], None);
        assert!(empty_statement.validate().is_err());
    }

    #[test]
    fn test_migration_runner_creation() {
        let runner = MigrationRunner::new();
        assert!(!runner.get_migrations().is_empty());
        assert!(runner.target_version() > 0);
    }

    #[test]
    fn test_migration_runner_validation() {
        let runner = MigrationRunner::new();
        assert!(runner.validate_migrations().is_ok());
    }

    #[test]
    fn test_pending_migrations() {
        let runner = MigrationRunner::new();
        let target = runner.target_version();

        // All migrations are pending from version 0
        let pending = runner.get_pending_migrations(0);
        assert_eq!(pending.len() as i64, target);

        // No migrations pending from target version
        let pending = runner.get_pending_migrations(target);
        assert!(pending.is_empty());

        // Some migrations pending from version 1
        if target > 1 {
            let pending = runner.get_pending_migrations(1);
            assert_eq!(pending.len() as i64, target - 1);
        }
    }

    #[test]
    fn test_rollback_migrations() {
        let runner = MigrationRunner::new();
        let target = runner.target_version();

        // No rollbacks needed when target >= current
        let rollbacks = runner.get_rollback_migrations(target, target);
        assert!(rollbacks.is_empty());

        let rollbacks = runner.get_rollback_migrations(target, target + 1);
        assert!(rollbacks.is_empty());

        // All migrations to rollback from target to 0
        let rollbacks = runner.get_rollback_migrations(target, 0);
        assert_eq!(rollbacks.len() as i64, target);

        // Partial rollback
        if target > 1 {
            let rollbacks = runner.get_rollback_migrations(target, 1);
            assert_eq!(rollbacks.len() as i64, target - 1);
        }
    }

    #[test]
    fn test_get_migration() {
        let runner = MigrationRunner::new();

        // Should find migration 1
        assert!(runner.get_migration(1).is_some());

        // Should not find non-existent migration
        assert!(runner.get_migration(99999).is_none());
    }

    #[test]
    fn test_migration_validation_duplicate_versions() {
        // This test would require creating a custom migration runner with duplicates
        // For now, we test that the default runner validates correctly
        let runner = MigrationRunner::new();
        assert!(runner.validate_migrations().is_ok());
    }

    #[test]
    fn test_migration_version_gaps() {
        // Test that migrations start at 1 and are sequential
        let runner = MigrationRunner::new();
        let versions: Vec<_> = runner.get_migrations().iter().map(|m| m.version).collect();

        let mut sorted_versions = versions.clone();
        sorted_versions.sort();

        for (i, &version) in sorted_versions.iter().enumerate() {
            assert_eq!(version, i as i64 + 1, "Version gap detected at position {}", i);
        }
    }

    #[test]
    fn test_core_migration_content() {
        let runner = MigrationRunner::new();
        let migration_1 = runner.get_migration(1).unwrap();

        assert_eq!(migration_1.version, 1);
        assert_eq!(migration_1.description, "Create core cost tracking tables");

        // Should have multiple SQL statements for tables and indexes
        assert!(migration_1.up_sql.len() >= 7); // 2 tables + 5 indexes

        // Should have rollback SQL
        assert!(migration_1.down_sql.is_some());
        let down_sql = migration_1.down_sql.as_ref().unwrap();
        assert!(down_sql.len() >= 7); // Corresponding rollback statements

        // Verify table creation statements are present
        let up_sql = migration_1.up_sql.join(" ");
        assert!(up_sql.contains("CREATE TABLE cost_sessions"));
        assert!(up_sql.contains("CREATE TABLE api_calls"));
        assert!(up_sql.contains("CREATE INDEX"));

        // Verify rollback statements
        let down_sql_text = down_sql.join(" ");
        assert!(down_sql_text.contains("DROP TABLE"));
        assert!(down_sql_text.contains("DROP INDEX"));
    }
}