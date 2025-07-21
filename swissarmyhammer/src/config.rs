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
    /// Create a new configuration instance with values from environment variables
    /// or defaults if environment variables are not set
    pub fn new() -> Self {
        let loader = EnvLoader::new("SWISSARMYHAMMER");

        Self {
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
        }
    }

    /// Get the global configuration instance
    pub fn global() -> &'static Self {
        static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
        CONFIG.get_or_init(Config::new)
    }

    /// Find the swissarmyhammer.yaml configuration file in the current directory
    /// Returns Some(path) if found, None if not found
    #[allow(dead_code)] // Will be used in future config loading implementation
    fn find_yaml_config_file() -> Option<std::path::PathBuf> {
        use std::path::Path;

        let config_path = Path::new("swissarmyhammer.yaml");

        if config_path.exists() && config_path.is_file() {
            tracing::debug!("Found configuration file: {:?}", config_path);
            Some(config_path.to_path_buf())
        } else {
            tracing::debug!(
                "No swissarmyhammer.yaml configuration file found in current directory"
            );
            None
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Ensure we're in a directory that doesn't have the config file
        let original_dir = std::env::current_dir().unwrap();
        let temp_dir = std::env::temp_dir();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Remove any existing config file in temp dir
        let config_path = temp_dir.join("swissarmyhammer.yaml");
        let _ = std::fs::remove_file(&config_path);

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_found() {
        use std::fs::File;
        use std::io::Write;

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

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Clean up
        std::env::set_current_dir(original_dir).unwrap();
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_path_handling() {
        use std::fs::File;
        use std::io::Write;

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
}
