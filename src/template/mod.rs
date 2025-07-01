use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;
use liquid::{ParserBuilder, Template};
use std::sync::Arc;

pub mod custom_filters;

pub struct TemplateEngine {
    // For future extensibility
    placeholder_regex: Regex,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self {
            placeholder_regex: Regex::new(r"\{\{([^}]+)\}\}")
                .expect("placeholder regex should be valid"),
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
        let triple_brace_regex = Regex::new(r"\{\{\{([^}]+)\}\}\}")
            .expect("triple brace regex should be valid");
        for cap in triple_brace_regex.captures_iter(template) {
            let full_match = cap.get(0)
                .ok_or_else(|| anyhow!("regex capture should have full match"))?
                .as_str();
            let var_name = cap.get(1)
                .ok_or_else(|| anyhow!("regex capture should have group 1"))?
                .as_str()
                .trim();
            
            if let Some(value) = arguments.get(var_name) {
                let replacement = format!("{{{}}}", self.value_to_string(value));
                result = result.replace(full_match, &replacement);
            }
        }
        
        // Now handle regular double braces
        for cap in self.placeholder_regex.captures_iter(&result.clone()) {
            let full_match = cap.get(0)
                .ok_or_else(|| anyhow!("regex capture should have full match"))?
                .as_str();
            let var_name = cap.get(1)
                .ok_or_else(|| anyhow!("regex capture should have group 1"))?
                .as_str()
                .trim();
            
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

/// Liquid-based template engine with advanced features
pub struct LiquidEngine {
    parser: liquid::Parser,
    // Cache for compiled templates
    template_cache: std::sync::Mutex<HashMap<String, Arc<Template>>>,
}

impl Default for LiquidEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LiquidEngine {
    pub fn new() -> Self {
        let parser = ParserBuilder::with_stdlib()
            .build()
            .expect("Failed to create Liquid parser");

        Self {
            parser,
            template_cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Process a template string with the given arguments using Liquid
    pub fn process(&self, template: &str, arguments: &HashMap<String, Value>) -> Result<String> {
        self.process_with_compatibility(template, arguments, true)
    }

    /// Process template with optional backward compatibility mode
    pub fn process_with_compatibility(&self, template: &str, arguments: &HashMap<String, Value>, backward_compatible: bool) -> Result<String> {
        let (processed_template, effective_args) = if backward_compatible {
            self.preprocess_for_compatibility(template, arguments)?
        } else {
            (template.to_string(), arguments.clone())
        };

        // Create cache key from processed template content
        let cache_key = processed_template.clone();
        
        // Check cache first
        {
            let cache = self.template_cache.lock().unwrap();
            if let Some(compiled_template) = cache.get(&cache_key) {
                return self.render_template(compiled_template, &effective_args);
            }
        }
        
        // Compile template
        let compiled_template = self.parser
            .parse(&processed_template)
            .map_err(|e| anyhow!("Failed to parse Liquid template: {}", e))?;
        
        let compiled_template = Arc::new(compiled_template);
        
        // Store in cache
        {
            let mut cache = self.template_cache.lock().unwrap();
            cache.insert(cache_key, compiled_template.clone());
        }
        
        let result = self.render_template(&compiled_template, &effective_args)?;
        
        // If using backward compatibility, restore undefined variable markers
        if backward_compatible {
            self.restore_undefined_variables(&result, template)
        } else {
            Ok(result)
        }
    }

    /// Preprocess template for backward compatibility with undefined variables
    fn preprocess_for_compatibility(&self, template: &str, arguments: &HashMap<String, Value>) -> Result<(String, HashMap<String, Value>)> {
        use regex::Regex;
        
        // First, identify raw blocks that should not be processed
        let raw_regex = Regex::new(r"(?s)\{\%\s*raw\s*\%\}.*?\{\%\s*endraw\s*\%\}")
            .map_err(|e| anyhow!("Failed to create raw block regex: {}", e))?;
        
        // Find all protected blocks and their positions  
        let for_regex = Regex::new(r"(?s)\{\%\s*for\s+.*?\{\%\s*endfor\s*\%\}")
            .map_err(|e| anyhow!("Failed to create for block regex: {}", e))?;
        
        let mut protected_blocks = Vec::new();
        protected_blocks.extend(raw_regex.find_iter(template));
        protected_blocks.extend(for_regex.find_iter(template));
        
        // Find simple variables without filters or pipes
        let var_regex = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}")
            .map_err(|e| anyhow!("Failed to create variable regex: {}", e))?;
        
        let effective_args = arguments.clone();
        let mut processed_template = template.to_string();
        
        // Find all simple variables and replace undefined ones with markers
        let variables_to_replace: Vec<_> = var_regex.captures_iter(template)
            .filter_map(|captures| {
                if let Some(var_match) = captures.get(1) {
                    let var_name = var_match.as_str();
                    let full_match = captures.get(0).unwrap();
                    let match_start = full_match.start();
                    let match_end = full_match.end();
                    
                    // Check if this variable is inside a protected block
                    let in_protected_block = protected_blocks.iter().any(|block| {
                        match_start >= block.start() && match_end <= block.end()
                    });
                    
                    // If variable is not defined and not in protected block
                    if !in_protected_block && !effective_args.contains_key(var_name) {
                        Some((var_name.to_string(), full_match.as_str().to_string()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
            
        // Replace variables with markers
        for (var_name, original_pattern) in variables_to_replace {
            let marker = format!("__UNDEFINED_VAR_{}__", var_name);
            processed_template = processed_template.replace(&original_pattern, &marker);
        }
        
        // Process the template with Liquid first, then restore undefined variables
        Ok((processed_template, effective_args))
    }

    /// Restore undefined variable markers back to original {{variable}} format
    fn restore_undefined_variables(&self, rendered: &str, _original_template: &str) -> Result<String> {
        use regex::Regex;
        
        let marker_regex = Regex::new(r"__UNDEFINED_VAR_([a-zA-Z_][a-zA-Z0-9_]*)__")
            .map_err(|e| anyhow!("Failed to create marker regex: {}", e))?;
        
        let mut result = rendered.to_string();
        
        for captures in marker_regex.captures_iter(rendered) {
            if let Some(var_match) = captures.get(1) {
                let var_name = var_match.as_str();
                let marker = format!("__UNDEFINED_VAR_{}__", var_name);
                let original_var = format!("{{{{ {} }}}}", var_name);
                result = result.replace(&marker, &original_var);
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

    /// Add environment variables to the context
    pub fn process_with_env(
        &self, 
        template: &str, 
        arguments: &HashMap<String, Value>,
        allowed_env_vars: &[String],
    ) -> Result<String> {
        let mut context = arguments.clone();
        
        // Add environment variables
        let mut env_vars = HashMap::new();
        for var_name in allowed_env_vars {
            if let Ok(value) = std::env::var(var_name) {
                env_vars.insert(var_name.clone(), Value::String(value));
            }
        }
        
        if !env_vars.is_empty() {
            context.insert("env".to_string(), Value::Object(env_vars.into_iter().collect()));
        }
        
        self.process(template, &context)
    }

    fn render_template(&self, template: &Template, arguments: &HashMap<String, Value>) -> Result<String> {
        // Convert HashMap<String, Value> to liquid::Object
        let mut liquid_object = liquid::Object::new();
        
        for (key, value) in arguments {
            let liquid_value = self.json_to_liquid_value(value);
            liquid_object.insert(key.clone().into(), liquid_value);
        }
        
        template
            .render(&liquid_object)
            .map_err(|e| anyhow!("Failed to render Liquid template: {}", e))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn json_to_liquid_value(&self, value: &Value) -> liquid::model::Value {
        match value {
            Value::String(s) => liquid::model::Value::scalar(s.clone()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    liquid::model::Value::scalar(i)
                } else if let Some(f) = n.as_f64() {
                    liquid::model::Value::scalar(f)
                } else {
                    liquid::model::Value::scalar(n.to_string())
                }
            }
            Value::Bool(b) => liquid::model::Value::scalar(*b),
            Value::Null => liquid::model::Value::Nil,
            Value::Array(arr) => {
                let liquid_array: Vec<liquid::model::Value> = arr.iter()
                    .map(|v| self.json_to_liquid_value(v))
                    .collect();
                liquid::model::Value::Array(liquid_array)
            }
            Value::Object(obj) => {
                let mut liquid_object = liquid::Object::new();
                for (key, value) in obj {
                    liquid_object.insert(key.clone().into(), self.json_to_liquid_value(value));
                }
                liquid::model::Value::Object(liquid_object)
            }
        }
    }
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

    // LiquidEngine tests
    mod liquid_engine_tests {
        use super::*;

        #[test]
        fn test_liquid_simple_substitution() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name }}, welcome to {{ place }}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));
            args.insert("place".to_string(), json!("Wonderland"));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice, welcome to Wonderland!");
        }

        #[test]
        fn test_liquid_backward_compatibility() {
            let engine = LiquidEngine::new();
            let template = "Hello {{name}}, welcome to {{place}}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));
            args.insert("place".to_string(), json!("Wonderland"));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice, welcome to Wonderland!");
        }

        #[test]
        fn test_liquid_filters() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name | upcase }}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("alice"));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello ALICE!");
        }

        #[test]
        fn test_liquid_if_condition() {
            let engine = LiquidEngine::new();
            let template = "{% if show_greeting %}Hello {{ name }}!{% endif %}";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));
            args.insert("show_greeting".to_string(), json!(true));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice!");
        }

        #[test]
        fn test_liquid_unless_condition() {
            let engine = LiquidEngine::new();
            let template = "{% unless hide_greeting %}Hello {{ name }}!{% endunless %}";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));
            args.insert("hide_greeting".to_string(), json!(false));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice!");
        }

        #[test]
        fn test_liquid_for_loop() {
            let engine = LiquidEngine::new();
            let template = "{% for item in items %}{{ item }}, {% endfor %}";
            let mut args = HashMap::new();
            args.insert("items".to_string(), json!(["apple", "banana", "cherry"]));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "apple, banana, cherry, ");
        }

        #[test]
        fn test_liquid_case_statement() {
            let engine = LiquidEngine::new();
            let template = r#"{% case fruit %}
            {% when 'apple' %}It's an apple!
            {% when 'banana' %}It's a banana!
            {% else %}Unknown fruit.
            {% endcase %}"#;
            let mut args = HashMap::new();
            args.insert("fruit".to_string(), json!("apple"));

            let result = engine.process(template, &args).unwrap();
            assert!(result.contains("It's an apple!"));
        }

        #[test]
        fn test_liquid_comments() {
            let engine = LiquidEngine::new();
            let template = "Hello {% comment %}This is a comment{% endcomment %}{{ name }}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice!");
        }

        #[test]
        fn test_liquid_raw_blocks() {
            let engine = LiquidEngine::new();
            let template = "Code: {% raw %}{{ not_processed }}{% endraw %}";
            let args = HashMap::new();

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Code: {{ not_processed }}");
        }

        #[test]
        fn test_liquid_default_filter() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name | default: 'Guest' }}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!(null)); // Provide null value for default filter to work

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Guest!");
        }

        #[test]
        fn test_liquid_truncate_filter() {
            let engine = LiquidEngine::new();
            let template = "{{ text | truncate: 10 }}";
            let mut args = HashMap::new();
            args.insert("text".to_string(), json!("This is a very long text"));

            let result = engine.process(template, &args).unwrap();
            assert!(result.len() <= 13); // 10 chars + "..."
        }

        #[test]
        fn test_liquid_env_variables() {
            let engine = LiquidEngine::new();
            let template = "Home: {{ env.HOME | default: '/unknown' }}";
            let args = HashMap::new();
            let allowed_vars = vec!["HOME".to_string()];

            let result = engine.process_with_env(template, &args, &allowed_vars).unwrap();
            assert!(result.starts_with("Home: "));
        }

        #[test]
        fn test_liquid_validation_required_arg() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name }}!";
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
        fn test_liquid_validation_default_value() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name }}!";
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
        fn test_liquid_complex_objects() {
            let engine = LiquidEngine::new();
            let template = "{{ user.name }} lives in {{ user.address.city }}";
            let mut args = HashMap::new();
            args.insert("user".to_string(), json!({
                "name": "Alice",
                "address": {
                    "city": "Wonderland",
                    "country": "Fantasyland"
                }
            }));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Alice lives in Wonderland");
        }

        #[test]
        fn test_liquid_array_access() {
            let engine = LiquidEngine::new();
            let template = "First item: {{ items[0] }}, Last item: {{ items[2] }}";
            let mut args = HashMap::new();
            args.insert("items".to_string(), json!(["apple", "banana", "cherry"]));

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "First item: apple, Last item: cherry");
        }

        #[test]
        fn test_liquid_template_caching() {
            let engine = LiquidEngine::new();
            let template = "Hello {{ name }}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));

            // First call should compile and cache
            let result1 = engine.process(template, &args).unwrap();
            assert_eq!(result1, "Hello Alice!");

            // Second call should use cached template
            args.insert("name".to_string(), json!("Bob"));
            let result2 = engine.process(template, &args).unwrap();
            assert_eq!(result2, "Hello Bob!");

            // Verify cache has one entry
            let cache = engine.template_cache.lock().unwrap();
            assert_eq!(cache.len(), 1);
        }

        #[test]
        fn test_liquid_error_handling() {
            let engine = LiquidEngine::new();
            let template = "{% if condition %}Hello{% endif"; // Missing closing tag
            let args = HashMap::new();

            let result = engine.process(template, &args);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Failed to parse Liquid template"));
        }

        #[test]
        fn test_liquid_undefined_variable_error() {
            let engine = LiquidEngine::new();
            let template = "Hello {{undefined_var}}!";
            let args = HashMap::new();

            // With backward compatibility (default), undefined variables should be left as-is
            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello {{ undefined_var }}!");
        }

        #[test]
        fn test_liquid_backward_compatibility_mode() {
            let engine = LiquidEngine::new();
            let template = "Hello {{name}}, welcome to {{place}}!";
            let mut args = HashMap::new();
            args.insert("name".to_string(), json!("Alice"));
            // place is undefined

            let result = engine.process(template, &args).unwrap();
            assert_eq!(result, "Hello Alice, welcome to {{ place }}!");
        }

        #[test]
        fn test_liquid_strict_mode() {
            let engine = LiquidEngine::new();
            let template = "Hello {{undefined_var}}!";
            let args = HashMap::new();

            // With strict mode (no backward compatibility), should error
            let result = engine.process_with_compatibility(template, &args, false);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Unknown variable"));
        }
    }
}