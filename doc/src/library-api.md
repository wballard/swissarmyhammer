# Library API Reference

This document provides comprehensive API documentation for the SwissArmyHammer Rust library.

## Core Types

### Prompt

The `Prompt` struct represents a single prompt with metadata and template content.

```rust
pub struct Prompt {
    pub name: String,
    pub content: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub arguments: Vec<ArgumentSpec>,
    pub file_path: Option<PathBuf>,
}
```

#### Methods

- `new(name: &str, content: &str) -> Self` - Create a new prompt
- `with_description(self, description: &str) -> Self` - Add a description (builder pattern)
- `with_category(self, category: &str) -> Self` - Add a category (builder pattern)
- `add_tag(self, tag: &str) -> Self` - Add a tag (builder pattern)
- `add_argument(self, arg: ArgumentSpec) -> Self` - Add an argument specification
- `render(&self, args: &HashMap<String, String>) -> Result<String>` - Render the prompt with arguments
- `validate_arguments(&self, args: &HashMap<String, String>) -> Result<()>` - Validate provided arguments

#### Example

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};
use std::collections::HashMap;

let prompt = Prompt::new("greet", "Hello {{name}}!")
    .with_description("A greeting prompt")
    .add_argument(ArgumentSpec {
        name: "name".to_string(),
        description: Some("Name to greet".to_string()),
        required: true,
        default: None,
        type_hint: Some("string".to_string()),
    });

let mut args = HashMap::new();
args.insert("name".to_string(), "World".to_string());
let result = prompt.render(&args)?;
// result: "Hello World!"
```

### ArgumentSpec

Defines the specification for a prompt argument.

```rust
pub struct ArgumentSpec {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub type_hint: Option<String>,
}
```

### PromptLibrary

The main interface for managing collections of prompts.

```rust
pub struct PromptLibrary {
    // internal fields...
}
```

#### Methods

- `new() -> Self` - Create a new empty library
- `add_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()>` - Load prompts from directory
- `add_prompt(&mut self, prompt: Prompt)` - Add a single prompt
- `get(&self, name: &str) -> Result<&Prompt>` - Get a prompt by name
- `list_prompts(&self) -> Vec<&Prompt>` - List all prompts
- `find_by_category(&self, category: &str) -> Vec<&Prompt>` - Find prompts by category
- `find_by_tag(&self, tag: &str) -> Vec<&Prompt>` - Find prompts by tag
- `remove(&mut self, name: &str) -> Option<Prompt>` - Remove a prompt

#### Example

```rust
use swissarmyhammer::PromptLibrary;

let mut library = PromptLibrary::new();
library.add_directory("./prompts")?;

let prompt = library.get("code-review")?;
let rendered = prompt.render(&args)?;
```

### PromptLoader

Handles loading prompts from various sources.

```rust
pub struct PromptLoader {
    // internal fields...
}
```

#### Methods

- `new() -> Self` - Create a new loader
- `load_file<P: AsRef<Path>>(&self, path: P) -> Result<Prompt>` - Load single prompt file
- `load_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Prompt>>` - Load all prompts from directory
- `load_string(&self, name: &str, content: &str) -> Result<Prompt>` - Load prompt from string

## Template Engine

### Template

Wrapper for Liquid templates with custom filters.

```rust
pub struct Template {
    // internal fields...
}
```

#### Methods

- `new(template_str: &str) -> Result<Self>` - Create template from string
- `render(&self, args: &HashMap<String, String>) -> Result<String>` - Render with arguments
- `raw(&self) -> &str` - Get the raw template string

### TemplateEngine

Manages template parsing and custom filters.

```rust
pub struct TemplateEngine {
    // internal fields...
}
```

#### Methods

- `new() -> Self` - Create new engine
- `default_parser() -> Parser` - Get default Liquid parser with custom filters
- `register_filter<F>(&mut self, name: &str, filter: F)` - Register custom filter

## Storage

### PromptStorage

High-level storage interface for prompts.

```rust
pub trait PromptStorage {
    fn store_prompt(&mut self, prompt: &Prompt) -> Result<()>;
    fn load_prompt(&self, name: &str) -> Result<Prompt>;
    fn list_prompts(&self) -> Result<Vec<String>>;
    fn delete_prompt(&mut self, name: &str) -> Result<()>;
}
```

### StorageBackend

Low-level storage abstraction.

```rust
pub trait StorageBackend {
    fn read(&self, key: &str) -> Result<Vec<u8>>;
    fn write(&mut self, key: &str, data: &[u8]) -> Result<()>;
    fn delete(&mut self, key: &str) -> Result<()>;
    fn list(&self) -> Result<Vec<String>>;
}
```

## Search

*Available with the `search` feature*

### SearchEngine

Full-text search functionality for prompts.

```rust
pub struct SearchEngine {
    // internal fields...
}
```

#### Methods

- `new() -> Result<Self>` - Create new search engine
- `index_prompt(&mut self, prompt: &Prompt) -> Result<()>` - Add prompt to search index
- `search(&self, query: &str) -> Result<Vec<SearchResult>>` - Search for prompts

### SearchResult

Represents a search result.

```rust
pub struct SearchResult {
    pub name: String,
    pub score: f32,
    pub snippet: Option<String>,
}
```

## MCP Integration

*Available with the `mcp` feature*

### McpServer

Model Context Protocol server implementation.

```rust
pub struct McpServer {
    // internal fields...
}
```

#### Methods

- `new(library: PromptLibrary) -> Self` - Create server with prompt library
- `run(&mut self) -> Result<()>` - Start the MCP server

## Plugin System

### SwissArmyHammerPlugin

Trait for creating plugins.

```rust
pub trait SwissArmyHammerPlugin {
    fn name(&self) -> &str;
    fn filters(&self) -> Vec<Box<dyn CustomLiquidFilter>>;
}
```

### CustomLiquidFilter

Trait for custom Liquid template filters.

```rust
pub trait CustomLiquidFilter {
    fn name(&self) -> &str;
    fn filter(&self, input: &str, args: &[&str]) -> Result<String>;
}
```

### PluginRegistry

Manages registered plugins and filters.

```rust
pub struct PluginRegistry {
    // internal fields...
}
```

#### Methods

- `new() -> Self` - Create new registry
- `register_plugin<P: SwissArmyHammerPlugin>(&mut self, plugin: P)` - Register plugin
- `get_filters(&self) -> Vec<&dyn CustomLiquidFilter>` - Get all registered filters

## Error Handling

### SwissArmyHammerError

Main error type for the library.

```rust
pub enum SwissArmyHammerError {
    Io(std::io::Error),
    Template(String),
    PromptNotFound(String),
    Config(String),
    Storage(String),
    Serialization(serde_yaml::Error),
    Other(String),
}
```

### Result Type

Convenient result type alias.

```rust
pub type Result<T> = std::result::Result<T, SwissArmyHammerError>;
```

## Feature Flags

The library supports several optional features:

- `search` - Enables full-text search functionality
- `mcp` - Enables Model Context Protocol server support

Enable features in your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { version = "0.1", features = ["search", "mcp"] }
```

## Complete Example

```rust
use swissarmyhammer::{PromptLibrary, ArgumentSpec, Result};
use std::collections::HashMap;

fn main() -> Result<()> {
    // Create library and load prompts
    let mut library = PromptLibrary::new();
    library.add_directory("./prompts")?;
    
    // Get a prompt
    let prompt = library.get("code-review")?;
    
    // Prepare arguments
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());
    args.insert("language".to_string(), "rust".to_string());
    
    // Render the prompt
    let rendered = prompt.render(&args)?;
    println!("{}", rendered);
    
    Ok(())
}
```

## Advanced Usage

### Custom Filters

Create custom Liquid filters for domain-specific transformations:

```rust
use swissarmyhammer::{CustomLiquidFilter, PluginRegistry, TemplateEngine};

struct UppercaseFilter;

impl CustomLiquidFilter for UppercaseFilter {
    fn name(&self) -> &str { "uppercase" }
    
    fn filter(&self, input: &str, _args: &[&str]) -> Result<String> {
        Ok(input.to_uppercase())
    }
}

let mut registry = PluginRegistry::new();
registry.register_filter("uppercase", Box::new(UppercaseFilter));
```

### Storage Backends

Implement custom storage backends:

```rust
use swissarmyhammer::{StorageBackend, Result};

struct DatabaseBackend {
    // database connection...
}

impl StorageBackend for DatabaseBackend {
    fn read(&self, key: &str) -> Result<Vec<u8>> {
        // Read from database
        todo!()
    }
    
    fn write(&mut self, key: &str, data: &[u8]) -> Result<()> {
        // Write to database
        todo!()
    }
    
    // ... implement other methods
}
```

For more examples and advanced usage patterns, see the [Library Examples](./library-examples.md) page.