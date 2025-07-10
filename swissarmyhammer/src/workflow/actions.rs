//! Workflow action execution system
//!
//! This module provides the action execution infrastructure for workflows,
//! including Claude integration, variable operations, and control flow actions.

use crate::workflow::action_parser::ActionParser;
use crate::workflow::error_utils::handle_claude_command_error;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

/// Errors that can occur during action execution
#[derive(Debug, Error)]
pub enum ActionError {
    /// Claude command execution failed
    #[error("Claude execution failed: {0}")]
    ClaudeError(String),
    /// Variable operation failed
    #[error("Variable operation failed: {0}")]
    VariableError(String),
    /// Action parsing failed
    #[error("Action parsing failed: {0}")]
    ParseError(String),
    /// Action execution timed out
    #[error("Action execution timed out after {timeout:?}")]
    Timeout {
        /// The timeout duration that was exceeded
        timeout: Duration,
    },
    /// Generic action execution error
    #[error("Action execution failed: {0}")]
    ExecutionError(String),
    /// IO error during action execution
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Result type for action operations
pub type ActionResult<T> = Result<T, ActionError>;

/// Context key for Claude response
const CLAUDE_RESPONSE_KEY: &str = "claude_response";

/// Context key for last action result
const LAST_ACTION_RESULT_KEY: &str = "last_action_result";

/// Context key for workflow execution stack (for circular dependency detection)
const WORKFLOW_STACK_KEY: &str = "_workflow_stack";

/// Trait for all workflow actions
#[async_trait::async_trait]
pub trait Action: Send + Sync {
    /// Execute the action with the given context
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value>;

    /// Get a description of what this action does
    fn description(&self) -> String;

    /// Get the action type name
    fn action_type(&self) -> &'static str;
}

/// Action that executes a prompt using Claude
#[derive(Debug, Clone)]
pub struct PromptAction {
    /// Name of the prompt to execute
    pub prompt_name: String,
    /// Arguments to pass to the prompt
    pub arguments: HashMap<String, String>,
    /// Variable name to store the result
    pub result_variable: Option<String>,
    /// Timeout for the Claude execution
    pub timeout: Duration,
    /// Whether to suppress stdout output (only log)
    ///
    /// When set to `true`, the Claude response will only be logged using the tracing
    /// framework and will not be printed to stderr. This is useful for workflows
    /// that need to capture the response programmatically without cluttering the output.
    ///
    /// The quiet mode can also be controlled via the `_quiet` context variable in workflows.
    pub quiet: bool,
}

impl PromptAction {
    /// Create a new prompt action
    pub fn new(prompt_name: String) -> Self {
        Self {
            prompt_name,
            arguments: HashMap::new(),
            result_variable: None,
            timeout: Duration::from_secs(300), // 5 minute default
            quiet: false,                      // Default to showing output
        }
    }

    /// Add an argument to the prompt
    pub fn with_argument(mut self, key: String, value: String) -> Self {
        self.arguments.insert(key, value);
        self
    }

    /// Set the result variable name
    pub fn with_result_variable(mut self, variable: String) -> Self {
        self.result_variable = Some(variable);
        self
    }

    /// Set the timeout for execution
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set whether to suppress stdout output
    pub fn with_quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Substitute variables in arguments using the context
    fn substitute_variables(&self, context: &HashMap<String, Value>) -> HashMap<String, String> {
        let mut substituted = HashMap::new();

        for (key, value) in &self.arguments {
            let substituted_value = substitute_variables_in_string(value, context);
            substituted.insert(key.clone(), substituted_value);
        }

        substituted
    }
}

#[async_trait::async_trait]
impl Action for PromptAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // Substitute variables in arguments
        let args = self.substitute_variables(context);

        // Build Claude command
        let mut cmd = Command::new("claude");
        cmd.arg("--dangerously-skip-permissions")
            .arg("--print")
            .arg("--output-format")
            .arg("stream-json")
            .arg(&self.prompt_name);

        // Add arguments with validation
        for (key, value) in args {
            // Validate key to prevent injection
            if !is_valid_argument_key(&key) {
                return Err(ActionError::ParseError(
                    format!("Invalid argument key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                ));
            }
            cmd.arg(format!("--{}", key));
            cmd.arg(value);
        }

        // Spawn process with proper cleanup on timeout
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        let child = cmd.spawn().map_err(|e| {
            ActionError::ClaudeError(format!("Failed to spawn claude command: {}", e))
        })?;

        // Execute with timeout
        let output = match timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(ActionError::ClaudeError(format!(
                    "Failed to execute claude command: {}",
                    e
                )))
            }
            Err(_) => {
                // Timeout occurred
                // Note: The child process should be automatically killed when dropped
                // tokio::process::Child implements Drop that kills the process
                return Err(ActionError::Timeout {
                    timeout: self.timeout,
                });
            }
        };

        // Use shared error handling utility
        let stdout = handle_claude_command_error(output)?;

        // Check if quiet mode is enabled in the context
        let quiet = self.quiet
            || context
                .get("_quiet")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

        // Process and display the JSON stream
        let response = parse_and_display_claude_response(&stdout, quiet)?;

        // Store result in context if variable name specified
        if let Some(var_name) = &self.result_variable {
            context.insert(var_name.clone(), response.clone());
        }

        // Always store in special last_action_result key
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
        context.insert(CLAUDE_RESPONSE_KEY.to_string(), response.clone());

        Ok(response)
    }

    fn description(&self) -> String {
        format!(
            "Execute prompt '{}' with arguments: {:?}",
            self.prompt_name, self.arguments
        )
    }

    fn action_type(&self) -> &'static str {
        "prompt"
    }
}

/// Action that pauses execution for a specified duration or waits for user input
#[derive(Debug, Clone)]
pub struct WaitAction {
    /// Duration to wait (None means wait for user input)
    pub duration: Option<Duration>,
    /// Message to display while waiting
    pub message: Option<String>,
}

impl WaitAction {
    /// Create a new wait action with duration
    pub fn new_duration(duration: Duration) -> Self {
        Self {
            duration: Some(duration),
            message: None,
        }
    }

    /// Create a new wait action for user input
    pub fn new_user_input() -> Self {
        Self {
            duration: None,
            message: None,
        }
    }

    /// Set the wait message
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

#[async_trait::async_trait]
impl Action for WaitAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        match self.duration {
            Some(duration) => {
                if let Some(message) = &self.message {
                    eprintln!("Waiting: {}", message);
                }
                tokio::time::sleep(duration).await;
            }
            None => {
                let message = self
                    .message
                    .as_deref()
                    .unwrap_or("Press Enter to continue...");
                eprintln!("{}", message);

                // Read from stdin with a reasonable timeout
                use tokio::io::{stdin, AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stdin());
                let mut line = String::new();

                // Use a 5-minute timeout for user input
                const USER_INPUT_TIMEOUT: Duration = Duration::from_secs(300);
                match timeout(USER_INPUT_TIMEOUT, reader.read_line(&mut line)).await {
                    Ok(Ok(_)) => {
                        // Successfully read input
                    }
                    Ok(Err(e)) => {
                        return Err(ActionError::IoError(e));
                    }
                    Err(_) => {
                        return Err(ActionError::Timeout {
                            timeout: USER_INPUT_TIMEOUT,
                        });
                    }
                }
            }
        }

        // Mark action as successful
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));

        Ok(Value::Null)
    }

    fn description(&self) -> String {
        match self.duration {
            Some(duration) => format!("Wait for {:?}", duration),
            None => "Wait for user input".to_string(),
        }
    }

    fn action_type(&self) -> &'static str {
        "wait"
    }
}

/// Action that logs a message
#[derive(Debug, Clone)]
pub struct LogAction {
    /// Message to log
    pub message: String,
    /// Log level
    pub level: LogLevel,
}

/// Log levels for LogAction
#[derive(Debug, Clone)]
pub enum LogLevel {
    /// Informational log level
    Info,
    /// Warning log level
    Warning,
    /// Error log level
    Error,
}

impl LogAction {
    /// Create a new log action
    pub fn new(message: String, level: LogLevel) -> Self {
        Self { message, level }
    }

    /// Create an info log action
    pub fn info(message: String) -> Self {
        Self::new(message, LogLevel::Info)
    }

    /// Create a warning log action
    pub fn warning(message: String) -> Self {
        Self::new(message, LogLevel::Warning)
    }

    /// Create an error log action
    pub fn error(message: String) -> Self {
        Self::new(message, LogLevel::Error)
    }
}

#[async_trait::async_trait]
impl Action for LogAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // Substitute variables in message
        let message = substitute_variables_in_string(&self.message, context);

        match self.level {
            LogLevel::Info => eprintln!("[INFO] {}", message),
            LogLevel::Warning => eprintln!("[WARNING] {}", message),
            LogLevel::Error => eprintln!("[ERROR] {}", message),
        }

        // Mark action as successful
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));

        Ok(Value::String(message))
    }

    fn description(&self) -> String {
        format!("Log message: {}", self.message)
    }

    fn action_type(&self) -> &'static str {
        "log"
    }
}

/// Action that sets a variable in the workflow context
#[derive(Debug, Clone)]
pub struct SetVariableAction {
    /// Variable name to set
    pub variable_name: String,
    /// Value to set (supports variable substitution)
    pub value: String,
}

/// Action that executes a sub-workflow
#[derive(Debug, Clone)]
pub struct SubWorkflowAction {
    /// Name of the workflow to execute
    pub workflow_name: String,
    /// Input variables to pass to the sub-workflow
    pub input_variables: HashMap<String, String>,
    /// Variable name to store the result
    pub result_variable: Option<String>,
    /// Timeout for the sub-workflow execution
    pub timeout: Duration,
}

impl SetVariableAction {
    /// Create a new set variable action
    pub fn new(variable_name: String, value: String) -> Self {
        Self {
            variable_name,
            value,
        }
    }
}

#[async_trait::async_trait]
impl Action for SetVariableAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // Substitute variables in value
        let substituted_value = substitute_variables_in_string(&self.value, context);

        // Try to parse as JSON first, fall back to string
        let json_value = match serde_json::from_str(&substituted_value) {
            Ok(v) => v,
            Err(_) => Value::String(substituted_value),
        };

        // Set the variable
        context.insert(self.variable_name.clone(), json_value.clone());

        // Mark action as successful
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));

        Ok(json_value)
    }

    fn description(&self) -> String {
        format!("Set variable '{}' to '{}'", self.variable_name, self.value)
    }

    fn action_type(&self) -> &'static str {
        "set_variable"
    }
}

/// Validate that an argument key is safe for command-line use
fn is_valid_argument_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Helper function to substitute variables in a string
/// Variables are referenced as ${variable_name}
fn substitute_variables_in_string(input: &str, context: &HashMap<String, Value>) -> String {
    let parser = ActionParser::new().expect("Failed to create ActionParser");
    parser
        .substitute_variables_safe(input, context)
        .unwrap_or_else(|_| input.to_string())
}

/// Parse Claude's streaming JSON response, log it, and optionally display as YAML
fn parse_and_display_claude_response(output: &str, quiet: bool) -> ActionResult<Value> {
    // Claude outputs streaming JSON, we need to collect all content
    let mut content = String::new();
    let mut parse_errors = Vec::new();
    let mut valid_json_found = false;

    for (line_num, line) in output.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        // Log the raw JSON stream
        tracing::debug!("Claude JSON stream: {}", line);

        match serde_json::from_str::<Value>(line) {
            Ok(json) => {
                valid_json_found = true;

                // Convert to YAML and log it unless quiet
                if !quiet {
                    if let Ok(yaml) = serde_yaml::to_string(&json) {
                        // Log YAML without the "---" document separator for cleaner output
                        let yaml_trimmed = yaml.trim_start_matches("---\n");
                        tracing::info!("Claude response:\n{}", yaml_trimmed);
                    }
                }

                // Extract content
                if let Some(Value::String(text)) = json.get("content") {
                    content.push_str(text);
                }
            }
            Err(e) => {
                // Collect parse errors for potential debugging
                parse_errors.push((line_num + 1, e.to_string()));
                tracing::warn!("Failed to parse JSON on line {}: {}", line_num + 1, e);
            }
        }
    }

    if content.is_empty() {
        if valid_json_found {
            // Valid JSON was found but no content field
            Ok(Value::String(String::new()))
        } else if !parse_errors.is_empty() {
            // No valid JSON found and we have parse errors
            Err(ActionError::ParseError(
                format!("Failed to parse Claude response. Found {} parse errors. First error at line {}: {}",
                    parse_errors.len(),
                    parse_errors[0].0,
                    parse_errors[0].1
                )
            ))
        } else {
            // No JSON lines found at all, return raw output
            Ok(Value::String(output.to_string()))
        }
    } else {
        Ok(Value::String(content))
    }
}

impl SubWorkflowAction {
    /// Create a new sub-workflow action
    pub fn new(workflow_name: String) -> Self {
        Self {
            workflow_name,
            input_variables: HashMap::new(),
            result_variable: None,
            timeout: Duration::from_secs(600), // 10 minute default
        }
    }

    /// Add an input variable to pass to the sub-workflow
    pub fn with_input(mut self, key: String, value: String) -> Self {
        self.input_variables.insert(key, value);
        self
    }

    /// Set the result variable name
    pub fn with_result_variable(mut self, variable: String) -> Self {
        self.result_variable = Some(variable);
        self
    }

    /// Set the timeout for execution
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Substitute variables in input values using the context
    fn substitute_variables(&self, context: &HashMap<String, Value>) -> HashMap<String, String> {
        let mut substituted = HashMap::new();

        for (key, value) in &self.input_variables {
            let substituted_value = substitute_variables_in_string(value, context);
            substituted.insert(key.clone(), substituted_value);
        }

        substituted
    }
}

#[async_trait::async_trait]
impl Action for SubWorkflowAction {
    async fn execute(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // Check for circular dependencies
        let workflow_stack = context
            .get(WORKFLOW_STACK_KEY)
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Check if this workflow is already in the execution stack
        for stack_item in &workflow_stack {
            if let Some(workflow_name) = stack_item.as_str() {
                if workflow_name == self.workflow_name {
                    return Err(ActionError::ExecutionError(format!(
                        "Circular dependency detected: workflow '{}' is already in the execution stack",
                        self.workflow_name
                    )));
                }
            }
        }

        // Substitute variables in input
        let substituted_inputs = self.substitute_variables(context);

        // Build arguments for the sub-workflow
        let mut args = vec![
            "--dangerously-skip-permissions".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "flow".to_string(),
            "run".to_string(),
            self.workflow_name.clone(),
        ];

        // Add workflow stack to track circular dependencies
        let mut new_stack = workflow_stack;
        new_stack.push(Value::String(self.workflow_name.clone()));

        // Add input variables as arguments
        for (key, value) in substituted_inputs {
            if !is_valid_argument_key(&key) {
                return Err(ActionError::ParseError(
                    format!("Invalid input variable key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                ));
            }
            args.push("--var".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Pass the workflow stack to the sub-workflow
        args.push("--var".to_string());
        args.push(format!(
            "{}={}",
            WORKFLOW_STACK_KEY,
            serde_json::to_string(&new_stack).unwrap_or_default()
        ));

        // Execute the sub-workflow using the 'flow' command
        let mut cmd = Command::new("swissarmyhammer");
        for arg in args {
            cmd.arg(arg);
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        let child = cmd.spawn().map_err(|e| {
            ActionError::ExecutionError(format!("Failed to spawn sub-workflow: {}", e))
        })?;

        // Execute with timeout
        let output = match timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(ActionError::ExecutionError(format!(
                    "Failed to execute sub-workflow: {}",
                    e
                )))
            }
            Err(_) => {
                return Err(ActionError::Timeout {
                    timeout: self.timeout,
                });
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ActionError::ExecutionError(format!(
                "Sub-workflow '{}' failed: {}",
                self.workflow_name, stderr
            )));
        }

        // Parse the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result = parse_workflow_output(&stdout)?;

        // Store result in context if variable name specified
        if let Some(var_name) = &self.result_variable {
            context.insert(var_name.clone(), result.clone());
        }

        // Mark action as successful
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));

        Ok(result)
    }

    fn description(&self) -> String {
        format!(
            "Execute sub-workflow '{}' with inputs: {:?}",
            self.workflow_name, self.input_variables
        )
    }

    fn action_type(&self) -> &'static str {
        "sub_workflow"
    }
}

/// Parse workflow execution output
fn parse_workflow_output(output: &str) -> ActionResult<Value> {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<Value>(output) {
        return Ok(json);
    }

    // Parse streaming JSON output
    let mut result = HashMap::new();
    let mut success = false;

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(Value::String(event_type)) = json.get("type") {
                match event_type.as_str() {
                    "workflow_completed" => {
                        success = true;
                        if let Some(Value::Object(obj)) = json.get("context") {
                            for (k, v) in obj {
                                result.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    "error" => {
                        if let Some(Value::String(error)) = json.get("message") {
                            return Err(ActionError::ExecutionError(format!(
                                "Sub-workflow error: {}",
                                error
                            )));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if !success {
        return Err(ActionError::ExecutionError(
            "Sub-workflow did not complete successfully".to_string(),
        ));
    }

    Ok(Value::Object(serde_json::Map::from_iter(result)))
}

/// Parse action from state description text
pub fn parse_action_from_description(description: &str) -> ActionResult<Option<Box<dyn Action>>> {
    let parser = ActionParser::new()?;
    let description = description.trim();

    // Parse different action patterns using the robust parser
    if let Some(prompt_action) = parser.parse_prompt_action(description)? {
        return Ok(Some(Box::new(prompt_action)));
    }

    if let Some(wait_action) = parser.parse_wait_action(description)? {
        return Ok(Some(Box::new(wait_action)));
    }

    if let Some(log_action) = parser.parse_log_action(description)? {
        return Ok(Some(Box::new(log_action)));
    }

    if let Some(set_action) = parser.parse_set_variable_action(description)? {
        return Ok(Some(Box::new(set_action)));
    }

    if let Some(sub_workflow_action) = parser.parse_sub_workflow_action(description)? {
        return Ok(Some(Box::new(sub_workflow_action)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::action_parser::ActionParser;

    #[test]
    fn test_variable_substitution() {
        let mut context = HashMap::new();
        context.insert("file".to_string(), Value::String("test.rs".to_string()));
        context.insert("count".to_string(), Value::Number(42.into()));

        let result =
            substitute_variables_in_string("Process ${file} with ${count} items", &context);
        assert_eq!(result, "Process test.rs with 42 items");
    }

    #[test]
    fn test_parse_prompt_action() {
        let parser = ActionParser::new().unwrap();
        let desc = r#"Execute prompt "analyze-code" with file="test.rs" verbose="true""#;
        let action = parser.parse_prompt_action(desc).unwrap().unwrap();

        assert_eq!(action.prompt_name, "analyze-code");
        assert_eq!(action.arguments.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(action.arguments.get("verbose"), Some(&"true".to_string()));
        assert!(!action.quiet); // Default should be false
    }

    #[test]
    fn test_prompt_action_with_quiet() {
        let action = PromptAction::new("test-prompt".to_string()).with_quiet(true);

        assert_eq!(action.prompt_name, "test-prompt");
        assert!(action.quiet);
        assert!(action.arguments.is_empty());
        assert!(action.result_variable.is_none());
    }

    #[test]
    fn test_prompt_action_builder_methods() {
        let mut args = HashMap::new();
        args.insert("key".to_string(), "value".to_string());

        let mut action = PromptAction::new("test".to_string())
            .with_quiet(true)
            .with_result_variable("result_var".to_string());

        // Add arguments manually since there's no with_arguments method
        action.arguments = args.clone();

        assert_eq!(action.prompt_name, "test");
        assert!(action.quiet);
        assert_eq!(action.result_variable, Some("result_var".to_string()));
        assert_eq!(action.arguments, args);
    }

    #[test]
    fn test_parse_wait_action() {
        let parser = ActionParser::new().unwrap();
        let action = parser
            .parse_wait_action("Wait for user confirmation")
            .unwrap()
            .unwrap();
        assert!(action.duration.is_none());

        let action = parser
            .parse_wait_action("Wait 30 seconds")
            .unwrap()
            .unwrap();
        assert_eq!(action.duration, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_parse_log_action() {
        let parser = ActionParser::new().unwrap();
        let action = parser
            .parse_log_action(r#"Log "Hello world""#)
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Hello world");

        let action = parser
            .parse_log_action(r#"Log error "Something failed""#)
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Something failed");
    }

    #[test]
    fn test_parse_set_variable_action() {
        let parser = ActionParser::new().unwrap();
        let action = parser
            .parse_set_variable_action(r#"Set result="${claude_response}""#)
            .unwrap()
            .unwrap();
        assert_eq!(action.variable_name, "result");
        assert_eq!(action.value, "${claude_response}");
    }

    #[tokio::test]
    async fn test_log_action_execution() {
        let action = LogAction::info("Test message".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await.unwrap();
        assert_eq!(result, Value::String("Test message".to_string()));
        assert_eq!(
            context.get(LAST_ACTION_RESULT_KEY),
            Some(&Value::Bool(true))
        );
    }

    #[tokio::test]
    async fn test_set_variable_action_execution() {
        const TEST_VAR: &str = "test_var";
        const TEST_VALUE: &str = "test_value";

        let action = SetVariableAction::new(TEST_VAR.to_string(), TEST_VALUE.to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await.unwrap();
        assert_eq!(result, Value::String(TEST_VALUE.to_string()));
        assert_eq!(
            context.get(TEST_VAR),
            Some(&Value::String(TEST_VALUE.to_string()))
        );
    }

    #[test]
    fn test_parse_sub_workflow_action() {
        let desc = r#"Run workflow "validation-workflow" with input="${data}""#;
        let action = parse_action_from_description(desc).unwrap().unwrap();
        assert_eq!(action.action_type(), "sub_workflow");
        assert_eq!(
            action.description(),
            r#"Execute sub-workflow 'validation-workflow' with inputs: {"input": "${data}"}"#
        );
    }

    #[tokio::test]
    async fn test_sub_workflow_circular_dependency_detection() {
        let action = SubWorkflowAction::new("workflow-a".to_string());
        let mut context = HashMap::new();

        // Simulate that workflow-a is already in the execution stack
        let workflow_stack = vec![
            Value::String("workflow-main".to_string()),
            Value::String("workflow-a".to_string()),
        ];
        context.insert(WORKFLOW_STACK_KEY.to_string(), Value::Array(workflow_stack));

        // This should fail with circular dependency error
        let result = action.execute(&mut context).await;
        assert!(result.is_err());

        let error = result.unwrap_err();
        match error {
            ActionError::ExecutionError(msg) => {
                assert!(msg.contains("Circular dependency detected"));
                assert!(msg.contains("workflow-a"));
            }
            _ => panic!("Expected ExecutionError for circular dependency"),
        }
    }

    #[test]
    fn test_sub_workflow_variable_substitution() {
        let mut action = SubWorkflowAction::new("validation-workflow".to_string());
        action
            .input_variables
            .insert("file".to_string(), "${current_file}".to_string());
        action
            .input_variables
            .insert("mode".to_string(), "strict".to_string());

        let mut context = HashMap::new();
        context.insert(
            "current_file".to_string(),
            Value::String("test.rs".to_string()),
        );

        let substituted = action.substitute_variables(&context);
        assert_eq!(substituted.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(substituted.get("mode"), Some(&"strict".to_string()));
    }

    #[test]
    fn test_parse_and_display_claude_response_quiet() {
        let json_output = r#"{"type":"content_block_start","content":"Hello"}
{"type":"content_block_delta","content":" world"}
{"type":"content_block_stop"}"#;

        // Test with quiet = true (should not print to stderr)
        let result = parse_and_display_claude_response(json_output, true).unwrap();
        assert_eq!(result, Value::String("Hello world".to_string()));

        // Test with quiet = false (would print to stderr but we can't easily test that)
        let result = parse_and_display_claude_response(json_output, false).unwrap();
        assert_eq!(result, Value::String("Hello world".to_string()));
    }

    #[test]
    fn test_parse_and_display_claude_response_invalid_json() {
        let invalid_output = "This is not JSON";

        // Should return the raw output when no valid JSON is found
        let result = parse_and_display_claude_response(invalid_output, true);

        // Actually it should return an error for invalid JSON with no valid lines
        assert!(result.is_err());
        if let Err(ActionError::ParseError(msg)) = result {
            assert!(msg.contains("Failed to parse Claude response"));
        } else {
            panic!("Expected ParseError");
        }
    }
}
