//! Tests for ShellAction

use crate::workflow::actions::*;
use crate::workflow::actions_tests::create_test_context;
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_shell_action_creation() {
    let action = ShellAction::new("echo hello".to_string());
    assert_eq!(action.command, "echo hello");
    assert_eq!(action.timeout, None);
    assert_eq!(action.result_variable, None);
    assert_eq!(action.working_dir, None);
    assert!(action.environment.is_empty());
}

#[test]
fn test_shell_action_with_timeout() {
    let timeout = Duration::from_secs(30);
    let action = ShellAction::new("echo hello".to_string()).with_timeout(timeout);

    assert_eq!(action.command, "echo hello");
    assert_eq!(action.timeout, Some(timeout));
}

#[test]
fn test_shell_action_with_result_variable() {
    let action =
        ShellAction::new("echo hello".to_string()).with_result_variable("output".to_string());

    assert_eq!(action.command, "echo hello");
    assert_eq!(action.result_variable, Some("output".to_string()));
}

#[test]
fn test_shell_action_with_working_dir() {
    let action = ShellAction::new("echo hello".to_string()).with_working_dir("/tmp".to_string());

    assert_eq!(action.command, "echo hello");
    assert_eq!(action.working_dir, Some("/tmp".to_string()));
}

#[test]
fn test_shell_action_with_environment() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/bin".to_string());
    env.insert("HOME".to_string(), "/home/user".to_string());

    let action = ShellAction::new("echo hello".to_string()).with_environment(env.clone());

    assert_eq!(action.command, "echo hello");
    assert_eq!(action.environment, env);
}

#[test]
fn test_shell_action_builder_chain() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());

    let action = ShellAction::new("echo hello".to_string())
        .with_timeout(Duration::from_secs(60))
        .with_result_variable("cmd_output".to_string())
        .with_working_dir("/tmp".to_string())
        .with_environment(env.clone());

    assert_eq!(action.command, "echo hello");
    assert_eq!(action.timeout, Some(Duration::from_secs(60)));
    assert_eq!(action.result_variable, Some("cmd_output".to_string()));
    assert_eq!(action.working_dir, Some("/tmp".to_string()));
    assert_eq!(action.environment, env);
}

#[test]
fn test_shell_action_description() {
    let action = ShellAction::new("ls -la".to_string());
    assert_eq!(action.description(), "Execute shell command: ls -la");
}

#[test]
fn test_shell_action_type() {
    let action = ShellAction::new("pwd".to_string());
    assert_eq!(action.action_type(), "shell");
}

#[test]
fn test_shell_action_variable_substitution() {
    let action = ShellAction::new("echo ${test_var}".to_string());
    let context = create_test_context();

    let substituted = action.substitute_string(&action.command, &context);
    assert_eq!(substituted, "echo test_value");
}

#[tokio::test]
async fn test_shell_action_basic_execution() {
    let action = ShellAction::new("echo hello world".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await.unwrap();

    // Check that context variables are set properly
    assert_eq!(context.get("success"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        context.get("failure"),
        Some(&serde_json::Value::Bool(false))
    );
    assert_eq!(
        context.get("exit_code"),
        Some(&serde_json::Value::Number(0.into()))
    );

    // Check that stdout contains our output
    let stdout = context.get("stdout").unwrap().as_str().unwrap();
    assert!(stdout.contains("hello world"));

    // Check that duration_ms is set
    assert!(context.contains_key("duration_ms"));

    // Check return value
    assert!(result.as_str().unwrap().contains("hello world"));
}

#[tokio::test]
async fn test_shell_action_failed_execution() {
    let action = ShellAction::new("exit 1".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await.unwrap();

    // Check that context variables are set properly for failure
    assert_eq!(
        context.get("success"),
        Some(&serde_json::Value::Bool(false))
    );
    assert_eq!(context.get("failure"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        context.get("exit_code"),
        Some(&serde_json::Value::Number(1.into()))
    );

    // Check that duration_ms is set
    assert!(context.contains_key("duration_ms"));

    // Check return value - should be false for failure
    assert_eq!(result, serde_json::Value::Bool(false));
}

#[test]
fn test_shell_action_as_any() {
    let action = ShellAction::new("echo test".to_string());
    let _any_ref = action.as_any();
    // Test passes if it compiles and doesn't panic
}

#[test]
fn test_shell_action_clone() {
    let mut env = HashMap::new();
    env.insert("TEST".to_string(), "value".to_string());

    let action1 = ShellAction::new("echo test".to_string())
        .with_timeout(Duration::from_secs(30))
        .with_result_variable("output".to_string())
        .with_working_dir("/tmp".to_string())
        .with_environment(env.clone());

    let action2 = action1.clone();

    assert_eq!(action1.command, action2.command);
    assert_eq!(action1.timeout, action2.timeout);
    assert_eq!(action1.result_variable, action2.result_variable);
    assert_eq!(action1.working_dir, action2.working_dir);
    assert_eq!(action1.environment, action2.environment);
}

#[test]
fn test_shell_action_debug() {
    let action = ShellAction::new("echo test".to_string()).with_timeout(Duration::from_secs(30));

    let debug_str = format!("{action:?}");
    assert!(debug_str.contains("ShellAction"));
    assert!(debug_str.contains("echo test"));
}
