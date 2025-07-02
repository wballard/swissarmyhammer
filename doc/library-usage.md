# Using SwissArmyHammer as a Rust Library

SwissArmyHammer can be used as a Rust library to integrate prompt management into your applications.

## Installation

Add SwissArmyHammer to your `Cargo.toml`:

```toml
[dependencies]
swissarmyhammer = "0.1"

# Optional features
swissarmyhammer = { version = "0.1", features = ["search", "mcp"] }
```

## Available Features

- `search` - Enable fuzzy search functionality
- `mcp` - Enable MCP server components
- `full` - Enable all features

## Basic Usage

### Creating a Prompt Library

```rust
use swissarmyhammer::{PromptLibrary, Prompt, ArgumentSpec};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new prompt library
    let mut library = PromptLibrary::new();
    
    // Add prompts from directories
    library.add_directory("./prompts")?;
    library.add_directory("/home/user/.prompts")?;
    
    // List all prompts
    for prompt in library.list()? {
        println!("Found prompt: {}", prompt.name);
    }
    
    Ok(())
}
```

### Creating Prompts Programmatically

```rust
use swissarmyhammer::{Prompt, ArgumentSpec};

// Create a prompt with builder pattern
let prompt = Prompt::new("greeting", "Hello {{ name }}!")
    .with_description("A simple greeting prompt")
    .with_category("examples")
    .with_tags(vec!["greeting".to_string(), "simple".to_string()])
    .add_argument(ArgumentSpec {
        name: "name".to_string(),
        description: Some("The name to greet".to_string()),
        required: true,
        default: None,
        type_hint: Some("string".to_string()),
    });

// Add to library
library.add(prompt)?;
```

### Rendering Prompts

```rust
// Get a prompt from the library
let prompt = library.get("code-review")?;

// Prepare arguments
let mut args = HashMap::new();
args.insert("code".to_string(), "fn main() { println!(\"Hello\"); }".to_string());
args.insert("language".to_string(), "rust".to_string());

// Render the prompt
let rendered = prompt.render(&args)?;
println!("Rendered prompt:\n{}", rendered);
```

## Advanced Usage

### Custom Storage Backend

```rust
use swissarmyhammer::{StorageBackend, Prompt, Result};

struct DatabaseStorage {
    // Your database connection
}

impl StorageBackend for DatabaseStorage {
    fn store(&mut self, prompt: Prompt) -> Result<()> {
        // Store prompt in database
        Ok(())
    }
    
    fn get(&self, name: &str) -> Result<Prompt> {
        // Retrieve from database
        todo!()
    }
    
    fn list(&self) -> Result<Vec<Prompt>> {
        // List all prompts
        todo!()
    }
    
    fn remove(&mut self, name: &str) -> Result<()> {
        // Remove from database
        Ok(())
    }
    
    fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        // Search prompts
        todo!()
    }
}
```

### Using the Template Engine Directly

```rust
use swissarmyhammer::{Template, TemplateEngine};
use std::collections::HashMap;

// Create a template engine
let engine = TemplateEngine::new();

// Create a template
let template = engine.parse("Hello {{ name | capitalize }}!")?;

// Render with arguments
let mut args = HashMap::new();
args.insert("name".to_string(), "world".to_string());

let result = engine.render(&template, &args)?;
assert_eq!(result, "Hello World!");
```

### Search Functionality

```rust
use swissarmyhammer::search::SearchEngine;

// Create a search engine
let search = SearchEngine::new();

// Index prompts
for prompt in library.list()? {
    search.index_prompt(&prompt)?;
}

// Search prompts
let results = search.search("code review")?;
for (prompt, score) in results {
    println!("Found: {} (score: {})", prompt.name, score);
}
```

## Working with Custom Filters

SwissArmyHammer includes many custom Liquid filters:

```rust
let template = r#"
Code:
{{ code | format_lang: "python" }}

Lines: {{ code | count_lines }}
Functions: {{ code | extract_functions | join: ", " }}
"#;

let prompt = Prompt::new("analysis", template);
```

Available custom filters include:
- Code filters: `format_lang`, `extract_functions`, `count_lines`, `dedent`
- Text filters: `slugify`, `word_wrap`, `indent`, `bullet_list`
- Data filters: `to_json`, `from_json`, `from_csv`, `from_yaml`
- Utility filters: `format_date`, `ordinal`, `highlight`, `sample`

## Error Handling

```rust
use swissarmyhammer::{Result, SwissArmyHammerError};

fn load_prompt(name: &str) -> Result<String> {
    let library = PromptLibrary::new();
    
    match library.get(name) {
        Ok(prompt) => {
            let args = HashMap::new();
            prompt.render(&args)
        }
        Err(SwissArmyHammerError::PromptNotFound(name)) => {
            eprintln!("Prompt '{}' not found", name);
            Err(SwissArmyHammerError::PromptNotFound(name))
        }
        Err(e) => {
            eprintln!("Error loading prompt: {}", e);
            Err(e)
        }
    }
}
```

## Integration Examples

### Web Service Integration

```rust
use axum::{Router, Json, extract::Path};
use swissarmyhammer::PromptLibrary;
use std::sync::Arc;

#[derive(serde::Serialize)]
struct PromptResponse {
    name: String,
    rendered: String,
}

async fn render_prompt(
    Path(name): Path<String>,
    library: Arc<PromptLibrary>,
    Json(args): Json<HashMap<String, String>>,
) -> Json<PromptResponse> {
    let prompt = library.get(&name).unwrap();
    let rendered = prompt.render(&args).unwrap();
    
    Json(PromptResponse { name, rendered })
}

let library = Arc::new(PromptLibrary::new());
let app = Router::new()
    .route("/prompts/:name", post(render_prompt))
    .with_state(library);
```

### CLI Tool Integration

```rust
use clap::Parser;
use swissarmyhammer::PromptLibrary;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    prompt: String,
    
    #[arg(short, long)]
    args: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let library = PromptLibrary::new();
    
    let prompt = library.get(&args.prompt)?;
    
    // Parse key=value arguments
    let mut render_args = HashMap::new();
    for arg in args.args {
        let parts: Vec<_> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            render_args.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    
    let rendered = prompt.render(&render_args)?;
    println!("{}", rendered);
    
    Ok(())
}
```

## Best Practices

1. **Error Handling**: Always handle prompt loading and rendering errors gracefully
2. **Caching**: Consider caching frequently used prompts for better performance
3. **Validation**: Validate arguments before rendering to provide better error messages
4. **Security**: Be careful when rendering user-provided content in templates
5. **Organization**: Use categories and tags to organize large prompt libraries

## Performance Considerations

- Prompt loading is lazy by default
- Directory scanning happens on `add_directory()` calls
- Templates are parsed once and cached
- Search indexing is incremental

## Thread Safety

The `PromptLibrary` is thread-safe and can be shared between threads using `Arc`:

```rust
use std::sync::Arc;
use std::thread;

let library = Arc::new(PromptLibrary::new());

let handles: Vec<_> = (0..4)
    .map(|_| {
        let lib = Arc::clone(&library);
        thread::spawn(move || {
            let prompt = lib.get("example").unwrap();
            // Use prompt
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```