//! Workflow action execution system
//!
//! This module provides the action execution infrastructure for workflows,
//! including Claude integration, variable operations, and control flow actions.

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
}

impl PromptAction {
    /// Create a new prompt action
    pub fn new(prompt_name: String) -> Self {
        Self {
            prompt_name,
            arguments: HashMap::new(),
            result_variable: None,
            timeout: Duration::from_secs(300), // 5 minute default
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

        // Add arguments
        for (key, value) in args {
            cmd.arg(format!("--{}", key));
            cmd.arg(value);
        }

        // Execute with timeout
        let output = timeout(self.timeout, cmd.output())
            .await
            .map_err(|_| ActionError::Timeout {
                timeout: self.timeout,
            })?
            .map_err(|e| {
                ActionError::ClaudeError(format!("Failed to execute claude command: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ActionError::ClaudeError(format!(
                "Claude command failed: {}",
                stderr
            )));
        }

        // Parse streaming JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let response = parse_claude_response(&stdout)?;

        // Store result in context if variable name specified
        if let Some(var_name) = &self.result_variable {
            context.insert(var_name.clone(), response.clone());
        }

        // Always store in special last_action_result key
        context.insert("last_action_result".to_string(), Value::Bool(true));
        context.insert("claude_response".to_string(), response.clone());

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

                // Read from stdin
                use tokio::io::{stdin, AsyncBufReadExt, BufReader};
                let mut reader = BufReader::new(stdin());
                let mut line = String::new();
                reader.read_line(&mut line).await?;
            }
        }

        // Mark action as successful
        context.insert("last_action_result".to_string(), Value::Bool(true));

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
        context.insert("last_action_result".to_string(), Value::Bool(true));

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
        context.insert("last_action_result".to_string(), Value::Bool(true));

        Ok(json_value)
    }

    fn description(&self) -> String {
        format!("Set variable '{}' to '{}'", self.variable_name, self.value)
    }

    fn action_type(&self) -> &'static str {
        "set_variable"
    }
}

/// Helper function to substitute variables in a string
/// Variables are referenced as ${variable_name}
fn substitute_variables_in_string(input: &str, context: &HashMap<String, Value>) -> String {
    let mut result = input.to_string();

    // Simple variable substitution - find ${variable_name} patterns
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            let replacement = context
                .get(var_name)
                .map(value_to_string)
                .unwrap_or_else(|| format!("${{{}}}", var_name)); // Keep original if not found

            result.replace_range(start..start + end + 1, &replacement);
        } else {
            break; // No closing brace found
        }
    }

    result
}

/// Convert a JSON Value to a string representation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

/// Parse Claude's streaming JSON response
fn parse_claude_response(output: &str) -> ActionResult<Value> {
    // Claude outputs streaming JSON, we need to collect all content
    let mut content = String::new();

    for line in output.lines() {
        if let Ok(json) = serde_json::from_str::<Value>(line) {
            if let Some(Value::String(text)) = json.get("content") {
                content.push_str(text);
            }
        }
    }

    if content.is_empty() {
        // If no content found, return the raw output
        Ok(Value::String(output.to_string()))
    } else {
        Ok(Value::String(content))
    }
}

/// Parse action from state description text
pub fn parse_action_from_description(description: &str) -> ActionResult<Option<Box<dyn Action>>> {
    let description = description.trim();

    // Parse different action patterns
    if let Some(prompt_action) = parse_prompt_action(description)? {
        return Ok(Some(Box::new(prompt_action)));
    }

    if let Some(wait_action) = parse_wait_action(description)? {
        return Ok(Some(Box::new(wait_action)));
    }

    if let Some(log_action) = parse_log_action(description)? {
        return Ok(Some(Box::new(log_action)));
    }

    if let Some(set_action) = parse_set_variable_action(description)? {
        return Ok(Some(Box::new(set_action)));
    }

    Ok(None)
}

/// Parse prompt action from description
/// Format: Execute prompt "prompt-name" with arg1="value1" arg2="value2"
fn parse_prompt_action(description: &str) -> ActionResult<Option<PromptAction>> {
    if !description.to_lowercase().starts_with("execute prompt") {
        return Ok(None);
    }

    // Extract prompt name
    let start_quote = description
        .find('"')
        .ok_or_else(|| ActionError::ParseError("Expected quoted prompt name".to_string()))?;
    let end_quote = description[start_quote + 1..].find('"').ok_or_else(|| {
        ActionError::ParseError("Expected closing quote for prompt name".to_string())
    })? + start_quote
        + 1;

    let prompt_name = description[start_quote + 1..end_quote].to_string();
    let mut action = PromptAction::new(prompt_name);

    // Parse arguments if present
    if let Some(with_pos) = description.find(" with ") {
        let args_part = &description[with_pos + 6..];
        for arg_pair in args_part.split_whitespace() {
            if let Some(eq_pos) = arg_pair.find('=') {
                let key = arg_pair[..eq_pos].to_string();
                let value = arg_pair[eq_pos + 1..].trim_matches('"').to_string();
                action.arguments.insert(key, value);
            }
        }
    }

    Ok(Some(action))
}

/// Parse wait action from description
/// Format: Wait for user confirmation OR Wait 30 seconds
fn parse_wait_action(description: &str) -> ActionResult<Option<WaitAction>> {
    let lower_desc = description.to_lowercase();

    if lower_desc.starts_with("wait for user") {
        return Ok(Some(
            WaitAction::new_user_input().with_message(description.to_string()),
        ));
    }

    if lower_desc.starts_with("wait ") {
        // Try to parse duration
        if let Some(seconds) = extract_duration_seconds(&lower_desc) {
            return Ok(Some(WaitAction::new_duration(Duration::from_secs(seconds))));
        }
    }

    Ok(None)
}

/// Parse log action from description
/// Format: Log "message" OR Log error "message"
fn parse_log_action(description: &str) -> ActionResult<Option<LogAction>> {
    let lower_desc = description.to_lowercase();

    if lower_desc.starts_with("log error") {
        if let Some(message) = extract_quoted_text(description) {
            return Ok(Some(LogAction::error(message)));
        }
    } else if lower_desc.starts_with("log warning") {
        if let Some(message) = extract_quoted_text(description) {
            return Ok(Some(LogAction::warning(message)));
        }
    } else if lower_desc.starts_with("log") {
        if let Some(message) = extract_quoted_text(description) {
            return Ok(Some(LogAction::info(message)));
        }
    }

    Ok(None)
}

/// Parse set variable action from description
/// Format: Set variable_name="${value}"
fn parse_set_variable_action(description: &str) -> ActionResult<Option<SetVariableAction>> {
    if !description.to_lowercase().starts_with("set ") {
        return Ok(None);
    }

    // Find the equals sign
    if let Some(eq_pos) = description.find('=') {
        let var_name = description[4..eq_pos].trim().to_string();
        let value = description[eq_pos + 1..].trim_matches('"').to_string();

        return Ok(Some(SetVariableAction::new(var_name, value)));
    }

    Ok(None)
}

/// Helper to extract quoted text from a string
fn extract_quoted_text(text: &str) -> Option<String> {
    let start_quote = text.find('"')?;
    let end_quote = text[start_quote + 1..].find('"')? + start_quote + 1;
    Some(text[start_quote + 1..end_quote].to_string())
}

/// Helper to extract duration in seconds from text
fn extract_duration_seconds(text: &str) -> Option<u64> {
    // Simple parser for "wait 30 seconds", "wait 5 minutes", etc.
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 3 {
        if let Ok(number) = parts[1].parse::<u64>() {
            let unit = parts[2].to_lowercase();
            return match unit.as_str() {
                "second" | "seconds" | "sec" | "s" => Some(number),
                "minute" | "minutes" | "min" | "m" => Some(number * 60),
                "hour" | "hours" | "h" => Some(number * 3600),
                _ => None,
            };
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let desc = r#"Execute prompt "analyze-code" with file="test.rs" verbose="true""#;
        let action = parse_prompt_action(desc).unwrap().unwrap();

        assert_eq!(action.prompt_name, "analyze-code");
        assert_eq!(action.arguments.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(action.arguments.get("verbose"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_wait_action() {
        let action = parse_wait_action("Wait for user confirmation")
            .unwrap()
            .unwrap();
        assert!(action.duration.is_none());

        let action = parse_wait_action("Wait 30 seconds").unwrap().unwrap();
        assert_eq!(action.duration, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_parse_log_action() {
        let action = parse_log_action(r#"Log "Hello world""#).unwrap().unwrap();
        assert_eq!(action.message, "Hello world");

        let action = parse_log_action(r#"Log error "Something failed""#)
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Something failed");
    }

    #[test]
    fn test_parse_set_variable_action() {
        let action = parse_set_variable_action(r#"Set result="${claude_response}""#)
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
        assert_eq!(context.get("last_action_result"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_set_variable_action_execution() {
        let action = SetVariableAction::new("test_var".to_string(), "test_value".to_string());
        let mut context = HashMap::new();

        let result = action.execute(&mut context).await.unwrap();
        assert_eq!(result, Value::String("test_value".to_string()));
        assert_eq!(
            context.get("test_var"),
            Some(&Value::String("test_value".to_string()))
        );
    }
}
