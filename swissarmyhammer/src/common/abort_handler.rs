//! Abort error handling utilities
//!
//! This module provides functionality to detect "ABORT ERROR" patterns in output
//! and trigger clean shutdown with proper error handling.

use crate::workflow::ActionError;

/// Pattern to match for abort errors in output text
pub const ABORT_ERROR_PATTERN: &str = "ABORT ERROR";

/// Pattern to match for Claude execution failures that should be treated as abort errors
pub const CLAUDE_EXECUTION_FAILED_PATTERN: &str =
    "Failed: Claude command failed: Claude execution failed";

/// Check if output contains an abort error pattern
///
/// Scans the given text for "ABORT ERROR" (case-sensitive) and the specific
/// Claude execution failure pattern "Failed: Claude command failed: Claude execution failed".
/// Returns an ActionError::AbortError if either pattern is found.
///
/// # Arguments
/// * `output` - The text to scan for abort errors
///
/// # Returns
/// * `Ok(())` if no abort error is found
/// * `Err(ActionError::AbortError)` if "ABORT ERROR" or Claude execution failure is found in the output
pub fn check_for_abort_error(output: &str) -> Result<(), ActionError> {
    if output.contains(ABORT_ERROR_PATTERN) {
        tracing::error!("Detected ABORT ERROR in output, triggering immediate shutdown");
        return Err(ActionError::AbortError(format!(
            "Found {} in output: {}",
            ABORT_ERROR_PATTERN,
            extract_abort_context(output)
        )));
    }

    if output.contains(CLAUDE_EXECUTION_FAILED_PATTERN) {
        tracing::error!("Detected Claude execution failure, triggering immediate shutdown");
        return Err(ActionError::AbortError(format!(
            "Claude execution failed - treating as ABORT ERROR: {}",
            extract_abort_context(output)
        )));
    }

    Ok(())
}

/// Extract context around the abort error for better error reporting
///
/// Returns the line containing the abort error plus some surrounding context
/// to help with debugging. Handles both ABORT ERROR and Claude execution failure patterns.
fn extract_abort_context(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains(ABORT_ERROR_PATTERN) || line.contains(CLAUDE_EXECUTION_FAILED_PATTERN) {
            // Get up to 2 lines before and after for context
            let start = i.saturating_sub(2);
            let end = if i + 3 < lines.len() {
                i + 3
            } else {
                lines.len()
            };

            let context_lines = &lines[start..end];
            return context_lines.join("\n");
        }
    }

    // If we somehow get here, just return a truncated version
    if output.len() > 200 {
        format!("{}...", &output[..200])
    } else {
        output.to_string()
    }
}

/// Check for abort error and handle appropriately based on compilation mode
///
/// This function provides different behavior based on compilation mode:
/// - In production/release: Immediately calls `std::process::exit(2)` for hard shutdown if error found
/// - In test mode: Returns ActionError::AbortError for testability
///
/// # Arguments
/// * `output` - The text to scan for abort errors
///
/// # Returns
/// * `Ok(())` if no abort error is found
/// * `Err(ActionError::AbortError)` if abort error found in test mode
/// * Never returns in production mode if abort error found (calls exit)
pub fn check_for_abort_error_and_exit(output: &str) -> Result<(), ActionError> {
    match check_for_abort_error(output) {
        Ok(()) => Ok(()),
        Err(action_error) => {
            if let ActionError::AbortError(msg) = &action_error {
                // In test mode, return the error for testability
                #[cfg(test)]
                {
                    #[allow(unused_variables)]
                    let _ = msg; // Variable used only in non-test builds
                    return Err(action_error);
                }

                // In all non-test builds (including debug), exit immediately
                #[cfg(not(test))]
                {
                    tracing::error!("ABORT ERROR detected - immediate shutdown: {}", msg);
                    std::process::exit(2);
                }
            }

            // This should never be reached in non-test builds due to exit above
            #[cfg(test)]
            return Err(action_error);

            #[cfg(not(test))]
            unreachable!("Should have exited above")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_abort_error() {
        let output = "This is normal output without any problems";
        assert!(check_for_abort_error(output).is_ok());
    }

    #[test]
    fn test_abort_error_detected() {
        let output = "Something went wrong\nABORT ERROR: Critical failure detected\nShutting down";
        let result = check_for_abort_error(output);
        assert!(result.is_err());

        if let Err(ActionError::AbortError(msg)) = result {
            assert!(msg.contains("ABORT ERROR"));
            assert!(msg.contains("Critical failure detected"));
        } else {
            panic!("Expected AbortError");
        }
    }

    #[test]
    fn test_abort_error_case_sensitive() {
        // Should not match lowercase
        let output = "abort error: something failed";
        assert!(check_for_abort_error(output).is_ok());

        // Should match exact case
        let output = "ABORT ERROR: something failed";
        assert!(check_for_abort_error(output).is_err());
    }

    #[test]
    fn test_extract_abort_context() {
        let output = "Line 1\nLine 2\nLine 3\nABORT ERROR: Failed here\nLine 5\nLine 6\nLine 7";
        let context = extract_abort_context(output);

        // Should include lines around the abort error
        assert!(context.contains("Line 2"));
        assert!(context.contains("ABORT ERROR: Failed here"));
        assert!(context.contains("Line 6"));
    }

    #[test]
    fn test_extract_abort_context_beginning() {
        let output = "ABORT ERROR: Failed at start\nLine 2\nLine 3";
        let context = extract_abort_context(output);

        // Should handle abort at beginning properly
        assert!(context.contains("ABORT ERROR: Failed at start"));
        assert!(context.contains("Line 2"));
        assert!(context.contains("Line 3"));
    }

    #[test]
    fn test_extract_abort_context_end() {
        let output = "Line 1\nLine 2\nABORT ERROR: Failed at end";
        let context = extract_abort_context(output);

        // Should handle abort at end properly
        assert!(context.contains("Line 1"));
        assert!(context.contains("Line 2"));
        assert!(context.contains("ABORT ERROR: Failed at end"));
    }

    #[test]
    fn test_extract_long_output() {
        let long_output = "x".repeat(300);
        let output = format!("Some text\n{long_output}\nABORT ERROR: failed");
        let context = extract_abort_context(&output);

        // Should contain the abort error
        assert!(context.contains("ABORT ERROR"));
        // Context should include surrounding lines but may not be shorter than original
        // if the abort error is at the end (which includes some long lines)
    }

    #[test]
    fn test_claude_execution_failed_pattern() {
        let output = "Failed: Claude command failed: Claude execution failed";
        let result = check_for_abort_error(output);
        assert!(result.is_err());

        if let Err(ActionError::AbortError(msg)) = result {
            assert!(msg.contains("Claude execution failed"));
        } else {
            panic!("Expected AbortError for Claude execution failure");
        }
    }

    #[test]
    fn test_claude_execution_failed_pattern_in_context() {
        let output = "Starting workflow\nProcessing request\nFailed: Claude command failed: Claude execution failed\nShutting down";
        let result = check_for_abort_error(output);
        assert!(result.is_err());

        if let Err(ActionError::AbortError(msg)) = result {
            assert!(msg.contains("Claude execution failed"));
            assert!(msg.contains("Processing request"));
        } else {
            panic!("Expected AbortError for Claude execution failure");
        }
    }

    #[test]
    fn test_similar_but_different_claude_errors() {
        // These should NOT trigger abort error
        let similar_errors = vec![
            "Failed: Claude command failed: Connection timeout",
            "Error: Claude command failed: Invalid API key",
            "Failed: Claude execution timeout",
            "Claude command failed: Rate limit exceeded",
        ];

        for error_output in similar_errors {
            let result = check_for_abort_error(error_output);
            assert!(
                result.is_ok(),
                "Should not trigger abort for: {error_output}"
            );
        }
    }

    #[test]
    fn test_check_for_abort_error_and_exit_in_test_mode() {
        // In test mode, should return errors instead of exiting
        let output_with_abort = "ABORT ERROR: Something went wrong";
        let result = check_for_abort_error_and_exit(output_with_abort);
        assert!(result.is_err(), "Should return error in test mode");

        if let Err(ActionError::AbortError(msg)) = result {
            assert!(msg.contains("ABORT ERROR"));
        } else {
            panic!("Expected AbortError in test mode");
        }

        // Should still work for normal output
        let normal_output = "Everything is fine";
        let result = check_for_abort_error_and_exit(normal_output);
        assert!(result.is_ok(), "Should return Ok for normal output");
    }
}
