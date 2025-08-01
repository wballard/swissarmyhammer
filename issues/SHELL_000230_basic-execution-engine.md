# Implement Basic Shell Execution Engine

Refer to ./specification/shell.md

## Overview

Implement the core shell command execution logic in the `ShellAction::execute` method. This step focuses on basic command execution using tokio's process management, without advanced features like timeout handling or environment variable processing.

## Objective

Replace the placeholder execute method with working shell command execution that can run commands and capture their output, following the established async patterns in the codebase.

## Tasks

### 1. Implement Basic Command Execution

Replace the placeholder execute method in `ShellAction` with actual command execution:

```rust
#[async_trait::async_trait]
impl Action for ShellAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // Substitute variables in command string
        let command = self.substitute_string(&self.command, context);
        
        tracing::info!("Executing shell command: {}", command);
        
        // Use tokio::process::Command for async execution
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c")
           .arg(&command)
           .stdout(std::process::Stdio::piped())
           .stderr(std::process::Stdio::piped());
        
        // Execute the command
        let output = cmd.output().await.map_err(|e| {
            ActionError::ExecutionError(format!("Failed to execute command: {}", e))
        })?;
        
        // Process results and set context variables
        let success = output.status.success();
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Set automatic variables in context
        context.insert("success".to_string(), Value::Bool(success));
        context.insert("failure".to_string(), Value::Bool(!success));
        context.insert("exit_code".to_string(), Value::Number(exit_code.into()));
        context.insert("stdout".to_string(), Value::String(stdout.clone()));
        context.insert("stderr".to_string(), Value::String(stderr));
        
        // Set result variable if specified
        if let Some(result_var) = &self.result_variable {
            context.insert(result_var.clone(), Value::String(stdout.clone()));
        }
        
        // Return appropriate result
        if success {
            Ok(Value::String(stdout))
        } else {
            Ok(Value::Bool(false)) // Don't fail the workflow, just indicate failure
        }
    }
}
```

### 2. Add Error Handling

Implement proper error handling for common failure scenarios:
- Command not found
- Permission denied
- Invalid command syntax
- Process spawn failures

### 3. Cross-Platform Command Execution

Ensure commands work on different platforms:
- Use `sh -c` on Unix systems
- Use `cmd /C` on Windows systems
- Detect platform using `cfg` attributes

```rust
#[cfg(target_os = "windows")]
fn create_command(command: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("cmd");
    cmd.args(["/C", command]);
    cmd
}

#[cfg(not(target_os = "windows"))]
fn create_command(command: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("sh");
    cmd.args(["-c", command]);
    cmd
}
```

### 4. Logging and Debugging

Add appropriate logging:
- Log command execution start
- Log execution completion with timing
- Log errors and failures
- Use `tracing` crate consistently

### 5. Variable Substitution Integration

Ensure variable substitution works correctly:
- Use the `VariableSubstitution` trait
- Handle variable substitution errors gracefully
- Test with various variable patterns

## Implementation Notes

- Keep this implementation simple and focused on basic execution
- Advanced features (timeout, working directory, environment) will be added in later steps
- Follow async patterns used in other actions like `PromptAction`
- Use `ActionError` variants appropriately

## Success Criteria

- [ ] Shell commands execute successfully
- [ ] Command output is captured correctly
- [ ] Context variables are set properly (success, failure, exit_code, stdout, stderr)
- [ ] Result variable is set when specified
- [ ] Error handling works for common failures
- [ ] Cross-platform execution works
- [ ] Integration with variable substitution works
- [ ] Proper logging is implemented

## Testing

Write unit tests for:
- Successful command execution
- Failed command execution (non-zero exit codes)
- Output capture and context variable setting
- Variable substitution in commands
- Error handling for invalid commands

Example test:
```rust
#[tokio::test]
async fn test_shell_action_basic_execution() {
    let action = ShellAction::new("echo hello world".to_string());
    let mut context = HashMap::new();
    
    let result = action.execute(&mut context).await.unwrap();
    
    assert_eq!(context.get("success"), Some(&Value::Bool(true)));
    assert_eq!(context.get("exit_code"), Some(&Value::Number(0.into())));
    assert!(context.get("stdout").unwrap().as_str().unwrap().contains("hello world"));
}
```

## Next Steps

After completing this step, proceed to implementing timeout handling and process cleanup.