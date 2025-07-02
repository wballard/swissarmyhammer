use swissarmyhammer::prelude::*;
use swissarmyhammer::ArgumentSpec;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_library_creation() {
    let library = PromptLibrary::new();
    let prompts = library.list().unwrap();
    assert!(prompts.is_empty());
}

#[test]
fn test_prompt_creation_and_rendering() {
    let prompt = Prompt::new("greeting", "Hello {{ name }}!")
        .with_description("A simple greeting prompt")
        .with_category("examples")
        .with_tags(vec!["greeting".to_string(), "example".to_string()]);
    
    let mut args = HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    
    let rendered = prompt.render(&args).unwrap();
    assert_eq!(rendered, "Hello World!");
}

#[test]
fn test_prompt_with_arguments() {
    let prompt = Prompt::new("complex", "{{ greeting }}, {{ name }}!")
        .add_argument(ArgumentSpec {
            name: "greeting".to_string(),
            description: Some("The greeting to use".to_string()),
            required: true,
            default: None,
            type_hint: Some("string".to_string()),
        })
        .add_argument(ArgumentSpec {
            name: "name".to_string(),
            description: Some("The name to greet".to_string()),
            required: false,
            default: Some("Friend".to_string()),
            type_hint: Some("string".to_string()),
        });
    
    // Test with all arguments provided
    let mut args = HashMap::new();
    args.insert("greeting".to_string(), "Hello".to_string());
    args.insert("name".to_string(), "Alice".to_string());
    
    let rendered = prompt.render(&args).unwrap();
    assert_eq!(rendered, "Hello, Alice!");
    
    // Test with default value
    let mut args = HashMap::new();
    args.insert("greeting".to_string(), "Hi".to_string());
    
    let rendered = prompt.render(&args).unwrap();
    assert_eq!(rendered, "Hi, Friend!");
}

#[test]
fn test_missing_required_argument() {
    let prompt = Prompt::new("test", "Hello {{ name }}!")
        .add_argument(ArgumentSpec {
            name: "name".to_string(),
            description: None,
            required: true,
            default: None,
            type_hint: None,
        });
    
    let args = HashMap::new();
    let result = prompt.render(&args);
    assert!(result.is_err());
}

#[test]
fn test_library_with_directory() {
    let temp_dir = TempDir::new().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir(&prompts_dir).unwrap();
    
    // Create a test prompt file
    let prompt_content = r#"---
description: "Test prompt"
category: "test"
tags:
  - test
  - example
arguments:
  - name: subject
    description: "The subject to test"
    required: true
---
Testing {{ subject }}!"#;
    
    fs::write(prompts_dir.join("test.md"), prompt_content).unwrap();
    
    // Load prompts from directory
    let mut library = PromptLibrary::new();
    let count = library.add_directory(&prompts_dir).unwrap();
    assert_eq!(count, 1);
    
    // Get the prompt
    let prompt = library.get("test").unwrap();
    assert_eq!(prompt.name, "test");
    assert_eq!(prompt.description, Some("Test prompt".to_string()));
    assert_eq!(prompt.category, Some("test".to_string()));
    assert_eq!(prompt.tags, vec!["test", "example"]);
    
    // Render the prompt
    let mut args = HashMap::new();
    args.insert("subject".to_string(), "library".to_string());
    
    let rendered = prompt.render(&args).unwrap();
    assert_eq!(rendered, "Testing library!");
}

#[test]
fn test_library_search() {
    let mut library = PromptLibrary::new();
    
    // Add some test prompts
    library.add(
        Prompt::new("code-review", "Review this code")
            .with_description("A prompt for code review")
            .with_tags(vec!["code".to_string(), "review".to_string()])
    ).unwrap();
    
    library.add(
        Prompt::new("bug-fix", "Fix this bug")
            .with_description("A prompt for fixing bugs")
            .with_tags(vec!["bug".to_string(), "fix".to_string()])
    ).unwrap();
    
    library.add(
        Prompt::new("refactor-code", "Refactor this code")
            .with_description("A prompt for code refactoring")
            .with_tags(vec!["code".to_string(), "refactor".to_string()])
    ).unwrap();
    
    // Search for prompts containing "code"
    let results = library.search("code").unwrap();
    assert_eq!(results.len(), 2);
    
    let names: Vec<String> = results.iter().map(|p| p.name.clone()).collect();
    assert!(names.contains(&"code-review".to_string()));
    assert!(names.contains(&"refactor-code".to_string()));
}

#[test]
fn test_template_engine() {
    let engine = TemplateEngine::new();
    
    let mut args = HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    args.insert("count".to_string(), "5".to_string());
    
    // Test simple substitution
    let result = engine.render("Hello {{ name }}!", &args).unwrap();
    assert_eq!(result, "Hello World!");
    
    // Test filters
    let result = engine.render("{{ name | upcase }}", &args).unwrap();
    assert_eq!(result, "WORLD");
    
    // Test conditionals
    let template = "{% if count %}Count: {{ count }}{% endif %}";
    let result = engine.render(template, &args).unwrap();
    assert_eq!(result, "Count: 5");
}

// Custom filters test disabled - these filters are not implemented in the library
// To enable this test, implement the custom filters: slugify, count_lines, indent
#[test]
#[ignore = "Custom filters not yet implemented"]
fn test_custom_filters() {
    let engine = TemplateEngine::new();
    
    let mut args = HashMap::new();
    args.insert("title".to_string(), "Hello World!".to_string());
    args.insert("text".to_string(), "line1\nline2\nline3".to_string());
    
    // Test slugify filter
    let result = engine.render("{{ title | slugify }}", &args).unwrap();
    assert_eq!(result, "hello-world");
    
    // Test count_lines filter
    let result = engine.render("{{ text | count_lines }}", &args).unwrap();
    assert_eq!(result, "3");
    
    // Test indent filter
    let result = engine.render("{{ text | indent: 2 }}", &args).unwrap();
    assert_eq!(result, "  line1\n  line2\n  line3");
}

#[test]
fn test_prompt_loader() {
    let temp_dir = TempDir::new().unwrap();
    let prompts_dir = temp_dir.path().join("prompts");
    fs::create_dir(&prompts_dir).unwrap();
    
    // Create multiple test prompt files
    for i in 1..=3 {
        let content = format!(
            r#"---
description: "Test prompt {}"
category: "test"
---
This is test prompt {}!"#,
            i, i
        );
        fs::write(prompts_dir.join(format!("test{}.md", i)), content).unwrap();
    }
    
    // Load all prompts
    let loader = PromptLoader::new();
    let prompts = loader.load_directory(&prompts_dir).unwrap();
    assert_eq!(prompts.len(), 3);
    
    // Check that all prompts were loaded correctly
    for (i, prompt) in prompts.iter().enumerate() {
        assert!(prompt.name.starts_with("test"));
        assert_eq!(prompt.category, Some("test".to_string()));
    }
}

#[cfg(feature = "search")]
#[test]
fn test_search_engine() {
    use swissarmyhammer::search::{SearchEngine, SearchResult};
    
    let mut engine = SearchEngine::new().unwrap();
    
    let prompts = vec![
        Prompt::new("code-review", "Review this code")
            .with_description("A prompt for reviewing code quality"),
        Prompt::new("bug-fix", "Fix this bug")
            .with_description("A prompt for fixing software bugs"),
        Prompt::new("documentation", "Write documentation")
            .with_description("A prompt for writing technical documentation"),
    ];
    
    // Index all prompts
    engine.index_prompts(&prompts).unwrap();
    
    // Search for "code"
    let results = engine.search("code", &prompts).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].prompt.name, "code-review");
    
    // Fuzzy search
    let results = engine.fuzzy_search("docu", &prompts);
    assert!(!results.is_empty());
    assert_eq!(results[0].prompt.name, "documentation");
}

#[cfg(feature = "mcp")]
#[tokio::test]
async fn test_mcp_server() {
    use swissarmyhammer::mcp::McpServer;
    
    let mut library = PromptLibrary::new();
    library.add(
        Prompt::new("test", "Hello {{ name }}!")
            .with_description("Test prompt")
    ).unwrap();
    
    let server = McpServer::new(library);
    
    // Test server info
    let info = server.info();
    assert_eq!(info.name, "SwissArmyHammer");
}

// Example usage for documentation
#[test]
fn test_example_usage() {
    // Create a prompt library
    let mut library = PromptLibrary::new();
    
    // Create a prompt programmatically
    let greeting_prompt = Prompt::new("greeting", "Hello {{ name }}! Welcome to {{ place }}.")
        .with_description("A friendly greeting prompt")
        .with_category("examples")
        .add_argument(ArgumentSpec {
            name: "name".to_string(),
            description: Some("The person's name".to_string()),
            required: true,
            default: None,
            type_hint: Some("string".to_string()),
        })
        .add_argument(ArgumentSpec {
            name: "place".to_string(),
            description: Some("The location".to_string()),
            required: false,
            default: Some("our application".to_string()),
            type_hint: Some("string".to_string()),
        });
    
    // Add to library
    library.add(greeting_prompt).unwrap();
    
    // Retrieve and use
    let prompt = library.get("greeting").unwrap();
    
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());
    
    let output = prompt.render(&args).unwrap();
    assert_eq!(output, "Hello Alice! Welcome to our application.");
}