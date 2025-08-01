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

#[tokio::test]
async fn test_shell_action_working_directory_validation() {
    // Test with valid existing directory
    let action = ShellAction::new("pwd".to_string()).with_working_dir("/tmp".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_ok());

    // Test with non-existent directory
    let action =
        ShellAction::new("pwd".to_string()).with_working_dir("/nonexistent/directory".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Working directory does not exist"));
}

#[tokio::test]
async fn test_shell_action_working_directory_path_traversal_prevention() {
    // Test path traversal attempts
    let action = ShellAction::new("pwd".to_string()).with_working_dir("../../../etc".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("cannot contain parent directory references"));
}

#[tokio::test]
async fn test_shell_action_environment_variable_validation() {
    // Test with valid environment variable names
    let mut env = HashMap::new();
    env.insert("VALID_VAR".to_string(), "value".to_string());
    env.insert("_UNDERSCORE_VAR".to_string(), "value".to_string());
    env.insert("VAR123".to_string(), "value".to_string());

    let action = ShellAction::new("echo $VALID_VAR".to_string()).with_environment(env);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_ok());

    // Test with invalid environment variable names
    let mut env = HashMap::new();
    env.insert("123INVALID".to_string(), "value".to_string()); // starts with number

    let action = ShellAction::new("echo test".to_string()).with_environment(env);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid environment variable name"));
}

#[tokio::test]
async fn test_shell_action_environment_variable_special_characters() {
    // Test with invalid characters in environment variable names
    let mut env = HashMap::new();
    env.insert("INVALID-VAR".to_string(), "value".to_string()); // hyphen not allowed

    let action = ShellAction::new("echo test".to_string()).with_environment(env);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid environment variable name"));
}

#[tokio::test]
async fn test_shell_action_working_directory_variable_substitution() {
    // Test variable substitution in working directory
    let action = ShellAction::new("pwd".to_string()).with_working_dir("${work_dir}".to_string());
    let mut context = HashMap::new();
    context.insert(
        "work_dir".to_string(),
        serde_json::Value::String("/tmp".to_string()),
    );

    let result = action.execute(&mut context).await;
    assert!(result.is_ok());

    let stdout = context.get("stdout").unwrap().as_str().unwrap();
    assert!(stdout.contains("/tmp"));
}

#[tokio::test]
async fn test_shell_action_environment_variable_substitution() {
    // Test variable substitution in environment variables
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "${test_value}".to_string());

    let action = ShellAction::new("echo $TEST_VAR".to_string()).with_environment(env);
    let mut context = HashMap::new();
    context.insert(
        "test_value".to_string(),
        serde_json::Value::String("substituted_value".to_string()),
    );

    let result = action.execute(&mut context).await;
    assert!(result.is_ok());

    let stdout = context.get("stdout").unwrap().as_str().unwrap();
    assert!(stdout.contains("substituted_value"));
}

#[test]
fn test_environment_variable_name_validation() {
    use crate::workflow::actions::is_valid_env_var_name;

    // Valid names
    assert!(is_valid_env_var_name("VAR"));
    assert!(is_valid_env_var_name("_VAR"));
    assert!(is_valid_env_var_name("VAR123"));
    assert!(is_valid_env_var_name("VAR_NAME"));
    assert!(is_valid_env_var_name("_123"));

    // Invalid names
    assert!(!is_valid_env_var_name(""));
    assert!(!is_valid_env_var_name("123VAR"));
    assert!(!is_valid_env_var_name("VAR-NAME"));
    assert!(!is_valid_env_var_name("VAR NAME"));
    assert!(!is_valid_env_var_name("VAR.NAME"));
    assert!(!is_valid_env_var_name("VAR@NAME"));
}

#[test]
fn test_working_directory_validation() {
    use crate::workflow::actions::validate_working_directory;

    // Valid paths
    assert!(validate_working_directory("/tmp").is_ok());
    assert!(validate_working_directory("relative/path").is_ok());
    assert!(validate_working_directory("/absolute/path").is_ok());

    // Invalid paths with parent directory references
    assert!(validate_working_directory("../parent").is_err());
    assert!(validate_working_directory("path/../parent").is_err());
    assert!(validate_working_directory("/absolute/../parent").is_err());
}

// Security validation tests
#[test]
fn test_command_validation_empty_command() {
    use crate::workflow::actions::validate_command;

    let result = validate_command("");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("cannot be empty"));
}

#[test]
fn test_command_validation_long_command() {
    use crate::workflow::actions::validate_command;

    let long_command = "a".repeat(5000); // Exceeds 4096 character limit
    let result = validate_command(&long_command);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("too long"));
}

#[test]
fn test_dangerous_pattern_detection() {
    use crate::workflow::actions::validate_dangerous_patterns;

    // Test dangerous patterns - these should log warnings but not fail
    assert!(validate_dangerous_patterns("rm -rf /").is_ok());
    assert!(validate_dangerous_patterns("sudo apt install package").is_ok());
    assert!(validate_dangerous_patterns("ssh user@host").is_ok());
    assert!(validate_dangerous_patterns("eval 'dangerous code'").is_ok());

    // All should pass but generate warnings in logs
}

#[test]
fn test_command_structure_validation_injection_patterns() {
    use crate::workflow::actions::validate_command_structure;

    // Test command injection patterns that should be blocked
    assert!(validate_command_structure("echo hello; rm -rf /").is_err());
    assert!(validate_command_structure("echo hello && rm file").is_err());
    assert!(validate_command_structure("echo hello || rm file").is_err());
    assert!(validate_command_structure("echo `dangerous`").is_err());
    assert!(validate_command_structure("echo $(dangerous)").is_err());
    assert!(validate_command_structure("echo hello\nrm file").is_err());
    assert!(validate_command_structure("echo hello\0rm file").is_err());

    // Test safe command
    assert!(validate_command_structure("echo hello world").is_ok());
}

#[test]
fn test_command_structure_validation_safe_pipes() {
    use crate::workflow::actions::validate_command_structure;

    // Test that simple pipes are allowed
    assert!(validate_command_structure("ls | grep test").is_ok());

    // Test that dangerous pipe combinations are blocked
    assert!(validate_command_structure("ls | nc -l 8080").is_err());
}

#[test]
fn test_safe_usage_validation() {
    use crate::workflow::actions::validate_safe_usage;

    // Test pipe validation
    assert!(validate_safe_usage("ls | grep test", "|").unwrap());
    assert!(!validate_safe_usage("ls | nc -l 8080", "|").unwrap());
    assert!(!validate_safe_usage("ls | grep | sort", "|").unwrap()); // Multiple pipes

    // Test that dangerous operators are not considered safe
    assert!(!validate_safe_usage("echo hello && rm file", "&&").unwrap());
    assert!(!validate_safe_usage("echo hello || rm file", "||").unwrap());
    assert!(!validate_safe_usage("echo hello; rm file", ";").unwrap());
}

#[test]
fn test_working_directory_security_validation() {
    use crate::workflow::actions::validate_working_directory_security;

    // Test normal directories
    assert!(validate_working_directory_security("/tmp").is_ok());
    assert!(validate_working_directory_security("relative/path").is_ok());

    // Test path traversal prevention (inherited from base validation)
    assert!(validate_working_directory_security("../parent").is_err());

    // Test sensitive directory warnings (should succeed but log warnings)
    assert!(validate_working_directory_security("/etc/passwd").is_ok()); // Logs warning
    assert!(validate_working_directory_security("/root/secret").is_ok()); // Logs warning
}

#[test]
fn test_environment_variables_security_validation() {
    use crate::workflow::actions::validate_environment_variables_security;
    use std::collections::HashMap;

    // Test valid environment variables
    let mut env = HashMap::new();
    env.insert("VALID_VAR".to_string(), "value".to_string());
    assert!(validate_environment_variables_security(&env).is_ok());

    // Test invalid variable names
    let mut env = HashMap::new();
    env.insert("123INVALID".to_string(), "value".to_string());
    assert!(validate_environment_variables_security(&env).is_err());

    // Test protected variables (should succeed but log warnings)
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/custom/path".to_string());
    assert!(validate_environment_variables_security(&env).is_ok()); // Logs warning

    // Test value too long
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "x".repeat(2000));
    assert!(validate_environment_variables_security(&env).is_err());

    // Test invalid characters in values
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "value\0with\nnull".to_string());
    assert!(validate_environment_variables_security(&env).is_err());
}

#[test]
fn test_shell_action_timeout_validation() {
    use std::time::Duration;

    // Test valid timeout
    let action = ShellAction::new("echo test".to_string()).with_timeout(Duration::from_secs(300));
    assert!(action.validate_timeout().is_ok());

    // Test timeout too large
    let action = ShellAction::new("echo test".to_string()).with_timeout(Duration::from_secs(4000)); // Exceeds 1 hour limit
    assert!(action.validate_timeout().is_err());

    // Test zero timeout
    let action = ShellAction::new("echo test".to_string()).with_timeout(Duration::from_millis(0));
    assert!(action.validate_timeout().is_err());

    // Test default timeout when none specified
    let action = ShellAction::new("echo test".to_string());
    let timeout = action.validate_timeout().unwrap();
    assert_eq!(timeout, Duration::from_secs(300)); // DEFAULT_TIMEOUT
}

// Integration tests for security validation in execute method
#[tokio::test]
async fn test_shell_action_security_command_injection_prevention() {
    // Test that command injection patterns are blocked
    let action = ShellAction::new("echo hello; rm -rf /".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("unsafe command pattern"));
}

#[tokio::test]
async fn test_shell_action_security_empty_command() {
    let action = ShellAction::new("".to_string());
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("cannot be empty"));
}

#[tokio::test]
async fn test_shell_action_security_long_command() {
    let long_command = "echo ".to_string() + &"a".repeat(5000);
    let action = ShellAction::new(long_command);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("too long"));
}

#[tokio::test]
async fn test_shell_action_security_environment_variable_validation() {
    // Test invalid environment variable names
    let mut env = HashMap::new();
    env.insert("123INVALID".to_string(), "value".to_string());

    let action = ShellAction::new("echo test".to_string()).with_environment(env);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Invalid environment variable name"));
}

#[tokio::test]
async fn test_shell_action_security_environment_variable_too_long() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "x".repeat(2000));

    let action = ShellAction::new("echo test".to_string()).with_environment(env);
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("value too long"));
}

#[tokio::test]
async fn test_shell_action_security_timeout_too_large() {
    let action = ShellAction::new("echo test".to_string()).with_timeout(Duration::from_secs(4000)); // Exceeds limit
    let mut context = HashMap::new();

    let result = action.execute(&mut context).await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Timeout too large"));
}

#[tokio::test]
async fn test_shell_action_security_sensitive_directory_warning() {
    // This should succeed but generate warnings in logs
    let action = ShellAction::new("ls".to_string()).with_working_dir("/etc".to_string());
    let mut context = HashMap::new();

    // This test would need the /etc directory to exist, so we expect it to either:
    // 1. Succeed with warnings logged, or
    // 2. Fail due to directory not existing (but not due to security validation)
    let result = action.execute(&mut context).await;
    if let Err(error) = result {
        let error_msg = error.to_string();
        // Should not fail due to security validation, only due to directory existence
        assert!(!error_msg.contains("unsafe command pattern"));
        assert!(!error_msg.contains("cannot contain parent"));
    }
}

#[tokio::test]
async fn test_shell_action_security_dangerous_pattern_warning() {
    // These should execute but generate security warnings in logs
    let dangerous_commands = [
        "sudo echo test",
        "rm file.txt", // Not rm -rf which is more dangerous
        "ssh-keygen -t rsa",
    ];

    for cmd in &dangerous_commands {
        let action = ShellAction::new(cmd.to_string());
        let mut context = HashMap::new();

        // These should succeed (dangerous patterns only generate warnings)
        // but we can't test the actual execution without the commands being available
        // So we test that security validation doesn't block them
        let result = action.execute(&mut context).await;
        if let Err(error) = result {
            let error_msg = error.to_string();
            // Should not fail due to dangerous pattern detection
            assert!(
                !error_msg.contains("dangerous command pattern"),
                "Command '{}' was blocked by dangerous pattern detection: {}",
                cmd,
                error_msg
            );
        }
    }
}
