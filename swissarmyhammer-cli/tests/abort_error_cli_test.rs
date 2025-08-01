//! Integration tests for CLI ABORT ERROR handling
//!
//! This test verifies that when the CLI encounters output containing "ABORT ERROR",
//! it properly exits with a non-zero exit code (EXIT_ERROR = 2).

use anyhow::Result;
use assert_cmd::Command;

/// Test that the abort.md prompt triggers proper ABORT ERROR handling
#[test]
fn test_abort_prompt_exits_with_error_code() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["prompt", "test", "abort"])
        .output()?;

    // The command should fail with exit code 2 (EXIT_ERROR)
    assert!(
        !output.status.success(),
        "Command should fail when ABORT ERROR is detected"
    );

    // Verify the exit code is specifically EXIT_ERROR (2)
    assert_eq!(
        output.status.code(),
        Some(2),
        "Exit code should be 2 (EXIT_ERROR) when ABORT ERROR is detected"
    );

    // Verify that the error message contains "ABORT ERROR"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ABORT ERROR"),
        "Error output should contain 'ABORT ERROR': {stderr}"
    );

    Ok(())
}

/// Test that the abort prompt with --raw flag also triggers ABORT ERROR handling
#[test]
fn test_abort_prompt_raw_exits_with_error_code() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["prompt", "test", "abort", "--raw"])
        .output()?;

    // The command should fail with exit code 2 (EXIT_ERROR)
    assert!(
        !output.status.success(),
        "Raw command should fail when ABORT ERROR is detected"
    );

    // Verify the exit code is specifically EXIT_ERROR (2)
    assert_eq!(
        output.status.code(),
        Some(2),
        "Exit code should be 2 (EXIT_ERROR) when ABORT ERROR is detected in raw mode"
    );

    Ok(())
}

/// Test that normal prompts still work correctly (don't trigger ABORT ERROR)
#[test]
fn test_normal_prompt_succeeds() -> Result<()> {
    // Create a simple test by running a known prompt that should not contain ABORT ERROR
    // We'll use the prompt list command as a baseline
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["prompt", "list", "--format", "json"])
        .output()?;

    // This command should succeed
    assert!(
        output.status.success(),
        "Normal prompt commands should succeed: stderr = {}",
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(())
}

/// Test that we can detect ABORT ERROR in different cases within prompt output
#[test]
fn test_abort_error_variations() -> Result<()> {
    // Test the exact abort prompt which should contain "ABORT ERROR"
    let output = Command::cargo_bin("swissarmyhammer")
        .unwrap()
        .args(["prompt", "test", "abort", "--raw"])
        .output()?;

    // Verify the stdout contains the expected abort instruction
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("ABORT ERROR"),
        "Abort prompt output should contain 'ABORT ERROR': {stdout}"
    );

    // The command should still fail due to ABORT ERROR detection
    assert!(
        !output.status.success(),
        "Command should fail when output contains ABORT ERROR"
    );
    assert_eq!(output.status.code(), Some(2), "Should exit with code 2");

    Ok(())
}
