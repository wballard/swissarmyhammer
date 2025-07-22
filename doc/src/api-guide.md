# API Guide

This guide provides comprehensive usage patterns and examples for the SwissArmyHammer Rust API.

## Overview

SwissArmyHammer provides a rich Rust API for programmatic prompt management, template rendering, workflow orchestration, and memoranda handling. The API is designed with both synchronous and asynchronous patterns in mind.

### Core Components

- **[PromptLibrary](#promptlibrary)** - Main interface for prompt management
- **[Prompt](#prompt)** - Individual prompt representation
- **[PromptResolver](#promptresolver)** - Advanced prompt loading and resolution
- **[TemplateEngine](#templateengine)** - Low-level template rendering
- **[Workflows](#workflows)** - State-based execution workflows
- **[Memoranda](#memoranda)** - Structured note management
- **[Storage](#storage)** - Pluggable storage backends

## Quick Start

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec, Prompt};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a library and load prompts
    let mut library = PromptLibrary::new();
    library.add_directory("./.swissarmyhammer/prompts")?;
    
    // Use a prompt
    let prompt = library.get("code-review")?;
    let mut args = HashMap::new();
    args.insert("language".to_string(), "rust".to_string());
    
    let result = prompt.render(&args)?;
    println!("{}", result);
    
    Ok(())
}
```

---

## PromptLibrary

The `PromptLibrary` is the primary interface for managing collections of prompts.

### Creation and Initialization

```rust
use swissarmyhammer::PromptLibrary;

// Create an empty library
let mut library = PromptLibrary::new();

// Load from directories (common pattern)
if std::path::Path::new("./.swissarmyhammer/prompts").exists() {
    let count = library.add_directory("./.swissarmyhammer/prompts")?;
    println!("Loaded {} prompts", count);
}

// Load from multiple sources
library.add_directory("./global/prompts")?;
library.add_directory("./project/prompts")?;
```

### Adding Prompts Programmatically

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};

let prompt = Prompt::new("greeting", "Hello {{name}}!")
    .with_description("A simple greeting")
    .add_argument(ArgumentSpec {
        name: "name".to_string(),
        description: Some("Person's name".to_string()),
        required: true,
        default: Some("World".to_string()),
        type_hint: Some("string".to_string()),
    });

library.add(prompt)?;
```

### Retrieving and Using Prompts

```rust
use std::collections::HashMap;

// Get by name
let prompt = library.get("greeting")?;

// Render with arguments
let mut args = HashMap::new();
args.insert("name".to_string(), "Alice".to_string());
let result = prompt.render(&args)?;
```

### Listing and Discovery

```rust
// List all prompts
for prompt in library.list()? {
    println!("{}: {}", prompt.name, 
             prompt.description.as_deref().unwrap_or("No description"));
}

// Filter by category
for prompt in library.list()? {
    if prompt.category.as_deref() == Some("development") {
        println!("Dev prompt: {}", prompt.name);
    }
}

// Search by content
let matches = library.search("code review")?;
for prompt in matches {
    println!("Found: {}", prompt.name);
}
```

---

## Prompt

Individual prompts encapsulate template content and metadata.

### Creating Prompts

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};

let prompt = Prompt::new("code-review", r#"
Review this {{language}} code:

```{{language}}
{{code}}
```

Focus on:
- Best practices
- Potential bugs
- Performance
"#)
.with_description("Comprehensive code review prompt")
.with_category("development")
.with_tags(vec!["code".to_string(), "review".to_string()])
.add_argument(ArgumentSpec {
    name: "language".to_string(),
    description: Some("Programming language".to_string()),
    required: true,
    default: None,
    type_hint: Some("string".to_string()),
})
.add_argument(ArgumentSpec {
    name: "code".to_string(),
    description: Some("Code to review".to_string()),
    required: true,
    default: None,
    type_hint: Some("text".to_string()),
});
```

### Argument Validation

```rust
// Check required arguments
if let Err(missing) = prompt.validate_arguments(&args) {
    eprintln!("Missing arguments: {:?}", missing);
    return Ok(());
}

// Get argument specifications
for arg in &prompt.arguments {
    println!("Arg: {} (required: {})", arg.name, arg.required);
    if let Some(default) = &arg.default {
        println!("  Default: {}", default);
    }
}
```

### Template Features

```rust
// Basic variable substitution
let template = "Hello {{name}}!";

// Conditionals
let template = r#"
{% if urgent %}
**URGENT**: {{message}}
{% else %}
{{message}}
{% endif %}
"#;

// Loops
let template = r#"
{% for item in items %}
- {{item.name}}: {{item.description}}
{% endfor %}
"#;

// Custom filters (if registered)
let template = "{{text | upper | truncate: 50}}";
```

---

## PromptResolver

For advanced prompt loading and resolution scenarios.

### Basic Usage

```rust
use swissarmyhammer::PromptResolver;

let resolver = PromptResolver::new();

// Get all available prompts from standard locations
let prompts = resolver.resolve_all()?;

// Resolve specific prompt by name
if let Some(prompt) = resolver.resolve("code-review")? {
    println!("Found prompt: {}", prompt.name);
}
```

### Custom Search Paths

```rust
use swissarmyhammer::{PromptResolver, FileSource};

let mut resolver = PromptResolver::new();
resolver.add_source(FileSource::from_directory("./custom/prompts")?);

// Resolve from custom sources
let prompts = resolver.resolve_all()?;
```

### Source Priority

```rust
// Sources are resolved in order, later sources override earlier ones
resolver.add_source(FileSource::from_directory("./global")?);    // Lower priority
resolver.add_source(FileSource::from_directory("./project")?);   // Medium priority  
resolver.add_source(FileSource::from_directory("./local")?);     // Higher priority

// "code-review" from ./local will override ./project and ./global
let prompt = resolver.resolve("code-review")?;
```

---

## TemplateEngine

Low-level template rendering with custom filters and context.

### Basic Rendering

```rust
use swissarmyhammer::TemplateEngine;
use std::collections::HashMap;

let engine = TemplateEngine::new();
let template = engine.parse("Hello {{name}}!")?;

let mut context = HashMap::new();
context.insert("name".to_string(), "World".to_string());

let result = template.render(&context)?;
```

### Custom Filters

```rust
use swissarmyhammer::{TemplateEngine, CustomLiquidFilter};
use liquid::ValueView;

struct UppercaseFilter;

impl CustomLiquidFilter for UppercaseFilter {
    fn name(&self) -> &'static str {
        "upper"
    }
    
    fn filter(&self, input: &dyn ValueView) -> Result<String, Box<dyn std::error::Error>> {
        Ok(input.to_kstr().to_uppercase())
    }
}

let mut engine = TemplateEngine::new();
engine.register_filter(Box::new(UppercaseFilter))?;

let template = engine.parse("{{name | upper}}")?;
// Renders: "ALICE" from input "alice"
```

### Advanced Context

```rust
use serde_json::json;

let context = json!({
    "user": {
        "name": "Alice",
        "role": "developer"
    },
    "items": [
        {"name": "Task 1", "done": true},
        {"name": "Task 2", "done": false}
    ]
});

let template = r#"
User: {{user.name}} ({{user.role}})
Pending tasks:
{% for item in items %}
{% unless item.done %}
- {{item.name}}
{% endunless %}
{% endfor %}
"#;
```

---

## Workflows

State-based execution for complex multi-step processes.

### Defining Workflows

```rust
use swissarmyhammer::{Workflow, State, Transition};

let workflow = Workflow::new("code-review-process")
    .with_description("Complete code review workflow")
    .add_state(State::new("start")
        .with_prompt("code-review-request"))
    .add_state(State::new("review")
        .with_prompt("perform-code-review"))
    .add_state(State::new("feedback")
        .with_prompt("format-feedback"))
    .add_state(State::new("complete"))
    .add_transition(Transition::new("start", "review")
        .with_condition("review_requested"))
    .add_transition(Transition::new("review", "feedback")
        .with_condition("review_complete"))
    .add_transition(Transition::new("feedback", "complete"));
```

### Executing Workflows

```rust
use std::collections::HashMap;

// Start a workflow run
let mut context = HashMap::new();
context.insert("code".to_string(), "fn main() {}".to_string());
context.insert("language".to_string(), "rust".to_string());

let mut run = workflow.start_run(context)?;

// Execute step by step
while !run.is_complete() {
    let current_state = run.current_state();
    println!("Current state: {:?}", current_state);
    
    // Get the prompt for this state
    if let Some(prompt_name) = current_state.prompt_name() {
        let prompt = library.get(prompt_name)?;
        let result = prompt.render(run.context())?;
        
        // Process result and advance
        run.set_variable("result", result);
        run.advance("next")?;
    }
}

let final_result = run.get_variable("result");
```

### Workflow Persistence

```rust
use swissarmyhammer::{WorkflowRun, WorkflowRunStatus};

// Save workflow state
let run_id = run.id();
let status = run.status(); // InProgress, Completed, Failed
let state_data = run.serialize()?;

// Later, restore workflow
let restored_run = WorkflowRun::deserialize(&state_data)?;
```

---

## Memoranda

Structured note and memo management.

### Creating Memos

```rust
use swissarmyhammer::{Memo, CreateMemoRequest};

let memo_request = CreateMemoRequest {
    title: "Meeting Notes".to_string(),
    content: r#"
# Team Meeting 2024-01-15

## Attendees
- Alice, Bob, Charlie

## Action Items
- [ ] Review PR #123
- [ ] Update documentation
    "#.to_string(),
};

// This would typically be used with MCP or storage layer
```

### Searching Memos

```rust
use swissarmyhammer::SearchMemosRequest;

let search_request = SearchMemosRequest {
    query: "meeting action items".to_string(),
};

// Returns SearchMemosResponse with matching memos
// Implementation depends on storage backend
```

---

## Storage

Pluggable storage backends for prompts and data.

### File System Storage

```rust
use swissarmyhammer::{PromptStorage, StorageBackend};
use std::path::PathBuf;

// Default file system storage
let storage = PromptStorage::new_filesystem(PathBuf::from("./.swissarmyhammer"))?;

// Store and retrieve prompts
storage.store_prompt(&prompt)?;
let retrieved = storage.get_prompt("greeting")?;

// List all stored prompts
let all_prompts = storage.list_prompts()?;
```

### Custom Storage Backend

```rust
use swissarmyhammer::{StorageBackend, Prompt};
use async_trait::async_trait;

struct DatabaseStorage {
    connection: DatabaseConnection,
}

#[async_trait]
impl StorageBackend for DatabaseStorage {
    async fn store_prompt(&self, prompt: &Prompt) -> Result<(), SwissArmyHammerError> {
        // Store in database
        Ok(())
    }
    
    async fn get_prompt(&self, name: &str) -> Result<Option<Prompt>, SwissArmyHammerError> {
        // Retrieve from database
        Ok(None)
    }
    
    async fn list_prompts(&self) -> Result<Vec<Prompt>, SwissArmyHammerError> {
        // List all prompts
        Ok(vec![])
    }
}
```

---

## Error Handling

SwissArmyHammer uses a comprehensive error system.

### Error Types

```rust
use swissarmyhammer::{SwissArmyHammerError, Result};

fn handle_errors() -> Result<()> {
    match library.get("nonexistent") {
        Ok(prompt) => println!("Found: {}", prompt.name),
        Err(SwissArmyHammerError::PromptNotFound(name)) => {
            eprintln!("Prompt '{}' not found", name);
        }
        Err(SwissArmyHammerError::Template(msg)) => {
            eprintln!("Template error: {}", msg);
        }
        Err(SwissArmyHammerError::Io(err)) => {
            eprintln!("IO error: {}", err);
        }
        Err(err) => {
            eprintln!("Other error: {}", err);
        }
    }
    
    Ok(())
}
```

### Result Chaining

```rust
// Chain operations safely
let result = library
    .get("code-review")?
    .render(&args)?;

// Or with explicit error handling
let prompt = library.get("code-review").map_err(|e| {
    eprintln!("Failed to get prompt: {}", e);
    e
})?;
```

---

## Advanced Patterns

### Plugin System

```rust
use swissarmyhammer::{SwissArmyHammerPlugin, PluginRegistry};

struct CustomPlugin;

impl SwissArmyHammerPlugin for CustomPlugin {
    fn name(&self) -> &'static str {
        "custom-plugin"
    }
    
    fn initialize(&self, registry: &mut PluginRegistry) -> Result<()> {
        // Register custom filters, templates, etc.
        Ok(())
    }
}

let mut registry = PluginRegistry::new();
registry.register(Box::new(CustomPlugin))?;
```

### Configuration Management

```rust
use swissarmyhammer::Config;

let config = Config::builder()
    .prompt_directories(vec!["./prompts", "./shared/prompts"])
    .template_cache_size(1000)
    .enable_file_watching(true)
    .build()?;

let library = PromptLibrary::with_config(config)?;
```

### Async Patterns

```rust
use swissarmyhammer::PromptResolver;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resolver = PromptResolver::new();
    
    // Async prompt resolution
    let prompts = resolver.resolve_all_async().await?;
    
    // Concurrent prompt rendering
    let tasks: Vec<_> = prompts.into_iter()
        .map(|prompt| {
            let args = args.clone();
            tokio::spawn(async move {
                prompt.render(&args)
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    
    Ok(())
}
```

---

## Performance Considerations

### Caching

```rust
// PromptLibrary caches loaded prompts automatically
let library = PromptLibrary::new();
library.add_directory("./prompts")?; // Loads once

// Multiple gets use cached versions
let prompt1 = library.get("greeting")?; // From cache
let prompt2 = library.get("greeting")?; // From cache
```

### Memory Management

```rust
use std::sync::Arc;

// Share prompts across threads
let prompt = Arc::new(library.get("greeting")?);
let handles: Vec<_> = (0..4).map(|_| {
    let prompt = Arc::clone(&prompt);
    let args = args.clone();
    std::thread::spawn(move || {
        prompt.render(&args)
    })
}).collect();
```

### Batch Operations

```rust
// Process multiple prompts efficiently
let prompt_names = vec!["greeting", "farewell", "code-review"];
let results: Result<Vec<_>, _> = prompt_names
    .iter()
    .map(|name| library.get(name)?.render(&args))
    .collect();
```

---

## Testing

### Unit Testing Prompts

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_greeting_prompt() {
        let prompt = Prompt::new("greeting", "Hello {{name}}!");
        
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        
        let result = prompt.render(&args).unwrap();
        assert_eq!(result, "Hello World!");
    }
    
    #[test]
    fn test_missing_argument() {
        let prompt = Prompt::new("greeting", "Hello {{name}}!")
            .add_argument(ArgumentSpec {
                name: "name".to_string(),
                required: true,
                ..Default::default()
            });
        
        let args = HashMap::new(); // Missing 'name'
        assert!(prompt.render(&args).is_err());
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_library_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let prompt_content = r#"---
title: Test Prompt
---
Hello {{name}}!"#;
        
        std::fs::write(
            temp_dir.path().join("test.md"),
            prompt_content
        ).unwrap();
        
        let mut library = PromptLibrary::new();
        library.add_directory(temp_dir.path()).unwrap();
        
        let prompt = library.get("test").unwrap();
        assert_eq!(prompt.title.unwrap(), "Test Prompt");
    }
}
```

---

## Best Practices

### 1. Error Handling
- Always use `Result<T>` return types
- Provide meaningful error messages
- Chain operations with `?` operator

### 2. Resource Management
- Cache `PromptLibrary` instances when possible
- Use `Arc<>` for sharing across threads
- Clean up temporary resources

### 3. Template Design
- Keep templates focused and reusable
- Use clear argument names
- Provide default values where appropriate
- Document expected arguments

### 4. Performance
- Load prompts once, use many times
- Consider async patterns for I/O intensive operations
- Use batch operations for multiple prompts

### 5. Testing
- Unit test individual prompts
- Integration test library loading
- Mock storage backends for testing

## See Also

- [Library Examples](./library-examples.md) - Practical usage examples
- [API Reference](./library-api.md) - Complete API documentation
- [rustdoc Documentation](./api/swissarmyhammer/index.html) - Generated API docs
- [MCP Protocol](./mcp-protocol.md) - Model Context Protocol integration