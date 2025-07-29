# SwissArmyHammer Data Structures and Type System Patterns

## Core Domain Types

**Prompt System Architecture**
```rust
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub content: String,
    pub arguments: Vec<ArgumentSpec>,
    pub source_path: Option<PathBuf>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct ArgumentSpec {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
    pub arg_type: Option<String>,
}
```

**Template Engine Types**
```rust
pub struct Template {
    liquid_template: liquid::Template,
    is_trusted: bool,  // Security context
}

pub struct TemplateEngine {
    parser: liquid::ParserBuilder,
    partials: Arc<PromptPartialSource>,
}
```

**Memoranda System**
```rust
pub struct Memo {
    pub id: MemoId,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Type-safe wrapper supporting multiple ID formats
pub struct MemoId(String);  // ULID-based with fallback support
```

## Type Safety Through Newtypes

**Domain-Specific Wrappers**
```rust
pub struct IssueName(pub String);      // Validated issue names
pub struct WorkflowName(String);       // Workflow identifiers  
pub struct StateId(String);           // State machine identifiers
pub struct WorkflowRunId(String);     // Runtime execution IDs
```

**Benefits of Newtype Pattern**
- Compile-time type safety
- Domain-specific validation
- Clear API boundaries
- Zero-runtime overhead

## Serialization Architecture

**Comprehensive Serde Integration**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]  // For wrapper types
pub struct MemoId(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    #[serde(flatten)]           // For flexible metadata
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(default)]           // For optional fields
    pub description: Option<String>,
}
```

**Schema Generation**
- `schemars::JsonSchema` for MCP tool definitions
- Automatic API documentation generation
- Type-safe JSON serialization/deserialization
- YAML front matter parsing for prompt files

## Error Type Hierarchy

**Structured Error Design**
```rust
#[derive(Debug, Error)]
#[non_exhaustive]  // Future extensibility
pub enum SwissArmyHammerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),    // Automatic conversion
    
    #[error("Git operation '{operation}' failed: {details}")]
    GitOperationFailed {      // Structured error data
        operation: String,
        details: String,
    },
}
```

**Domain-Specific Error Types**
- `WorkflowError`: State machine execution
- `ActionError`: Workflow action failures
- `ValidationError`: Content validation
- `StorageError`: Backend operations
- `McpError`: Protocol communication

## Collection and Container Patterns  

**Strategic Collection Usage**
```rust
// Template variables
HashMap<String, String>                    // Key-value mapping
HashMap<String, serde_json::Value>        // Flexible metadata

// Search and listing
Vec<Prompt>                              // Ordered collections
Vec<String>                              // Tags and arguments
HashSet<String>                          // Unique collections

// Concurrent access
DashMap<String, CachedValue>            // Thread-safe caching
Arc<Mutex<HashMap<K, V>>>               // Shared mutable state
```

**Performance Considerations**
- `DashMap` for high-concurrency scenarios
- `Vec` for ordered, indexed access
- `HashMap` for key-based lookup
- `HashSet` for membership testing

## Async and Concurrency Types

**Smart Pointer Usage**
```rust
Arc<T>                    // Shared ownership across threads
Box<dyn Trait>           // Trait objects for dynamic dispatch
Cow<'a, str>             // Copy-on-write for string efficiency
```

**Async Trait Patterns**
```rust
#[async_trait]
pub trait MemoStorage: Send + Sync {
    async fn create_memo(&self, title: String, content: String) -> Result<Memo>;
    async fn get_memo(&self, id: &MemoId) -> Result<Memo>;
    async fn search_memos(&self, query: &str) -> Result<Vec<SearchResult>>;
}
```

## Configuration and Environment Types

**Centralized Configuration**
```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub issue_branch_prefix: String,
    pub max_pending_issues_in_summary: usize,
    pub cache_ttl_seconds: u64,
    // ... with environment variable support
}
```

**Global State Management**
```rust
impl Config {
    pub fn global() -> &'static Self {
        static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
        CONFIG.get_or_init(Config::new)
    }
}
```

## Storage Abstraction Patterns

**Pluggable Storage Backend**
```rust
pub trait StorageBackend: Send + Sync {
    fn store(&mut self, prompt: Prompt) -> Result<()>;
    fn get(&self, name: &str) -> Result<Prompt>;
    fn list(&self) -> Result<Vec<Prompt>>;
    fn remove(&mut self, name: &str) -> Result<()>;
    fn search(&self, query: &str) -> Result<Vec<SearchResult>>;
}
```

**Concrete Implementations**
- `MemoryStorage`: HashMap-based for testing
- `FileSystemStorage`: Persistent with DashMap caching
- Future: Database backends, cloud storage

## Request/Response Pattern

**Structured API Types**
```rust
// Request types
pub struct CreateMemoRequest {
    pub title: String,
    pub content: String,
}

pub struct SearchMemosRequest {
    pub query: String,
    pub limit: Option<usize>,
}

// Response types  
pub struct ListMemosResponse {
    pub memos: Vec<Memo>,
    pub total_count: usize,
}

pub struct SearchMemosResponse {
    pub results: Vec<SearchResult>,
    pub query: String,
    pub total_matches: usize,
}
```

This type system design emphasizes safety through the Rust type system, performance through efficient data structures, and maintainability through clear abstractions and consistent patterns.