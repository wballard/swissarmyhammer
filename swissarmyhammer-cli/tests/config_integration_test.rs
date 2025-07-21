//! Integration tests for CLI configuration management
//!
//! Tests the integration of the YAML configuration system with the CLI layer,
//! ensuring that configuration is properly loaded, validated, and accessible
//! throughout the CLI commands.

use std::fs;
use tempfile::TempDir;
use swissarmyhammer_cli::cli::{Cli, Commands};
use swissarmyhammer::config::Config;

#[cfg(test)]
mod config_cli_tests {
    use super::*;

    #[test]
    fn test_config_show_command_exists() {
        // Test that we can parse a config show command
        let result = Cli::try_parse_from_args([
            "swissarmyhammer", 
            "config", 
            "show"
        ]);
        
        assert!(result.is_ok(), "Config show command should be parseable");
        
        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Config { action }) => {
                    // This will fail initially because we haven't implemented it yet
                    // but we're writing the test first (TDD)
                    assert!(matches!(action, swissarmyhammer_cli::cli::ConfigAction::Show));
                }
                _ => panic!("Expected Config command"),
            }
        }
    }

    #[test]
    fn test_config_validate_command_exists() {
        // Test that we can parse a config validate command
        let result = Cli::try_parse_from_args([
            "swissarmyhammer", 
            "config", 
            "validate"
        ]);
        
        assert!(result.is_ok(), "Config validate command should be parseable");
    }

    #[test]
    fn test_config_init_command_exists() {
        // Test that we can parse a config init command
        let result = Cli::try_parse_from_args([
            "swissarmyhammer", 
            "config", 
            "init"
        ]);
        
        assert!(result.is_ok(), "Config init command should be parseable");
    }

    #[test]
    fn test_config_guide_command_exists() {
        // Test that we can parse a config guide command
        let result = Cli::try_parse_from_args([
            "swissarmyhammer", 
            "config", 
            "guide"
        ]);
        
        assert!(result.is_ok(), "Config guide command should be parseable");
    }
}

#[cfg(test)]
mod config_initialization_tests {
    use super::*;

    #[test]
    #[serial_test::serial]
    fn test_cli_loads_config_successfully() {
        // Create a temporary directory with a valid config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("swissarmyhammer.yaml");
        
        fs::write(&config_path, r#"
base_branch: "develop"
"#).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clear any environment variables that might interfere
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");

        // Test that Config::new() works with our YAML file
        let config = Config::new();
        assert_eq!(config.base_branch, "develop");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_cli_handles_invalid_config_gracefully() {
        // Create a temporary directory with an invalid config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("swissarmyhammer.yaml");
        
        // Write invalid YAML (empty base_branch)
        fs::write(&config_path, r#"
base_branch: ""
"#).unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Clear any environment variables that might interfere
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
        
        // Clean up any config files that might exist in home directories
        if let Some(home_dir) = dirs::home_dir() {
            let _ = std::fs::remove_file(home_dir.join("swissarmyhammer.yaml"));
            let config_dir = home_dir.join(".config").join("swissarmyhammer");
            let _ = std::fs::remove_file(config_dir.join("swissarmyhammer.yaml"));
        }

        // Test that Config::new() handles invalid config gracefully (falls back)
        let config = Config::new();
        // Should fall back to default value since YAML validation fails
        assert_eq!(config.base_branch, "main");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_cli_uses_env_vars_when_no_yaml() {
        // Test that environment variables work when no YAML file exists
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Set environment variable
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "env-test-branch");

        let config = Config::new();
        assert_eq!(config.base_branch, "env-test-branch");

        // Cleanup
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
        std::env::set_current_dir(original_dir).unwrap();
    }
}

#[cfg(test)]
mod config_command_handler_tests {

    // These tests will initially fail because we haven't implemented the handlers yet
    // But we're writing them first following TDD principles

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_show_displays_current_config() {
        // Test that config show command displays current configuration values
        // This should show base_branch, issue_branch_prefix, etc.
    }

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_validate_reports_valid_config() {
        // Test that config validate reports when configuration is valid
    }

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_validate_reports_invalid_config() {
        // Test that config validate reports configuration errors
    }

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_init_creates_example_file() {
        // Test that config init creates an example configuration file
    }

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_init_fails_if_file_exists() {
        // Test that config init fails if configuration file already exists
    }

    #[test]
    #[ignore] // Will be enabled once we implement the handler
    fn test_config_help_displays_help_text() {
        // Test that config help displays comprehensive help information
    }
}

#[cfg(test)]
mod doctor_config_tests {

    #[test]
    #[ignore] // Will be enabled once we implement doctor config checks
    fn test_doctor_includes_config_validation() {
        // Test that doctor command includes configuration validation checks
    }

    #[test]
    #[ignore] // Will be enabled once we implement doctor config checks  
    fn test_doctor_reports_config_file_status() {
        // Test that doctor command reports on configuration file status
    }
}