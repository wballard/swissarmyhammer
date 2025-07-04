# Library Examples

This guide provides practical examples of using SwissArmyHammer as a Rust library in your applications.

## Basic Usage

### Adding to Your Project

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = { git = "https://github.com/wballard/swissarmyhammer.git" }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

### Simple Example

```rust
use swissarmyhammer::{PromptManager, PromptArgument};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a prompt manager
    let manager = PromptManager::new()?;
    
    // Load prompts from default directories
    manager.load_prompts().await?;
    
    // Get a specific prompt
    let prompt = manager.get_prompt("code-review")?;
    
    // Prepare arguments
    let mut args = HashMap::new();
    args.insert("code".to_string(), r#"
        def calculate_sum(a, b):
            return a + b
    "#.to_string());
    args.insert("language".to_string(), "python".to_string());
    
    // Render the prompt
    let rendered = prompt.render(&args)?;
    println!("Rendered prompt:\n{}", rendered);
    
    Ok(())
}
```

## Advanced Examples

### Custom Prompt Directories

```rust
use swissarmyhammer::{PromptManager, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let mut config = Config::default();
    config.prompt_directories.push(PathBuf::from("./my-prompts"));
    config.prompt_directories.push(PathBuf::from("/opt/company/prompts"));
    
    // Create manager with custom config
    let manager = PromptManager::with_config(config)?;
    
    // Load prompts from all directories
    manager.load_prompts().await?;
    
    // List all available prompts
    for prompt in manager.list_prompts() {
        println!("Found prompt: {} - {}", prompt.name, prompt.title);
    }
    
    Ok(())
}
```

### Watching for Changes

```rust
use swissarmyhammer::{PromptManager, WatchEvent};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PromptManager::new()?;
    
    // Create a channel for watch events
    let (tx, mut rx) = mpsc::channel(100);
    
    // Start watching for changes
    manager.watch(tx).await?;
    
    // Handle watch events
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                WatchEvent::PromptAdded(name) => {
                    println!("New prompt added: {}", name);
                }
                WatchEvent::PromptModified(name) => {
                    println!("Prompt modified: {}", name);
                }
                WatchEvent::PromptRemoved(name) => {
                    println!("Prompt removed: {}", name);
                }
            }
        }
    });
    
    // Keep the program running
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    
    Ok(())
}
```

### MCP Server Implementation

```rust
use swissarmyhammer::{PromptManager, MCPServer, MCPRequest, MCPResponse};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create prompt manager
    let manager = PromptManager::new()?;
    manager.load_prompts().await?;
    
    // Create MCP server
    let server = MCPServer::new(manager);
    
    // Listen on TCP socket
    let listener = TcpListener::bind("127.0.0.1:3333").await?;
    println!("MCP server listening on 127.0.0.1:3333");
    
    loop {
        let (mut socket, addr) = listener.accept().await?;
        let server = server.clone();
        
        // Handle each connection
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];
            
            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Error reading from {}: {}", addr, e);
                        return;
                    }
                };
                
                // Parse request
                if let Ok(request) = serde_json::from_slice::<MCPRequest>(&buffer[..n]) {
                    // Handle request
                    let response = server.handle_request(request).await;
                    
                    // Send response
                    let response_bytes = serde_json::to_vec(&response).unwrap();
                    if let Err(e) = socket.write_all(&response_bytes).await {
                        eprintln!("Error writing to {}: {}", addr, e);
                        return;
                    }
                }
            }
        });
    }
}
```

### Custom Template Filters

```rust
use swissarmyhammer::{PromptManager, TemplateEngine, FilterFunction};
use liquid::ValueView;

fn create_custom_filters() -> Vec<(&'static str, FilterFunction)> {
    vec![
        // Custom filter to convert to snake_case
        ("snake_case", Box::new(|input: &dyn ValueView, _args: &[liquid::model::Value]| {
            let s = input.to_kstr().to_string();
            let snake = s.chars().fold(String::new(), |mut acc, ch| {
                if ch.is_uppercase() && !acc.is_empty() {
                    acc.push('_');
                }
                acc.push(ch.to_lowercase().next().unwrap());
                acc
            });
            Ok(liquid::model::Value::scalar(snake))
        })),
        
        // Custom filter to add line numbers
        ("line_numbers", Box::new(|input: &dyn ValueView, _args: &[liquid::model::Value]| {
            let s = input.to_kstr().to_string();
            let numbered = s.lines()
                .enumerate()
                .map(|(i, line)| format!("{:4}: {}", i + 1, line))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(liquid::model::Value::scalar(numbered))
        })),
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create template engine with custom filters
    let mut engine = TemplateEngine::new();
    for (name, filter) in create_custom_filters() {
        engine.register_filter(name, filter);
    }
    
    // Create prompt manager with custom engine
    let manager = PromptManager::with_engine(engine)?;
    
    // Use prompts with custom filters
    let template = r#"
    Function name: {{ function_name | snake_case }}
    
    Code with line numbers:
    {{ code | line_numbers }}
    "#;
    
    let mut args = HashMap::new();
    args.insert("function_name", "calculateTotalPrice");
    args.insert("code", "def hello():\n    print('Hello')\n    return True");
    
    let rendered = engine.render_str(template, &args)?;
    println!("{}", rendered);
    
    Ok(())
}
```

### Prompt Validation

```rust
use swissarmyhammer::{PromptManager, PromptValidator, ValidationRule};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom validation rules
    let rules = vec![
        ValidationRule::RequiredFields(vec!["name", "title", "description"]),
        ValidationRule::ArgumentTypes(HashMap::from([
            ("max_length", "integer"),
            ("temperature", "float"),
            ("enabled", "boolean"),
        ])),
        ValidationRule::TemplatePatterns(vec![
            r"\{\{[^}]+\}\}",  // Must use double braces
        ]),
    ];
    
    // Create validator
    let validator = PromptValidator::new(rules);
    
    // Create manager with validator
    let manager = PromptManager::with_validator(validator)?;
    
    // Load and validate prompts
    match manager.load_prompts().await {
        Ok(_) => println!("All prompts validated successfully"),
        Err(e) => eprintln!("Validation errors: {}", e),
    }
    
    // Validate a specific prompt file
    let prompt_content = std::fs::read_to_string("my-prompt.md")?;
    match manager.validate_prompt_content(&prompt_content) {
        Ok(prompt) => println!("Prompt '{}' is valid", prompt.name),
        Err(errors) => {
            println!("Validation errors:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
    
    Ok(())
}
```

### Batch Processing

```rust
use swissarmyhammer::{PromptManager, BatchProcessor};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PromptManager::new()?;
    manager.load_prompts().await?;
    
    // Create batch processor
    let processor = BatchProcessor::new(manager, 10); // 10 concurrent tasks
    
    // Prepare batch jobs
    let jobs = vec![
        ("code-review", HashMap::from([
            ("code", "def add(a, b): return a + b"),
            ("language", "python"),
        ])),
        ("api-docs", HashMap::from([
            ("api_spec", r#"{"endpoints": ["/users", "/posts"]}"#),
            ("format", "markdown"),
        ])),
        ("test-writer", HashMap::from([
            ("code", "class Calculator { add(a, b) { return a + b; } }"),
            ("framework", "jest"),
        ])),
    ];
    
    // Process in parallel
    let results = processor.process_batch(jobs).await;
    
    // Handle results
    for (index, result) in results.iter().enumerate() {
        match result {
            Ok(rendered) => {
                println!("Job {} completed:", index + 1);
                println!("{}\n", rendered);
            }
            Err(e) => {
                eprintln!("Job {} failed: {}", index + 1, e);
            }
        }
    }
    
    Ok(())
}
```

### Integration with AI Services

```rust
use swissarmyhammer::{PromptManager, AIServiceClient};
use async_trait::async_trait;

// Custom AI service implementation
struct OpenAIClient {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
impl AIServiceClient for OpenAIClient {
    async fn complete(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "model": "gpt-4",
                "messages": [{"role": "user", "content": prompt}],
                "temperature": 0.7,
            }))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("");
        
        Ok(content.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup prompt manager
    let manager = PromptManager::new()?;
    manager.load_prompts().await?;
    
    // Create AI client
    let ai_client = OpenAIClient {
        api_key: std::env::var("OPENAI_API_KEY")?,
        client: reqwest::Client::new(),
    };
    
    // Get and render prompt
    let prompt = manager.get_prompt("code-review")?;
    let args = HashMap::from([
        ("code", "def factorial(n): return 1 if n <= 1 else n * factorial(n-1)"),
        ("language", "python"),
    ]);
    let rendered = prompt.render(&args)?;
    
    // Send to AI service
    println!("Sending prompt to AI service...");
    let response = ai_client.complete(&rendered).await?;
    println!("AI Response:\n{}", response);
    
    Ok(())
}
```

### Web Server Integration

```rust
use swissarmyhammer::PromptManager;
use axum::{
    routing::{get, post},
    Router, Json, Extension,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
struct RenderRequest {
    prompt_name: String,
    arguments: HashMap<String, String>,
}

#[derive(Serialize)]
struct RenderResponse {
    rendered: String,
}

async fn list_prompts(
    Extension(manager): Extension<Arc<PromptManager>>
) -> impl IntoResponse {
    let prompts = manager.list_prompts();
    Json(prompts)
}

async fn render_prompt(
    Extension(manager): Extension<Arc<PromptManager>>,
    Json(request): Json<RenderRequest>,
) -> impl IntoResponse {
    match manager.get_prompt(&request.prompt_name) {
        Ok(prompt) => match prompt.render(&request.arguments) {
            Ok(rendered) => Ok(Json(RenderResponse { rendered })),
            Err(e) => Err((StatusCode::BAD_REQUEST, e.to_string())),
        },
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup prompt manager
    let manager = Arc::new(PromptManager::new()?);
    manager.load_prompts().await?;
    
    // Build web app
    let app = Router::new()
        .route("/prompts", get(list_prompts))
        .route("/render", post(render_prompt))
        .layer(Extension(manager));
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    println!("Web server listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### Testing Utilities

```rust
use swissarmyhammer::{PromptManager, TestHarness, TestCase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = PromptManager::new()?;
    manager.load_prompts().await?;
    
    // Create test harness
    let harness = TestHarness::new(manager);
    
    // Define test cases
    let test_cases = vec![
        TestCase {
            prompt_name: "code-review",
            arguments: HashMap::from([
                ("code", "def divide(a, b): return a / b"),
                ("language", "python"),
            ]),
            expected_contains: vec!["division by zero", "error handling"],
            expected_not_contains: vec!["syntax error"],
        },
        TestCase {
            prompt_name: "api-docs",
            arguments: HashMap::from([
                ("api_spec", r#"{"version": "1.0"}"#),
            ]),
            expected_contains: vec!["API Documentation", "version"],
            expected_not_contains: vec!["error", "invalid"],
        },
    ];
    
    // Run tests
    let results = harness.run_tests(test_cases).await;
    
    // Report results
    for (test, result) in results {
        match result {
            Ok(_) => println!("✓ {} passed", test.prompt_name),
            Err(e) => println!("✗ {} failed: {}", test.prompt_name, e),
        }
    }
    
    Ok(())
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use swissarmyhammer::{PromptManager, SwissArmyHammerError};

#[tokio::main]
async fn main() {
    match run_app().await {
        Ok(_) => println!("Application completed successfully"),
        Err(e) => {
            eprintln!("Application error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_app() -> Result<(), SwissArmyHammerError> {
    let manager = PromptManager::new()
        .map_err(|e| SwissArmyHammerError::Initialization(e.to_string()))?;
    
    // Handle different error types
    match manager.load_prompts().await {
        Ok(_) => println!("Prompts loaded successfully"),
        Err(SwissArmyHammerError::IoError(e)) => {
            eprintln!("File system error: {}", e);
            return Err(SwissArmyHammerError::IoError(e));
        }
        Err(SwissArmyHammerError::ParseError(e)) => {
            eprintln!("Prompt parsing error: {}", e);
            // Continue with partial prompts
        }
        Err(e) => return Err(e),
    }
    
    // Safely get and render prompt
    let prompt_name = "code-review";
    let prompt = manager.get_prompt(prompt_name)
        .map_err(|_| SwissArmyHammerError::PromptNotFound(prompt_name.to_string()))?;
    
    let args = HashMap::from([("code", "print('hello')")]);
    let rendered = prompt.render(&args)
        .map_err(|e| SwissArmyHammerError::RenderError(e.to_string()))?;
    
    println!("Rendered: {}", rendered);
    Ok(())
}
```

## Performance Optimization

### Caching and Pooling

```rust
use swissarmyhammer::{PromptManager, CacheConfig, ConnectionPool};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure caching
    let cache_config = CacheConfig {
        max_size: 100_000_000, // 100MB
        ttl: Duration::from_secs(3600),
        strategy: CacheStrategy::LRU,
    };
    
    // Create connection pool for MCP
    let pool = ConnectionPool::builder()
        .max_connections(100)
        .connection_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .build()?;
    
    // Create optimized manager
    let manager = PromptManager::builder()
        .cache_config(cache_config)
        .connection_pool(pool)
        .parallel_load(true)
        .build()?;
    
    // Benchmark loading
    let start = std::time::Instant::now();
    manager.load_prompts().await?;
    println!("Loaded prompts in {:?}", start.elapsed());
    
    // Benchmark rendering with cache
    let mut total_time = Duration::ZERO;
    for i in 0..1000 {
        let start = std::time::Instant::now();
        let prompt = manager.get_prompt("code-review")?;
        let args = HashMap::from([("code", format!("test {}", i))]);
        let _ = prompt.render(&args)?;
        total_time += start.elapsed();
    }
    println!("Average render time: {:?}", total_time / 1000);
    
    Ok(())
}
```

## Next Steps

- Review the [Library API](./library-api.md) reference
- Learn about [Library Usage](./library-usage.md) patterns
- See [Integration Examples](./examples.md) for more use cases
- Check the [API Documentation](./api-reference.md) for detailed information