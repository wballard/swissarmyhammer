//! Workflow action execution system
//!
//! This module provides the action execution infrastructure for workflows,
//! including Claude integration, variable operations, and control flow actions.

use crate::workflow::action_parser::ActionParser;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::process::Command;
use tokio::time::timeout;

/// Macro to implement the as_any() method for Action trait implementations
macro_rules! impl_as_any {
    () => {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    };
}

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
    /// Rate limit error with retry time
    #[error("Rate limit reached. Please wait {wait_time:?} and try again. Details: {message}")]
    RateLimit {
        /// The error message
        message: String,
        /// How long to wait before retrying
        wait_time: Duration,
    },
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

    /// For testing: allow downcasting
    #[doc(hidden)]
    fn as_any(&self) -> &dyn std::any::Any;
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
    /// Maximum number of retries for rate limit errors
    pub max_retries: u32,
    // TODO: Future enhancement - Add configurable retry strategy
    // pub retry_strategy: RetryStrategy,
    // where RetryStrategy could be:
    // - WaitUntilNextHour (current behavior)
    // - ExponentialBackoff { base: Duration, max: Duration }
    // - FixedDelay(Duration)
    // - Custom(Box<dyn Fn(u32) -> Duration>)
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
            max_retries: 2,                    // Default to 2 retries for rate limits
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

    /// Set the maximum number of retries for rate limit errors
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
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
        let mut retries = 0;

        loop {
            // Execute the command and handle rate limit errors
            match self.execute_once(context).await {
                Ok(response) => return Ok(response),
                Err(ActionError::RateLimit { message, wait_time }) => {
                    if retries >= self.max_retries {
                        // Max retries exceeded, return the rate limit error
                        return Err(ActionError::RateLimit { message, wait_time });
                    }

                    retries += 1;
                    tracing::warn!(
                        "Rate limit reached (attempt {}/{}). Waiting {:?} until next hour...",
                        retries,
                        self.max_retries + 1,
                        wait_time
                    );

                    // Wait until the next hour
                    tokio::time::sleep(wait_time).await;

                    tracing::info!("Retrying after rate limit wait...");
                    // Continue to retry
                }
                Err(e) => return Err(e), // Other errors are not retried
            }
        }
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

    impl_as_any!();
}

impl PromptAction {
    /// Render the prompt using swissarmyhammer prompt test
    async fn render_prompt_with_swissarmyhammer(
        &self,
        context: &HashMap<String, Value>,
    ) -> ActionResult<String> {
        // Substitute variables in arguments
        let args = self.substitute_variables(context);

        // Build the command to render the prompt
        // Use the current executable path to ensure we call the right binary
        let current_exe =
            std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("swissarmyhammer"));

        // In test environment, current_exe might point to test binary, so try to find the actual swissarmyhammer binary
        let cmd_binary = if current_exe.to_string_lossy().contains("test")
            || current_exe.to_string_lossy().contains("deps")
        {
            // We're in a test, try to find the actual binary
            if let Ok(cargo_target) = std::env::var("CARGO_TARGET_DIR") {
                let debug_binary = std::path::PathBuf::from(cargo_target)
                    .join("debug")
                    .join("swissarmyhammer");
                if debug_binary.exists() {
                    debug_binary
                } else {
                    std::path::PathBuf::from("swissarmyhammer")
                }
            } else {
                // Fallback to target/debug/swissarmyhammer relative to workspace
                let workspace_dir =
                    std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
                let workspace_root = std::path::Path::new(&workspace_dir)
                    .parent()
                    .unwrap_or(std::path::Path::new("."));
                let debug_binary = workspace_root
                    .join("target")
                    .join("debug")
                    .join("swissarmyhammer");
                if debug_binary.exists() {
                    debug_binary
                } else {
                    std::path::PathBuf::from("swissarmyhammer")
                }
            }
        } else {
            current_exe
        };

        let mut cmd = Command::new(&cmd_binary);
        cmd.arg("prompt")
            .arg("test")
            .arg(&self.prompt_name)
            .arg("--raw"); // Get raw output without formatting

        // Add arguments
        for (key, value) in &args {
            if !is_valid_argument_key(key) {
                return Err(ActionError::ParseError(
                    format!("Invalid argument key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                ));
            }
            cmd.arg("--arg");
            cmd.arg(format!("{}={}", key, value));
        }

        // Set up process pipes
        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::null());

        // Execute the command
        let output = cmd.output().await.map_err(|e| {
            ActionError::ClaudeError(format!(
                "Failed to render prompt with swissarmyhammer: {}",
                e
            ))
        })?;

        // Check for errors
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ActionError::ClaudeError(format!(
                "Failed to render prompt '{}': {}",
                self.prompt_name, stderr
            )));
        }

        // Get the rendered prompt
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Execute the command once without retry logic
    ///
    /// This method performs a single execution attempt of the Claude command.
    /// Rate limit errors are propagated to the caller for retry handling.
    ///
    /// # Arguments
    /// * `context` - The workflow execution context
    ///
    /// # Returns
    /// * `Ok(Value)` - The command response on success
    /// * `Err(ActionError)` - Various errors including rate limits
    async fn execute_once(&self, context: &mut HashMap<String, Value>) -> ActionResult<Value> {
        // First, render the prompt using swissarmyhammer
        let rendered_prompt = self.render_prompt_with_swissarmyhammer(context).await?;

        // Log the actual prompt being sent to Claude
        tracing::debug!("Piping prompt to Claude:\n{}", rendered_prompt);

        // Check if quiet mode is enabled in the context
        let quiet = self.quiet
            || context
                .get("_quiet")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

        // Execute the rendered prompt with Claude
        // Find claude in PATH or use common locations
        let claude_path = which::which("claude")
            .or_else(|_| {
                // Check common installation paths
                let home = std::env::var("HOME").unwrap_or_default();
                let possible_paths = vec![
                    format!("{}/.claude/local/claude", home),
                    "/usr/local/bin/claude".to_string(),
                    "/opt/claude/claude".to_string(),
                ];

                for path in possible_paths {
                    if std::path::Path::new(&path).exists() {
                        return Ok(std::path::PathBuf::from(path));
                    }
                }

                Err(which::Error::CannotFindBinaryPath)
            })
            .map_err(|e| {
                ActionError::ClaudeError(format!(
                    "Claude CLI not found. Make sure 'claude' is installed and available in your PATH. Error: {}",
                    e
                ))
            })?;
        let mut cmd = Command::new(&claude_path);

        // Claude CLI arguments
        cmd.arg("--dangerously-skip-permissions")
            .arg("--print")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--verbose");

        // Set up the command to pipe prompt via stdin
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if !quiet {
            tracing::info!(
                "Executing prompt '{}' with Claude at: {}",
                self.prompt_name,
                claude_path.display()
            );
        }

        // Spawn the Claude process
        let mut child = cmd.spawn().map_err(|e| {
            ActionError::ClaudeError(format!("Failed to spawn Claude command: {}", e))
        })?;

        // Write the prompt to Claude's stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(rendered_prompt.as_bytes())
                .await
                .map_err(|e| {
                    ActionError::ClaudeError(format!("Failed to write prompt to Claude: {}", e))
                })?;
            stdin.shutdown().await.map_err(|e| {
                ActionError::ClaudeError(format!("Failed to close Claude stdin: {}", e))
            })?;
        }

        // Get stdout for streaming
        let stdout = child.stdout.take().ok_or_else(|| {
            ActionError::ClaudeError("Failed to capture Claude stdout".to_string())
        })?;

        // Read stdout line by line for streaming JSON
        use tokio::io::{AsyncBufReadExt, BufReader};
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let mut response_text = String::new();
        let mut _got_result = false;

        // Get timeout from context or use default
        // Use 30 seconds per line as a reasonable default for streaming
        let line_timeout = Duration::from_secs(30);

        tracing::debug!("Using line timeout: {:?}", line_timeout);

        // Read lines with timeout
        loop {
            match timeout(line_timeout, lines.next_line()).await {
                Ok(Ok(Some(line))) => {
                    // Format JSON as YAML for better readability
                    let formatted_output = format_claude_output_as_yaml(&line);
                    if formatted_output != line {
                        // If it was formatted as YAML, log it with proper structure
                        tracing::debug!("Claude output line:\n{}", formatted_output);
                    } else {
                        // If not JSON, log as is
                        tracing::debug!("Claude output line: {}", line);
                    }

                    if line.trim().is_empty() {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(&line) {
                        // Look for the final result
                        if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
                            response_text = result.to_string();
                            _got_result = true;
                            break;
                        }
                        // Also check for assistant messages with text content
                        if json.get("type").and_then(|t| t.as_str()) == Some("assistant") {
                            if let Some(message) = json.get("message").and_then(|m| m.as_object()) {
                                if let Some(content_array) =
                                    message.get("content").and_then(|c| c.as_array())
                                {
                                    for content_item in content_array {
                                        if let Some(text) =
                                            content_item.get("text").and_then(|t| t.as_str())
                                        {
                                            response_text.push_str(text);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Ok(None)) => {
                    // End of stream
                    break;
                }
                Ok(Err(e)) => {
                    tracing::error!("Error reading Claude output: {}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - kill the process and return error
                    tracing::error!("Timeout reading Claude output after {:?}", line_timeout);
                    tracing::error!("Response so far: {} characters", response_text.len());
                    let _ = child.kill().await;
                    // If we have some response, use it rather than erroring
                    if !response_text.is_empty() {
                        break;
                    }
                    return Err(ActionError::Timeout {
                        timeout: line_timeout,
                    });
                }
            }
        }

        // Wait for process to complete with a short timeout
        let wait_result = timeout(Duration::from_secs(5), child.wait()).await;
        let status = match wait_result {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                return Err(ActionError::ClaudeError(format!(
                    "Failed to wait for Claude: {}",
                    e
                )))
            }
            Err(_) => {
                // Process didn't exit cleanly, kill it
                let _ = child.kill().await;
                return Err(ActionError::ClaudeError(
                    "Claude process failed to exit cleanly".to_string(),
                ));
            }
        };

        if !status.success() {
            return Err(ActionError::ClaudeError(
                "Claude execution failed".to_string(),
            ));
        }

        let response_text = response_text.trim();

        if response_text.is_empty() {
            tracing::warn!("No response received from Claude");
        } else {
            tracing::debug!(
                "Claude response received: {} characters",
                response_text.len()
            );
        }

        // Display the output as YAML
        if !quiet && !response_text.is_empty() {
            // Build the YAML output as a single string
            let mut yaml_output = String::new();
            yaml_output.push_str("\n---\n");
            yaml_output.push_str(&format!("prompt: {}\n", self.prompt_name));
            yaml_output.push_str("claude_response: |\n");
            for line in response_text.lines() {
                yaml_output.push_str(&format!("  {}\n", line));
            }
            yaml_output.push_str("---");

            // Log YAML output
            tracing::info!("{}", yaml_output);
        }

        // Create a response value
        let response = Value::String(response_text.to_string());

        // Store result in context if variable name specified
        if let Some(var_name) = &self.result_variable {
            context.insert(var_name.clone(), response.clone());
        }

        // Always store in special last_action_result key
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
        context.insert(CLAUDE_RESPONSE_KEY.to_string(), response.clone());

        Ok(response)
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
                    tracing::info!("Waiting: {}", message);
                }
                tokio::time::sleep(duration).await;
            }
            None => {
                let message = self
                    .message
                    .as_deref()
                    .unwrap_or("Press Enter to continue...");
                tracing::info!("{}", message);

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

    impl_as_any!();
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
            LogLevel::Info => tracing::info!("{}", message),
            LogLevel::Warning => tracing::warn!("{}", message),
            LogLevel::Error => tracing::error!("{}", message),
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

    impl_as_any!();
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

    impl_as_any!();
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

    impl_as_any!();
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

/// Format Claude output JSON line as YAML for better readability
#[cfg_attr(test, allow(dead_code))]
pub(crate) fn format_claude_output_as_yaml(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Try to parse as JSON
    match serde_json::from_str::<Value>(trimmed) {
        Ok(json_value) => {
            // Convert to YAML
            match serde_yaml::to_string(&json_value) {
                Ok(yaml) => yaml,
                Err(_) => trimmed.to_string(), // Fall back to original if YAML conversion fails
            }
        }
        Err(_) => trimmed.to_string(), // Return original if not valid JSON
    }
}

/// Parse action from state description text with liquid template rendering
pub fn parse_action_from_description_with_context(
    description: &str,
    context: &HashMap<String, Value>,
) -> ActionResult<Option<Box<dyn Action>>> {
    let rendered_description = if let Some(template_vars) = context.get("_template_vars") {
        // Extract template variables from context
        if let Some(vars_map) = template_vars.as_object() {
            // Convert to liquid Object
            let mut liquid_vars = liquid::Object::new();
            for (key, value) in vars_map {
                liquid_vars.insert(
                    key.clone().into(),
                    liquid::model::to_value(value).unwrap_or(liquid::model::Value::Nil),
                );
            }

            // Parse and render the template
            match liquid::ParserBuilder::with_stdlib()
                .build()
                .and_then(|parser| parser.parse(description))
            {
                Ok(template) => match template.render(&liquid_vars) {
                    Ok(rendered) => rendered,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to render liquid template: {}. Using original text.",
                            e
                        );
                        description.to_string()
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse liquid template: {}. Using original text.",
                        e
                    );
                    description.to_string()
                }
            }
        } else {
            description.to_string()
        }
    } else {
        description.to_string()
    };

    parse_action_from_description(&rendered_description)
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

    #[tokio::test]
    async fn test_prompt_action_with_rate_limit_retry() {
        let action = PromptAction::new("test-prompt".to_string());

        // The action should have default max_retries
        assert_eq!(action.max_retries, 2);

        // Verify other defaults
        assert_eq!(action.prompt_name, "test-prompt");
        assert_eq!(action.timeout, Duration::from_secs(300));
        assert!(!action.quiet);
        assert!(action.arguments.is_empty());
        assert!(action.result_variable.is_none());
    }

    #[test]
    fn test_prompt_action_with_max_retries_builder() {
        let action = PromptAction::new("test-prompt".to_string()).with_max_retries(5);

        assert_eq!(action.max_retries, 5);
        assert_eq!(action.prompt_name, "test-prompt");

        // Test chaining with other builders
        let action2 = PromptAction::new("test-prompt2".to_string())
            .with_quiet(true)
            .with_max_retries(0)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(action2.max_retries, 0);
        assert!(action2.quiet);
        assert_eq!(action2.timeout, Duration::from_secs(60));
    }
}
