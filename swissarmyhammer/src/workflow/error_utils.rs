//! Shared error handling utilities for command execution
//!
//! This module provides common error handling patterns to reduce code duplication
//! across the workflow execution system.

use crate::workflow::actions::{ActionError, ActionResult};
use std::process::Output;

/// Handle command execution results with consistent error formatting
///
/// This function standardizes error handling for command execution across the codebase.
/// It checks if the command succeeded and formats errors consistently.
///
/// # Arguments
/// * `result` - The output from a command execution
/// * `command_name` - The name of the command for error messages
///
/// # Returns
/// * `Ok(String)` - The stdout as a string if the command succeeded
/// * `Err(ActionError)` - A formatted error if the command failed
///
/// # Examples
/// ```rust
/// use std::process::Command;
/// use swissarmyhammer::workflow::handle_command_error;
///
/// let output = Command::new("echo").arg("hello").output().unwrap();
/// let result = handle_command_error(output, "echo");
/// assert!(result.is_ok());
/// ```
pub fn handle_command_error(result: Output, command_name: &str) -> ActionResult<String> {
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(ActionError::ExecutionError(format!(
            "{} command failed: {}",
            command_name, stderr
        )));
    }
    Ok(String::from_utf8_lossy(&result.stdout).into_owned())
}

/// Handle command execution results with custom error types
///
/// This function provides the same error handling pattern but allows for custom error types.
/// Useful when you need to integrate with different error systems.
///
/// # Arguments
/// * `result` - The output from a command execution
/// * `command_name` - The name of the command for error messages
/// * `error_mapper` - A function to map the error message to a custom error type
///
/// # Returns
/// * `Ok(String)` - The stdout as a string if the command succeeded
/// * `Err(E)` - A custom error if the command failed
pub fn handle_command_error_with_mapper<E>(
    result: Output,
    command_name: &str,
    error_mapper: impl FnOnce(String) -> E,
) -> Result<String, E> {
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(error_mapper(format!(
            "{} command failed: {}",
            command_name, stderr
        )));
    }
    Ok(String::from_utf8_lossy(&result.stdout).into_owned())
}

/// Check if a command execution succeeded
///
/// This is a simple helper to check command success without processing the output.
///
/// # Arguments
/// * `result` - The output from a command execution
///
/// # Returns
/// * `true` if the command succeeded
/// * `false` if the command failed
pub fn command_succeeded(result: &Output) -> bool {
    result.status.success()
}

/// Extract stderr as a string from command output
///
/// This helper function safely extracts stderr from command output as a UTF-8 string.
///
/// # Arguments
/// * `result` - The output from a command execution
///
/// # Returns
/// * String containing the stderr output
pub fn extract_stderr(result: &Output) -> String {
    String::from_utf8_lossy(&result.stderr).into_owned()
}

/// Extract stdout as a string from command output
///
/// This helper function safely extracts stdout from command output as a UTF-8 string.
///
/// # Arguments
/// * `result` - The output from a command execution
///
/// # Returns
/// * String containing the stdout output
pub fn extract_stdout(result: &Output) -> String {
    String::from_utf8_lossy(&result.stdout).into_owned()
}

/// Handle Claude command execution results with appropriate error type
///
/// This function provides specialized error handling for Claude command execution,
/// returning ActionError::ClaudeError for command failures.
///
/// # Arguments
/// * `result` - The output from a Claude command execution
///
/// # Returns
/// * `Ok(String)` - The stdout as a string if the command succeeded
/// * `Err(ActionError)` - A ClaudeError if the command failed
pub fn handle_claude_command_error(result: Output) -> ActionResult<String> {
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(ActionError::ClaudeError(format!(
            "Claude command failed: {}",
            stderr
        )));
    }
    Ok(String::from_utf8_lossy(&result.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn test_handle_command_error_success() {
        let output = Command::new("echo")
            .arg("hello")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let result = handle_command_error(output, "echo");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello");
    }

    #[test]
    fn test_handle_command_error_failure() {
        let output = Command::new("false")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let result = handle_command_error(output, "false");
        assert!(result.is_err());
        if let Err(ActionError::ExecutionError(msg)) = result {
            assert!(msg.contains("false command failed"));
        } else {
            panic!("Expected ExecutionError");
        }
    }

    #[test]
    fn test_command_succeeded() {
        let success_output = Command::new("echo")
            .arg("test")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let failure_output = Command::new("false")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        assert!(command_succeeded(&success_output));
        assert!(!command_succeeded(&failure_output));
    }

    #[test]
    fn test_extract_stdout_stderr() {
        let output = Command::new("echo")
            .arg("hello")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let stdout = extract_stdout(&output);
        let stderr = extract_stderr(&output);

        assert_eq!(stdout.trim(), "hello");
        assert_eq!(stderr.trim(), "");
    }

    #[test]
    fn test_handle_command_error_with_mapper() {
        let output = Command::new("false")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let result = handle_command_error_with_mapper(output, "false", |msg| {
            format!("Custom error: {}", msg)
        });

        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Custom error: false command failed"));
    }

    #[test]
    fn test_handle_claude_command_error_success() {
        let output = Command::new("echo")
            .arg("test")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let result = handle_claude_command_error(output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "test");
    }

    #[test]
    fn test_handle_claude_command_error_failure() {
        let output = Command::new("false")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let result = handle_claude_command_error(output);
        assert!(result.is_err());
        if let Err(ActionError::ClaudeError(msg)) = result {
            assert!(msg.contains("Claude command failed"));
        } else {
            panic!("Expected ClaudeError");
        }
    }
}
