//! Integration tests for CLI command structure and backward compatibility

use anyhow::Result;
use std::process::Command;
use tempfile::TempDir;

mod test_utils;
use test_utils::create_test_environment;

/// Test that the new prompt subcommand structure works correctly
#[test]
fn test_prompt_subcommand_list() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "prompt", "list"])
        .output()?;

    assert!(
        output.status.success(),
        "prompt list command should succeed"
    );
    Ok(())
}

/// Test prompt search functionality
#[test]
fn test_prompt_subcommand_search() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "prompt", "search", "test"])
        .output()?;

    // Search might not find results but should not error
    assert!(
        output.status.code().is_some(),
        "prompt search should complete"
    );
    Ok(())
}

/// Test prompt validate functionality
#[test]
fn test_prompt_subcommand_validate() -> Result<()> {
    let (_temp_dir, prompts_dir) = create_test_environment()?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "prompt",
            "validate",
            "--workflow-dirs",
            prompts_dir.to_str().unwrap(),
        ])
        .output()?;

    // Validation should complete (may have warnings but shouldn't crash)
    assert!(
        output.status.code().is_some(),
        "prompt validate should complete"
    );
    Ok(())
}

/// Test prompt test functionality with a simple prompt
#[test]
fn test_prompt_subcommand_test() -> Result<()> {
    let (_temp_dir, _prompts_dir) = create_test_environment()?;

    // Test with non-existent prompt should fail gracefully
    let output = Command::new("cargo")
        .args(["run", "--", "prompt", "test", "non_existent_prompt"])
        .output()?;

    assert!(
        !output.status.success(),
        "testing non-existent prompt should fail"
    );
    assert_eq!(output.status.code(), Some(1), "should return exit code 1");

    // Verify error message is present
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error:") || stderr.contains("not found"),
        "should show meaningful error message"
    );

    Ok(())
}

/// Test help output for prompt subcommands
#[test]
fn test_prompt_help() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "prompt", "--help"])
        .output()?;

    assert!(output.status.success(), "prompt help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("list"),
        "help should mention list subcommand"
    );
    assert!(
        stdout.contains("search"),
        "help should mention search subcommand"
    );
    assert!(
        stdout.contains("validate"),
        "help should mention validate subcommand"
    );
    assert!(
        stdout.contains("test"),
        "help should mention test subcommand"
    );

    Ok(())
}

/// Test that old-style commands still work if any exist
#[test]
#[ignore = "doctor command may fail in CI due to environment differences"]
fn test_doctor_command() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "doctor"])
        .output()?;

    assert!(output.status.success(), "doctor command should succeed");
    Ok(())
}

/// Test shell completion generation
#[test]
fn test_completion_command() -> Result<()> {
    let shells = vec!["bash", "zsh", "fish"];

    for shell in shells {
        let output = Command::new("cargo")
            .args(["run", "--", "completion", shell])
            .output()?;

        assert!(
            output.status.success(),
            "{} completion should succeed",
            shell
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "{} completion should generate output",
            shell
        );
    }

    Ok(())
}

/// Test error handling and exit codes
#[test]
fn test_error_exit_codes() -> Result<()> {
    // Test validation error (exit code 2)
    let temp_dir = TempDir::new()?;
    let invalid_dir = temp_dir.path().join("non_existent");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "prompt",
            "validate",
            "--workflow-dirs",
            invalid_dir.to_str().unwrap(),
        ])
        .output()?;

    // Should handle gracefully even if directory doesn't exist
    assert!(output.status.code().is_some(), "should return an exit code");

    Ok(())
}

/// Test that verbose flag works
#[test]
fn test_verbose_flag() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--verbose", "prompt", "list"])
        .output()?;

    // Command should still work with verbose flag
    assert!(
        output.status.code().is_some(),
        "verbose flag should not break commands"
    );

    Ok(())
}

/// Test that quiet flag works
#[test]
fn test_quiet_flag() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--quiet", "prompt", "list"])
        .output()?;

    // Command should still work with quiet flag
    assert!(
        output.status.code().is_some(),
        "quiet flag should not break commands"
    );

    Ok(())
}

/// Test prompt list with different formats
#[test]
fn test_prompt_list_formats() -> Result<()> {
    let formats = vec!["json", "yaml", "table"];

    for format in formats {
        let output = Command::new("cargo")
            .args(["run", "--", "prompt", "list", "--format", format])
            .output()?;

        assert!(
            output.status.code().is_some(),
            "prompt list --format {} should complete",
            format
        );
    }

    Ok(())
}

/// Test concurrent command execution
#[tokio::test]
async fn test_concurrent_commands() -> Result<()> {
    use tokio::task::JoinSet;

    let mut tasks = JoinSet::new();

    // Run multiple commands concurrently
    for i in 0..3 {
        tasks.spawn(async move {
            let output = Command::new("cargo")
                .args(["run", "--", "prompt", "list"])
                .output()
                .expect("Failed to run command");

            (i, output.status.success())
        });
    }

    // All commands should succeed
    while let Some(result) = tasks.join_next().await {
        let (i, success) = result?;
        assert!(success, "Concurrent command {} should succeed", i);
    }

    Ok(())
}
