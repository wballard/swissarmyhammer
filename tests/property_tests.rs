use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use swissarmyhammer::template::TemplateEngine;

proptest! {
    #[test]
    fn test_template_engine_no_placeholders(s: String) {
        // Template without placeholders should remain unchanged
        let engine = TemplateEngine::new();
        let args = HashMap::new();
        
        // Only test strings without placeholder patterns
        if !s.contains("{{") && !s.contains("}}") {
            let result = engine.process(&s, &args).unwrap();
            assert_eq!(result, s);
        }
    }
    
    #[test]
    fn test_template_engine_with_values(
        template in "[a-zA-Z0-9 ]*\\{\\{[a-zA-Z_][a-zA-Z0-9_]*\\}\\}[a-zA-Z0-9 ]*",
        value in "[a-zA-Z0-9]+",
    ) {
        // Extract variable name from template
        if let Some(start) = template.find("{{") {
            if let Some(end) = template.find("}}") {
                let var_name = &template[start + 2..end];
                
                let engine = TemplateEngine::new();
                let mut args = HashMap::new();
                args.insert(var_name.to_string(), Value::String(value.clone()));
                
                let result = engine.process(&template, &args).unwrap();
                
                // Result should contain the value but not the placeholder
                assert!(result.contains(&value));
                assert!(!result.contains(&format!("{{{{{}}}}}", var_name)));
            }
        }
    }
    
    #[test]
    fn test_template_engine_missing_values(
        template in "[a-zA-Z0-9 ]*\\{\\{[a-zA-Z_][a-zA-Z0-9_]*\\}\\}[a-zA-Z0-9 ]*",
    ) {
        // Template with placeholders but no arguments should keep placeholders
        let engine = TemplateEngine::new();
        let args = HashMap::new();
        
        let result = engine.process(&template, &args).unwrap();
        
        // Result should still contain the placeholder
        assert!(result.contains("{{"));
        assert!(result.contains("}}"));
    }
    
    #[test]
    fn test_template_engine_idempotent(
        s: String,
        args in prop::collection::hash_map(
            "[a-zA-Z_][a-zA-Z0-9_]*",
            prop_oneof![
                Just(Value::String("test".to_string())),
                Just(Value::Number(42.into())),
                Just(Value::Bool(true)),
            ],
            0..5
        )
    ) {
        // Processing a template twice should give the same result
        let engine = TemplateEngine::new();
        
        let result1 = engine.process(&s, &args).unwrap();
        let result2 = engine.process(&s, &args).unwrap();
        
        assert_eq!(result1, result2);
    }
}

#[cfg(test)]
mod prompt_property_tests {
    use super::*;
    use swissarmyhammer::prompts::{Prompt, PromptArgument};
    
    proptest! {
        #[test]
        fn test_prompt_argument_validation(
            name in "[a-zA-Z_][a-zA-Z0-9_]*",
            description in ".*",
            required: bool,
        ) {
            let arg = PromptArgument {
                name: name.clone(),
                description: description.clone(),
                required,
                default: None,
            };
            
            // Argument name should be preserved
            assert_eq!(arg.name, name);
            
            // Required flag should be preserved
            assert_eq!(arg.required, required);
        }
        
        #[test]
        fn test_prompt_creation(
            name in "[a-zA-Z0-9_]+",
            content in "[a-zA-Z0-9\n ]+",
            source_path in "[a-zA-Z0-9/]+\\.md",
        ) {
            let prompt = Prompt::new(
                name.clone(),
                content.clone(),
                source_path.clone(),
            );
            
            // All fields should be preserved
            assert_eq!(prompt.name, name);
            assert_eq!(prompt.content, content);
            assert_eq!(prompt.source_path, source_path);
        }
    }
}