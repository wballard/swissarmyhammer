# SwissArmyHammer API Reference

## Core Types

### PromptLibrary

The main entry point for managing prompts.

```rust
pub struct PromptLibrary {
    // ...
}

impl PromptLibrary {
    /// Create a new empty prompt library
    pub fn new() -> Self
    
    /// Add a single prompt
    pub fn add(&mut self, prompt: Prompt) -> Result<()>
    
    /// Add all prompts from a directory
    pub fn add_directory(&mut self, path: impl AsRef<Path>) -> Result<usize>
    
    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Result<Prompt>
    
    /// List all prompts
    pub fn list(&self) -> Result<Vec<Prompt>>
    
    /// Search prompts by query
    pub fn search(&self, query: &str) -> Result<Vec<Prompt>>
    
    /// Remove a prompt
    pub fn remove(&mut self, name: &str) -> Result<()>
    
    /// Check if a prompt exists
    pub fn contains(&self, name: &str) -> bool
    
    /// Get total count
    pub fn len(&self) -> usize
    
    /// Check if empty
    pub fn is_empty(&self) -> bool
}
```

### Prompt

Represents a single prompt with metadata and template.

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

impl Prompt {
    /// Create a new prompt
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self
    
    /// Render the prompt with arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String>
    
    /// Builder methods
    pub fn with_description(mut self, desc: impl Into<String>) -> Self
    pub fn with_category(mut self, category: impl Into<String>) -> Self
    pub fn with_tags(mut self, tags: Vec<String>) -> Self
    pub fn add_argument(mut self, arg: ArgumentSpec) -> Self
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self
}
```

### ArgumentSpec

Defines a prompt argument/variable.

```rust
pub struct ArgumentSpec {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub type_hint: Option<String>,
}

impl ArgumentSpec {
    /// Create a new required argument
    pub fn required(name: impl Into<String>) -> Self
    
    /// Create a new optional argument
    pub fn optional(name: impl Into<String>) -> Self
    
    /// Builder methods
    pub fn with_description(mut self, desc: impl Into<String>) -> Self
    pub fn with_default(mut self, default: impl Into<String>) -> Self
    pub fn with_type_hint(mut self, hint: impl Into<String>) -> Self
}
```

### Template

Low-level template handling.

```rust
pub struct Template {
    // ...
}

impl Template {
    /// Parse a template string
    pub fn new(template_str: &str) -> Result<Self>
    
    /// Render with arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String>
    
    /// Get the raw template string
    pub fn raw(&self) -> &str
}
```

### TemplateEngine

Template engine with custom filters.

```rust
pub struct TemplateEngine {
    // ...
}

impl TemplateEngine {
    /// Create with default configuration
    pub fn new() -> Self
    
    /// Create with custom parser
    pub fn with_parser(parser: liquid::Parser) -> Self
    
    /// Parse a template string
    pub fn parse(&self, template_str: &str) -> Result<Template>
    
    /// Render a template string directly
    pub fn render(&self, template_str: &str, args: &HashMap<String, String>) -> Result<String>
}
```

## Storage Traits

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
    
    /// Search prompts
    fn search(&self, query: &str) -> Result<Vec<Prompt>>;
    
    /// Check existence (default implementation provided)
    fn exists(&self, name: &str) -> Result<bool> {
        self.get(name).map(|_| true).or_else(|e| match e {
            SwissArmyHammerError::PromptNotFound(_) => Ok(false),
            _ => Err(e),
        })
    }
    
    /// Get count (default implementation provided)
    fn count(&self) -> Result<usize> {
        self.list().map(|prompts| prompts.len())
    }
}
```

### Built-in Storage Implementations

```rust
/// In-memory storage
pub struct MemoryStorage { /* ... */ }

/// File system storage
pub struct FileStorage { /* ... */ }

impl FileStorage {
    /// Create file storage for a directory
    pub fn new(path: impl AsRef<Path>) -> Result<Self>
    
    /// Watch for file changes
    pub fn watch(&mut self) -> Result<()>
    
    /// Stop watching
    pub fn unwatch(&mut self)
}
```

## Search Module

### SearchEngine

Full-text search functionality (requires `search` feature).

```rust
#[cfg(feature = "search")]
pub struct SearchEngine {
    // ...
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self
    
    /// Index a prompt
    pub fn index_prompt(&mut self, prompt: &Prompt) -> Result<()>
    
    /// Remove from index
    pub fn remove_prompt(&mut self, name: &str) -> Result<()>
    
    /// Search with fuzzy matching
    pub fn search(&self, query: &str) -> Result<Vec<(Prompt, f32)>>
    
    /// Clear all indexed data
    pub fn clear(&mut self) -> Result<()>
}
```

## MCP Module

### McpServer

MCP protocol server (requires `mcp` feature).

```rust
#[cfg(feature = "mcp")]
pub struct McpServer {
    // ...
}

impl McpServer {
    /// Create server with prompt library
    pub fn new(library: PromptLibrary) -> Self
    
    /// Get server info
    pub fn info(&self) -> ServerInfo
    
    /// Run the server
    pub async fn run(self, shutdown: oneshot::Receiver<()>) -> Result<()>
}

pub struct ServerInfo {
    pub name: String,
    pub version: String,
}
```

## Error Types

### SwissArmyHammerError

Main error type for the library.

```rust
#[derive(Debug, thiserror::Error)]
pub enum SwissArmyHammerError {
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),
    
    #[error("Template error: {0}")]
    Template(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    
    #[error("Search error: {0}")]
    Search(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
}

pub type Result<T> = std::result::Result<T, SwissArmyHammerError>;
```

## Prelude

Common imports for convenience.

```rust
pub mod prelude {
    pub use crate::{
        Prompt,
        PromptLibrary,
        ArgumentSpec,
        Template,
        TemplateEngine,
        Result,
        SwissArmyHammerError,
    };
}

// Usage:
use swissarmyhammer::prelude::*;
```

## Feature Flags

- `default`: Basic functionality
- `search`: Enable search engine
- `mcp`: Enable MCP server
- `full`: All features

## Examples

### Basic Usage

```rust
use swissarmyhammer::prelude::*;
use std::collections::HashMap;

let mut library = PromptLibrary::new();
library.add_directory("./prompts")?;

let prompt = library.get("example")?;
let mut args = HashMap::new();
args.insert("name".to_string(), "World".to_string());

let result = prompt.render(&args)?;
```

### Custom Storage

```rust
use swissarmyhammer::{StorageBackend, MemoryStorage};

let storage = MemoryStorage::new();
let mut library = PromptLibrary::with_storage(Box::new(storage));
```

### Search Integration

```rust
#[cfg(feature = "search")]
{
    use swissarmyhammer::search::SearchEngine;
    
    let mut search = SearchEngine::new();
    for prompt in library.list()? {
        search.index_prompt(&prompt)?;
    }
    
    let results = search.search("code review")?;
}
```

## Thread Safety

- `PromptLibrary`: Thread-safe with `Arc<RwLock<_>>` internally
- `SearchEngine`: Thread-safe with `Arc<RwLock<_>>` internally
- `StorageBackend`: Must implement `Send + Sync`

## Performance Notes

- Prompts are loaded lazily when possible
- Templates are parsed once and cached
- Directory scanning is optimized with parallel processing
- Search indexing is incremental