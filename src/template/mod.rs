use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;

pub struct TemplateEngine {
    // For future extensibility
    placeholder_regex: Regex,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self {
            placeholder_regex: Regex::new(r"\{\{([^}]+)\}\}").unwrap(),
        }
    }
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process a template string with the given arguments
    pub fn process(&self, template: &str, arguments: &HashMap<String, Value>) -> Result<String> {
        let mut result = template.to_string();
        
        // Handle triple braces specially - {{{var}}} becomes {value}
        let triple_brace_regex = Regex::new(r"\{\{\{([^}]+)\}\}\}").unwrap();
        for cap in triple_brace_regex.captures_iter(template) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str().trim();
            
            if let Some(value) = arguments.get(var_name) {
                let replacement = format!("{{{}}}", self.value_to_string(value));
                result = result.replace(full_match, &replacement);
            }
        }
        
        // Now handle regular double braces
        for cap in self.placeholder_regex.captures_iter(&result.clone()) {
            let full_match = cap.get(0).unwrap().as_str();
            let var_name = cap.get(1).unwrap().as_str().trim();
            
            if let Some(value) = arguments.get(var_name) {
                let replacement = self.value_to_string(value);
                result = result.replace(full_match, &replacement);
            }
        }
        
        Ok(result)
    }

    /// Process template with validation against expected arguments
    pub fn process_with_validation(
        &self,
        template: &str,
        arguments: &HashMap<String, Value>,
        expected_args: &[TemplateArgument],
    ) -> Result<String> {
        // First, validate required arguments
        for arg in expected_args {
            if arg.required && !arguments.contains_key(&arg.name) {
                return Err(anyhow!(
                    "Missing required argument '{}'{}", 
                    arg.name,
                    arg.description
                        .as_ref()
                        .map(|d| format!(": {}", d))
                        .unwrap_or_default()
                ));
            }
        }
        
        // Build effective arguments map with defaults
        let mut effective_args = arguments.clone();
        for arg in expected_args {
            if !effective_args.contains_key(&arg.name) {
                if let Some(default) = &arg.default_value {
                    effective_args.insert(arg.name.clone(), Value::String(default.clone()));
                }
            }
        }
        
        self.process(template, &effective_args)
    }
    
    /// Convert a JSON value to a string representation
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            _ => value.to_string(), // For arrays and objects, use JSON representation
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_substitution() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}, welcome to {{place}}!";
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Alice"));
        args.insert("place".to_string(), json!("Wonderland"));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "Hello Alice, welcome to Wonderland!");
    }

    #[test]
    fn test_missing_argument_no_validation() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}!";
        let args = HashMap::new();

        let result = engine.process(template, &args).unwrap();
        // Without validation, missing args are left as-is
        assert_eq!(result, "Hello {{name}}!");
    }

    #[test]
    fn test_required_argument_validation() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}!";
        let args = HashMap::new();
        let expected_args = vec![
            TemplateArgument {
                name: "name".to_string(),
                description: Some("The person's name".to_string()),
                required: true,
                default_value: None,
            }
        ];

        let result = engine.process_with_validation(template, &args, &expected_args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required"));
    }

    #[test]
    fn test_default_value() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}!";
        let args = HashMap::new();
        let expected_args = vec![
            TemplateArgument {
                name: "name".to_string(),
                description: Some("The person's name".to_string()),
                required: false,
                default_value: Some("Guest".to_string()),
            }
        ];

        let result = engine.process_with_validation(template, &args, &expected_args).unwrap();
        assert_eq!(result, "Hello Guest!");
    }

    #[test]
    fn test_numeric_value() {
        let engine = TemplateEngine::new();
        let template = "The answer is {{number}}";
        let mut args = HashMap::new();
        args.insert("number".to_string(), json!(42));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "The answer is 42");
    }

    #[test]
    fn test_boolean_value() {
        let engine = TemplateEngine::new();
        let template = "Is ready: {{ready}}";
        let mut args = HashMap::new();
        args.insert("ready".to_string(), json!(true));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "Is ready: true");
    }

    #[test]
    fn test_multiple_occurrences() {
        let engine = TemplateEngine::new();
        let template = "{{name}} says hello. {{name}} is happy!";
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("Bob"));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "Bob says hello. Bob is happy!");
    }

    #[test]
    fn test_nested_braces() {
        let engine = TemplateEngine::new();
        let template = "Code: {{{example}}}";
        let mut args = HashMap::new();
        args.insert("example".to_string(), json!("function() {}"));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "Code: {function() {}}");
    }

    #[test]
    fn test_special_characters() {
        let engine = TemplateEngine::new();
        let template = "Path: {{path}}";
        let mut args = HashMap::new();
        args.insert("path".to_string(), json!("/home/user/file.txt"));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "Path: /home/user/file.txt");
    }

    #[test]
    fn test_empty_template() {
        let engine = TemplateEngine::new();
        let template = "";
        let args = HashMap::new();

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_placeholders() {
        let engine = TemplateEngine::new();
        let template = "This is plain text.";
        let mut args = HashMap::new();
        args.insert("unused".to_string(), json!("value"));

        let result = engine.process(template, &args).unwrap();
        assert_eq!(result, "This is plain text.");
    }
}