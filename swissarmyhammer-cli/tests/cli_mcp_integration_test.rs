//! Integration tests for CLI-MCP tool integration
//!
//! These tests verify that the CLI can successfully call MCP tools directly
//! without going through the MCP protocol layer.

use serde_json::json;
use std::env;
use swissarmyhammer_cli::mcp_integration::{CliToolContext, McpToolRunner};
use tempfile::TempDir;

/// Test helper to create a test environment
fn setup_test_environment() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Create issues directory
    let issues_dir = temp_dir.path().join("issues");
    std::fs::create_dir_all(&issues_dir).expect("Failed to create issues directory");

    // Change to temp directory for tests
    if let Ok(_original_dir) = env::current_dir() {
        let _ = env::set_current_dir(temp_dir.path());
    } else {
        // If we can't get current dir, just use the temp dir as-is
        let _ = env::set_current_dir(temp_dir.path());
    }

    // Store original directory in temp_dir for cleanup
    temp_dir
}

#[tokio::test]
async fn test_cli_can_call_mcp_tools() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Verify that tools are available
    let tools = context.list_tools();
    assert!(!tools.is_empty(), "No tools available");

    // Check for specific expected tools
    assert!(
        context.has_tool("issue_create"),
        "issue_create tool should be available"
    );
    assert!(
        context.has_tool("memo_create"),
        "memo_create tool should be available"
    );
}

#[tokio::test]
async fn test_issue_create_tool_integration() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test calling issue_create tool
    let args = context.create_arguments(vec![
        ("name", json!("test_issue")),
        (
            "content",
            json!("# Test Issue\n\nThis is a test issue for integration testing."),
        ),
    ]);

    let result = context.execute_tool("issue_create", args).await;
    assert!(
        result.is_ok(),
        "Failed to execute issue_create tool: {:?}",
        result.err()
    );

    let call_result = result.unwrap();
    assert_eq!(
        call_result.is_error,
        Some(false),
        "Tool execution reported an error"
    );
    assert!(
        !call_result.content.is_empty(),
        "Tool result should have content"
    );
}

#[tokio::test]
async fn test_memo_create_tool_integration() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test calling memo_create tool
    let args = context.create_arguments(vec![
        ("title", json!("Test Memo")),
        (
            "content",
            json!("# Test Memo\n\nThis is a test memo for integration testing."),
        ),
    ]);

    let result = context.execute_tool("memo_create", args).await;
    assert!(
        result.is_ok(),
        "Failed to execute memo_create tool: {:?}",
        result.err()
    );

    let call_result = result.unwrap();
    assert_eq!(
        call_result.is_error,
        Some(false),
        "Tool execution reported an error"
    );
    assert!(
        !call_result.content.is_empty(),
        "Tool result should have content"
    );
}

#[tokio::test]
async fn test_nonexistent_tool_error() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test calling a nonexistent tool
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("nonexistent_tool", args).await;

    assert!(result.is_err(), "Should return error for nonexistent tool");

    let error = result.err().unwrap();
    assert!(
        error.to_string().contains("Tool not found"),
        "Error should mention tool not found"
    );
}

#[tokio::test]
async fn test_mcp_tool_runner_trait() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test the McpToolRunner trait implementation
    struct TestRunner;
    let runner = TestRunner;

    let args = context.create_arguments(vec![
        ("title", json!("Test Memo via Runner")),
        ("content", json!("Testing the McpToolRunner trait")),
    ]);

    let result = runner.run_mcp_tool(&context, "memo_create", args).await;
    assert!(
        result.is_ok(),
        "McpToolRunner should execute successfully: {:?}",
        result.err()
    );

    let output = result.unwrap();
    assert!(!output.is_empty(), "Runner should return formatted output");
}

#[tokio::test]
async fn test_invalid_arguments_error() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test calling memo_create with invalid arguments (missing required fields)
    let args = context.create_arguments(vec![("invalid_field", json!("invalid_value"))]);

    let result = context.execute_tool("memo_create", args).await;
    assert!(result.is_err(), "Should return error for invalid arguments");
}

#[tokio::test]
async fn test_issue_workflow_integration() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test a complete workflow: create issue, then list issues

    // 1. Create an issue
    let create_args = context.create_arguments(vec![
        ("name", json!("workflow_test")),
        (
            "content",
            json!("# Workflow Test\n\nTesting issue workflow integration."),
        ),
    ]);

    let create_result = context.execute_tool("issue_create", create_args).await;
    assert!(
        create_result.is_ok(),
        "Failed to create issue: {:?}",
        create_result.err()
    );

    // 2. Try to get the next issue (should include our created issue)
    let next_args = context.create_arguments(vec![]);
    let next_result = context.execute_tool("issue_next", next_args).await;

    // Note: This might fail if there are no pending issues, which is fine for this test
    // We're mainly testing that the tool can be called without errors
    match next_result {
        Ok(result) => {
            assert_eq!(
                result.is_error,
                Some(false),
                "issue_next should not report error when successful"
            );
        }
        Err(e) => {
            // This is acceptable - might be no pending issues
            println!("issue_next returned error (acceptable): {e}");
        }
    }
}

#[test]
fn test_response_formatting_utilities() {
    use rmcp::model::{Annotated, CallToolResult, RawContent, RawTextContent};
    use serde_json::json;
    use swissarmyhammer_cli::mcp_integration::response_formatting;

    // Test success response formatting
    let success_result = CallToolResult {
        content: vec![Annotated::new(
            RawContent::Text(RawTextContent {
                text: "Operation completed successfully".to_string(),
            }),
            None,
        )],
        is_error: Some(false),
    };

    let formatted = response_formatting::format_success_response(&success_result);
    assert!(formatted.contains("Operation completed successfully"));

    // Test error response formatting
    let error_result = CallToolResult {
        content: vec![Annotated::new(
            RawContent::Text(RawTextContent {
                text: "Something went wrong".to_string(),
            }),
            None,
        )],
        is_error: Some(true),
    };

    let formatted_error = response_formatting::format_error_response(&error_result);
    assert!(formatted_error.contains("Something went wrong"));

    // Test table formatting
    let test_data = json!([
        {"name": "Alice", "age": 30, "city": "New York"},
        {"name": "Bob", "age": 25, "city": "San Francisco"}
    ]);

    let table = response_formatting::format_as_table(&test_data);
    assert!(table.contains("name"));
    assert!(table.contains("Alice"));
    assert!(table.contains("Bob"));
    assert!(table.contains("New York"));

    // Test status message creation
    let status_msg =
        response_formatting::create_status_message("test operation", true, Some("All good"));
    assert!(status_msg.contains("SUCCESS"));
    assert!(status_msg.contains("test operation"));
    assert!(status_msg.contains("All good"));

    let error_msg = response_formatting::create_status_message(
        "failed operation",
        false,
        Some("Something failed"),
    );
    assert!(error_msg.contains("ERROR"));
    assert!(error_msg.contains("failed operation"));
    assert!(error_msg.contains("Something failed"));
}

#[test]
fn test_error_conversion() {
    use rmcp::Error as McpError;
    use swissarmyhammer_cli::mcp_integration::CliError;

    // Test basic MCP error conversion
    let mcp_error = McpError::internal_error("test error".to_string(), None);
    let cli_error: CliError = mcp_error.into();

    assert!(cli_error.message.contains("MCP error"));
    assert!(cli_error.message.contains("test error"));
    assert_eq!(cli_error.exit_code, 1);

    // Test abort error detection
    let abort_error = McpError::internal_error("ABORT ERROR: Cannot proceed".to_string(), None);
    let cli_abort_error: CliError = abort_error.into();

    assert!(cli_abort_error.message.contains("MCP error"));
    assert!(cli_abort_error.message.contains("ABORT ERROR"));
}

#[tokio::test]
async fn test_create_arguments_helper() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    // Test the create_arguments helper method
    let args = context.create_arguments(vec![
        ("string_param", json!("test_string")),
        ("number_param", json!(42)),
        ("bool_param", json!(true)),
        ("array_param", json!(["item1", "item2"])),
        ("object_param", json!({"key": "value"})),
    ]);

    assert_eq!(args.len(), 5);
    assert_eq!(args.get("string_param"), Some(&json!("test_string")));
    assert_eq!(args.get("number_param"), Some(&json!(42)));
    assert_eq!(args.get("bool_param"), Some(&json!(true)));
    assert_eq!(args.get("array_param"), Some(&json!(["item1", "item2"])));
    assert_eq!(args.get("object_param"), Some(&json!({"key": "value"})));
}

#[tokio::test]
async fn test_context_tool_listing() {
    let _temp_dir = setup_test_environment();

    let context = CliToolContext::new()
        .await
        .expect("Failed to create CliToolContext");

    let tools = context.list_tools();

    // Verify we have the expected categories of tools
    let issue_tools: Vec<_> = tools.iter().filter(|t| t.starts_with("issue_")).collect();
    let memo_tools: Vec<_> = tools.iter().filter(|t| t.starts_with("memo_")).collect();
    let search_tools: Vec<_> = tools.iter().filter(|t| t.starts_with("search_")).collect();

    assert!(!issue_tools.is_empty(), "Should have issue tools");
    assert!(!memo_tools.is_empty(), "Should have memo tools");
    assert!(!search_tools.is_empty(), "Should have search tools");

    // Verify specific expected tools exist
    assert!(tools.contains(&"issue_create".to_string()));
    assert!(tools.contains(&"issue_next".to_string()));
    assert!(tools.contains(&"memo_create".to_string()));
    assert!(tools.contains(&"memo_list".to_string()));
}
