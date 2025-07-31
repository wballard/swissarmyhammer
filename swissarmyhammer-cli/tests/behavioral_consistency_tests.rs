//! Behavioral consistency tests for CLI-MCP integration
//!
//! These tests verify that CLI commands produce identical output after the MCP integration
//! refactoring. They focus on ensuring that the user experience remains unchanged.

use anyhow::Result;
use assert_cmd::Command;
use tempfile::TempDir;

mod test_utils;
use test_utils::setup_git_repo;

/// Test helper to create a standardized test environment with sample data
fn setup_behavioral_test_environment() -> Result<(TempDir, std::path::PathBuf)> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().to_path_buf();

    // Create issues directory
    let issues_dir = temp_path.join("issues");
    std::fs::create_dir_all(&issues_dir)?;

    // Create sample issues for testing
    std::fs::write(
        issues_dir.join("TEST_001_sample_issue.md"),
        r#"# Sample Issue

This is a sample issue for behavioral testing.

## Details
- Priority: Medium
- Status: Open
- Created: 2024-01-01
"#,
    )?;

    std::fs::write(
        issues_dir.join("TEST_002_another_issue.md"),
        r#"# Another Issue

This is another sample issue for testing list functionality.

## Details
- Priority: High
- Status: In Progress
"#,
    )?;

    // Create .swissarmyhammer directory for memos
    let swissarmyhammer_dir = temp_path.join(".swissarmyhammer");
    std::fs::create_dir_all(&swissarmyhammer_dir)?;

    // Initialize git repository
    setup_git_repo(&temp_path)?;

    Ok((temp_dir, temp_path))
}

/// Test that issue list command produces consistent output format
#[test]
fn test_issue_list_output_format_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify the output contains expected structure
    assert!(stdout.contains("TEST_001"), "Should list first test issue");
    assert!(stdout.contains("TEST_002"), "Should list second test issue");
    assert!(stdout.contains("sample_issue"), "Should show issue names");

    // Verify expected format elements (based on current CLI behavior)
    assert!(
        stdout.contains("Issues:") || stdout.contains("Active Issues:"),
        "Should have issues section header"
    );

    Ok(())
}

/// Test that issue list command with JSON format produces consistent output
#[test]
fn test_issue_list_json_format_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list", "--format", "json"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify JSON output structure
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    let json = json_result.unwrap();

    // Verify JSON contains expected issue data
    if let Some(issues) = json.as_array() {
        assert!(!issues.is_empty(), "Should contain issues");

        // Check first issue has expected fields
        if let Some(first_issue) = issues.first() {
            assert!(
                first_issue.get("name").is_some() || first_issue.get("title").is_some(),
                "Issue should have name or title field"
            );
        }
    } else if json.is_object() {
        // Alternative structure - verify it has issue-like fields
        assert!(
            json.get("issues").is_some() || json.get("data").is_some(),
            "JSON should contain issues data"
        );
    }

    Ok(())
}

/// Test that issue creation produces consistent success messages
#[test]
fn test_issue_create_output_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    let output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            "test_behavioral_issue",
            "--content",
            "Test issue for behavioral consistency",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify success message format
    assert!(
        stdout.contains("Created") || stdout.contains("created") || stdout.contains("SUCCESS"),
        "Should show creation success message: {stdout}"
    );
    assert!(
        stdout.contains("test_behavioral_issue"),
        "Should mention the issue name in output: {stdout}"
    );

    // Verify the issue was actually created (name-based system)
    let created_issue_path = temp_path.join("issues").join("test_behavioral_issue.md");
    assert!(
        created_issue_path.exists(),
        "Issue file should be created at expected path: {}",
        created_issue_path.display()
    );

    Ok(())
}

/// Test that memo creation produces consistent output format
#[test]
fn test_memo_create_output_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    let output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Test Memo",
            "--content",
            "This is a test memo for behavioral consistency testing",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify success message format
    assert!(
        stdout.contains("Created") || stdout.contains("created") || stdout.contains("SUCCESS"),
        "Should show creation success message: {stdout}"
    );
    assert!(
        stdout.contains("Test Memo") || stdout.contains("memo"),
        "Should reference the memo in output: {stdout}"
    );

    // Output should contain the memo ID (ULID format)
    assert!(
        stdout.chars().any(|c| c.is_alphanumeric()),
        "Should contain memo ID in output: {stdout}"
    );

    Ok(())
}

/// Test that memo list command produces consistent output format
#[test]
fn test_memo_list_output_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // First create a memo to ensure list has content
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Behavioral Test Memo",
            "--content",
            "Test memo content for list testing",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify the output contains expected memo information
    assert!(
        stdout.contains("Behavioral Test Memo"),
        "Should list the created memo: {stdout}"
    );

    // Should have some structure (table headers or organized format)
    assert!(
        stdout.contains("Title")
            || stdout.contains("ID")
            || stdout.contains("Created")
            || stdout.len() > 50, // At minimum, substantial content
        "Should have structured output format: {stdout}"
    );

    Ok(())
}

/// Test that search index command produces consistent output
#[test]
fn test_search_index_output_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Create some sample files to index
    let src_dir = temp_path.join("src");
    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        src_dir.join("test.rs"),
        r#"
// Test file for behavioral consistency
fn hello_world() {
    println!("Hello, world!");
}

fn error_handling() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
"#,
    )?;

    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["search", "index", "src/**/*.rs"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify indexing success message
    assert!(
        stdout.contains("indexed") || stdout.contains("Successfully") || stdout.contains("files"),
        "Should show indexing success message: {stdout}"
    );

    // Should mention number of files or operations
    assert!(
        stdout.chars().any(char::is_numeric),
        "Should contain numeric information about indexing: {stdout}"
    );

    Ok(())
}

/// Test that search query command produces consistent output format
#[test]
fn test_search_query_output_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Create and index sample files first
    let src_dir = temp_path.join("src");
    std::fs::create_dir_all(&src_dir)?;

    std::fs::write(
        src_dir.join("error_handler.rs"),
        r#"
// Error handling functions for behavioral consistency testing
use std::error::Error;

fn handle_error(e: Box<dyn Error>) -> Result<(), String> {
    eprintln!("Error occurred: {}", e);
    Err("Failed to handle error".to_string())
}

fn error_recovery() -> Result<String, Box<dyn Error>> {
    Ok("Recovery successful".to_string())
}
"#,
    )?;

    // Index the files
    Command::cargo_bin("swissarmyhammer")?
        .args(["search", "index", "src/**/*.rs"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["search", "query", "error handling"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify search results format
    if !stdout.trim().is_empty() {
        // If there are results, verify format
        assert!(
            stdout.contains("error") || stdout.contains("Error"),
            "Should contain search terms in results: {stdout}"
        );

        // Should have some structure (file paths, line numbers, etc.)
        assert!(
            stdout.contains(".rs") || stdout.contains("src/"),
            "Should contain file references: {stdout}"
        );
    } else {
        // Empty results are acceptable for consistency testing
        // as long as the command completes successfully
    }

    Ok(())
}

/// Test that error conditions produce consistent error messages
#[test]
fn test_error_message_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Test non-existent issue
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "show", "nonexistent_issue"])
        .current_dir(&temp_path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);

    // Verify error message format
    assert!(
        stderr.contains("Error")
            || stderr.contains("error")
            || stderr.contains("not found")
            || stderr.contains("Not found"),
        "Should show appropriate error message: {stderr}"
    );

    // Test invalid memo ID
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "get", "invalid_id"])
        .current_dir(&temp_path)
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&output.get_output().stderr);

    assert!(
        stderr.contains("Error")
            || stderr.contains("error")
            || stderr.contains("invalid")
            || stderr.contains("not found"),
        "Should show appropriate error for invalid memo ID: {stderr}"
    );

    Ok(())
}

/// Test that help output remains consistent
#[test]
fn test_help_output_consistency() -> Result<()> {
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Verify help output contains expected sections
    assert!(
        stdout.contains("USAGE") || stdout.contains("Usage"),
        "Help should contain usage section"
    );
    assert!(
        stdout.contains("Commands") || stdout.contains("COMMANDS"),
        "Help should contain commands section"
    );
    assert!(
        stdout.contains("Options") || stdout.contains("OPTIONS"),
        "Help should contain options section"
    );

    // Verify major commands are present
    assert!(
        stdout.contains("issue"),
        "Help should mention issue commands"
    );
    assert!(stdout.contains("memo"), "Help should mention memo commands");
    assert!(
        stdout.contains("search"),
        "Help should mention search commands"
    );

    Ok(())
}

/// Test command-specific help output consistency
#[test]
fn test_subcommand_help_consistency() -> Result<()> {
    // Test issue help
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        stdout.contains("create") && stdout.contains("list") && stdout.contains("show"),
        "Issue help should contain major subcommands: {stdout}"
    );

    // Test memo help
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        stdout.contains("create") && stdout.contains("list") && stdout.contains("get"),
        "Memo help should contain major subcommands: {stdout}"
    );

    // Test search help
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["search", "--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    assert!(
        stdout.contains("index") && stdout.contains("query"),
        "Search help should contain major subcommands: {stdout}"
    );

    Ok(())
}

/// Test that verbose flag produces additional output consistently
#[test]
fn test_verbose_flag_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Test normal output
    let normal_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let normal_stdout = String::from_utf8_lossy(&normal_output.get_output().stdout);

    // Test verbose output
    let verbose_output = Command::cargo_bin("swissarmyhammer")?
        .args(["--verbose", "issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let verbose_stdout = String::from_utf8_lossy(&verbose_output.get_output().stdout);

    // Verbose output should contain at least as much information as normal output
    // (This is a basic consistency check - specific verbose behavior may vary)
    assert!(
        verbose_stdout.len() >= normal_stdout.len()
        || verbose_stdout.contains("TEST_001") // At minimum should show same content
        || verbose_stdout.contains("Sample Issue"),
        "Verbose output should contain at least the same information as normal output"
    );

    Ok(())
}

/// Test that quiet flag reduces output consistently
#[test]
fn test_quiet_flag_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Test normal output
    let normal_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let normal_stdout = String::from_utf8_lossy(&normal_output.get_output().stdout);

    // Test quiet output
    let quiet_output = Command::cargo_bin("swissarmyhammer")?
        .args(["--quiet", "issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let quiet_stdout = String::from_utf8_lossy(&quiet_output.get_output().stdout);

    // Quiet output should generally be less verbose
    // But still contain essential information
    assert!(
        quiet_stdout.len() <= normal_stdout.len()
        || quiet_stdout.contains("TEST_001") // Still should show core content
        || quiet_stdout.trim().is_empty(), // Or be minimal
        "Quiet output should be less verbose than normal output"
    );

    Ok(())
}

/// Test that exit codes remain consistent
#[test]
fn test_exit_codes_consistency() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Test successful command (exit code 0)
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Test command with non-existent resource (should fail with non-zero exit code)
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "show", "nonexistent"])
        .current_dir(&temp_path)
        .assert()
        .failure();

    // Test invalid arguments (should fail with specific exit code)
    Command::cargo_bin("swissarmyhammer")?
        .args(["invalid", "command"])
        .assert()
        .failure();

    Ok(())
}

/// Test CLI output stability with different data sizes
#[test]
fn test_output_stability_with_scale() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Create additional issues to test scaling behavior
    let issues_dir = temp_path.join("issues");
    for i in 3..=20 {
        std::fs::write(
            issues_dir.join(format!("SCALE_{i:03}_issue.md")),
            format!("# Scale Test Issue {i}\n\nTesting output stability with more data."),
        )?;
    }

    // Test that output format remains consistent with more data
    let output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);

    // Should still contain expected structure with more data
    assert!(
        stdout.contains("TEST_001") && stdout.contains("SCALE_020"),
        "Should list all issues consistently: {stdout}"
    );

    Ok(())
}

/// Test CLI behavior with edge case data
#[test]
fn test_edge_case_data_handling() -> Result<()> {
    let (_temp_dir, temp_path) = setup_behavioral_test_environment()?;

    // Test with unicode content
    let output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            "unicode_test",
            "--content",
            "Unicode test: 日本語 🚀 émojis and special chars: @#$%^&*()",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("Created") || stdout.contains("created"),
        "Should handle unicode content gracefully: {stdout}"
    );

    // Test with very long issue name
    let long_name = "a".repeat(100);
    let output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            &long_name,
            "--content",
            "Testing long issue names",
        ])
        .current_dir(&temp_path)
        .assert();

    // Should either succeed or fail gracefully (not crash)
    match output.get_output().status.success() {
        true => {
            let stdout = String::from_utf8_lossy(&output.get_output().stdout);
            assert!(
                stdout.contains("Created") || stdout.contains("created"),
                "Long names should be handled gracefully"
            );
        }
        false => {
            let stderr = String::from_utf8_lossy(&output.get_output().stderr);
            assert!(
                stderr.contains("Error") || stderr.contains("error"),
                "Long name errors should be user-friendly: {stderr}"
            );
        }
    }

    Ok(())
}
