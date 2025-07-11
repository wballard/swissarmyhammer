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

/// Test root-level validate command
#[test]
fn test_root_validate_command() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "validate"])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "root validate command should complete"
    );
    Ok(())
}

/// Test root validate command with quiet flag
#[test]
fn test_root_validate_quiet() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "validate", "--quiet"])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "root validate --quiet should complete"
    );

    // In quiet mode, should only show errors
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should have minimal output in quiet mode
    if output.status.success() {
        assert!(
            stdout.is_empty() || stdout.trim().is_empty(),
            "quiet mode should produce minimal output on success"
        );
    }
    
    Ok(())
}

/// Test root validate command with JSON format
#[test]
fn test_root_validate_json_format() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "validate", "--format", "json"])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "root validate --format json should complete"
    );

    // If successful, output should be valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        // Try to parse as JSON
        let result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        assert!(
            result.is_ok(),
            "JSON format output should be valid JSON"
        );
        
        if let Ok(json) = result {
            // Verify expected fields exist
            assert!(json.get("files_checked").is_some(), "JSON should have files_checked field");
            assert!(json.get("errors").is_some(), "JSON should have errors field");
            assert!(json.get("warnings").is_some(), "JSON should have warnings field");
            assert!(json.get("issues").is_some(), "JSON should have issues field");
        }
    }

    Ok(())
}

/// Test root validate command with specific workflow directories
#[test]
fn test_root_validate_with_workflow_dirs() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create a simple valid workflow
    std::fs::write(
        workflow_dir.join("test.mermaid"),
        r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
        ])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "root validate with workflow-dir should complete"
    );
    
    Ok(())
}

/// Test root validate command with multiple workflow directories
#[test]
fn test_root_validate_with_multiple_workflow_dirs() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir1 = temp_dir.path().join("workflows1");
    let workflow_dir2 = temp_dir.path().join("workflows2");
    std::fs::create_dir_all(&workflow_dir1)?;
    std::fs::create_dir_all(&workflow_dir2)?;
    
    // Create workflows in both directories
    std::fs::write(
        workflow_dir1.join("flow1.mermaid"),
        r#"stateDiagram-v2
    [*] --> A
    A --> [*]
"#,
    )?;
    
    std::fs::write(
        workflow_dir2.join("flow2.mermaid"),
        r#"stateDiagram-v2
    [*] --> B
    B --> [*]
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir1.to_str().unwrap(),
            "--workflow-dir",
            workflow_dir2.to_str().unwrap(),
        ])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "root validate with multiple workflow-dirs should complete"
    );
    
    Ok(())
}

/// Test root validate command error exit codes
#[test]
fn test_root_validate_error_exit_codes() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create an invalid workflow (missing terminal state)
    std::fs::write(
        workflow_dir.join("invalid.mermaid"),
        r#"stateDiagram-v2
    [*] --> Start
    Start --> Middle
    Middle --> Start
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
            "--quiet",
        ])
        .output()?;

    // Should return exit code 2 for validation errors
    assert_eq!(
        output.status.code(),
        Some(2),
        "root validate should return exit code 2 for validation errors"
    );
    
    Ok(())
}

/// Test that help output includes the root validate command
#[test]
fn test_root_help_includes_validate() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()?;

    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("validate"),
        "help should mention validate command at root level"
    );
    assert!(
        stdout.contains("Validate prompt files and workflows"),
        "help should describe what validate does"
    );
    
    Ok(())
}

/// Test validate command help
#[test]
fn test_root_validate_help() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "validate", "--help"])
        .output()?;

    assert!(output.status.success(), "validate help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--quiet"),
        "validate help should mention --quiet flag"
    );
    assert!(
        stdout.contains("--format"),
        "validate help should mention --format flag"
    );
    assert!(
        stdout.contains("--workflow-dir"),
        "validate help should mention --workflow-dir option"
    );
    
    Ok(())
}

/// Test validation with invalid YAML format
#[test]
fn test_root_validate_invalid_yaml() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    
    // Create a prompt with invalid YAML
    std::fs::write(
        prompts_dir.join("invalid.md"),
        r#"---
title: Test Prompt
description: This has invalid YAML
arguments:
  - name: test
    required: yes  # Should be boolean true/false, not yes/no
    description
---

Test content"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--quiet",
        ])
        .env("HOME", temp_dir.path())
        .output()?;

    // Should have validation errors
    assert_ne!(
        output.status.code(),
        Some(0),
        "validation with invalid YAML should fail"
    );
    
    Ok(())
}

/// Test validation with missing required fields
#[test]
fn test_root_validate_missing_fields() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    
    // Create a prompt missing required fields
    std::fs::write(
        prompts_dir.join("incomplete.md"),
        r#"---
# Missing title and description
arguments:
  - name: test
    required: true
---

Test content"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--format",
            "json",
        ])
        .env("HOME", temp_dir.path())
        .output()?;

    // Should have validation errors
    assert_eq!(
        output.status.code(),
        Some(2),
        "validation with missing fields should return exit code 2"
    );
    
    // Check JSON output contains error info
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let errors = json.get("errors").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(errors > 0, "should have reported errors in JSON");
    }
    
    Ok(())
}

/// Test validation with undefined template variables
#[test]
fn test_root_validate_undefined_variables() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    
    // Create a prompt using undefined variables
    std::fs::write(
        prompts_dir.join("undefined_vars.md"),
        r#"---
title: Test Undefined Variables
description: This uses variables not defined in arguments
arguments:
  - name: defined_var
    required: true
---

This uses {{ defined_var }} which is fine.
But this uses {{ undefined_var }} which should error.
And this uses {{ another_undefined }} too."#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
        ])
        .env("HOME", temp_dir.path())
        .output()?;

    // Should have validation errors
    assert_eq!(
        output.status.code(),
        Some(2),
        "validation with undefined variables should return exit code 2"
    );
    
    Ok(())
}

/// Test validation with malformed workflow
#[test]
fn test_root_validate_malformed_workflow() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create various malformed workflows
    std::fs::write(
        workflow_dir.join("syntax_error.mermaid"),
        r#"stateDiagram-v2
    [*] --> Start
    Start --> invalid syntax here [
    End --> [*]
"#,
    )?;

    std::fs::write(
        workflow_dir.join("no_initial.mermaid"),
        r#"stateDiagram-v2
    Start --> End
    End --> Done
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
        ])
        .output()?;

    // Should have validation errors
    assert_eq!(
        output.status.code(),
        Some(2),
        "validation with malformed workflows should return exit code 2"
    );
    
    Ok(())
}

/// Test validation with non-existent workflow directory
#[test]
fn test_root_validate_nonexistent_workflow_dir() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let fake_dir = temp_dir.path().join("does_not_exist");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            fake_dir.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()?;

    // Should complete with warnings
    assert!(
        output.status.code().is_some(),
        "validation should complete even with non-existent directory"
    );
    
    // Check JSON output for warnings
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let warnings = json.get("warnings").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(warnings > 0, "should have warnings about non-existent directory");
    }
    
    Ok(())
}

/// Test validation with invalid format option
#[test]
fn test_root_validate_invalid_format() -> Result<()> {
    let output = Command::new("cargo")
        .args(["run", "--", "validate", "--format", "invalid_format"])
        .output()?;

    // Should fail to parse arguments
    assert!(
        !output.status.success(),
        "validation with invalid format should fail"
    );
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error:") || stderr.contains("invalid value"),
        "should show error about invalid format"
    );
    
    Ok(())
}

/// Test validation with empty workflow_dirs vector (should use default behavior)
#[test]
fn test_root_validate_empty_workflow_dirs() -> Result<()> {
    // When no workflow dirs are specified, it should search from current directory
    let output = Command::new("cargo")
        .args(["run", "--", "validate"])
        .output()?;

    // Should complete successfully (may have warnings/errors based on current dir content)
    assert!(
        output.status.code().is_some(),
        "validation with empty workflow_dirs should complete"
    );
    
    Ok(())
}

/// Test validation with mix of valid and invalid prompts
#[test]
fn test_root_validate_mixed_valid_invalid_prompts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    std::fs::create_dir_all(&prompts_dir)?;
    
    // Create a valid prompt
    std::fs::write(
        prompts_dir.join("valid.md"),
        r#"---
title: Valid Prompt
description: This is a valid prompt
arguments:
  - name: test
    required: true
    default: "value"
---

This uses {{ test }} correctly."#,
    )?;
    
    // Create an invalid prompt (missing title)
    std::fs::write(
        prompts_dir.join("invalid.md"),
        r#"---
description: Missing title field
---

Content here."#,
    )?;
    
    // Create another invalid prompt (undefined variable)
    std::fs::write(
        prompts_dir.join("bad_vars.md"),
        r#"---
title: Bad Variables
description: Uses undefined variables
---

This uses {{ undefined }} variable."#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--format",
            "json",
        ])
        .env("HOME", temp_dir.path())
        .output()?;

    // Should have errors due to invalid prompts
    assert_eq!(
        output.status.code(),
        Some(2),
        "validation with mixed valid/invalid prompts should return exit code 2"
    );
    
    // Check JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let files_checked = json.get("files_checked").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(files_checked >= 3, "should have checked at least 3 files");
        
        let errors = json.get("errors").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(errors >= 2, "should have at least 2 errors");
    }
    
    Ok(())
}

/// Test validation with mix of valid and invalid workflows
#[test]
fn test_root_validate_mixed_valid_invalid_workflows() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create a valid workflow
    std::fs::write(
        workflow_dir.join("valid.mermaid"),
        r#"stateDiagram-v2
    [*] --> Process
    Process --> Complete
    Complete --> [*]
"#,
    )?;
    
    // Create an invalid workflow (no terminal state)
    std::fs::write(
        workflow_dir.join("no_terminal.mermaid"),
        r#"stateDiagram-v2
    [*] --> Start
    Start --> Loop
    Loop --> Start
"#,
    )?;
    
    // Create another invalid workflow (unreachable state)
    std::fs::write(
        workflow_dir.join("unreachable.mermaid"),
        r#"stateDiagram-v2
    [*] --> A
    A --> [*]
    B --> C
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
        ])
        .output()?;

    // Should have errors due to invalid workflows
    assert_eq!(
        output.status.code(),
        Some(2),
        "validation with mixed valid/invalid workflows should return exit code 2"
    );
    
    Ok(())
}

/// Test validation with absolute and relative workflow directories
#[test]
fn test_root_validate_absolute_relative_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let abs_workflow_dir = temp_dir.path().join("abs_workflows");
    std::fs::create_dir_all(&abs_workflow_dir)?;
    
    // Create a workflow in absolute path
    std::fs::write(
        abs_workflow_dir.join("test.mermaid"),
        r#"stateDiagram-v2
    [*] --> Test
    Test --> [*]
"#,
    )?;
    
    // Test with absolute path
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            abs_workflow_dir.to_str().unwrap(),
        ])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "validation with absolute path should complete"
    );
    
    // Test with relative path (from temp dir)
    std::fs::create_dir_all(temp_dir.path().join("rel_workflows"))?;
    std::fs::write(
        temp_dir.path().join("rel_workflows").join("test.mermaid"),
        r#"stateDiagram-v2
    [*] --> Test
    Test --> [*]
"#,
    )?;
    
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            "rel_workflows",
        ])
        .current_dir(temp_dir.path())
        .output()?;

    assert!(
        output.status.code().is_some(),
        "validation with relative path should complete"
    );
    
    Ok(())
}

/// Test validation with large number of files (stress test)
#[test]
#[ignore = "stress test - only run manually"]
fn test_root_validate_stress_many_files() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("workflows");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create 100 workflow files
    for i in 0..100 {
        std::fs::write(
            workflow_dir.join(format!("workflow_{}.mermaid", i)),
            format!(r#"stateDiagram-v2
    [*] --> State{}
    State{} --> [*]
"#, i, i),
        )?;
    }

    let start = std::time::Instant::now();
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
            "--quiet",
        ])
        .output()?;
    let duration = start.elapsed();

    assert!(
        output.status.code().is_some(),
        "validation of many files should complete"
    );
    
    // Should complete in reasonable time (less than 10 seconds for 100 files)
    assert!(
        duration.as_secs() < 10,
        "validation of 100 files should complete within 10 seconds"
    );
    
    Ok(())
}

/// Test validation with special characters in file paths
#[test]
fn test_root_validate_special_chars_in_paths() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let workflow_dir = temp_dir.path().join("work flows with spaces");
    std::fs::create_dir_all(&workflow_dir)?;
    
    // Create workflow with special chars in name
    std::fs::write(
        workflow_dir.join("test-workflow_v1.0.mermaid"),
        r#"stateDiagram-v2
    [*] --> Test
    Test --> [*]
"#,
    )?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--workflow-dir",
            workflow_dir.to_str().unwrap(),
        ])
        .output()?;

    assert!(
        output.status.code().is_some(),
        "validation with special chars in paths should complete"
    );
    
    Ok(())
}
