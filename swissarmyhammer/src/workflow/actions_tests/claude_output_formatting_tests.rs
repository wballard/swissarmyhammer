//! Tests for Claude output formatting functionality

use crate::workflow::actions::format_claude_output_as_yaml;

#[test]
fn test_format_claude_output_as_yaml() {
    // Test JSON object formatting
    let json_line = r#"{"type":"user","message":{"role":"user","content":[{"tool_use_id":"toolu_01HHxb5NnvSfUxwCTQptWWaE","type":"tool_result","content":"Test content"}]},"parent_tool_use_id":null,"session_id":"e99afa02-75bc-4f2f-baef-68d5e071f023"}"#;

    let formatted = format_claude_output_as_yaml(json_line);

    // The output should be in YAML format
    assert!(formatted.contains("type: user"));
    assert!(formatted.contains("message:"));
    assert!(formatted.contains("role: user"));
    assert!(formatted.contains("content:"));
    assert!(formatted.contains("tool_use_id: toolu_01HHxb5NnvSfUxwCTQptWWaE"));
    assert!(formatted.contains("type: tool_result"));
    assert!(formatted.contains("content: Test content"));
    assert!(formatted.contains("parent_tool_use_id: null"));
    assert!(formatted.contains("session_id: e99afa02-75bc-4f2f-baef-68d5e071f023"));
}

#[test]
fn test_format_claude_output_as_yaml_invalid_json() {
    // Test that invalid JSON returns the original string
    let invalid_json = "not a json string";
    let formatted = format_claude_output_as_yaml(invalid_json);
    assert_eq!(formatted, invalid_json);
}

#[test]
fn test_format_claude_output_as_yaml_empty_string() {
    // Test empty string
    let empty = "";
    let formatted = format_claude_output_as_yaml(empty);
    assert_eq!(formatted, empty);
}

#[test]
fn test_format_claude_output_as_yaml_whitespace() {
    // Test whitespace-only string
    let whitespace = "   \n   ";
    let formatted = format_claude_output_as_yaml(whitespace);
    assert_eq!(formatted, whitespace.trim());
}

#[test]
fn test_format_claude_output_as_yaml_nested_objects() {
    // Test deeply nested JSON object
    let nested_json = r#"{"level1":{"level2":{"level3":{"value":"deep"}}}}"#;
    let formatted = format_claude_output_as_yaml(nested_json);

    assert!(formatted.contains("level1:"));
    assert!(formatted.contains("level2:"));
    assert!(formatted.contains("level3:"));
    assert!(formatted.contains("value: deep"));
}

#[test]
fn test_format_claude_output_as_yaml_arrays() {
    // Test JSON with arrays
    let json_with_array = r#"{"items":["one","two","three"],"count":3}"#;
    let formatted = format_claude_output_as_yaml(json_with_array);

    assert!(formatted.contains("items:"));
    assert!(formatted.contains("- one"));
    assert!(formatted.contains("- two"));
    assert!(formatted.contains("- three"));
    assert!(formatted.contains("count: 3"));
}
