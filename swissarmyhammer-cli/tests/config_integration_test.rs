//! Integration tests for CLI configuration management
//!
//! Tests the integration of the YAML configuration system with the CLI layer,
//! ensuring that configuration is properly loaded, validated, and accessible
//! throughout the CLI commands.

use std::fs;
use swissarmyhammer::config::Config;
use swissarmyhammer_cli::cli::{Cli, Commands};
use tempfile::TempDir;

#[cfg(test)]
mod config_cli_tests {
    use super::*;

    #[test]
    fn test_config_show_command_exists() {
        // Test that we can parse a config show command
        let result = Cli::try_parse_from_args(["swissarmyhammer", "config", "show"]);

        assert!(result.is_ok(), "Config show command should be parseable");

        if let Ok(cli) = result {
            match cli.command {
                Some(Commands::Config { action }) => {
                    // This will fail initially because we haven't implemented it yet
                    // but we're writing the test first (TDD)
                    assert!(matches!(
                        action,
                        swissarmyhammer_cli::cli::ConfigAction::Show
                    ));
                }
                _ => panic!("Expected Config command"),
            }
        }
    }

    #[test]
    fn test_config_validate_command_exists() {
        // Test that we can parse a config validate command
        let result = Cli::try_parse_from_args(["swissarmyhammer", "config", "validate"]);

        assert!(
            result.is_ok(),
            "Config validate command should be parseable"
        );
    }

    #[test]
    fn test_config_init_command_exists() {
        // Test that we can parse a config init command
        let result = Cli::try_parse_from_args(["swissarmyhammer", "config", "init"]);

        assert!(result.is_ok(), "Config init command should be parseable");
    }

    #[test]
    fn test_config_guide_command_exists() {
        // Test that we can parse a config guide command
        let result = Cli::try_parse_from_args(["swissarmyhammer", "config", "guide"]);

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

        fs::write(
            &config_path,
            r#"
base_branch: "develop"
"#,
        )
        .unwrap();

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
        fs::write(
            &config_path,
            r#"
base_branch: ""
"#,
        )
        .unwrap();

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
    use super::*;

    // These tests will initially fail because we haven't implemented the handlers yet
    // But we're writing them first following TDD principles

    #[test]
    #[serial_test::serial]
    fn test_config_show_displays_current_config() {
        // Test that config show command displays current configuration values
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("show").output().unwrap();

        assert!(output.status.success(), "Config show should succeed");
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should display configuration values
        assert!(stdout.contains("📋 Current Configuration:"));
        assert!(stdout.contains("base_branch:"));
        assert!(stdout.contains("issue_branch_prefix:"));
    }

    #[test]
    #[serial_test::serial]
    fn test_config_validate_reports_valid_config() {
        // Test that config validate reports when configuration is valid
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("validate").output().unwrap();

        assert!(output.status.success(), "Config validate should succeed");
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should report configuration is valid
        assert!(stdout.contains("✅ Configuration is valid"));
    }

    #[test]
    #[serial_test::serial]
    fn test_config_validate_reports_invalid_config() {
        // Test that config validate reports configuration errors
        use assert_cmd::Command;

        // Set invalid environment variable to create invalid final configuration
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "");

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("validate").output().unwrap();

        assert!(
            !output.status.success(),
            "Config validate should fail with invalid config"
        );
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should report validation failure
        assert!(stdout.contains("❌ Configuration validation failed"));

        // Clean up environment variable
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_init_creates_example_file() {
        // Test that config init creates an example configuration file
        use assert_cmd::Command;

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("init").output().unwrap();

        assert!(output.status.success(), "Config init should succeed");
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should report successful creation
        assert!(stdout.contains("✅ Created example configuration file"));

        // File should exist
        assert!(temp_dir.path().join("swissarmyhammer.yaml").exists());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_config_init_fails_if_file_exists() {
        // Test that config init fails if configuration file already exists
        use assert_cmd::Command;

        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create existing config file
        let config_path = temp_dir.path().join("swissarmyhammer.yaml");
        fs::write(&config_path, "base_branch: main").unwrap();

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("init").output().unwrap();

        assert!(
            !output.status.success(),
            "Config init should fail when file exists"
        );
        let stderr = String::from_utf8(output.stderr).unwrap();

        // Should report that file already exists
        assert!(stderr.contains("❌ Configuration file already exists"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_config_guide_displays_help_text() {
        // Test that config guide displays comprehensive help information
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("config").arg("guide").output().unwrap();

        assert!(output.status.success(), "Config guide should succeed");
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should display comprehensive help
        assert!(stdout.contains("📖 Configuration Help"));
        assert!(stdout.contains("SwissArmyHammer supports configuration via"));
        assert!(stdout.contains("Configuration file locations searched"));
        assert!(stdout.contains("Example configuration file"));
    }
}

#[cfg(test)]
mod doctor_config_tests {

    #[test]
    #[serial_test::serial]
    fn test_doctor_includes_config_validation() {
        // Test that doctor command includes configuration validation checks
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("doctor").output().unwrap();

        // Doctor may exit with non-zero status due to warnings, which is expected behavior
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should include configuration checks
        assert!(
            stdout.contains("Configuration:") || stdout.contains("SwissArmyHammer configuration"),
            "Doctor output should mention configuration: {}",
            stdout
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_doctor_reports_config_file_status() {
        // Test that doctor command reports on configuration file status
        use assert_cmd::Command;

        let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
        let output = cmd.arg("doctor").output().unwrap();

        // Doctor may exit with non-zero status due to warnings, which is expected behavior
        let stdout = String::from_utf8(output.stdout).unwrap();

        // Should report on config file status (either found or not found)
        assert!(
            stdout.contains("Configuration:")
                || stdout.contains("SwissArmyHammer configuration")
                || stdout.contains("configuration file"),
            "Doctor output should report configuration status: {}",
            stdout
        );
    }
}
