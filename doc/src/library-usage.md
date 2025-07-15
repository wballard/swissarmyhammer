# Rust Library Guide

SwissArmyHammer is available as a Rust library (`swissarmyhammer`) that you can integrate into your own applications. This guide covers installation, basic usage, and advanced integration patterns.

## Installation

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { git = "https://github.com/wballard/swissarmyhammer", features = ["full"] }
```

### Feature Flags

Control which functionality to include:

```toml
[dependencies]
swissarmyhammer = { 
  git = "https://github.com/wballard/swissarmyhammer", 
  features = ["prompts", "templates", "search", "mcp"] 
}
```

Available features:
- `prompts` - Core prompt management (always enabled)
- `templates` - Liquid template engine with custom filters
- `search` - Full-text search capabilities
- `mcp` - Model Context Protocol server support
- `storage` - Advanced storage backends
- `full` - All features enabled

## Quick Start

### Basic Prompt Library

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new prompt library
    let mut library = PromptLibrary::new();
    
    // Add prompts from a directory
    let count = library.add_directory("./.swissarmyhammer/prompts")?;
    println!("Loaded {} prompts from directory", count);
    
    // List available prompts
    for prompt_id in library.list() {
        println!("Available prompt: {}", prompt_id);
    }
    
    // Get a specific prompt
    let prompt = library.get("code-review")?;
    println!("Name: {}", prompt.name);
    if let Some(description) = &prompt.description {
        println!("Description: {}", description);
    }
    
    // Prepare arguments
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());
    args.insert("language".to_string(), "rust".to_string());
    
    // Render the prompt
    let rendered = prompt.render(&args)?;
    println!("Rendered prompt:\n{}", rendered);
    
    Ok(())
}
```

### Custom Prompt Creation

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};

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
```

## Core Components

### PromptLibrary

The main interface for managing collections of prompts.

```rust
use swissarmyhammer::PromptLibrary;

let mut library = PromptLibrary::new();

// Add prompts from various sources
library.add_directory("./.swissarmyhammer/prompts")?;
library.add_from_file("./special-prompt.md")?;

// Query prompts
let prompts = library.list();
let prompt = library.get("prompt-id")?;
let filtered = library.filter_by_category("review");

// Search prompts (if search feature is enabled)
let results = library.search("code review")?
```

### Prompt

Individual prompt with metadata and template.

```rust
use swissarmyhammer::Prompt;
use std::collections::HashMap;

// Load from file using PromptLoader
let loader = swissarmyhammer::PromptLoader::new();
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

Advanced template processing with custom filters.

```rust
use swissarmyhammer::template::{TemplateEngine, TemplateContext};

let engine = TemplateEngine::new();

let template = "Hello {{ name | capitalize }}! Today is {{ 'now' | format_date: '%Y-%m-%d' }}.";

let mut context = TemplateContext::new();
context.insert("name", "alice");

let result = engine.render(template, &context)?;
println!("{}", result); // "Hello Alice! Today is 2024-01-15."
```

## Advanced Usage

### Custom Storage Backend

Implement your own storage for prompts:

```rust
use swissarmyhammer::storage::{StorageBackend, PromptSource};
use async_trait::async_trait;

struct DatabaseStorage {
    // Your database connection
    db: Database,
}

#[async_trait]
impl StorageBackend for DatabaseStorage {
    async fn list_prompts(&self) -> Result<Vec<String>, StorageError> {
        // Implement database query
        todo!()
    }
    
    async fn get_prompt(&self, id: &str) -> Result<PromptSource, StorageError> {
        // Implement database retrieval
        todo!()
    }
    
    async fn save_prompt(&mut self, id: &str, source: &PromptSource) -> Result<(), StorageError> {
        // Implement database storage
        todo!()
    }
}

// Use custom storage
let storage = DatabaseStorage::new(db);
let mut library = PromptLibrary::with_storage(storage);
```

### Search Integration

Advanced search capabilities:

```rust
use swissarmyhammer::search::{SearchEngine, SearchQuery, SearchResult};

let mut search_engine = SearchEngine::new();

// Index prompts
search_engine.index_prompt("code-review", &prompt).await?;

// Perform searches
let query = SearchQuery::new("code review")
    .with_field("title")
    .with_limit(10)
    .case_sensitive(false);

let results: Vec<SearchResult> = search_engine.search(&query)?;

for result in results {
    println!("Found: {} (score: {:.2})", result.id, result.score);
}
```

### MCP Server Integration

Embed MCP server functionality:

```rust
use swissarmyhammer::mcp::{McpServer, McpConfig};

let config = McpConfig {
    name: "my-prompt-server".to_string(),
    version: "1.0.0".to_string(),
    // ... other config
};

let mut library = PromptLibrary::new();
library.add_directory("./.swissarmyhammer/prompts").await?;

let server = McpServer::new(config, library);

// Run MCP server
server.serve().await?;
```

### Event System

React to library events:

```rust
use swissarmyhammer::events::{EventHandler, PromptEvent};

struct MyEventHandler;

impl EventHandler for MyEventHandler {
    fn handle_prompt_added(&self, id: &str) {
        println!("Prompt added: {}", id);
    }
    
    fn handle_prompt_updated(&self, id: &str) {
        println!("Prompt updated: {}", id);
    }
    
    fn handle_prompt_removed(&self, id: &str) {
        println!("Prompt removed: {}", id);
    }
}

let mut library = PromptLibrary::new();
library.add_event_handler(Box::new(MyEventHandler));
```

### File Watching

Automatically reload prompts when files change:

```rust
use swissarmyhammer::watcher::FileWatcher;

let mut library = PromptLibrary::new();
library.add_directory("./.swissarmyhammer/prompts").await?;

// Start watching for file changes
let _watcher = FileWatcher::new("./.swissarmyhammer/prompts", move |event| {
    match event {
        FileEvent::Created(path) => {
            if let Err(e) = library.reload_file(&path) {
                eprintln!("Failed to load {}: {}", path.display(), e);
            }
        }
        FileEvent::Modified(path) => {
            if let Err(e) = library.reload_file(&path) {
                eprintln!("Failed to reload {}: {}", path.display(), e);
            }
        }
        FileEvent::Deleted(path) => {
            library.remove_file(&path);
        }
    }
});

// Keep the watcher alive
std::thread::sleep(std::time::Duration::from_secs(60));
```

## Integration Examples

### Web Server Integration

```rust
use axum::{extract::Path, http::StatusCode, response::Json, routing::get, Router};
use swissarmyhammer::PromptLibrary;
use std::sync::Arc;
use tokio::sync::RwLock;

type SharedLibrary = Arc<RwLock<PromptLibrary>>;

async fn list_prompts(library: SharedLibrary) -> Json<Vec<String>> {
    let lib = library.read().await;
    Json(lib.list_prompts())
}

async fn get_prompt(
    Path(id): Path<String>,
    library: SharedLibrary,
) -> Result<Json<String>, StatusCode> {
    let lib = library.read().await;
    match lib.get(&id) {
        Ok(prompt) => Ok(Json(prompt.title().to_string())),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[tokio::main]
async fn main() {
    let mut library = PromptLibrary::new();
    library.add_directory("./.swissarmyhammer/prompts").await
        .expect("Failed to load prompts directory");
    let shared_library = Arc::new(RwLock::new(library));

    let app = Router::new()
        .route("/prompts", get({
            let lib = shared_library.clone();
            move || list_prompts(lib)
        }))
        .route("/prompts/:id", get({
            let lib = shared_library.clone();
            move |path| get_prompt(path, lib)
        }));

    let addr = "0.0.0.0:3000".parse()
        .expect("Failed to parse server address");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}
```

### CLI Tool Integration

```rust
use clap::{Arg, Command};
use swissarmyhammer::PromptLibrary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("my-prompt-tool")
        .arg(Arg::new("prompt")
            .help("Prompt ID to render")
            .required(true)
            .index(1))
        .arg(Arg::new("args")
            .help("Template arguments as key=value pairs")
            .multiple_values(true)
            .short('a')
            .long("arg"))
        .get_matches();

    let mut library = PromptLibrary::new();
    library.add_directory("./.swissarmyhammer/prompts").await?;

    let prompt_id = matches.value_of("prompt")
        .expect("Prompt ID is required");
    let prompt = library.get(prompt_id)?;

    let mut args = std::collections::HashMap::new();
    if let Some(arg_values) = matches.values_of("args") {
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
use swissarmyhammer::{PromptLibrary, storage::FileSystemStorage};

#[derive(Serialize, Deserialize)]
struct AppConfig {
    prompt_directories: Vec<String>,
    default_arguments: std::collections::HashMap<String, String>,
    search_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt_directories: vec!["./.swissarmyhammer/prompts".to_string()],
            default_arguments: std::collections::HashMap::new(),
            search_enabled: true,
        }
    }
}

async fn setup_library(config: &AppConfig) -> Result<PromptLibrary, Box<dyn std::error::Error>> {
    let mut library = PromptLibrary::new();
    
    for dir in &config.prompt_directories {
        library.add_directory(dir).await?;
    }
    
    if config.search_enabled {
        library.enable_search()?;
    }
    
    Ok(library)
}
```

## Error Handling

SwissArmyHammer uses comprehensive error types:

```rust
use swissarmyhammer::error::{SwissArmyHammerError, PromptError, TemplateError};

match library.get("nonexistent") {
    Ok(prompt) => {
        // Handle success
    }
    Err(SwissArmyHammerError::PromptNotFound(id)) => {
        eprintln!("Prompt '{}' not found", id);
    }
    Err(SwissArmyHammerError::Template(TemplateError::RenderError(msg))) => {
        eprintln!("Template rendering failed: {}", msg);
    }
    Err(SwissArmyHammerError::Io(io_err)) => {
        eprintln!("I/O error: {}", io_err);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

## Testing

SwissArmyHammer provides testing utilities:

```rust
use swissarmyhammer::testing::{MockPromptLibrary, PromptTestCase};

#[tokio::test]
async fn test_prompt_rendering() {
    let mut library = MockPromptLibrary::new();
    
    let test_case = PromptTestCase::new("test-prompt")
        .with_template("Hello {{ name }}!")
        .with_argument("name", "World")
        .expect_output("Hello World!");
    
    library.add_test_prompt(test_case);
    
    let prompt = library.get("test-prompt")
        .expect("Test prompt should exist");
    let mut args = std::collections::HashMap::new();
    args.insert("name".to_string(), "World".to_string());
    
    let result = prompt.render(&args)
        .expect("Template rendering should succeed");
    assert_eq!(result, "Hello World!");
}
```

## Performance Considerations

### Memory Usage
- Prompt libraries cache parsed templates in memory
- Large collections may require custom storage backends
- Use lazy loading for better memory efficiency

### Concurrency
- `PromptLibrary` is `Send + Sync` when used with appropriate storage
- Template rendering is thread-safe
- Consider using `Arc<RwLock<PromptLibrary>>` for shared access

### Best Practices
- Prefer batch operations for multiple prompts
- Cache rendered templates when arguments don't change
- Use feature flags to include only needed functionality
- Implement proper error handling for production use

## Migration from CLI

If you're migrating from using the CLI to the library:

```rust
// CLI equivalent: swissarmyhammer search "code review"
let results = library.search("code review")?;

// CLI equivalent: swissarmyhammer test prompt-id --arg key=value
let prompt = library.get("prompt-id")?;
let mut args = HashMap::new();
args.insert("key".to_string(), "value".to_string());
let rendered = prompt.render(&args)?;

// CLI equivalent: swissarmyhammer export --all output.tar.gz
library.export_all("output.tar.gz", ExportFormat::TarGz)?;
```

## See Also

- [Library API Reference](./library-api.md) - Complete API documentation
- [Integration Examples](./library-examples.md) - More integration patterns
- [Custom Filters](./custom-filters.md) - Template customization
- [Advanced Prompts](./advanced-prompts.md) - Complex template patterns