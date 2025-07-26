# Rust Library Guide

SwissArmyHammer is available as a Rust library (`swissarmyhammer`) that you can integrate into your own applications. This guide covers installation, basic usage, and integration patterns.

## Installation

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { git = "https://github.com/wballard/swissarmyhammer" }
```

## Quick Start

### Basic Prompt Library

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec, Prompt};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new prompt library
    let mut library = PromptLibrary::new();
    
    // Create a simple prompt
    let prompt = Prompt::new("greet", "Hello {{name}}!")
        .with_description("A greeting prompt")
        .add_argument(ArgumentSpec {
            name: "name".to_string(),
            description: Some("Name to greet".to_string()),
            required: true,
            default: None,
            type_hint: Some("string".to_string()),
        });
    
    // Add prompt to library
    library.add(prompt)?;
    
    // Add prompts from a directory
    if std::path::Path::new("./.swissarmyhammer/prompts").exists() {
        let count = library.add_directory("./.swissarmyhammer/prompts")?;
        println!("Loaded {} prompts from directory", count);
    }
    
    // List available prompts
    let prompts = library.list()?;
    for prompt in &prompts {
        println!("Available prompt: {}", prompt.name);
    }
    
    // Get a specific prompt
    let prompt = library.get("greet")?;
    println!("Name: {}", prompt.name);
    if let Some(description) = &prompt.description {
        println!("Description: {}", description);
    }
    
    // Prepare arguments
    let mut args = HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    
    // Render the prompt
    let rendered = prompt.render(&args)?;
    println!("Rendered prompt:\n{}", rendered);
    
    Ok(())
}
```

### Custom Prompt Creation

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};
use std::collections::HashMap;

fn create_custom_prompt() -> Result<Prompt, Box<dyn std::error::Error>> {
    let template = r#"
# Code Review: {{ focus | capitalize }}

Please review this code:

```
{{ code }}
```

{% if focus == "security" %}
Focus specifically on security vulnerabilities and best practices.
{% elsif focus == "performance" %}
Focus on performance optimizations and efficiency.
{% else %}
Perform a general code review covering style, bugs, and maintainability.
{% endif %}
"#;
    
    let prompt = Prompt::new("custom-code-review", template)
        .with_description("A custom code review prompt")
        .with_category("development")
        .add_argument(ArgumentSpec {
            name: "code".to_string(),
            description: Some("Code to review".to_string()),
            required: true,
            default: None,
            type_hint: Some("string".to_string()),
        })
        .add_argument(ArgumentSpec {
            name: "focus".to_string(),
            description: Some("Review focus area".to_string()),
            required: false,
            default: Some("general".to_string()),
            type_hint: Some("string".to_string()),
        });
    
    Ok(prompt)
}

// Test the custom prompt
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prompt = create_custom_prompt()?;
    
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());
    args.insert("focus".to_string(), "security".to_string());
    
    let rendered = prompt.render(&args)?;
    println!("{}", rendered);
    
    Ok(())
}
```

## Core Components

### PromptLibrary

The main interface for managing collections of prompts.

```rust
use swissarmyhammer::PromptLibrary;

// Create a new library
let mut library = PromptLibrary::new();

// Add prompts from various sources
library.add_directory("./.swissarmyhammer/prompts")?;

// Query prompts
let prompts = library.list()?;
let prompt = library.get("prompt-name")?;

// Search prompts
let results = library.search("code review")?;

// Render a prompt directly
let mut args = std::collections::HashMap::new();
args.insert("key".to_string(), "value".to_string());
let rendered = library.render_prompt("prompt-name", &args)?;
```

### Prompt

Individual prompt with metadata and template.

```rust
use swissarmyhammer::{Prompt, PromptLoader};
use std::collections::HashMap;

// Load from file using PromptLoader
let loader = PromptLoader::new();
let prompt = loader.load_file("./.swissarmyhammer/prompts/review.md")?;

// Access metadata
println!("Name: {}", prompt.name);
if let Some(description) = &prompt.description {
    println!("Description: {}", description);
}
for arg in &prompt.arguments {
    println!("Argument: {} (required: {})", arg.name, arg.required);
}

// Render with arguments
let mut args = HashMap::new();
args.insert("code".to_string(), "example code".to_string());
let rendered = prompt.render(&args)?;
```

### Template Engine

Template processing with Liquid syntax.

```rust
use swissarmyhammer::template::Template;
use std::collections::HashMap;

let template = Template::new("Hello {{ name | capitalize }}! Today is {{ date }}.")?;

let mut variables = HashMap::new();
variables.insert("name".to_string(), "alice".to_string());
variables.insert("date".to_string(), "2024-01-15".to_string());

let result = template.render(&variables)?;
println!("{}", result); // "Hello Alice! Today is 2024-01-15."
```

## Advanced Usage

### Loading Prompts from String

```rust
use swissarmyhammer::PromptLoader;

let loader = PromptLoader::new();
let content = r#"---
title: My Prompt
description: A custom prompt
arguments:
  - name: input
    description: The input text
    required: true
---

Process this input: {{ input }}
"#;

let prompt = loader.load_from_string("my-prompt", content)?;
```

### Search and Filter

```rust
use swissarmyhammer::PromptLibrary;

let mut library = PromptLibrary::new();
library.add_directory("./.swissarmyhammer/prompts")?;

// Search for prompts
let results = library.search("code review")?;
for prompt in results {
    println!("Found: {} - {}", prompt.name, 
             prompt.description.unwrap_or_default());
}
```

## Integration Examples

### Simple CLI Tool

```rust
use clap::{Arg, Command};
use swissarmyhammer::PromptLibrary;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("my-prompt-tool")
        .arg(Arg::new("prompt")
            .help("Prompt ID to render")
            .required(true)
            .index(1))
        .arg(Arg::new("args")
            .help("Template arguments as key=value pairs")
            .action(clap::ArgAction::Append)
            .short('a')
            .long("arg"))
        .get_matches();

    let mut library = PromptLibrary::new();
    library.add_directory("./.swissarmyhammer/prompts")?;

    let prompt_id = matches.get_one::<String>("prompt")
        .expect("Prompt ID is required");
    let prompt = library.get(prompt_id)?;

    let mut args = HashMap::new();
    if let Some(arg_values) = matches.get_many::<String>("args") {
        for arg in arg_values {
            if let Some((key, value)) = arg.split_once('=') {
                args.insert(key.to_string(), value.to_string());
            }
        }
    }

    let rendered = prompt.render(&args)?;
    println!("{}", rendered);

    Ok(())
}
```

### Configuration Management

```rust
use serde::{Deserialize, Serialize};
use swissarmyhammer::PromptLibrary;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct AppConfig {
    prompt_directories: Vec<String>,
    default_arguments: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt_directories: vec!["./.swissarmyhammer/prompts".to_string()],
            default_arguments: HashMap::new(),
        }
    }
}

fn setup_library(config: &AppConfig) -> Result<PromptLibrary, Box<dyn std::error::Error>> {
    let mut library = PromptLibrary::new();
    
    for dir in &config.prompt_directories {
        library.add_directory(dir)?;
    }
    
    Ok(library)
}
```

## Error Handling

SwissArmyHammer uses comprehensive error types:

```rust
use swissarmyhammer::{SwissArmyHammerError, PromptLibrary};

let library = PromptLibrary::new();

match library.get("nonexistent") {
    Ok(prompt) => {
        // Handle success
        println!("Found prompt: {}", prompt.name);
    }
    Err(SwissArmyHammerError::PromptNotFound(id)) => {
        eprintln!("Prompt '{}' not found", id);
    }
    Err(SwissArmyHammerError::TemplateError(msg)) => {
        eprintln!("Template error: {}", msg);
    }
    Err(SwissArmyHammerError::IoError(io_err)) => {
        eprintln!("I/O error: {}", io_err);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

## Working with Workflow System

```rust
use swissarmyhammer::{Workflow, WorkflowRun, State};

// The workflow system is available but requires more complex setup
// Refer to the workflow module documentation for detailed examples
```

## Working with Issues and Memoranda

```rust
use swissarmyhammer::{
    CreateMemoRequest, Memo, 
    issues::filesystem::FileSystemIssueStorage
};

// These modules provide issue tracking and memo functionality
// See the respective module documentation for usage examples
```

## Best Practices

### Memory Usage
- Prompt libraries cache parsed templates in memory
- For large collections, consider periodically reloading
- Use the search functionality to find specific prompts efficiently

### Error Handling
- Always handle `SwissArmyHammerError` variants appropriately
- Use `?` operator for error propagation in functions returning `Result`
- Log errors appropriately for debugging

### Performance
- Template rendering is generally fast
- Cache commonly used prompts in your application
- Use batch operations when working with multiple prompts

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use swissarmyhammer::{PromptLibrary, Prompt, ArgumentSpec};
    use std::collections::HashMap;

    #[test]
    fn test_prompt_rendering() {
        let prompt = Prompt::new("test", "Hello {{name}}!")
            .add_argument(ArgumentSpec {
                name: "name".to_string(),
                description: Some("Name to greet".to_string()),
                required: true,
                default: None,
                type_hint: Some("string".to_string()),
            });

        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        
        let result = prompt.render(&args).unwrap();
        assert_eq!(result, "Hello World!");
    }
}
```

## See Also

- [Built-in Prompts](./builtin-prompts.md) - Available built-in prompts
- [Creating Prompts](./creating-prompts.md) - How to create custom prompts
- [CLI Reference](./cli-reference.md) - Command-line interface documentation