# API Reference

This page provides a comprehensive reference for the SwissArmyHammer Rust library API. The library is designed to be used as a dependency in your own Rust projects for managing prompts and templates.

## Getting Started

See [INSTALLATION.md](../../INSTALLATION.md) for installation instructions.

## Core Types

### Prompt

The central type representing a prompt with metadata and template content.

```rust
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub template: String,
    pub arguments: Vec<ArgumentSpec>,
    pub source: Option<PathBuf>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

#### Methods

```rust
impl Prompt {
    /// Create a new prompt with name and template
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self

    /// Render the prompt with given arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String>

    /// Add an argument specification
    pub fn add_argument(self, arg: ArgumentSpec) -> Self

    /// Set the description
    pub fn with_description(self, description: impl Into<String>) -> Self

    /// Set the category
    pub fn with_category(self, category: impl Into<String>) -> Self

    /// Set tags
    pub fn with_tags(self, tags: Vec<String>) -> Self
}
```

#### Example

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};
use std::collections::HashMap;

// Create a prompt
let prompt = Prompt::new("greet", "Hello {{name}}!")
    .with_description("A simple greeting prompt")
    .with_category("examples")
    .add_argument(ArgumentSpec {
        name: "name".to_string(),
        description: Some("The name to greet".to_string()),
        required: true,
        default: None,
        type_hint: Some("string".to_string()),
    });

// Render with arguments
let mut args = HashMap::new();
args.insert("name".to_string(), "World".to_string());
let result = prompt.render(&args)?;
assert_eq!(result, "Hello World!");
```

### ArgumentSpec

Specification for template arguments.

```rust
pub struct ArgumentSpec {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub type_hint: Option<String>,
}
```

## Prompt Management

### PromptLibrary

High-level interface for managing collections of prompts.

```rust
pub struct PromptLibrary;

impl PromptLibrary {
    /// Create a new empty library
    pub fn new() -> Self

    /// Create library with custom storage backend
    pub fn with_storage(storage: Box<dyn StorageBackend>) -> Self

    /// Add prompts from a directory
    pub fn add_directory(&mut self, path: impl AsRef<Path>) -> Result<usize>

    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Result<Prompt>

    /// List all prompts
    pub fn list(&self) -> Result<Vec<Prompt>>

    /// Search prompts by query
    pub fn search(&self, query: &str) -> Result<Vec<Prompt>>

    /// Add a single prompt
    pub fn add(&mut self, prompt: Prompt) -> Result<()>

    /// Remove a prompt by name
    pub fn remove(&mut self, name: &str) -> Result<()>
}
```

#### Example

```rust
use swissarmyhammer::PromptLibrary;

// Create a library and load prompts from directory
let mut library = PromptLibrary::new();
library.add_directory("./prompts")?;

// Get and render a prompt
let prompt = library.get("code-review")?;
let rendered = prompt.render(&args)?;
```

### PromptLoader

Lower-level interface for loading prompts from files.

```rust
pub struct PromptLoader;

impl PromptLoader {
    /// Create a new loader
    pub fn new() -> Self

    /// Load all prompts from a directory
    pub fn load_directory(&self, path: impl AsRef<Path>) -> Result<Vec<Prompt>>

    /// Load a single prompt from a file
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<Prompt>
}
```

## Template System

### Template

Wrapper around Liquid templates with SwissArmyHammer-specific filters.

```rust
pub struct Template;

impl Template {
    /// Create a new template from string
    pub fn new(template_str: &str) -> Result<Self>

    /// Render template with arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String>

    /// Get raw template string
    pub fn raw(&self) -> &str
}
```

### TemplateEngine

Low-level template engine interface.

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    /// Create new template engine
    pub fn new() -> Self

    /// Create with custom Liquid parser
    pub fn with_parser(parser: liquid::Parser) -> Self

    /// Get default parser with SwissArmyHammer filters
    pub fn default_parser() -> liquid::Parser

    /// Parse template string
    pub fn parse(&self, template_str: &str) -> Result<Template>

    /// Parse and render in one step
    pub fn render(&self, template_str: &str, args: &HashMap<String, String>) -> Result<String>
}
```

## Storage System

### StorageBackend

Trait for implementing custom storage backends.

```rust
pub trait StorageBackend: Send + Sync {
    /// Store a prompt
    fn store(&mut self, prompt: Prompt) -> Result<()>;

    /// Get a prompt by name
    fn get(&self, name: &str) -> Result<Prompt>;

    /// List all prompts
    fn list(&self) -> Result<Vec<Prompt>>;

    /// Remove a prompt
    fn remove(&mut self, name: &str) -> Result<()>;

    /// Search prompts by query
    fn search(&self, query: &str) -> Result<Vec<Prompt>>;

    /// Check if prompt exists
    fn exists(&self, name: &str) -> Result<bool>;

    /// Get total count
    fn count(&self) -> Result<usize>;
}
```

### Built-in Storage Implementations

#### MemoryStorage

In-memory storage for testing and temporary use.

```rust
pub struct MemoryStorage;

impl MemoryStorage {
    pub fn new() -> Self
}
```

#### FileSystemStorage

File-based storage using markdown files.

```rust
pub struct FileSystemStorage;

impl FileSystemStorage {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self>
    pub fn reload_cache(&self) -> Result<()>
}
```

#### PromptStorage

High-level storage interface with backend abstraction.

```rust
pub struct PromptStorage;

impl PromptStorage {
    /// Create with custom backend
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self

    /// Create memory-based storage
    pub fn memory() -> Self

    /// Create file-system based storage
    pub fn file_system(path: impl AsRef<Path>) -> Result<Self>

    // ... same methods as StorageBackend trait
}
```

## Search System (Feature: "search")

### SearchResult

Result from search operations with scoring.

```rust
pub struct SearchResult {
    pub prompt: Prompt,
    pub score: f32,
}
```

### SearchEngine

Full-text and fuzzy search capabilities.

```rust
pub struct SearchEngine;

impl SearchEngine {
    /// Create new search engine
    pub fn new() -> Result<Self>

    /// Create with prompts from directory
    pub fn with_directory(path: impl AsRef<Path>) -> Result<Self>

    /// Index a single prompt
    pub fn index_prompt(&mut self, prompt: &Prompt) -> Result<()>

    /// Index multiple prompts
    pub fn index_prompts(&mut self, prompts: &[Prompt]) -> Result<()>

    /// Commit index changes
    pub fn commit(&mut self) -> Result<()>

    /// Full-text search
    pub fn search(&self, query: &str, prompts: &[Prompt]) -> Result<Vec<SearchResult>>

    /// Fuzzy string matching
    pub fn fuzzy_search(&self, query: &str, prompts: &[Prompt]) -> Vec<SearchResult>

    /// Combined full-text and fuzzy search
    pub fn hybrid_search(&self, query: &str, prompts: &[Prompt]) -> Result<Vec<SearchResult>>
}
```

## MCP Server Support (Feature: "mcp")

### McpServer

Model Context Protocol server implementation.

```rust
pub struct McpServer;

impl McpServer {
    /// Create MCP server with prompt library
    pub fn new(library: PromptLibrary) -> Self

    /// Get reference to library
    pub fn library(&self) -> &Arc<RwLock<PromptLibrary>>

    /// Get server information
    pub fn info(&self) -> ServerInfo
}
```

### ServerInfo

Server metadata.

```rust
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
```

## Error Handling

### SwissArmyHammerError

Main error type for the library.

```rust
pub enum SwissArmyHammerError {
    /// IO operation failed
    Io(std::io::Error),
    
    /// Template parsing or rendering failed
    Template(String),
    
    /// Prompt not found
    PromptNotFound(String),
    
    /// Invalid configuration
    Config(String),
    
    /// Storage backend error
    Storage(String),
    
    /// Serialization/deserialization error
    Serialization(serde_yaml::Error),
    
    /// Other errors
    Other(String),
}
```

### Result

Type alias for `std::result::Result<T, SwissArmyHammerError>`.

```rust
pub type Result<T> = std::result::Result<T, SwissArmyHammerError>;
```

## Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `default` | `["async", "search", "watch"]` | Standard async + search + file watching |
| `async` | Async/await support | `tokio` |
| `search` | Full-text and fuzzy search | `tantivy`, `fuzzy-matcher` |
| `watch` | File watching for auto-reload | `notify` |
| `mcp` | Model Context Protocol server | `rmcp` |
| `full` | All features enabled | All of the above |

## Constants

### VERSION

Library version string.

```rust
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

## Prelude

For convenient imports:

```rust
use swissarmyhammer::prelude::*;
```

This imports:
- `Prompt`, `PromptLibrary`, `PromptLoader`, `ArgumentSpec`
- `PromptStorage`, `StorageBackend`
- `Template`, `TemplateEngine`
- `Result`, `SwissArmyHammerError`
- `McpServer` (if `mcp` feature enabled)
- `SearchEngine`, `SearchResult` (if `search` feature enabled)

## Complete Example

```rust
use swissarmyhammer::prelude::*;
use std::collections::HashMap;

fn main() -> Result<()> {
    // Create a prompt library
    let mut library = PromptLibrary::new();
    
    // Add prompts from directory
    library.add_directory("./prompts")?;
    
    // Get a prompt
    let prompt = library.get("code-review")?;
    
    // Prepare arguments
    let mut args = HashMap::new();
    args.insert("code".to_string(), "fn main() {}".to_string());
    args.insert("language".to_string(), "rust".to_string());
    
    // Render the prompt
    let rendered = prompt.render(&args)?;
    println!("{}", rendered);
    
    // Search for prompts
    let results = library.search("debug")?;
    for prompt in results {
        println!("Found: {}", prompt.name);
    }
    
    Ok(())
}
```

For more examples, see the [Examples](./examples.md) page.