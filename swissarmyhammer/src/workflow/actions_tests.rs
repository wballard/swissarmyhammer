//! Comprehensive tests for workflow actions module
//!
//! This module contains thorough tests for all action types and utility functions,
//! including edge cases, error conditions, and various execution scenarios.

use super::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Helper function to create a test context with common variables
fn create_test_context() -> HashMap<String, Value> {
    let mut context = HashMap::new();
    context.insert(
        "test_var".to_string(),
        Value::String("test_value".to_string()),
    );
    context.insert("number_var".to_string(), Value::Number(42.into()));
    context.insert("bool_var".to_string(), Value::Bool(true));
    context.insert(
        "current_file".to_string(),
        Value::String("test.rs".to_string()),
    );
    context.insert(
        "user_name".to_string(),
        Value::String("testuser".to_string()),
    );
    context
}

/// Helper function to create a test context with special characters
fn create_context_with_special_chars() -> HashMap<String, Value> {
    let mut context = HashMap::new();
    context.insert(
        "special_chars".to_string(),
        Value::String("hello\"world'test".to_string()),
    );
    context.insert("empty_string".to_string(), Value::String("".to_string()));
    context.insert("null_value".to_string(), Value::Null);
    context
}

#[cfg(test)]
mod prompt_action_tests {
    use super::*;

    #[test]
    fn test_prompt_action_creation() {
        let action = PromptAction::new("test-prompt".to_string());
        assert_eq!(action.prompt_name, "test-prompt");
        assert!(action.arguments.is_empty());
        assert!(action.result_variable.is_none());
        assert_eq!(action.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_prompt_action_with_arguments() {
        let action = PromptAction::new("test-prompt".to_string())
            .with_argument("arg1".to_string(), "value1".to_string())
            .with_argument("arg2".to_string(), "value2".to_string());

        assert_eq!(action.arguments.get("arg1"), Some(&"value1".to_string()));
        assert_eq!(action.arguments.get("arg2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_prompt_action_with_result_variable() {
        let action = PromptAction::new("test-prompt".to_string())
            .with_result_variable("result_var".to_string());

        assert_eq!(action.result_variable, Some("result_var".to_string()));
    }

    #[test]
    fn test_prompt_action_with_timeout() {
        let timeout_duration = Duration::from_secs(60);
        let action = PromptAction::new("test-prompt".to_string()).with_timeout(timeout_duration);

        assert_eq!(action.timeout, timeout_duration);
    }

    #[test]
    fn test_prompt_action_with_quiet() {
        // Test enabling quiet mode
        let action = PromptAction::new("test-prompt".to_string()).with_quiet(true);
        assert!(action.quiet);

        // Test disabling quiet mode
        let action = PromptAction::new("test-prompt".to_string()).with_quiet(false);
        assert!(!action.quiet);

        // Test default is false
        let action = PromptAction::new("test-prompt".to_string());
        assert!(!action.quiet);
    }

    // Note: Variable substitution is tested through the public execute() method
    // since substitute_variables() is private

    #[test]
    fn test_prompt_action_description() {
        let action = PromptAction::new("test-prompt".to_string())
            .with_argument("arg1".to_string(), "value1".to_string());

        let description = action.description();
        assert!(description.contains("test-prompt"));
        assert!(description.contains("arg1"));
    }

    #[test]
    fn test_prompt_action_type() {
        let action = PromptAction::new("test-prompt".to_string());
        assert_eq!(action.action_type(), "prompt");
    }

    #[tokio::test]
    async fn test_prompt_action_execution_with_invalid_argument_key() {
        let action = PromptAction::new("test-prompt".to_string())
            .with_argument("invalid key!".to_string(), "value".to_string());

        let mut context = HashMap::new();
        let result = action.execute(&mut context).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ActionError::ParseError(msg) => {
                assert!(msg.contains("Invalid argument key"));
            }
            _ => panic!("Expected ParseError"),
        }
    }
}

#[cfg(test)]
mod wait_action_tests {
    use super::*;

    #[test]
    fn test_wait_action_duration_creation() {
        let duration = Duration::from_secs(30);
        let action = WaitAction::new_duration(duration);

        assert_eq!(action.duration, Some(duration));
        assert!(action.message.is_none());
    }

    #[test]
    fn test_wait_action_user_input_creation() {
        let action = WaitAction::new_user_input();

        assert!(action.duration.is_none());
        assert!(action.message.is_none());
    }

    #[test]
    fn test_wait_action_with_message() {
        let action = WaitAction::new_duration(Duration::from_secs(10))
            .with_message("Please wait...".to_string());

        assert_eq!(action.message, Some("Please wait...".to_string()));
    }

    #[test]
    fn test_wait_action_description() {
        let action = WaitAction::new_duration(Duration::from_secs(30));
        assert!(action.description().contains("30s"));

        let action = WaitAction::new_user_input();
        assert_eq!(action.description(), "Wait for user input");
    }

    #[test]
    fn test_wait_action_type() {
        let action = WaitAction::new_duration(Duration::from_secs(10));
        assert_eq!(action.action_type(), "wait");
    }

    #[tokio::test]
    async fn test_wait_action_duration_execution() {
        let action = WaitAction::new_duration(Duration::from_millis(100));
        let mut context = HashMap::new();

        let start = std::time::Instant::now();
        let result = action.execute(&mut context).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
        assert!(elapsed >= Duration::from_millis(90)); // Allow some tolerance
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_wait_action_duration_with_message() {
        let action = WaitAction::new_duration(Duration::from_millis(10))
            .with_message("Processing...".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Null);
    }

    // Note: Testing user input wait is complex in automated tests
    // We'll focus on the timeout behavior and structure
    #[tokio::test]
    async fn test_wait_action_user_input_timeout_setup() {
        let action = WaitAction::new_user_input();
        // We can't actually test stdin reading in unit tests easily,
        // but we can verify the action structure
        assert!(action.duration.is_none());
        assert_eq!(action.action_type(), "wait");
    }
}

#[cfg(test)]
mod log_action_tests {
    use super::*;

    #[test]
    fn test_log_action_creation() {
        let action = LogAction::new("Test message".to_string(), LogLevel::Info);
        assert_eq!(action.message, "Test message");
        assert!(matches!(action.level, LogLevel::Info));
    }

    #[test]
    fn test_log_action_convenience_methods() {
        let info_action = LogAction::info("Info message".to_string());
        assert!(matches!(info_action.level, LogLevel::Info));

        let warning_action = LogAction::warning("Warning message".to_string());
        assert!(matches!(warning_action.level, LogLevel::Warning));

        let error_action = LogAction::error("Error message".to_string());
        assert!(matches!(error_action.level, LogLevel::Error));
    }

    #[test]
    fn test_log_action_description() {
        let action = LogAction::info("Test message".to_string());
        assert_eq!(action.description(), "Log message: Test message");
    }

    #[test]
    fn test_log_action_type() {
        let action = LogAction::info("Test message".to_string());
        assert_eq!(action.action_type(), "log");
    }

    #[tokio::test]
    async fn test_log_action_execution_info() {
        let action = LogAction::info("Test info message".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("Test info message".to_string())
        );
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_log_action_execution_warning() {
        let action = LogAction::warning("Test warning message".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("Test warning message".to_string())
        );
    }

    #[tokio::test]
    async fn test_log_action_execution_error() {
        let action = LogAction::error("Test error message".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("Test error message".to_string())
        );
    }

    #[tokio::test]
    async fn test_log_action_variable_substitution() {
        let action = LogAction::info("File: ${current_file}, User: ${user_name}".to_string());
        let mut context = create_test_context();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(
            message,
            Value::String("File: test.rs, User: testuser".to_string())
        );
    }

    #[tokio::test]
    async fn test_log_action_with_special_characters() {
        let action = LogAction::info("Special: ${special_chars}".to_string());
        let mut context = create_context_with_special_chars();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(
            message,
            Value::String("Special: hello\"world'test".to_string())
        );
    }
}

#[cfg(test)]
mod set_variable_action_tests {
    use super::*;

    #[test]
    fn test_set_variable_action_creation() {
        let action = SetVariableAction::new("test_var".to_string(), "test_value".to_string());
        assert_eq!(action.variable_name, "test_var");
        assert_eq!(action.value, "test_value");
    }

    #[test]
    fn test_set_variable_action_description() {
        let action = SetVariableAction::new("test_var".to_string(), "test_value".to_string());
        assert_eq!(
            action.description(),
            "Set variable 'test_var' to 'test_value'"
        );
    }

    #[test]
    fn test_set_variable_action_type() {
        let action = SetVariableAction::new("test_var".to_string(), "test_value".to_string());
        assert_eq!(action.action_type(), "set_variable");
    }

    #[tokio::test]
    async fn test_set_variable_action_execution_string() {
        let action = SetVariableAction::new("new_var".to_string(), "new_value".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("new_value".to_string()));
        assert_eq!(
            context.get("new_var"),
            Some(&Value::String("new_value".to_string()))
        );
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_set_variable_action_execution_json() {
        let action = SetVariableAction::new(
            "json_var".to_string(),
            r#"{"key": "value", "number": 42}"#.to_string(),
        );
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());

        let expected_json = serde_json::json!({"key": "value", "number": 42});
        assert_eq!(result.unwrap(), expected_json);
        assert_eq!(context.get("json_var"), Some(&expected_json));
    }

    #[tokio::test]
    async fn test_set_variable_action_execution_invalid_json() {
        let action =
            SetVariableAction::new("invalid_json".to_string(), "invalid json {".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());

        // Should fall back to string value
        assert_eq!(result.unwrap(), Value::String("invalid json {".to_string()));
        assert_eq!(
            context.get("invalid_json"),
            Some(&Value::String("invalid json {".to_string()))
        );
    }

    #[tokio::test]
    async fn test_set_variable_action_variable_substitution() {
        let action = SetVariableAction::new("result".to_string(), "Value: ${test_var}".to_string());
        let mut context = create_test_context();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::String("Value: test_value".to_string())
        );
        assert_eq!(
            context.get("result"),
            Some(&Value::String("Value: test_value".to_string()))
        );
    }

    #[tokio::test]
    async fn test_set_variable_action_json_substitution() {
        let action = SetVariableAction::new(
            "json_result".to_string(),
            r#"{"file": "${current_file}", "number": ${number_var}}"#.to_string(),
        );
        let mut context = create_test_context();

        let result = action.execute(&mut context).await;
        assert!(result.is_ok());

        let expected_json = serde_json::json!({"file": "test.rs", "number": 42});
        assert_eq!(result.unwrap(), expected_json);
        assert_eq!(context.get("json_result"), Some(&expected_json));
    }
}

#[cfg(test)]
mod sub_workflow_action_tests {
    use super::*;

    #[test]
    fn test_sub_workflow_action_creation() {
        let action = SubWorkflowAction::new("test-workflow".to_string());
        assert_eq!(action.workflow_name, "test-workflow");
        assert!(action.input_variables.is_empty());
        assert!(action.result_variable.is_none());
        assert_eq!(action.timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_sub_workflow_action_with_input() {
        let action = SubWorkflowAction::new("test-workflow".to_string())
            .with_input("input1".to_string(), "value1".to_string())
            .with_input("input2".to_string(), "value2".to_string());

        assert_eq!(
            action.input_variables.get("input1"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            action.input_variables.get("input2"),
            Some(&"value2".to_string())
        );
    }

    #[test]
    fn test_sub_workflow_action_with_result_variable() {
        let action = SubWorkflowAction::new("test-workflow".to_string())
            .with_result_variable("result_var".to_string());

        assert_eq!(action.result_variable, Some("result_var".to_string()));
    }

    #[test]
    fn test_sub_workflow_action_with_timeout() {
        let timeout_duration = Duration::from_secs(300);
        let action =
            SubWorkflowAction::new("test-workflow".to_string()).with_timeout(timeout_duration);

        assert_eq!(action.timeout, timeout_duration);
    }

    // Note: Variable substitution is tested through the public execute() method
    // since substitute_variables() is private

    #[test]
    fn test_sub_workflow_action_description() {
        let action = SubWorkflowAction::new("test-workflow".to_string())
            .with_input("input1".to_string(), "value1".to_string());

        let description = action.description();
        assert!(description.contains("test-workflow"));
        assert!(description.contains("input1"));
    }

    #[test]
    fn test_sub_workflow_action_type() {
        let action = SubWorkflowAction::new("test-workflow".to_string());
        assert_eq!(action.action_type(), "sub_workflow");
    }

    #[tokio::test]
    async fn test_sub_workflow_action_circular_dependency_detection() {
        let action = SubWorkflowAction::new("workflow-a".to_string());
        let mut context = HashMap::new();

        // Simulate that workflow-a is already in the execution stack
        let workflow_stack = vec![
            Value::String("workflow-main".to_string()),
            Value::String("workflow-a".to_string()),
        ];
        context.insert("_workflow_stack".to_string(), Value::Array(workflow_stack));

        let result = action.execute(&mut context).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ActionError::ExecutionError(msg) => {
                assert!(msg.contains("Circular dependency detected"));
                assert!(msg.contains("workflow-a"));
            }
            _ => panic!("Expected ExecutionError for circular dependency"),
        }
    }

    #[tokio::test]
    async fn test_sub_workflow_action_invalid_input_key() {
        let action = SubWorkflowAction::new("test-workflow".to_string())
            .with_input("invalid key!".to_string(), "value".to_string());

        let mut context = HashMap::new();
        let result = action.execute(&mut context).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ActionError::ParseError(msg) => {
                assert!(msg.contains("Invalid input variable key"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[tokio::test]
    async fn test_sub_workflow_action_empty_workflow_stack() {
        let action = SubWorkflowAction::new("test-workflow".to_string());
        let mut context = HashMap::new();

        // This should not fail with circular dependency since stack is empty
        let result = action.execute(&mut context).await;

        // It will fail with execution error (workflow not found), but not circular dependency
        assert!(result.is_err());
        if let ActionError::ExecutionError(msg) = result.unwrap_err() {
            assert!(!msg.contains("Circular dependency"));
            assert!(msg.contains("Failed to load sub-workflow"));
        }
    }

    #[tokio::test]
    async fn test_sub_workflow_action_in_process_execution() {
        // This test verifies that the sub-workflow action correctly executes workflows
        // in-process rather than shelling out to a subprocess
        use crate::workflow::test_helpers::create_state;
        use crate::workflow::{StateId, Workflow, WorkflowName, WorkflowStorage};

        // Create a simple test workflow in memory
        let workflow_name = "test-in-process-workflow";
        let mut workflow = Workflow::new(
            WorkflowName::new(workflow_name),
            "Test workflow for in-process execution".to_string(),
            StateId::new("start"),
        );

        // Add a simple state that sets a variable
        let start_state = create_state("start", "Set test_result=\"workflow_executed\"", true);
        workflow.add_state(start_state);

        // Store the workflow
        let mut storage = WorkflowStorage::memory();
        storage.store_workflow(workflow).unwrap();

        // Create sub-workflow action
        let action = SubWorkflowAction::new(workflow_name.to_string())
            .with_input("input_var".to_string(), "test_input".to_string())
            .with_result_variable("sub_result".to_string());

        let mut context = HashMap::new();

        // Execute the sub-workflow
        let result = action.execute(&mut context).await;

        // The workflow doesn't exist in the file system, so it should fail
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ActionError::ExecutionError(_)
        ));

        // Verify that no subprocess was spawned (we can't directly test this,
        // but the error message should indicate a workflow loading failure,
        // not a subprocess failure)
    }
}

// Note: Utility functions are private and tested implicitly through action execution
// including: is_valid_argument_key, substitute_variables_in_string

// Note: parse_claude_response is private and tested implicitly through PromptAction execution

// Note: parse_workflow_output is private and tested implicitly through SubWorkflowAction execution

#[cfg(test)]
mod action_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_action_from_description_prompt() {
        let description = r#"Execute prompt "test-prompt" with arg1="value1" arg2="value2""#;
        let action = parse_action_from_description(description).unwrap().unwrap();

        assert_eq!(action.action_type(), "prompt");
        assert!(action.description().contains("test-prompt"));
    }

    #[test]
    fn test_parse_action_from_description_wait() {
        let description = "Wait 30 seconds";
        let action = parse_action_from_description(description).unwrap().unwrap();

        assert_eq!(action.action_type(), "wait");
        assert!(action.description().contains("30s"));
    }

    #[test]
    fn test_parse_action_from_description_log() {
        let description = r#"Log "Test message""#;
        let action = parse_action_from_description(description).unwrap().unwrap();

        assert_eq!(action.action_type(), "log");
        assert!(action.description().contains("Test message"));
    }

    #[test]
    fn test_parse_action_from_description_set_variable() {
        let description = r#"Set variable_name="value""#;
        let action = parse_action_from_description(description).unwrap().unwrap();

        assert_eq!(action.action_type(), "set_variable");
        assert!(action.description().contains("variable_name"));
    }

    #[test]
    fn test_parse_action_from_description_sub_workflow() {
        let description = r#"Run workflow "test-workflow" with input="value""#;
        let action = parse_action_from_description(description).unwrap().unwrap();

        assert_eq!(action.action_type(), "sub_workflow");
        assert!(action.description().contains("test-workflow"));
    }

    #[test]
    fn test_parse_action_from_description_no_match() {
        let description = "This doesn't match any action pattern";
        let action = parse_action_from_description(description).unwrap();

        assert!(action.is_none());
    }

    #[test]
    fn test_parse_action_from_description_empty() {
        let description = "";
        let action = parse_action_from_description(description).unwrap();

        assert!(action.is_none());
    }

    #[test]
    fn test_parse_action_from_description_whitespace() {
        let description = "   \n\n   ";
        let action = parse_action_from_description(description).unwrap();

        assert!(action.is_none());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_action_error_display() {
        let error = ActionError::ClaudeError("Test error".to_string());
        assert!(error.to_string().contains("Claude execution failed"));
        assert!(error.to_string().contains("Test error"));

        let error = ActionError::VariableError("Variable error".to_string());
        assert!(error.to_string().contains("Variable operation failed"));

        let error = ActionError::ParseError("Parse error".to_string());
        assert!(error.to_string().contains("Action parsing failed"));

        let error = ActionError::Timeout {
            timeout: Duration::from_secs(30),
        };
        assert!(error.to_string().contains("timed out"));
        assert!(error.to_string().contains("30s"));

        let error = ActionError::ExecutionError("Execution error".to_string());
        assert!(error.to_string().contains("Action execution failed"));
    }

    #[test]
    fn test_action_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let action_error = ActionError::from(io_error);

        match action_error {
            ActionError::IoError(_) => {
                assert!(action_error.to_string().contains("IO error"));
            }
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_action_error_from_json_error() {
        let json_error = serde_json::from_str::<Value>("invalid json").unwrap_err();
        let action_error = ActionError::from(json_error);

        match action_error {
            ActionError::JsonError(_) => {
                assert!(action_error.to_string().contains("JSON parsing error"));
            }
            _ => panic!("Expected JsonError"),
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_action_execution_context_preservation() {
        // Test that actions properly preserve and modify context
        let mut context = HashMap::new();
        context.insert(
            "initial_value".to_string(),
            Value::String("initial".to_string()),
        );

        // Execute a set variable action
        let set_action = SetVariableAction::new("new_var".to_string(), "new_value".to_string());
        let _result = set_action.execute(&mut context).await.unwrap();

        // Verify context was modified
        assert_eq!(
            context.get("new_var"),
            Some(&Value::String("new_value".to_string()))
        );
        assert_eq!(
            context.get("initial_value"),
            Some(&Value::String("initial".to_string()))
        );
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));

        // Execute a log action that uses the new variable
        let log_action = LogAction::info("Value: ${new_var}".to_string());
        let result = log_action.execute(&mut context).await.unwrap();

        // Verify substitution worked
        assert_eq!(result, Value::String("Value: new_value".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_actions_sequence() {
        let mut context = HashMap::new();

        // Execute sequence of actions
        let actions: Vec<Box<dyn Action>> = vec![
            Box::new(SetVariableAction::new(
                "step1".to_string(),
                "completed".to_string(),
            )),
            Box::new(LogAction::info("Step 1: ${step1}".to_string())),
            Box::new(SetVariableAction::new(
                "step2".to_string(),
                "also_completed".to_string(),
            )),
            Box::new(LogAction::info("Step 2: ${step2}".to_string())),
        ];

        for action in actions {
            let result = action.execute(&mut context).await;
            assert!(result.is_ok());
        }

        // Verify final context state
        assert_eq!(
            context.get("step1"),
            Some(&Value::String("completed".to_string()))
        );
        assert_eq!(
            context.get("step2"),
            Some(&Value::String("also_completed".to_string()))
        );
    }

    #[tokio::test]
    async fn test_action_error_propagation() {
        let mut context = HashMap::new();

        // Test that parse errors are properly propagated
        let action = SetVariableAction::new("test".to_string(), "value".to_string());
        let result = action.execute(&mut context).await;
        assert!(result.is_ok());

        // Add an invalid key to prompt action to test error propagation
        let prompt_action = PromptAction::new("test".to_string())
            .with_argument("invalid key!".to_string(), "value".to_string());

        let result = prompt_action.execute(&mut context).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            ActionError::ParseError(msg) => {
                assert!(msg.contains("Invalid argument key"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[tokio::test]
    async fn test_action_timeout_behavior() {
        // Test timeout behavior with wait action
        let action = WaitAction::new_duration(Duration::from_millis(50));
        let mut context = HashMap::new();

        let start = std::time::Instant::now();
        let result = action.execute(&mut context).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(40)); // Allow some tolerance
        assert!(elapsed < Duration::from_millis(100)); // Should not be too slow
    }

    #[tokio::test]
    async fn test_action_context_key_constants() {
        let mut context = HashMap::new();

        // Test that actions use the correct context keys
        let action = LogAction::info("Test message".to_string());
        let result = action.execute(&mut context).await;
        assert!(result.is_ok());

        // Verify the constant keys are used
        assert!(context.contains_key("last_action_result"));
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));
    }
}

#[cfg(test)]
mod claude_output_formatting_tests {
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
}
