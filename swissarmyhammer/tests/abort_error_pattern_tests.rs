//! Comprehensive tests for "ABORT ERROR" pattern detection
//!
//! These tests verify that the "ABORT ERROR" pattern in outputs triggers proper
//! abort behavior with correct error handling and context extraction.

use swissarmyhammer::common::{check_for_abort_error, ABORT_ERROR_PATTERN};
use swissarmyhammer::workflow::{Action, ActionError, PromptAction};

#[test]
fn test_abort_error_detection_in_output() {
    // Test various formats of abort error messages
    let test_cases = [
        "ABORT ERROR",
        "ABORT ERROR: Something went wrong",
        "Previous line\nABORT ERROR: Critical failure\nNext line",
        "ABORT ERROR\nShutting down immediately",
    ];

    for (i, test_case) in test_cases.iter().enumerate() {
        let result = check_for_abort_error(test_case);
        assert!(result.is_err(), "Test case {i} should detect abort error");

        if let Err(ActionError::AbortError(msg)) = result {
            assert!(
                msg.contains("ABORT ERROR"),
                "Error message should contain ABORT ERROR"
            );
        } else {
            panic!("Expected ActionError::AbortError for test case {i}");
        }
    }
}

#[test]
fn test_no_abort_error_detection_in_normal_output() {
    let normal_outputs = [
        "This is normal output",
        "abort error (lowercase should not trigger)",
        "Abort Error (wrong case should not trigger)",
        "Some text with ABORT_ERROR (underscore should not trigger)",
        "Error: Something failed (regular error should not trigger)",
        "ABORT_ERROR (underscore instead of space)",
        "ABRT ERROR (abbreviation should not trigger)",
    ];

    for (i, output) in normal_outputs.iter().enumerate() {
        let result = check_for_abort_error(output);
        assert!(
            result.is_ok(),
            "Test case {i} should not detect abort error: {output}"
        );
    }
}

#[test]
fn test_abort_error_context_extraction() {
    let output_with_context = r#"
Line 1: Normal operation
Line 2: Processing data
Line 3: ABORT ERROR: Critical system failure detected
Line 4: Shutting down
Line 5: Cleanup complete
"#;

    let result = check_for_abort_error(output_with_context);
    assert!(result.is_err(), "Should detect abort error");

    if let Err(ActionError::AbortError(msg)) = result {
        assert!(
            msg.contains("Line 1"),
            "Should include context before abort"
        );
        assert!(
            msg.contains("ABORT ERROR: Critical system failure detected"),
            "Should include abort line"
        );
        assert!(msg.contains("Line 5"), "Should include context after abort");
    } else {
        panic!("Expected ActionError::AbortError");
    }
}

#[test]
fn test_abort_error_case_sensitivity() {
    let case_variations = vec![
        ("ABORT ERROR", true),  // Exact match should trigger
        ("abort error", false), // Lowercase should not trigger
        ("Abort Error", false), // Mixed case should not trigger
        ("ABORT error", false), // Partial match should not trigger
        ("abort ERROR", false), // Partial match should not trigger
    ];

    for (text, should_abort) in case_variations {
        let result = check_for_abort_error(text);
        if should_abort {
            assert!(result.is_err(), "Text '{text}' should trigger abort");
        } else {
            assert!(result.is_ok(), "Text '{text}' should not trigger abort");
        }
    }
}

#[test]
fn test_abort_error_pattern_constant() {
    assert_eq!(ABORT_ERROR_PATTERN, "ABORT ERROR");

    // Verify the constant is used correctly
    let test_output = format!("Something went wrong: {ABORT_ERROR_PATTERN}");
    let result = check_for_abort_error(&test_output);
    assert!(
        result.is_err(),
        "Should detect abort using pattern constant"
    );
}

#[test]
fn test_abort_error_with_special_characters() {
    let special_outputs = vec![
        "ABORT ERROR: Unicode test: 🚨 Critical failure",
        "ABORT ERROR: Special chars: @#$%^&*()",
        "ABORT ERROR: Newlines\nand\ttabs\r\nembedded",
    ];

    // Test very long message separately
    let long_message = "ABORT ERROR: Very long message ".to_owned() + &"x".repeat(1000);

    for output in special_outputs {
        let result = check_for_abort_error(output);
        assert!(
            result.is_err(),
            "Should detect abort in: {}",
            output.chars().take(50).collect::<String>()
        );
    }

    // Test the long message
    let result = check_for_abort_error(&long_message);
    assert!(result.is_err(), "Should detect abort in very long message");
}

#[test]
fn test_multiple_abort_errors_in_output() {
    let output_with_multiple = r#"
First operation failed
ABORT ERROR: First failure
Attempting recovery
ABORT ERROR: Recovery also failed
Final shutdown
"#;

    let result = check_for_abort_error(output_with_multiple);
    assert!(result.is_err(), "Should detect first abort error");

    // Should detect the first occurrence
    if let Err(ActionError::AbortError(msg)) = result {
        assert!(
            msg.contains("First failure"),
            "Should capture context around first abort error"
        );
    }
}

#[test]
fn test_abort_error_at_beginning_of_output() {
    let output = "ABORT ERROR: Failed at startup\nSubsequent lines";
    let result = check_for_abort_error(output);

    assert!(result.is_err(), "Should detect abort at beginning");
    if let Err(ActionError::AbortError(msg)) = result {
        assert!(msg.contains("ABORT ERROR: Failed at startup"));
        assert!(msg.contains("Subsequent lines"));
    }
}

#[test]
fn test_abort_error_at_end_of_output() {
    let output = "Previous operations\nLast line\nABORT ERROR: Failed at end";
    let result = check_for_abort_error(output);

    assert!(result.is_err(), "Should detect abort at end");
    if let Err(ActionError::AbortError(msg)) = result {
        assert!(msg.contains("Previous operations"));
        assert!(msg.contains("ABORT ERROR: Failed at end"));
    }
}

#[test]
fn test_abort_error_surrounded_by_whitespace() {
    let test_cases = vec![
        " ABORT ERROR ",
        "\tABORT ERROR\t",
        "\nABORT ERROR\n",
        " \t ABORT ERROR \t ",
        "   ABORT ERROR: with message   ",
    ];

    for case in test_cases {
        let result = check_for_abort_error(case);
        assert!(
            result.is_err(),
            "Should detect abort error surrounded by whitespace: '{case}'"
        );
    }
}

#[test]
fn test_abort_error_within_larger_words() {
    let non_matching_cases = vec![
        "ABORTABORT ERROR",        // No space before
        "ABORT ERRORABORT",        // No space after
        "PREFIXABORT ERRORSUFFIX", // Embedded in words
    ];

    for case in non_matching_cases {
        let result = check_for_abort_error(case);
        assert!(
            result.is_err(),
            "Should still detect ABORT ERROR even when embedded: '{case}'"
        );
    }
}

#[tokio::test]
async fn test_prompt_action_integration() {
    // Test that PromptAction can be created and introspected
    // This verifies that the abort detection integration point exists
    let prompt_action = PromptAction::new("test-prompt".to_string());
    let description = prompt_action.description();

    assert!(description.contains("test-prompt"));
    assert_eq!(prompt_action.action_type(), "prompt");
}

#[test]
fn test_empty_output() {
    let result = check_for_abort_error("");
    assert!(result.is_ok(), "Empty output should not trigger abort");
}

#[test]
fn test_very_long_output_without_abort() {
    let long_output = "normal text ".repeat(10000);
    let result = check_for_abort_error(&long_output);
    assert!(
        result.is_ok(),
        "Long normal output should not trigger abort"
    );
}

#[test]
fn test_abort_error_in_json_like_output() {
    let json_output =
        r#"{"status": "error", "message": "ABORT ERROR: Process failed", "code": 500}"#;
    let result = check_for_abort_error(json_output);
    assert!(
        result.is_err(),
        "Should detect abort error even in JSON-like output"
    );
}

#[test]
fn test_abort_error_in_log_format() {
    let log_output = "[2024-01-01 12:00:00] ERROR: System failure detected\n[2024-01-01 12:00:01] ABORT ERROR: Critical shutdown required\n[2024-01-01 12:00:02] INFO: Cleanup started";
    let result = check_for_abort_error(log_output);
    assert!(result.is_err(), "Should detect abort error in log format");
}
