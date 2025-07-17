//! Action parsing utilities for workflow state descriptions

use crate::workflow::actions::{
    AbortAction, ActionError, ActionResult, LogAction, LogLevel, PromptAction, SetVariableAction,
    SubWorkflowAction, WaitAction,
};
use chumsky::prelude::*;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

// Type alias for parser error type
type ParserError<'a> = extra::Err<Rich<'a, char>>;

/// Robust action parser using chumsky parser combinators
pub struct ActionParser;

impl ActionParser {
    /// Create a new action parser
    pub fn new() -> ActionResult<Self> {
        Ok(Self)
    }

    // Helper parsers

    /// Parse whitespace (spaces and tabs)
    fn whitespace<'a>() -> impl Parser<'a, &'a str, (), ParserError<'a>> {
        one_of(" \t").repeated().at_least(1).ignored()
    }

    /// Parse optional whitespace
    fn opt_whitespace<'a>() -> impl Parser<'a, &'a str, (), ParserError<'a>> {
        one_of(" \t").repeated().ignored()
    }

    /// Parse a case-insensitive word
    fn case_insensitive<'a>(word: &'static str) -> impl Parser<'a, &'a str, (), ParserError<'a>> {
        any()
            .filter(move |c: &char| c.is_alphabetic())
            .repeated()
            .exactly(word.len())
            .collect::<String>()
            .try_map(move |s, span| {
                if s.to_lowercase() == word.to_lowercase() {
                    Ok(())
                } else {
                    Err(Rich::custom(
                        span,
                        format!("expected '{}', found '{}'", word, s),
                    ))
                }
            })
            .ignored()
    }

    /// Parse a quoted string
    fn quoted_string<'a>() -> impl Parser<'a, &'a str, String, ParserError<'a>> {
        just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
    }

    /// Parse an identifier (variable/argument name)
    fn identifier<'a>() -> impl Parser<'a, &'a str, String, ParserError<'a>> {
        any()
            .filter(|c: &char| c.is_alphabetic() || *c == '_')
            .then(
                any()
                    .filter(|c: &char| c.is_alphanumeric() || *c == '_')
                    .repeated()
                    .collect::<String>(),
            )
            .map(|(first, rest)| format!("{}{}", first, rest))
    }

    /// Parse an argument key (allows hyphens)
    fn argument_key<'a>() -> impl Parser<'a, &'a str, String, ParserError<'a>> {
        any()
            .filter(|c: &char| c.is_alphabetic() || *c == '_')
            .then(
                any()
                    .filter(|c: &char| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .repeated()
                    .collect::<String>(),
            )
            .map(|(first, rest)| format!("{}{}", first, rest))
    }

    /// Parse a prompt action from description
    /// Format: Execute prompt "prompt-name" with arg1="value1" arg2="value2"
    pub fn parse_prompt_action(&self, description: &str) -> ActionResult<Option<PromptAction>> {
        let parser = Self::case_insensitive("execute")
            .then_ignore(Self::whitespace())
            .then_ignore(Self::case_insensitive("prompt"))
            .then_ignore(Self::whitespace())
            .ignore_then(Self::quoted_string())
            .then(
                Self::whitespace()
                    .ignore_then(Self::case_insensitive("with"))
                    .ignore_then(Self::whitespace())
                    .then(
                        Self::argument_key()
                            .then_ignore(just('='))
                            .then(Self::quoted_string())
                            .separated_by(Self::whitespace())
                            .collect::<Vec<(String, String)>>(),
                    )
                    .map(|(_, args)| args)
                    .or_not(),
            );

        match parser.parse(description.trim()).into_result() {
            Ok((prompt_name, args)) => {
                let mut action = PromptAction::new(prompt_name);

                if let Some(arguments) = args {
                    for (key, value) in arguments {
                        if key == "result" {
                            action = action.with_result_variable(value);
                        } else {
                            if !self.is_valid_argument_key(&key) {
                                return Err(ActionError::ParseError(
                                    format!("Invalid argument key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                                ));
                            }
                            action.arguments.insert(key, value);
                        }
                    }
                }

                Ok(Some(action))
            }
            Err(_) => Ok(None),
        }
    }

    /// Parse a wait action from description
    /// Format: Wait for user confirmation OR Wait 30 seconds
    pub fn parse_wait_action(&self, description: &str) -> ActionResult<Option<WaitAction>> {
        // Parser for "Wait for user" is now handled by string contains check below

        // Parser for duration units
        let duration_unit = choice((
            Self::case_insensitive("seconds")
                .to("seconds".to_string())
                .or(Self::case_insensitive("second").to("second".to_string())),
            Self::case_insensitive("minutes")
                .to("minutes".to_string())
                .or(Self::case_insensitive("minute").to("minute".to_string())),
            Self::case_insensitive("hours")
                .to("hours".to_string())
                .or(Self::case_insensitive("hour").to("hour".to_string())),
            Self::case_insensitive("sec").to("sec".to_string()),
            Self::case_insensitive("min").to("min".to_string()),
            just("h").to("h".to_string()),
            just("s").to("s".to_string()),
            just("m").to("m".to_string()),
        ));

        // Parser for "Wait N units"
        let wait_duration = Self::case_insensitive("wait")
            .then_ignore(Self::whitespace())
            .ignore_then(text::int(10).from_str::<u64>().unwrapped())
            .then_ignore(Self::whitespace())
            .then(duration_unit)
            .then(
                Self::whitespace()
                    .then(any().repeated().collect::<String>())
                    .map(|(_, msg)| msg)
                    .or_not(),
            );

        // Try to parse as user wait first
        let lower_desc = description.to_lowercase();
        if lower_desc.contains("wait for user") {
            return Ok(Some(
                WaitAction::new_user_input().with_message(description.to_string()),
            ));
        }

        // Try to parse as duration wait
        match wait_duration.parse(description.trim()).into_result() {
            Ok(((duration_value, unit), message)) => {
                let duration = self.parse_duration_unit(duration_value, &unit)?;
                let mut action = WaitAction::new_duration(duration);

                if let Some(msg) = message {
                    action = action.with_message(msg);
                }

                Ok(Some(action))
            }
            Err(_) => Ok(None),
        }
    }

    /// Parse a log action from description
    /// Format: Log "message" OR Log error "message"
    pub fn parse_log_action(&self, description: &str) -> ActionResult<Option<LogAction>> {
        let log_level = choice((
            Self::case_insensitive("error").to(LogLevel::Error),
            Self::case_insensitive("warning").to(LogLevel::Warning),
        ));

        let parser = Self::case_insensitive("log")
            .then_ignore(Self::whitespace())
            .ignore_then(log_level.then_ignore(Self::whitespace()).or_not())
            .then(Self::quoted_string());

        match parser.parse(description.trim()).into_result() {
            Ok((level, message)) => {
                let level = level.unwrap_or(LogLevel::Info);
                Ok(Some(LogAction::new(message, level)))
            }
            Err(_) => Ok(None),
        }
    }

    /// Parse a set variable action from description
    /// Format: Set variable_name="${value}"
    pub fn parse_set_variable_action(
        &self,
        description: &str,
    ) -> ActionResult<Option<SetVariableAction>> {
        let value_parser = choice((
            Self::quoted_string(),
            none_of('"').repeated().at_least(1).collect::<String>(),
        ));

        let parser = Self::case_insensitive("set")
            .then_ignore(Self::whitespace())
            .ignore_then(Self::identifier())
            .then_ignore(Self::opt_whitespace())
            .then_ignore(just('='))
            .then_ignore(Self::opt_whitespace())
            .then(value_parser);

        match parser.parse(description.trim()).into_result() {
            Ok((var_name, value)) => {
                // Validate variable name
                if !self.is_valid_variable_name(&var_name) {
                    return Err(ActionError::ParseError(
                        format!("Invalid variable name '{}': must start with letter or underscore and contain only alphanumeric characters and underscores", var_name)
                    ));
                }

                Ok(Some(SetVariableAction::new(var_name, value)))
            }
            Err(_) => Ok(None),
        }
    }

    /// Parse an abort action from description
    /// Format: Abort "error message" OR Abort with message "error message"
    pub fn parse_abort_action(&self, description: &str) -> ActionResult<Option<AbortAction>> {
        // Parser for simple "Abort" followed by quoted message
        let simple_abort = Self::case_insensitive("abort")
            .then_ignore(Self::whitespace())
            .ignore_then(Self::quoted_string());

        // Parser for "Abort with message" format
        let abort_with_message = Self::case_insensitive("abort")
            .then_ignore(Self::whitespace())
            .then_ignore(Self::case_insensitive("with"))
            .then_ignore(Self::whitespace())
            .then_ignore(Self::case_insensitive("message"))
            .then_ignore(Self::whitespace())
            .ignore_then(Self::quoted_string());

        // Try both formats
        let parser = choice((abort_with_message, simple_abort));

        match parser.parse(description.trim()).into_result() {
            Ok(message) => Ok(Some(AbortAction::new(message))),
            Err(_) => Ok(None),
        }
    }

    /// Parse a sub-workflow action from description
    /// Format: Run workflow "workflow-name" with input1="value1" input2="value2"
    /// Format: Delegate to "workflow-name" with input="${data}"
    pub fn parse_sub_workflow_action(
        &self,
        description: &str,
    ) -> ActionResult<Option<SubWorkflowAction>> {
        // Parser for "Run workflow" or "Delegate to"
        let run_workflow = Self::case_insensitive("run")
            .then_ignore(Self::whitespace())
            .then_ignore(Self::case_insensitive("workflow"));

        let delegate_to = Self::case_insensitive("delegate")
            .then_ignore(Self::whitespace())
            .then_ignore(Self::case_insensitive("to").or_not());

        let workflow_prefix = choice((run_workflow.to(()), delegate_to.to(())));

        // Parser for single input format
        let single_input = Self::case_insensitive("input")
            .then_ignore(just('='))
            .then(choice((
                Self::quoted_string(),
                none_of(' ').repeated().at_least(1).collect::<String>(),
            )));

        // Parser for arguments
        let argument_parser = Self::argument_key()
            .then_ignore(just('='))
            .then(Self::quoted_string())
            .separated_by(Self::whitespace())
            .collect::<Vec<(String, String)>>();

        let parser = workflow_prefix
            .then_ignore(Self::whitespace())
            .ignore_then(Self::quoted_string())
            .then(
                Self::whitespace()
                    .then_ignore(Self::case_insensitive("with"))
                    .then_ignore(Self::whitespace())
                    .then(choice((
                        single_input.map(|(_, value)| vec![("input".to_string(), value)]),
                        argument_parser,
                    )))
                    .map(|(_, args)| args)
                    .or_not(),
            );

        match parser.parse(description.trim()).into_result() {
            Ok((workflow_name, args)) => {
                let mut action = SubWorkflowAction::new(workflow_name);

                if let Some(arguments) = args {
                    for (key, value) in arguments {
                        if key == "result" {
                            action = action.with_result_variable(value);
                        } else {
                            if !self.is_valid_argument_key(&key) {
                                return Err(ActionError::ParseError(
                                    format!("Invalid input variable key '{}': must contain only alphanumeric characters, hyphens, and underscores", key)
                                ));
                            }
                            action.input_variables.insert(key, value);
                        }
                    }
                }

                Ok(Some(action))
            }
            Err(_) => Ok(None),
        }
    }

    /// Safely substitute variables in a string using regex
    pub fn substitute_variables_safe(
        &self,
        input: &str,
        context: &HashMap<String, Value>,
    ) -> ActionResult<String> {
        let var_regex = Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_.-]*)\}").map_err(|e| {
            ActionError::ParseError(format!("Failed to compile variable regex: {}", e))
        })?;

        let result = var_regex.replace_all(input, |caps: &regex::Captures| {
            let var_name = &caps[1];
            context
                .get(var_name)
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
            _ => Err(ActionError::ParseError(format!(
                "Invalid duration unit: {}",
                unit
            ))),
        }
    }

    /// Validate that an argument key is safe for command-line use
    fn is_valid_argument_key(&self, key: &str) -> bool {
        !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Validate that a variable name is valid
    fn is_valid_variable_name(&self, name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_alphanumeric() || c == '_')
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
        let action = parser
            .parse_prompt_action("Execute prompt \"analyze-code\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.prompt_name, "analyze-code");
        assert!(action.arguments.is_empty());

        // Test prompt with arguments
        let action = parser
            .parse_prompt_action(
                "Execute prompt \"analyze-code\" with file=\"test.rs\" verbose=\"true\"",
            )
            .unwrap()
            .unwrap();
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
        let result = parser
            .parse_wait_action("Wait for user confirmation")
            .unwrap();
        assert!(
            result.is_some(),
            "Failed to parse 'Wait for user confirmation'"
        );
        let action = result.unwrap();
        assert!(action.duration.is_none());

        // Test duration wait
        let action = parser
            .parse_wait_action("Wait 30 seconds")
            .unwrap()
            .unwrap();
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
        let action = parser
            .parse_log_action("Log \"Hello world\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Hello world");
        assert!(matches!(action.level, LogLevel::Info));

        // Test error log
        let action = parser
            .parse_log_action("Log error \"Something failed\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Something failed");
        assert!(matches!(action.level, LogLevel::Error));

        // Test warning log
        let action = parser
            .parse_log_action("Log warning \"Be careful\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Be careful");
        assert!(matches!(action.level, LogLevel::Warning));
    }

    #[test]
    fn test_parse_set_variable_action() {
        let parser = ActionParser::new().unwrap();

        // Test basic set
        let action = parser
            .parse_set_variable_action("Set result=\"success\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.variable_name, "result");
        assert_eq!(action.value, "success");

        // Test with variable substitution
        let action = parser
            .parse_set_variable_action("Set output=\"${claude_response}\"")
            .unwrap()
            .unwrap();
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

        let result = parser
            .substitute_variables_safe("Process ${file} with ${count} items", &context)
            .unwrap();
        assert_eq!(result, "Process test.rs with 42 items");

        // Test with missing variable
        let result = parser
            .substitute_variables_safe("Process ${missing} file", &context)
            .unwrap();
        assert_eq!(result, "Process ${missing} file");
    }

    #[test]
    fn test_parse_sub_workflow_action() {
        let parser = ActionParser::new().unwrap();

        // Test "Run workflow" format
        let action = parser
            .parse_sub_workflow_action("Run workflow \"validation-workflow\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.workflow_name, "validation-workflow");
        assert!(action.input_variables.is_empty());

        // Test "Run workflow" with arguments
        let action = parser
            .parse_sub_workflow_action(
                "Run workflow \"analyze-code\" with file=\"test.rs\" mode=\"strict\"",
            )
            .unwrap()
            .unwrap();
        assert_eq!(action.workflow_name, "analyze-code");
        assert_eq!(
            action.input_variables.get("file"),
            Some(&"test.rs".to_string())
        );
        assert_eq!(
            action.input_variables.get("mode"),
            Some(&"strict".to_string())
        );

        // Test "Delegate to" format
        let action = parser
            .parse_sub_workflow_action("Delegate to \"validation-workflow\" with input=\"${data}\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.workflow_name, "validation-workflow");
        assert_eq!(
            action.input_variables.get("input"),
            Some(&"${data}".to_string())
        );

        // Test case insensitive
        let action = parser
            .parse_sub_workflow_action("run workflow \"test-workflow\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.workflow_name, "test-workflow");

        // Test invalid format
        let result = parser.parse_sub_workflow_action("Run workflow test-workflow");
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_abort_action() {
        let parser = ActionParser::new().unwrap();

        // Test simple abort format
        let action = parser
            .parse_abort_action("Abort \"Test error message\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Test error message");

        // Test abort with message format
        let action = parser
            .parse_abort_action("Abort with message \"Critical error occurred\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "Critical error occurred");

        // Test case insensitive
        let action = parser
            .parse_abort_action("abort \"test error\"")
            .unwrap()
            .unwrap();
        assert_eq!(action.message, "test error");

        // Test invalid format
        let result = parser.parse_abort_action("Abort test error");
        assert!(result.unwrap().is_none());
    }
}
