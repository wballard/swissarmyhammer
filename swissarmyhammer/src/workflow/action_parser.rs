//! Action parsing utilities for workflow state descriptions

use crate::workflow::actions::{
    ActionError, ActionResult, LogAction, LogLevel, PromptAction, SetVariableAction, SubWorkflowAction, WaitAction,
};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Robust action parser using regex patterns
pub struct ActionParser {
    /// Regex for parsing prompt actions
    prompt_regex: Regex,
    /// Regex for parsing wait actions with duration
    wait_duration_regex: Regex,
    /// Regex for parsing log actions
    log_regex: Regex,
    /// Regex for parsing set variable actions
    set_variable_regex: Regex,
    /// Regex for parsing arguments
    argument_regex: Regex,
    /// Regex for parsing sub-workflow actions
    sub_workflow_regex: Regex,
}

impl ActionParser {
    /// Create a new action parser with compiled regex patterns
    pub fn new() -> ActionResult<Self> {
        Ok(Self {
            prompt_regex: Regex::new(r#"^[Ee]xecute\s+prompt\s+"([^"]+)"(?:\s+with\s+(.+))?$"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile prompt regex: {}", e)))?,
            wait_duration_regex: Regex::new(r#"^[Ww]ait\s+(\d+)\s+(seconds?|minutes?|hours?|sec|min|h|s|m)(?:\s+(.+))?$"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile wait duration regex: {}", e)))?,
            log_regex: Regex::new(r#"^[Ll]og\s+(?:(error|warning)\s+)?"([^"]+)"$"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile log regex: {}", e)))?,
            set_variable_regex: Regex::new(r#"^[Ss]et\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*"?([^"]*)"?$"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile set variable regex: {}", e)))?,
            argument_regex: Regex::new(r#"([a-zA-Z_][a-zA-Z0-9_-]*)="([^"]*)"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile argument regex: {}", e)))?,
            sub_workflow_regex: Regex::new(r#"^(?:[Rr]un\s+workflow|[Dd]elegate(?:\s+to)?)\s+"([^"]+)"(?:\s+with\s+(.+))?$"#)
                .map_err(|e| ActionError::ParseError(format!("Failed to compile sub-workflow regex: {}", e)))?,
        })
    }

    /// Parse a prompt action from description
    /// Format: Execute prompt "prompt-name" with arg1="value1" arg2="value2"
    pub fn parse_prompt_action(&self, description: &str) -> ActionResult<Option<PromptAction>> {
        if let Some(captures) = self.prompt_regex.captures(description.trim()) {
            let prompt_name = captures.get(1).unwrap().as_str().to_string();
            let mut action = PromptAction::new(prompt_name);

            // Parse arguments if present
            if let Some(args_match) = captures.get(2) {
                let args_str = args_match.as_str();
                for arg_capture in self.argument_regex.captures_iter(args_str) {
                    if let (Some(key), Some(value)) = (arg_capture.get(1), arg_capture.get(2)) {
                        let key = key.as_str().to_string();
                        let value = value.as_str().to_string();
                        
                        // Validate key format
                        if !self.is_valid_argument_key(&key) {
                            return Err(ActionError::ParseError(
                                format!("Invalid argument key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                            ));
                        }
                        
                        action.arguments.insert(key, value);
                    }
                }
            }

            return Ok(Some(action));
        }

        Ok(None)
    }

    /// Parse a wait action from description
    /// Format: Wait for user confirmation OR Wait 30 seconds
    pub fn parse_wait_action(&self, description: &str) -> ActionResult<Option<WaitAction>> {
        let lower_desc = description.to_lowercase();

        // Check for user input wait
        if lower_desc.contains("wait for user") {
            return Ok(Some(
                WaitAction::new_user_input().with_message(description.to_string()),
            ));
        }

        // Check for duration wait
        if let Some(captures) = self.wait_duration_regex.captures(description.trim()) {
            let duration_value: u64 = captures.get(1).unwrap().as_str().parse()
                .map_err(|_| ActionError::ParseError("Invalid duration value".to_string()))?;
            let unit = captures.get(2).unwrap().as_str().to_lowercase();
            let message = captures.get(3).map(|m| m.as_str().to_string());

            let duration = self.parse_duration_unit(duration_value, &unit)?;
            let mut action = WaitAction::new_duration(duration);
            
            if let Some(msg) = message {
                action = action.with_message(msg);
            }

            return Ok(Some(action));
        }

        Ok(None)
    }

    /// Parse a log action from description
    /// Format: Log "message" OR Log error "message"
    pub fn parse_log_action(&self, description: &str) -> ActionResult<Option<LogAction>> {
        if let Some(captures) = self.log_regex.captures(description.trim()) {
            let level_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let message = captures.get(2).unwrap().as_str().to_string();

            let level = match level_str.to_lowercase().as_str() {
                "error" => LogLevel::Error,
                "warning" => LogLevel::Warning,
                _ => LogLevel::Info,
            };

            return Ok(Some(LogAction::new(message, level)));
        }

        Ok(None)
    }

    /// Parse a set variable action from description
    /// Format: Set variable_name="${value}"
    pub fn parse_set_variable_action(&self, description: &str) -> ActionResult<Option<SetVariableAction>> {
        if let Some(captures) = self.set_variable_regex.captures(description.trim()) {
            let var_name = captures.get(1).unwrap().as_str().to_string();
            let value = captures.get(2).unwrap().as_str().to_string();

            // Validate variable name
            if !self.is_valid_variable_name(&var_name) {
                return Err(ActionError::ParseError(
                    format!("Invalid variable name '{}': must start with letter or underscore and contain only alphanumeric characters and underscores", var_name)
                ));
            }

            return Ok(Some(SetVariableAction::new(var_name, value)));
        }

        Ok(None)
    }

    /// Parse a sub-workflow action from description
    /// Format: Run workflow "workflow-name" with input1="value1" input2="value2"
    /// Format: Delegate to "workflow-name" with input="${data}"
    pub fn parse_sub_workflow_action(&self, description: &str) -> ActionResult<Option<SubWorkflowAction>> {
        if let Some(captures) = self.sub_workflow_regex.captures(description.trim()) {
            let workflow_name = captures.get(1).unwrap().as_str().to_string();
            let mut action = SubWorkflowAction::new(workflow_name);

            // Parse input variables if present
            if let Some(inputs_match) = captures.get(2) {
                let inputs_str = inputs_match.as_str();
                
                // Check if it's a single input without quotes (e.g., with input="${data}")
                if inputs_str.starts_with("input=") {
                    let value = inputs_str.strip_prefix("input=").unwrap_or("");
                    let value = value.trim_matches('"');
                    action.input_variables.insert("input".to_string(), value.to_string());
                } else {
                    // Parse multiple arguments
                    for arg_capture in self.argument_regex.captures_iter(inputs_str) {
                        if let (Some(key), Some(value)) = (arg_capture.get(1), arg_capture.get(2)) {
                            let key = key.as_str().to_string();
                            let value = value.as_str().to_string();
                            
                            // Validate key format
                            if !self.is_valid_argument_key(&key) {
                                return Err(ActionError::ParseError(
                                    format!("Invalid input variable key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                                ));
                            }
                            
                            action.input_variables.insert(key, value);
                        }
                    }
                }
            }

            return Ok(Some(action));
        }

        Ok(None)
    }

    /// Safely substitute variables in a string using regex
    pub fn substitute_variables_safe(&self, input: &str, context: &HashMap<String, Value>) -> ActionResult<String> {
        let var_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_.-]*)\}")
            .map_err(|e| ActionError::ParseError(format!("Failed to compile variable regex: {}", e)))?;
        
        let result = var_regex.replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            context.get(var_name)
                .map(|v| self.value_to_string(v))
                .unwrap_or_else(|| format!("${{{}}}", var_name))
        });

        Ok(result.into_owned())
    }

    /// Parse duration unit string into Duration
    fn parse_duration_unit(&self, value: u64, unit: &str) -> ActionResult<Duration> {
        match unit {
            "second" | "seconds" | "sec" | "s" => Ok(Duration::from_secs(value)),
            "minute" | "minutes" | "min" | "m" => Ok(Duration::from_secs(value * 60)),
            "hour" | "hours" | "h" => Ok(Duration::from_secs(value * 3600)),
            _ => Err(ActionError::ParseError(
                format!("Invalid duration unit: {}", unit)
            )),
        }
    }

    /// Validate that an argument key is safe for command-line use
    fn is_valid_argument_key(&self, key: &str) -> bool {
        !key.is_empty() && 
        key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Validate that a variable name is valid
    fn is_valid_variable_name(&self, name: &str) -> bool {
        !name.is_empty() && 
        name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') &&
        name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// Convert a JSON Value to a string representation
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) | Value::Object(_) => value.to_string(),
        }
    }
}

impl Default for ActionParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default ActionParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prompt_action() {
        let parser = ActionParser::new().unwrap();
        
        // Test basic prompt
        let action = parser.parse_prompt_action("Execute prompt \"analyze-code\"").unwrap().unwrap();
        assert_eq!(action.prompt_name, "analyze-code");
        assert!(action.arguments.is_empty());

        // Test prompt with arguments
        let action = parser.parse_prompt_action("Execute prompt \"analyze-code\" with file=\"test.rs\" verbose=\"true\"").unwrap().unwrap();
        assert_eq!(action.prompt_name, "analyze-code");
        assert_eq!(action.arguments.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(action.arguments.get("verbose"), Some(&"true".to_string()));

        // Test invalid format
        let result = parser.parse_prompt_action("Execute prompt analyze-code");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_wait_action() {
        let parser = ActionParser::new().unwrap();
        
        // Test user input wait
        let action = parser.parse_wait_action("Wait for user confirmation").unwrap().unwrap();
        assert!(action.duration.is_none());

        // Test duration wait
        let action = parser.parse_wait_action("Wait 30 seconds").unwrap().unwrap();
        assert_eq!(action.duration, Some(Duration::from_secs(30)));

        // Test duration with different units
        let action = parser.parse_wait_action("Wait 5 minutes").unwrap().unwrap();
        assert_eq!(action.duration, Some(Duration::from_secs(300)));

        // Test invalid format
        let result = parser.parse_wait_action("Wait invalid");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_log_action() {
        let parser = ActionParser::new().unwrap();
        
        // Test info log
        let action = parser.parse_log_action("Log \"Hello world\"").unwrap().unwrap();
        assert_eq!(action.message, "Hello world");
        assert!(matches!(action.level, LogLevel::Info));

        // Test error log
        let action = parser.parse_log_action("Log error \"Something failed\"").unwrap().unwrap();
        assert_eq!(action.message, "Something failed");
        assert!(matches!(action.level, LogLevel::Error));

        // Test warning log
        let action = parser.parse_log_action("Log warning \"Be careful\"").unwrap().unwrap();
        assert_eq!(action.message, "Be careful");
        assert!(matches!(action.level, LogLevel::Warning));
    }

    #[test]
    fn test_parse_set_variable_action() {
        let parser = ActionParser::new().unwrap();
        
        // Test basic set
        let action = parser.parse_set_variable_action("Set result=\"success\"").unwrap().unwrap();
        assert_eq!(action.variable_name, "result");
        assert_eq!(action.value, "success");

        // Test with variable substitution
        let action = parser.parse_set_variable_action("Set output=\"${claude_response}\"").unwrap().unwrap();
        assert_eq!(action.variable_name, "output");
        assert_eq!(action.value, "${claude_response}");

        // Test invalid variable name
        let result = parser.parse_set_variable_action("Set 123invalid=\"value\"");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_variable_substitution() {
        let parser = ActionParser::new().unwrap();
        let mut context = HashMap::new();
        context.insert("file".to_string(), Value::String("test.rs".to_string()));
        context.insert("count".to_string(), Value::Number(42.into()));

        let result = parser.substitute_variables_safe("Process ${file} with ${count} items", &context).unwrap();
        assert_eq!(result, "Process test.rs with 42 items");

        // Test with missing variable
        let result = parser.substitute_variables_safe("Process ${missing} file", &context).unwrap();
        assert_eq!(result, "Process ${missing} file");
    }

    #[test]
    fn test_parse_sub_workflow_action() {
        let parser = ActionParser::new().unwrap();
        
        // Test "Run workflow" format
        let action = parser.parse_sub_workflow_action("Run workflow \"validation-workflow\"").unwrap().unwrap();
        assert_eq!(action.workflow_name, "validation-workflow");
        assert!(action.input_variables.is_empty());

        // Test "Run workflow" with arguments
        let action = parser.parse_sub_workflow_action("Run workflow \"analyze-code\" with file=\"test.rs\" mode=\"strict\"").unwrap().unwrap();
        assert_eq!(action.workflow_name, "analyze-code");
        assert_eq!(action.input_variables.get("file"), Some(&"test.rs".to_string()));
        assert_eq!(action.input_variables.get("mode"), Some(&"strict".to_string()));

        // Test "Delegate to" format
        let action = parser.parse_sub_workflow_action("Delegate to \"validation-workflow\" with input=\"${data}\"").unwrap().unwrap();
        assert_eq!(action.workflow_name, "validation-workflow");
        assert_eq!(action.input_variables.get("input"), Some(&"${data}".to_string()));

        // Test case insensitive
        let action = parser.parse_sub_workflow_action("run workflow \"test-workflow\"").unwrap().unwrap();
        assert_eq!(action.workflow_name, "test-workflow");

        // Test invalid format
        let result = parser.parse_sub_workflow_action("Run workflow test-workflow");
        assert!(result.unwrap().is_none());
    }
}