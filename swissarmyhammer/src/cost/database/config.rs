//! Database configuration for cost analytics

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Default database file path
const DEFAULT_DB_FILE_PATH: &str = "./costs.db";

/// Default connection timeout in seconds
const DEFAULT_CONNECTION_TIMEOUT_SECONDS: u64 = 30;

/// Default maximum number of database connections
const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Default data retention period in days
const DEFAULT_RETENTION_DAYS: u32 = 365;

/// Minimum allowed connection timeout (1 second)
const MIN_CONNECTION_TIMEOUT_SECONDS: u64 = 1;

/// Maximum allowed connection timeout (300 seconds = 5 minutes)
const MAX_CONNECTION_TIMEOUT_SECONDS: u64 = 300;

/// Minimum number of database connections
const MIN_CONNECTIONS: u32 = 1;

/// Maximum number of database connections
const MAX_CONNECTIONS: u32 = 100;

/// Minimum data retention days
const MIN_RETENTION_DAYS: u32 = 1;

/// Maximum data retention days (10 years)
const MAX_RETENTION_DAYS: u32 = 3650;

/// Database configuration errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum DatabaseConfigError {
    /// Invalid connection timeout
    #[error("Connection timeout must be between {min} and {max} seconds, got: {value}")]
    InvalidConnectionTimeout { value: u64, min: u64, max: u64 },

    /// Invalid max connections
    #[error("Max connections must be between {min} and {max}, got: {value}")]
    InvalidMaxConnections { value: u32, min: u32, max: u32 },

    /// Invalid retention days
    #[error("Retention days must be between {min} and {max}, got: {value}")]
    InvalidRetentionDays { value: u32, min: u32, max: u32 },

    /// Invalid file path
    #[error("Database file path cannot be empty")]
    EmptyFilePath,
}

/// Database configuration for cost analytics storage
///
/// This configuration controls the optional SQLite database used for
/// advanced cost analytics and historical reporting.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::cost::database::DatabaseConfig;
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let config = DatabaseConfig {
///     enabled: true,
///     file_path: PathBuf::from("./analytics.db"),
///     connection_timeout: Duration::from_secs(60),
///     max_connections: 20,
///     retention_days: 90,
/// };
///
/// assert!(config.validate().is_ok());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Enable database storage for cost analytics
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Path to the SQLite database file
    #[serde(default = "default_file_path")]
    pub file_path: PathBuf,

    /// Database connection timeout
    #[serde(
        default = "default_connection_timeout",
        with = "duration_serde",
        alias = "connection_timeout_seconds"
    )]
    pub connection_timeout: Duration,

    /// Maximum number of concurrent database connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Data retention period in days
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            file_path: default_file_path(),
            connection_timeout: default_connection_timeout(),
            max_connections: default_max_connections(),
            retention_days: default_retention_days(),
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a database configuration with custom values
    pub fn with_values(
        enabled: bool,
        file_path: PathBuf,
        connection_timeout: Duration,
        max_connections: u32,
        retention_days: u32,
    ) -> Result<Self, DatabaseConfigError> {
        let config = Self {
            enabled,
            file_path,
            connection_timeout,
            max_connections,
            retention_days,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the database configuration
    pub fn validate(&self) -> Result<(), DatabaseConfigError> {
        // Validate file path
        if self.file_path.as_os_str().is_empty() {
            return Err(DatabaseConfigError::EmptyFilePath);
        }

        // Validate connection timeout
        let timeout_secs = self.connection_timeout.as_secs();
        if timeout_secs < MIN_CONNECTION_TIMEOUT_SECONDS
            || timeout_secs > MAX_CONNECTION_TIMEOUT_SECONDS
        {
            return Err(DatabaseConfigError::InvalidConnectionTimeout {
                value: timeout_secs,
                min: MIN_CONNECTION_TIMEOUT_SECONDS,
                max: MAX_CONNECTION_TIMEOUT_SECONDS,
            });
        }

        // Validate max connections
        if self.max_connections < MIN_CONNECTIONS || self.max_connections > MAX_CONNECTIONS {
            return Err(DatabaseConfigError::InvalidMaxConnections {
                value: self.max_connections,
                min: MIN_CONNECTIONS,
                max: MAX_CONNECTIONS,
            });
        }

        // Validate retention days
        if self.retention_days < MIN_RETENTION_DAYS || self.retention_days > MAX_RETENTION_DAYS {
            return Err(DatabaseConfigError::InvalidRetentionDays {
                value: self.retention_days,
                min: MIN_RETENTION_DAYS,
                max: MAX_RETENTION_DAYS,
            });
        }

        Ok(())
    }

    /// Check if database storage is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the database file path as a string
    pub fn file_path_str(&self) -> Option<&str> {
        self.file_path.to_str()
    }
}

// Default value functions
fn default_enabled() -> bool {
    false
}

fn default_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_DB_FILE_PATH)
}

fn default_connection_timeout() -> Duration {
    Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)
}

fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

fn default_retention_days() -> u32 {
    DEFAULT_RETENTION_DAYS
}

// Custom serde module for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();

        assert!(!config.enabled);
        assert_eq!(config.file_path, PathBuf::from(DEFAULT_DB_FILE_PATH));
        assert_eq!(
            config.connection_timeout,
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)
        );
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(config.retention_days, DEFAULT_RETENTION_DAYS);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_new() {
        let config = DatabaseConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_with_values_valid() {
        let config = DatabaseConfig::with_values(
            true,
            PathBuf::from("./test.db"),
            Duration::from_secs(60),
            20,
            180,
        );

        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.enabled);
        assert_eq!(config.file_path, PathBuf::from("./test.db"));
        assert_eq!(config.connection_timeout, Duration::from_secs(60));
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.retention_days, 180);
    }

    #[test]
    fn test_database_config_validation_empty_path() {
        let config = DatabaseConfig {
            file_path: PathBuf::new(),
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(result, Err(DatabaseConfigError::EmptyFilePath)));
    }

    #[test]
    fn test_database_config_validation_invalid_timeout() {
        // Too short
        let config = DatabaseConfig {
            connection_timeout: Duration::from_secs(0),
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidConnectionTimeout { value: 0, .. })
        ));

        // Too long
        let config = DatabaseConfig {
            connection_timeout: Duration::from_secs(400),
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidConnectionTimeout { value: 400, .. })
        ));
    }

    #[test]
    fn test_database_config_validation_invalid_max_connections() {
        // Too few
        let config = DatabaseConfig {
            max_connections: 0,
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidMaxConnections { value: 0, .. })
        ));

        // Too many
        let config = DatabaseConfig {
            max_connections: 1000,
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidMaxConnections { value: 1000, .. })
        ));
    }

    #[test]
    fn test_database_config_validation_invalid_retention_days() {
        // Too short
        let config = DatabaseConfig {
            retention_days: 0,
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidRetentionDays { value: 0, .. })
        ));

        // Too long
        let config = DatabaseConfig {
            retention_days: 5000,
            ..DatabaseConfig::default()
        };

        let result = config.validate();
        assert!(matches!(
            result,
            Err(DatabaseConfigError::InvalidRetentionDays { value: 5000, .. })
        ));
    }

    #[test]
    fn test_database_config_is_enabled() {
        let mut config = DatabaseConfig::default();
        assert!(!config.is_enabled());

        config.enabled = true;
        assert!(config.is_enabled());
    }

    #[test]
    fn test_database_config_file_path_str() {
        let config = DatabaseConfig {
            file_path: PathBuf::from("./test.db"),
            ..DatabaseConfig::default()
        };

        assert_eq!(config.file_path_str(), Some("./test.db"));
    }

    #[test]
    fn test_database_config_serialization() {
        let config = DatabaseConfig {
            enabled: true,
            file_path: PathBuf::from("./analytics.db"),
            connection_timeout: Duration::from_secs(45),
            max_connections: 25,
            retention_days: 200,
        };

        // Test serialization
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("enabled: true"));
        assert!(yaml.contains("./analytics.db"));
        assert!(yaml.contains("max_connections: 25"));
        assert!(yaml.contains("retention_days: 200"));

        // Test deserialization
        let deserialized: DatabaseConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized, config);
    }

    #[test]
    fn test_database_config_yaml_with_connection_timeout_seconds() {
        let yaml = r#"
enabled: true
file_path: "./test.db"
connection_timeout_seconds: 120
max_connections: 15
retention_days: 90
"#;

        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.file_path, PathBuf::from("./test.db"));
        assert_eq!(config.connection_timeout, Duration::from_secs(120));
        assert_eq!(config.max_connections, 15);
        assert_eq!(config.retention_days, 90);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_partial_yaml() {
        let yaml = r#"
enabled: true
file_path: "./custom.db"
"#;

        let config: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.file_path, PathBuf::from("./custom.db"));
        // Should use defaults for other fields
        assert_eq!(
            config.connection_timeout,
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECONDS)
        );
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(config.retention_days, DEFAULT_RETENTION_DAYS);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_database_config_edge_cases() {
        // Test minimum valid values
        let config = DatabaseConfig::with_values(
            true,
            PathBuf::from("./min.db"),
            Duration::from_secs(MIN_CONNECTION_TIMEOUT_SECONDS),
            MIN_CONNECTIONS,
            MIN_RETENTION_DAYS,
        );
        assert!(config.is_ok());

        // Test maximum valid values
        let config = DatabaseConfig::with_values(
            true,
            PathBuf::from("./max.db"),
            Duration::from_secs(MAX_CONNECTION_TIMEOUT_SECONDS),
            MAX_CONNECTIONS,
            MAX_RETENTION_DAYS,
        );
        assert!(config.is_ok());
    }
}
