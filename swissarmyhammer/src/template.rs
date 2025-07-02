//! Template engine and rendering functionality

use crate::{Result, SwissArmyHammerError};
use liquid::{Object, Parser};
use std::collections::HashMap;

/// Template wrapper for Liquid templates
pub struct Template {
    parser: Parser,
    template_str: String,
}

impl Template {
    /// Create a new template from a string
    pub fn new(template_str: &str) -> Result<Self> {
        let parser = TemplateEngine::default_parser();
        // Validate the template by trying to parse it
        parser
            .parse(template_str)
            .map_err(|e| SwissArmyHammerError::Template(e.to_string()))?;
        
        Ok(Self { 
            parser,
            template_str: template_str.to_string(),
        })
    }
    
    /// Render the template with given arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String> {
        let template = self.parser
            .parse(&self.template_str)
            .map_err(|e| SwissArmyHammerError::Template(e.to_string()))?;
            
        let mut object = Object::new();
        for (key, value) in args {
            object.insert(key.clone().into(), liquid::model::Value::scalar(value.clone()));
        }
        
        template
            .render(&object)
            .map_err(|e| SwissArmyHammerError::Template(e.to_string()))
    }
    
    /// Get the raw template string
    pub fn raw(&self) -> &str {
        &self.template_str
    }
}

/// Template engine with Liquid configuration
pub struct TemplateEngine {
    parser: liquid::Parser,
}

impl TemplateEngine {
    /// Create a new template engine with default configuration
    pub fn new() -> Self {
        Self {
            parser: Self::default_parser(),
        }
    }
    
    /// Create a new template engine with custom parser
    pub fn with_parser(parser: liquid::Parser) -> Self {
        Self { parser }
    }
    
    /// Create a default parser
    pub fn default_parser() -> liquid::Parser {
        liquid::ParserBuilder::with_stdlib()
            .build()
            .expect("Failed to build Liquid parser")
    }
    
    /// Parse a template string
    pub fn parse(&self, template_str: &str) -> Result<Template> {
        Template::new(template_str)
    }
    
    /// Render a template string with arguments
    pub fn render(&self, template_str: &str, args: &HashMap<String, String>) -> Result<String> {
        let template = self.parse(template_str)?;
        template.render(args)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_template() {
        let template = Template::new("Hello {{ name }}!").unwrap();
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        
        let result = template.render(&args).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_empty_template() {
        let engine = TemplateEngine::new();
        let args = HashMap::new();
        
        let result = engine.render("", &args).unwrap();
        assert_eq!(result, "");
    }
    
    #[test]
    fn test_no_placeholders() {
        let engine = TemplateEngine::new();
        let args = HashMap::new();
        
        let result = engine.render("Hello World!", &args).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_multiple_occurrences() {
        let engine = TemplateEngine::new();
        let mut args = HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        
        let result = engine.render("Hello {{ name }}! Nice to meet you, {{ name }}.", &args).unwrap();
        assert_eq!(result, "Hello Alice! Nice to meet you, Alice.");
    }
    
    #[test]
    fn test_special_characters() {
        let engine = TemplateEngine::new();
        let mut args = HashMap::new();
        args.insert("code".to_string(), "<script>alert('XSS')</script>".to_string());
        
        let result = engine.render("Code: {{ code }}", &args).unwrap();
        assert_eq!(result, "Code: <script>alert('XSS')</script>");
    }
    
    #[test]
    fn test_numeric_value() {
        let engine = TemplateEngine::new();
        let mut args = HashMap::new();
        args.insert("count".to_string(), "42".to_string());
        
        let result = engine.render("Count: {{ count }}", &args).unwrap();
        assert_eq!(result, "Count: 42");
    }
    
    #[test]
    fn test_boolean_value() {
        let engine = TemplateEngine::new();
        let mut args = HashMap::new();
        args.insert("enabled".to_string(), "true".to_string());
        
        let result = engine.render("Enabled: {{ enabled }}", &args).unwrap();
        assert_eq!(result, "Enabled: true");
    }
    
    #[test]
    fn test_missing_argument_no_validation() {
        let engine = TemplateEngine::new();
        let args = HashMap::new();
        
        let result = engine.render("Hello {{ name }}!", &args);
        // Liquid throws an error for undefined variables
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown variable"));
    }
    
    #[test]
    fn test_default_value() {
        let engine = TemplateEngine::new();
        let mut args = HashMap::new();
        args.insert("greeting".to_string(), "Hello".to_string());
        args.insert("name".to_string(), "".to_string()); // Provide empty value
        
        let template = "{{ greeting }}, {{ name }}!";
        let result = engine.render(template, &args).unwrap();
        assert_eq!(result, "Hello, !");
    }
    
    #[test]
    fn test_required_argument_validation() {
        let template = Template::new("Hello {{ name }}!").unwrap();
        let args = HashMap::new();
        
        // Liquid will error on undefined variables
        let result = template.render(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown variable"));
    }
}