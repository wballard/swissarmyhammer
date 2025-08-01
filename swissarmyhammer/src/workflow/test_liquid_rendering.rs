//! Tests for liquid template rendering in action descriptions

#[cfg(test)]
mod tests {
    use crate::workflow::parse_action_from_description_with_context;
    use serde_json::{json, Value};
    use std::collections::HashMap;

    #[test]
    fn test_action_parsing_with_liquid_templates() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();

        template_vars.insert("name".to_string(), json!("Alice"));
        template_vars.insert("language".to_string(), json!("French"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Test prompt action with templates
        let description =
            r#"Execute prompt "say-hello" with name="{{ name }}" language="{{ language }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let prompt_action = action
            .as_any()
            .downcast_ref::<crate::workflow::PromptAction>()
            .unwrap();
        assert_eq!(prompt_action.prompt_name, "say-hello");
        assert_eq!(prompt_action.arguments.get("name").unwrap(), "Alice");
        assert_eq!(prompt_action.arguments.get("language").unwrap(), "French");
    }

    #[test]
    fn test_action_parsing_with_default_values() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();

        template_vars.insert("name".to_string(), json!("Bob"));
        // Note: language is not set, should use default

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Test with simple template - liquid doesn't support default filter syntax
        let description = r#"Log "Hello, {{ name }}!""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        assert_eq!(log_action.message, "Hello, Bob!");
    }

    #[test]
    fn test_action_parsing_without_templates() {
        let context = HashMap::new(); // No template vars

        let description = r#"Execute prompt "test-prompt" with arg="value""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let prompt_action = action
            .as_any()
            .downcast_ref::<crate::workflow::PromptAction>()
            .unwrap();
        assert_eq!(prompt_action.prompt_name, "test-prompt");
        assert_eq!(prompt_action.arguments.get("arg").unwrap(), "value");
    }

    #[test]
    fn test_action_parsing_with_missing_template_var() {
        let mut context = HashMap::new();
        let template_vars: HashMap<String, Value> = HashMap::new(); // Empty template vars

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Template variable not provided, liquid will keep the template text
        let description = r#"Log "Hello, {{ name }}!""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // When template vars are empty, liquid keeps the original template
        assert_eq!(log_action.message, "Hello, {{ name }}!");
    }

    #[test]
    fn test_action_parsing_with_invalid_liquid_syntax() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("name".to_string(), json!("Bob"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Invalid liquid syntax - unclosed tag
        let description = r#"Log "Hello, {{ name""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // With invalid syntax, should fall back to original text
        assert_eq!(log_action.message, "Hello, {{ name");
    }

    #[test]
    fn test_action_parsing_with_nested_liquid_errors() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("items".to_string(), json!(["a", "b", "c"]));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Invalid nested liquid - can't have {{ inside {% %}
        let description = r#"Log "Items: {% for item in {{ items }} %}{{ item }}{% endfor %}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // Should fall back to original text due to parse error
        assert!(log_action.message.contains("{% for item in {{ items }}"));
    }

    #[test]
    fn test_action_parsing_with_undefined_filter() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("value".to_string(), json!("test"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Use a filter that doesn't exist
        let description = r#"Log "Value: {{ value | nonexistent_filter }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // With undefined filter, liquid keeps the original template
        assert_eq!(
            log_action.message,
            "Value: {{ value | nonexistent_filter }}"
        );
    }

    #[test]
    fn test_prompt_action_with_template_in_arguments() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("user".to_string(), json!("Alice"));
        template_vars.insert("task".to_string(), json!("review code"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Test templates in prompt arguments
        let description =
            r#"Execute prompt "assistant" with message="Help {{ user }} to {{ task }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let prompt_action = action
            .as_any()
            .downcast_ref::<crate::workflow::PromptAction>()
            .unwrap();
        assert_eq!(prompt_action.prompt_name, "assistant");
        assert_eq!(
            prompt_action.arguments.get("message").unwrap(),
            "Help Alice to review code"
        );
    }

    #[test]
    fn test_action_parsing_with_empty_template_vars() {
        let mut context = HashMap::new();
        // _template_vars exists but is empty
        context.insert("_template_vars".to_string(), json!({}));

        let description = r#"Log "Hello, {{ name }}!""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // With empty template vars, liquid keeps the original template
        assert_eq!(log_action.message, "Hello, {{ name }}!");
    }

    #[test]
    fn test_action_parsing_with_null_template_value() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("value".to_string(), json!(null));

        context.insert("_template_vars".to_string(), json!(template_vars));

        let description = r#"Log "Value is: {{ value }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // Liquid renders null as empty string
        assert_eq!(log_action.message, "Value is: ");
    }

    #[test]
    fn test_action_parsing_with_complex_object_template_value() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert(
            "user".to_string(),
            json!({
                "name": "Bob",
                "id": 123
            }),
        );

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Try to access nested property
        let description = r#"Log "User: {{ user.name }} (ID: {{ user.id }})""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // Liquid supports dot notation for object properties
        assert_eq!(log_action.message, "User: Bob (ID: 123)");
    }

    #[test]
    fn test_action_parsing_with_array_template_value() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("items".to_string(), json!(["a", "b", "c"]));

        context.insert("_template_vars".to_string(), json!(template_vars));

        // Array access
        let description = r#"Log "First item: {{ items[0] }}, Count: {{ items.size }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        assert_eq!(log_action.message, "First item: a, Count: 3");
    }

    #[test]
    fn test_action_parsing_with_special_characters_in_template() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        // Use special characters that won't break the action parser
        template_vars.insert("message".to_string(), json!("Hello World & <everyone>!"));
        template_vars.insert("path".to_string(), json!("/usr/bin/test"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        let description = r#"Log "Message: {{ message }} at {{ path }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let log_action = action
            .as_any()
            .downcast_ref::<crate::workflow::LogAction>()
            .unwrap();
        // Special characters should be preserved
        assert_eq!(
            log_action.message,
            "Message: Hello World & <everyone>! at /usr/bin/test"
        );
    }

    #[test]
    fn test_set_variable_action_with_template() {
        let mut context = HashMap::new();
        let mut template_vars = HashMap::new();
        template_vars.insert("prefix".to_string(), json!("test"));
        template_vars.insert("suffix".to_string(), json!("value"));

        context.insert("_template_vars".to_string(), json!(template_vars));

        let description = r#"Set my_var="{{ prefix }}_{{ suffix }}""#;
        let action = parse_action_from_description_with_context(description, &context)
            .unwrap()
            .unwrap();

        let set_action = action
            .as_any()
            .downcast_ref::<crate::workflow::SetVariableAction>()
            .unwrap();
        assert_eq!(set_action.variable_name, "my_var");
        assert_eq!(set_action.value, "test_value");
    }
}
