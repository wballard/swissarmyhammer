# Implement Working Directory and Environment Variables

Refer to ./specification/shell.md

## Overview

Enhance the shell action execution to support custom working directories and environment variables as specified in the shell action specification. This adds important flexibility for shell command execution contexts.

## Objective

Extend the `ShellAction::execute` method to handle working directory changes and environment variable injection, enabling more sophisticated shell command execution scenarios.

## Tasks

### 1. Implement Working Directory Support

Modify the command execution to support custom working directories:

```rust
async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
    let command = self.substitute_string(&self.command, context);
    let start_time = std::time::Instant::now();
    
    // Create the command
    let mut cmd = create_command(&command);
    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());
    
    // Set working directory if specified
    if let Some(working_dir) = &self.working_dir {
        let dir = self.substitute_string(working_dir, context);
        let path = std::path::Path::new(&dir);
        
        // Validate working directory exists and is accessible
        if !path.exists() {
            return Err(ActionError::ExecutionError(
                format!("Working directory does not exist: {}", dir)
            ));
        }
        
        if !path.is_dir() {
            return Err(ActionError::ExecutionError(
                format!("Working directory is not a directory: {}", dir)
            ));
        }
        
        cmd.current_dir(path);
        tracing::debug!("Set working directory to: {}", dir);
    }
    
    // ... rest of execution logic
}
```

### 2. Implement Environment Variable Support

Add environment variable injection to command execution:

```rust
// Apply environment variables if specified
if !self.environment.is_empty() {
    for (key, value) in &self.environment {
        let substituted_key = self.substitute_string(key, context);
        let substituted_value = self.substitute_string(value, context);
        
        // Validate environment variable names
        if !is_valid_env_var_name(&substituted_key) {
            return Err(ActionError::ExecutionError(
                format!("Invalid environment variable name: {}", substituted_key)
            ));
        }
        
        cmd.env(&substituted_key, &substituted_value);
        tracing::debug!("Set environment variable: {}={}", substituted_key, substituted_value);
    }
}
```

### 3. Add Environment Variable Validation

Implement validation for environment variable names:

```rust
fn is_valid_env_var_name(name: &str) -> bool {
    // Environment variable names should start with letter or underscore
    // and contain only letters, digits, and underscores
    if name.is_empty() {
        return false;
    }
    
    let mut chars = name.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
    }
    
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
```

### 4. Enhance Parser for New Parameters

Update the parser to handle working directory and environment parameters:

```rust
// In action_parser.rs, enhance parse_shell_action to handle:
// Shell "command" with working_dir="/path/to/dir"
// Shell "command" with env={"VAR1": "value1", "VAR2": "value2"}
// Shell "command" with timeout=30 working_dir="/tmp" env={"DEBUG": "1"}

pub fn parse_shell_action(&self, description: &str) -> ActionResult<Option<ShellAction>> {
    // Existing parser logic...
    
    // Add parameter parsing for working_dir and env
    let working_dir_parser = Self::case_insensitive("working_dir")
        .then_ignore(Self::opt_whitespace())
        .then_ignore(just('='))
        .then_ignore(Self::opt_whitespace())
        .ignore_then(Self::quoted_string());
    
    let env_parser = Self::case_insensitive("env")
        .then_ignore(Self::opt_whitespace())
        .then_ignore(just('='))
        .then_ignore(Self::opt_whitespace())
        .ignore_then(/* JSON parsing for environment variables */);
    
    // Integrate into main parameter parsing logic
}
```

### 5. Add JSON Environment Variable Parsing

Implement JSON parsing for environment variables in the parser:

```rust
fn parse_environment_json(json_str: &str) -> ActionResult<HashMap<String, String>> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| ActionError::ParseError(format!("Invalid environment JSON: {}", e)))?;
    
    if let serde_json::Value::Object(obj) = json_value {
        let mut env_map = HashMap::new();
        for (key, value) in obj {
            if let serde_json::Value::String(str_value) = value {
                env_map.insert(key, str_value);
            } else {
                return Err(ActionError::ParseError(
                    format!("Environment variable values must be strings, found: {:?}", value)
                ));
            }
        }
        Ok(env_map)
    } else {
        Err(ActionError::ParseError(
            "Environment variables must be specified as a JSON object".to_string()
        ))
    }
}
```

### 6. Add Variable Substitution for New Fields

Ensure variable substitution works in working directory and environment variables:

```rust
impl VariableSubstitution for ShellAction {
    fn substitute_variables(&self, context: &HashMap<String, Value>) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        
        // Substitute in command
        vars.insert("command".to_string(), self.substitute_string(&self.command, context));
        
        // Substitute in working directory
        if let Some(working_dir) = &self.working_dir {
            vars.insert("working_dir".to_string(), self.substitute_string(working_dir, context));
        }
        
        // Substitute in environment variables
        for (key, value) in &self.environment {
            let sub_key = self.substitute_string(key, context);
            let sub_value = self.substitute_string(value, context);
            vars.insert(format!("env_{}", sub_key), sub_value);
        }
        
        vars
    }
}
```

### 7. Security Considerations

Add security validations:
- Validate working directory paths don't escape sandbox
- Sanitize environment variable names and values
- Prevent sensitive environment variable overrides
- Log working directory and environment changes

```rust
fn validate_working_directory(path: &str) -> ActionResult<()> {
    let path = std::path::Path::new(path);
    
    // Check for path traversal attempts
    if path.components().any(|comp| matches!(comp, std::path::Component::ParentDir)) {
        return Err(ActionError::ExecutionError(
            "Working directory cannot contain parent directory references".to_string()
        ));
    }
    
    // Additional security checks can be added here
    Ok(())
}
```

## Success Criteria

- [ ] Working directory can be set and commands execute in the correct directory
- [ ] Environment variables are properly injected into command execution
- [ ] Variable substitution works in working directory and environment values
- [ ] Parser supports new syntax for working_dir and env parameters
- [ ] JSON environment variable parsing works correctly
- [ ] Security validations prevent dangerous operations
- [ ] Error handling works for invalid directories and environment variables
- [ ] Cross-platform compatibility maintained

## Testing

Write comprehensive tests for:
- Working directory changes
- Environment variable injection
- Variable substitution in paths and environment
- JSON environment variable parsing
- Security validations
- Error handling for invalid paths and variables

Example tests:
```rust
#[tokio::test]
async fn test_shell_action_working_directory() {
    let action = ShellAction::new("pwd".to_string())
        .with_working_dir("/tmp".to_string());
    let mut context = HashMap::new();
    
    let result = action.execute(&mut context).await.unwrap();
    
    assert_eq!(context.get("success"), Some(&Value::Bool(true)));
    assert!(context.get("stdout").unwrap().as_str().unwrap().contains("/tmp"));
}

#[tokio::test]
async fn test_shell_action_environment_variables() {
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    
    let action = ShellAction::new("echo $TEST_VAR".to_string())
        .with_environment(env);
    let mut context = HashMap::new();
    
    let result = action.execute(&mut context).await.unwrap();
    
    assert_eq!(context.get("success"), Some(&Value::Bool(true)));
    assert!(context.get("stdout").unwrap().as_str().unwrap().contains("test_value"));
}
```

## Next Steps

After completing this step, proceed to implementing comprehensive security validation and command sanitization.