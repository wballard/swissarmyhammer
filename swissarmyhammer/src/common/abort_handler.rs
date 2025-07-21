//! Abort error handling utilities
//!
//! This module provides functionality to detect "ABORT ERROR" patterns in output
//! and trigger clean shutdown with proper error handling.

use crate::workflow::ActionError;

/// Pattern to match for abort errors in output text
pub const ABORT_ERROR_PATTERN: &str = "ABORT ERROR";

/// Check if output contains an abort error pattern
///
/// Scans the given text for "ABORT ERROR" (case-sensitive) and returns
/// an ActionError::AbortError if found.
///
/// # Arguments
/// * `output` - The text to scan for abort errors
///
/// # Returns
/// * `Ok(())` if no abort error is found
/// * `Err(ActionError::AbortError)` if "ABORT ERROR" is found in the output
pub fn check_for_abort_error(output: &str) -> Result<(), ActionError> {
    if output.contains(ABORT_ERROR_PATTERN) {
        tracing::error!("Detected ABORT ERROR in output, triggering immediate shutdown");
        return Err(ActionError::AbortError(format!(
            "Found {} in output: {}",
            ABORT_ERROR_PATTERN,
            extract_abort_context(output)
        )));
    }
    Ok(())
}

/// Extract context around the abort error for better error reporting
///
/// Returns the line containing the abort error plus some surrounding context
/// to help with debugging.
fn extract_abort_context(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains(ABORT_ERROR_PATTERN) {
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
}
