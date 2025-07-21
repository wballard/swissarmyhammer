//! Configuration management for SwissArmyHammer
//!
//! This module provides centralized configuration management with environment variable support
//! and sensible defaults for all configurable constants throughout the application.

use crate::common::env_loader::EnvLoader;
use serde::Deserialize;

const DEFAULT_BASE_BRANCH: &str = "main";

/// Configuration settings for the SwissArmyHammer application
#[derive(Debug, Clone)]
pub struct Config {
    /// Prefix for issue branches (default: "issue/")
    pub issue_branch_prefix: String,
    /// Width for issue numbers in display (default: 6)
    pub issue_number_width: usize,
    /// Maximum number of pending issues to display in summary (default: 5)
    pub max_pending_issues_in_summary: usize,
    /// Minimum issue number allowed (default: 1)
    pub min_issue_number: u32,
    /// Maximum issue number allowed (default: 999_999)
    pub max_issue_number: u32,
    /// Number of digits for issue numbering in filenames (default: 6)
    pub issue_number_digits: usize,
    /// Maximum content length for issue content (default: 50000)
    pub max_content_length: usize,
    /// Maximum line length for issue content (default: 10000)
    pub max_line_length: usize,
    /// Maximum issue name length (default: 100)
    pub max_issue_name_length: usize,
    /// Cache TTL in seconds (default: 300, i.e., 5 minutes)
    pub cache_ttl_seconds: u64,
    /// Maximum cache size (default: 1000)
    pub cache_max_size: usize,
    /// Base number for virtual issue numbering (default: 500_000)
    pub virtual_issue_number_base: u32,
    /// Range for virtual issue numbers (default: 500_000, so virtual numbers go from base to base+range-1)
    pub virtual_issue_number_range: u32,
    /// Base branch for pull requests (default: "main")
    pub base_branch: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            issue_branch_prefix: "issue/".to_string(),
            issue_number_width: 6,
            max_pending_issues_in_summary: 5,
            min_issue_number: 1,
            max_issue_number: 999_999,
            issue_number_digits: 6,
            max_content_length: 50000,
            max_line_length: 10000,
            max_issue_name_length: 100,
            cache_ttl_seconds: 300,
            cache_max_size: 1000,
            virtual_issue_number_base: 500_000,
            virtual_issue_number_range: 500_000,
            base_branch: DEFAULT_BASE_BRANCH.to_string(),
        }
    }
}

impl Config {
    /// Create a new configuration instance with values from environment variables,
    /// YAML configuration file, or defaults (in that order of precedence)
    pub fn new() -> Self {
        let loader = EnvLoader::new("SWISSARMYHAMMER");

        // Start with environment variables or defaults
        let mut config = Self {
            issue_branch_prefix: loader.load_string("ISSUE_BRANCH_PREFIX", "issue/"),
            issue_number_width: loader.load_parsed("ISSUE_NUMBER_WIDTH", 6),
            max_pending_issues_in_summary: loader.load_parsed("MAX_PENDING_ISSUES_IN_SUMMARY", 5),
            min_issue_number: loader.load_parsed("MIN_ISSUE_NUMBER", 1),
            max_issue_number: loader.load_parsed("MAX_ISSUE_NUMBER", 999_999),
            issue_number_digits: loader.load_parsed("ISSUE_NUMBER_DIGITS", 6),
            max_content_length: loader.load_parsed("MAX_CONTENT_LENGTH", 50000),
            max_line_length: loader.load_parsed("MAX_LINE_LENGTH", 10000),
            max_issue_name_length: loader.load_parsed("MAX_ISSUE_NAME_LENGTH", 100),
            cache_ttl_seconds: loader.load_parsed("CACHE_TTL_SECONDS", 300),
            cache_max_size: loader.load_parsed("CACHE_MAX_SIZE", 1000),
            virtual_issue_number_base: loader.load_parsed("VIRTUAL_ISSUE_NUMBER_BASE", 500_000),
            virtual_issue_number_range: loader.load_parsed("VIRTUAL_ISSUE_NUMBER_RANGE", 500_000),
            base_branch: loader.load_string("BASE_BRANCH", DEFAULT_BASE_BRANCH),
        };

        // Apply YAML configuration if available (only overrides values not set by environment)
        if let Some(yaml_path) = Self::find_yaml_config_file() {
            match std::fs::read_to_string(&yaml_path) {
                Ok(yaml_content) => {
                    match serde_yaml::from_str::<YamlConfig>(&yaml_content) {
                        Ok(yaml_config) => {
                            // Validate the loaded configuration
                            if let Err(validation_error) = yaml_config.validate() {
                                tracing::warn!(
                                    "Invalid YAML configuration in {:?}: {}. Using default values instead.",
                                    yaml_path,
                                    validation_error
                                );
                                return config; // Return without applying invalid config
                            }

                            tracing::debug!(
                                "Loaded and validated YAML configuration from {:?}",
                                yaml_path
                            );
                            // YAML config only applies to values that weren't overridden by environment
                            Self::apply_yaml_config_selectively(&mut config, &yaml_config, &loader);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse YAML configuration from {:?}: {}",
                                yaml_path,
                                e
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to read YAML configuration file {:?}: {}",
                        yaml_path,
                        e
                    );
                }
            }
        }

        config
    }

    /// Get the global configuration instance
    pub fn global() -> &'static Self {
        static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
        CONFIG.get_or_init(Config::new)
    }

    /// Find the swissarmyhammer.yaml configuration file in multiple locations
    ///
    /// Searches for a YAML configuration file to supplement environment variable
    /// and default configuration values. The search order is:
    /// 1. Current working directory: `swissarmyhammer.yaml`
    /// 2. Home directory: `~/.config/swissarmyhammer/swissarmyhammer.yaml`
    /// 3. Home directory root: `~/swissarmyhammer.yaml`
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Path to the first configuration file found and readable
    /// * `None` - No configuration file found in any location or file is not accessible
    ///
    /// # Examples
    /// ```
    /// use swissarmyhammer::config::Config;
    ///
    /// if let Some(config_path) = Config::find_yaml_config_file() {
    ///     println!("Found config at: {:?}", config_path);
    /// }
    /// ```
    pub fn find_yaml_config_file() -> Option<std::path::PathBuf> {
        use std::path::PathBuf;

        let config_filename = "swissarmyhammer.yaml";

        // Define search paths in order of priority
        let mut search_paths = Vec::new();

        // 1. Current working directory
        search_paths.push(PathBuf::from(config_filename));

        // 2. Home directory .config/swissarmyhammer/ subdirectory
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(
                home_dir
                    .join(".config")
                    .join("swissarmyhammer")
                    .join(config_filename),
            );
        }

        // 3. Home directory root
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(home_dir.join(config_filename));
        }

        // Search through each path
        for config_path in search_paths {
            match Self::check_config_file(&config_path) {
                Some(path) => {
                    tracing::debug!("Found configuration file: {:?}", path);
                    return Some(path);
                }
                None => continue,
            }
        }

        tracing::debug!("No swissarmyhammer.yaml configuration file found in any search location");
        None
    }

    /// Check if a configuration file exists and is readable
    fn check_config_file(config_path: &std::path::Path) -> Option<std::path::PathBuf> {
        match config_path.try_exists() {
            Ok(true) if config_path.is_file() => {
                // Additional permission check
                match std::fs::File::open(config_path) {
                    Ok(_) => Some(config_path.to_path_buf()),
                    Err(e) => {
                        tracing::warn!(
                            "Configuration file {:?} exists but cannot be read: {}",
                            config_path,
                            e
                        );
                        None
                    }
                }
            }
            Ok(false) => None, // File does not exist
            Ok(true) => {
                tracing::debug!(
                    "Found {:?} but it is not a file (possibly a directory)",
                    config_path
                );
                None
            }
            Err(e) => {
                tracing::warn!(
                    "Error checking for configuration file {:?}: {}",
                    config_path,
                    e
                );
                None
            }
        }
    }

    /// Apply YAML configuration selectively, only overriding values not set by environment
    fn apply_yaml_config_selectively(
        config: &mut Config,
        yaml_config: &YamlConfig,
        _loader: &EnvLoader,
    ) {
        // Only apply YAML base_branch if environment variable is not set
        if std::env::var("SWISSARMYHAMMER_BASE_BRANCH").is_err() {
            if let Some(ref base_branch) = yaml_config.base_branch {
                config.base_branch = base_branch.clone();
            }
        }
    }

    /// Reset the global configuration (for testing purposes)
    #[cfg(test)]
    pub fn reset_global() {
        // This is a workaround since OnceLock doesn't have a reset method
        // We can't actually reset the global config in tests due to OnceLock's design
        // Tests should use Config::new() directly instead of global() for testing env vars
    }
}

/// Configuration loaded from swissarmyhammer.yaml file
#[derive(Debug, Clone, Default, Deserialize)]
pub struct YamlConfig {
    /// Base branch for pull requests
    pub base_branch: Option<String>,
}

impl YamlConfig {
    /// Apply YAML configuration values to an existing Config
    /// YAML values take precedence over existing values
    pub fn apply_to_config(&self, config: &mut Config) {
        if let Some(ref base_branch) = self.base_branch {
            config.base_branch = base_branch.clone();
        }
    }

    /// Validate the YAML configuration for correctness
    ///
    /// Performs validation checks on the loaded YAML configuration to ensure:
    /// - Branch names are valid (no invalid characters, not empty)
    /// - All values are within acceptable ranges
    ///
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(String)` - Configuration is invalid with description of the error
    ///
    /// # Examples
    /// ```
    /// use swissarmyhammer::config::YamlConfig;
    ///
    /// let config = YamlConfig {
    ///     base_branch: Some("main".to_string()),
    /// };
    /// assert!(config.validate().is_ok());
    ///
    /// let invalid_config = YamlConfig {
    ///     base_branch: Some("".to_string()),
    /// };
    /// assert!(invalid_config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Validate base_branch if provided
        if let Some(ref base_branch) = self.base_branch {
            Self::validate_branch_name(base_branch)?;
        }

        Ok(())
    }

    /// Validate that a branch name is acceptable for git usage
    fn validate_branch_name(branch_name: &str) -> Result<(), String> {
        // Check for empty branch name
        if branch_name.is_empty() {
            return Err("Branch name cannot be empty".to_string());
        }

        // Check for whitespace-only branch name
        if branch_name.trim().is_empty() {
            return Err("Branch name cannot be whitespace only".to_string());
        }

        // Check branch name length (reasonable limit)
        if branch_name.len() > 255 {
            return Err("Branch name is too long (maximum 255 characters)".to_string());
        }

        // Check for invalid characters that git doesn't allow
        let invalid_chars = ['\0', ' ', '~', '^', ':', '?', '*', '[', '\\'];
        for &invalid_char in &invalid_chars {
            if branch_name.contains(invalid_char) {
                return Err(format!(
                    "Branch name '{}' contains invalid character '{}'",
                    branch_name, invalid_char
                ));
            }
        }

        // Check for sequences that git doesn't allow
        if branch_name.contains("..") {
            return Err("Branch name cannot contain consecutive dots '..'".to_string());
        }

        // Check that it doesn't start or end with certain characters
        if branch_name.starts_with('.') || branch_name.ends_with('.') {
            return Err("Branch name cannot start or end with '.'".to_string());
        }

        if branch_name.starts_with('/') || branch_name.ends_with('/') {
            return Err("Branch name cannot start or end with '/'".to_string());
        }

        if branch_name.ends_with(".lock") {
            return Err("Branch name cannot end with '.lock'".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to ensure thread-safe working directory modification for tests
    static WORKING_DIR_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.issue_branch_prefix, "issue/");
        assert_eq!(config.issue_number_width, 6);
        assert_eq!(config.max_pending_issues_in_summary, 5);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 999_999);
        assert_eq!(config.issue_number_digits, 6);
        assert_eq!(config.max_content_length, 50000);
        assert_eq!(config.max_line_length, 10000);
        assert_eq!(config.max_issue_name_length, 100);
        assert_eq!(config.virtual_issue_number_base, 500_000);
        assert_eq!(config.virtual_issue_number_range, 500_000);
        assert_eq!(config.base_branch, "main");
    }

    #[test]
    fn test_config_new() {
        // Clean up any environment variables from other tests
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX");
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY");
        std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER");
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS");
        std::env::remove_var("SWISSARMYHAMMER_MAX_CONTENT_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_LINE_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NAME_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE");
        std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");

        let config = Config::new();
        // Should use defaults when environment variables are not set
        assert_eq!(config.issue_branch_prefix, "issue/");
        assert_eq!(config.issue_number_width, 6);
        assert_eq!(config.max_pending_issues_in_summary, 5);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 999_999);
        assert_eq!(config.issue_number_digits, 6);
        assert_eq!(config.max_content_length, 50000);
        assert_eq!(config.max_line_length, 10000);
        assert_eq!(config.max_issue_name_length, 100);
        assert_eq!(config.virtual_issue_number_base, 500_000);
        assert_eq!(config.virtual_issue_number_range, 500_000);
        assert_eq!(config.base_branch, "main");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_with_env_vars() {
        // Save original env vars if they exist
        let orig_prefix = std::env::var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX").ok();
        let orig_width = std::env::var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH").ok();
        let orig_max_pending = std::env::var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY").ok();
        let orig_max_number = std::env::var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER").ok();
        let orig_digits = std::env::var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS").ok();
        let orig_virtual_base = std::env::var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE").ok();
        let orig_virtual_range = std::env::var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE").ok();
        let orig_base_branch = std::env::var("SWISSARMYHAMMER_BASE_BRANCH").ok();

        // Set test values
        std::env::set_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX", "feature/");
        std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH", "8");
        std::env::set_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY", "10");
        std::env::set_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER", "9999999");
        std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS", "7");
        std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE", "600000");
        std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE", "400000");
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "develop");

        let config = Config::new();
        assert_eq!(config.issue_branch_prefix, "feature/");
        assert_eq!(config.issue_number_width, 8);
        assert_eq!(config.max_pending_issues_in_summary, 10);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 9_999_999);
        assert_eq!(config.issue_number_digits, 7);
        assert_eq!(config.virtual_issue_number_base, 600_000);
        assert_eq!(config.virtual_issue_number_range, 400_000);
        assert_eq!(config.base_branch, "develop");

        // Restore original env vars or remove if they didn't exist
        match orig_prefix {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX"),
        }
        match orig_width {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH"),
        }
        match orig_max_pending {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY", val),
            None => std::env::remove_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY"),
        }
        match orig_max_number {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER", val),
            None => std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER"),
        }
        match orig_digits {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS"),
        }
        match orig_virtual_base {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE", val),
            None => std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE"),
        }
        match orig_virtual_range {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE", val),
            None => std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE"),
        }
        match orig_base_branch {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", val),
            None => std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH"),
        }
    }

    #[test]
    fn test_yaml_config_default() {
        let yaml_config = YamlConfig::default();
        assert!(yaml_config.base_branch.is_none());
    }

    #[test]
    fn test_yaml_config_apply_to_config_with_values() {
        let yaml_config = YamlConfig {
            base_branch: Some("develop".to_string()),
        };
        let mut config = Config::default();

        // Verify initial state
        assert_eq!(config.base_branch, "main");

        // Apply YAML config
        yaml_config.apply_to_config(&mut config);

        // Verify YAML config took precedence
        assert_eq!(config.base_branch, "develop");
    }

    #[test]
    fn test_yaml_config_apply_to_config_with_none() {
        let yaml_config = YamlConfig::default(); // all fields are None
        let mut config = Config::default();

        // Save original value
        let original_base_branch = config.base_branch.clone();

        // Apply YAML config with None values
        yaml_config.apply_to_config(&mut config);

        // Verify original values are preserved
        assert_eq!(config.base_branch, original_base_branch);
    }

    #[test]
    fn test_yaml_config_deserialization() {
        let yaml_content = r#"
base_branch: "feature/test"
"#;

        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(yaml_config.base_branch, Some("feature/test".to_string()));
    }

    #[test]
    fn test_yaml_config_partial_deserialization() {
        let yaml_content = r#"
# Empty YAML config
"#;

        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(yaml_config.base_branch.is_none());
    }

    #[test]
    fn test_yaml_config_apply_overwrites_existing_values() {
        let yaml_config = YamlConfig {
            base_branch: Some("staging".to_string()),
        };

        // Create config with non-default value
        let mut config = Config {
            base_branch: "custom".to_string(),
            ..Default::default()
        };

        // Verify initial custom value
        assert_eq!(config.base_branch, "custom");

        // Apply YAML config
        yaml_config.apply_to_config(&mut config);

        // Verify YAML config overwrote the existing value
        assert_eq!(config.base_branch, "staging");
    }

    #[test]
    fn test_find_yaml_config_file_not_found() {
        let _guard = WORKING_DIR_MUTEX.lock().expect("Failed to lock working directory mutex");

        // Ensure we're in a directory that doesn't have the config file
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = std::env::temp_dir();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Remove any existing config file in temp dir
        let config_path = temp_dir.join("swissarmyhammer.yaml");
        let _ = std::fs::remove_file(&config_path);

        // Remove any existing config files in home directory locations to ensure clean test
        let mut backup_paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_path = home_dir.join("swissarmyhammer.yaml");
            let xdg_config_path = home_dir
                .join(".config")
                .join("swissarmyhammer")
                .join("swissarmyhammer.yaml");

            // Backup existing files if they exist
            if home_config_path.exists() {
                let backup_path = home_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&home_config_path, &backup_path);
                backup_paths.push((home_config_path.clone(), backup_path));
            }
            if xdg_config_path.exists() {
                let backup_path = xdg_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&xdg_config_path, &backup_path);
                backup_paths.push((xdg_config_path.clone(), backup_path));
            }
        }

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Restore backed up files
        for (original, backup) in backup_paths {
            let _ = std::fs::rename(backup, original);
        }

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_found() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX.lock().expect("Failed to lock working directory mutex");

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create config file in the test directory
        let config_path = test_dir.join("swissarmyhammer.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: test").unwrap();
        drop(file);

        // Change to test directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        let result = Config::find_yaml_config_file();
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "swissarmyhammer.yaml");

        // Clean up
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_directory_not_file() {
        let _guard = WORKING_DIR_MUTEX.lock().expect("Failed to lock working directory mutex");

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_dir_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create a directory with the config name
        let config_dir = test_dir.join("swissarmyhammer.yaml");
        std::fs::create_dir_all(&config_dir).unwrap();

        // Change to test directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        // Remove any existing config files in home directory locations to ensure clean test
        let mut backup_paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_path = home_dir.join("swissarmyhammer.yaml");
            let xdg_config_path = home_dir
                .join(".config")
                .join("swissarmyhammer")
                .join("swissarmyhammer.yaml");

            // Backup existing files if they exist
            if home_config_path.exists() {
                let backup_path = home_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&home_config_path, &backup_path);
                backup_paths.push((home_config_path.clone(), backup_path));
            }
            if xdg_config_path.exists() {
                let backup_path = xdg_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&xdg_config_path, &backup_path);
                backup_paths.push((xdg_config_path.clone(), backup_path));
            }
        }

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Restore backed up files
        for (original, backup) in backup_paths {
            let _ = std::fs::rename(backup, original);
        }

        // Clean up
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_path_handling() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX.lock().expect("Failed to lock working directory mutex");

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_path_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create config file in the test directory
        let config_path = test_dir.join("swissarmyhammer.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: test").unwrap();
        drop(file);

        // Change to test directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        let result = Config::find_yaml_config_file();
        assert!(result.is_some());
        let found_path = result.unwrap();

        // Verify the path is properly constructed
        assert!(found_path.is_file());
        assert_eq!(found_path.file_name().unwrap(), "swissarmyhammer.yaml");

        // Clean up
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_validation_valid() {
        let yaml_config = YamlConfig {
            base_branch: Some("main".to_string()),
        };
        assert!(yaml_config.validate().is_ok());

        let yaml_config_none = YamlConfig { base_branch: None };
        assert!(yaml_config_none.validate().is_ok());
    }

    #[test]
    fn test_yaml_config_validation_invalid_branch_names() {
        // Empty branch name
        let yaml_config = YamlConfig {
            base_branch: Some("".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Whitespace only branch name
        let yaml_config = YamlConfig {
            base_branch: Some("   ".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Branch name with invalid characters
        let invalid_chars = vec![' ', '~', '^', ':', '?', '*', '[', '\\'];
        for invalid_char in invalid_chars {
            let yaml_config = YamlConfig {
                base_branch: Some(format!("branch{}", invalid_char)),
            };
            assert!(yaml_config.validate().is_err());
        }

        // Branch name with consecutive dots
        let yaml_config = YamlConfig {
            base_branch: Some("branch..name".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Branch name starting/ending with dot
        let yaml_config = YamlConfig {
            base_branch: Some(".branch".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        let yaml_config = YamlConfig {
            base_branch: Some("branch.".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Branch name starting/ending with slash
        let yaml_config = YamlConfig {
            base_branch: Some("/branch".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        let yaml_config = YamlConfig {
            base_branch: Some("branch/".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Branch name ending with .lock
        let yaml_config = YamlConfig {
            base_branch: Some("branch.lock".to_string()),
        };
        assert!(yaml_config.validate().is_err());

        // Branch name too long
        let yaml_config = YamlConfig {
            base_branch: Some("a".repeat(256)),
        };
        assert!(yaml_config.validate().is_err());
    }

    #[test]
    fn test_yaml_config_validation_valid_branch_names() {
        let valid_names = vec![
            "main",
            "develop",
            "feature/new-feature",
            "bugfix/issue-123",
            "release/v1.0.0",
            "hotfix/critical-fix",
            "user/john/feature",
        ];

        for valid_name in valid_names {
            let yaml_config = YamlConfig {
                base_branch: Some(valid_name.to_string()),
            };
            assert!(
                yaml_config.validate().is_ok(),
                "Expected '{}' to be valid but validation failed",
                valid_name
            );
        }
    }
}
