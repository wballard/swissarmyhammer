# Migrating from CLI to Library

This guide helps you transition from using SwissArmyHammer as a CLI tool to integrating it as a library in your Rust applications.

## Overview of Changes

The library refactoring in v0.1 introduced:

- Separate library (`swissarmyhammer`) and CLI (`swissarmyhammer-cli`) crates
- Clean public API with modular structure
- Optional features for search and MCP functionality
- Thread-safe implementations
- Custom storage backend support

## CLI vs Library Comparison

### Loading Prompts

**CLI approach:**
```bash
# Prompts automatically loaded from standard directories
swissarmyhammer list
```

**Library approach:**
```rust
use swissarmyhammer::PromptLibrary;

let mut library = PromptLibrary::new();

// Explicitly load directories
library.add_directory("~/.swissarmyhammer/prompts")?;
library.add_directory("./.swissarmyhammer/prompts")?;
```

### Using a Prompt

**CLI approach:**
```bash
swissarmyhammer test code-review --arg code="fn main() {}"
```

**Library approach:**
```rust
use std::collections::HashMap;

let prompt = library.get("code-review")?;

let mut args = HashMap::new();
args.insert("code".to_string(), "fn main() {}".to_string());

let rendered = prompt.render(&args)?;
println!("{}", rendered);
```

### Searching Prompts

**CLI approach:**
```bash
swissarmyhammer search "code review" --fuzzy
```

**Library approach:**
```rust
// Basic search
let results = library.search("code review")?;

// With search engine (requires 'search' feature)
use swissarmyhammer::search::SearchEngine;

let mut search = SearchEngine::new();
for prompt in library.list()? {
    search.index_prompt(&prompt)?;
}

let results = search.search("code review")?;
for (prompt, score) in results {
    println!("{}: {}", prompt.name, score);
}
```

## Common Migration Patterns

### 1. Configuration Loading

Instead of relying on CLI's automatic configuration:

```rust
use swissarmyhammer::PromptLibrary;
use dirs;

pub fn create_configured_library() -> Result<PromptLibrary, Box<dyn Error>> {
    let mut library = PromptLibrary::new();
    
    // Load from standard locations like CLI does
    if let Some(data_dir) = dirs::data_dir() {
        let builtin = data_dir.join("swissarmyhammer/prompts");
        if builtin.exists() {
            library.add_directory(&builtin)?;
        }
    }
    
    if let Some(home) = dirs::home_dir() {
        let user = home.join(".swissarmyhammer/prompts");
        if user.exists() {
            library.add_directory(&user)?;
        }
    }
    
    let local = Path::new(".swissarmyhammer/prompts");
    if local.exists() {
        library.add_directory(local)?;
    }
    
    Ok(library)
}
```

### 2. Validation

Replace CLI validation with programmatic validation:

**CLI:**
```bash
swissarmyhammer validate my-prompt.md
```

**Library:**
```rust
use swissarmyhammer::{Prompt, ArgumentSpec};

fn validate_prompt(prompt: &Prompt) -> Vec<String> {
    let mut errors = Vec::new();
    
    // Check required fields
    if prompt.description.is_none() {
        errors.push("Missing description".to_string());
    }
    
    // Validate template syntax
    match prompt.render(&HashMap::new()) {
        Err(e) if !is_missing_variable_error(&e) => {
            errors.push(format!("Template error: {}", e));
        }
        _ => {}
    }
    
    // Check arguments
    for arg in &prompt.arguments {
        if arg.required && arg.default.is_some() {
            errors.push(format!(
                "Argument '{}' is required but has a default", 
                arg.name
            ));
        }
    }
    
    errors
}
```

### 3. Interactive Testing

Replace CLI's interactive test command:

**CLI:**
```bash
swissarmyhammer test my-prompt
```

**Library:**
```rust
use dialoguer::{Input, theme::ColorfulTheme};
use swissarmyhammer::Prompt;

fn interactive_test(prompt: &Prompt) -> Result<String, Box<dyn Error>> {
    let mut args = HashMap::new();
    
    // Collect required arguments
    for arg in &prompt.arguments {
        if arg.required || arg.default.is_none() {
            let input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(&format!("{} ({})", 
                    arg.name, 
                    arg.description.as_deref().unwrap_or("required")
                ))
                .interact_text()?;
            
            args.insert(arg.name.clone(), input);
        } else if let Some(default) = &arg.default {
            args.insert(arg.name.clone(), default.clone());
        }
    }
    
    // Render and return
    prompt.render(&args)
}
```

### 4. File Watching

Implement file watching like the CLI:

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

fn watch_prompts(library: Arc<Mutex<PromptLibrary>>) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2))?;
    
    // Watch directories
    watcher.watch("~/.swissarmyhammer/prompts", RecursiveMode::Recursive)?;
    watcher.watch("./.swissarmyhammer/prompts", RecursiveMode::Recursive)?;
    
    // Handle events
    loop {
        match rx.recv() {
            Ok(event) => {
                println!("Reloading prompts due to: {:?}", event);
                let mut lib = library.lock().unwrap();
                *lib = create_configured_library()?;
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}
```

### 5. Export/Import

Replace CLI export/import commands:

```rust
use swissarmyhammer::PromptLibrary;
use std::fs;
use tar::Builder;

fn export_prompts(library: &PromptLibrary, output: &Path) -> Result<(), Box<dyn Error>> {
    let file = fs::File::create(output)?;
    let mut archive = Builder::new(file);
    
    for prompt in library.list()? {
        // Create YAML front matter
        let front_matter = serde_yaml::to_string(&prompt)?;
        let content = format!("---\n{}\n---\n\n{}", front_matter, prompt.template);
        
        // Add to archive
        let path = format!("prompts/{}.md", prompt.name);
        let mut header = tar::Header::new_gnu();
        header.set_path(&path)?;
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        
        archive.append(&header, content.as_bytes())?;
    }
    
    archive.finish()?;
    Ok(())
}
```

## Advanced Migration Topics

### Custom Storage Backend

The library allows custom storage implementations:

```rust
use swissarmyhammer::{StorageBackend, Prompt, Result};

struct DatabaseStorage {
    connection: sqlx::PgPool,
}

#[async_trait]
impl StorageBackend for DatabaseStorage {
    async fn store(&mut self, prompt: Prompt) -> Result<()> {
        sqlx::query!(
            "INSERT INTO prompts (name, content, metadata) VALUES ($1, $2, $3)",
            prompt.name,
            serde_json::to_string(&prompt)?,
            serde_json::to_value(&prompt.metadata)?
        )
        .execute(&self.connection)
        .await?;
        Ok(())
    }
    
    // Implement other required methods...
}
```

### Web Service Integration

Transform CLI functionality into web endpoints:

```rust
use axum::{Router, Json, extract::{Path, State}};
use swissarmyhammer::PromptLibrary;
use std::sync::Arc;

type SharedLibrary = Arc<RwLock<PromptLibrary>>;

async fn list_prompts(State(library): State<SharedLibrary>) -> Json<Vec<String>> {
    let lib = library.read().unwrap();
    let prompts = lib.list()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.name)
        .collect();
    Json(prompts)
}

async fn render_prompt(
    Path(name): Path<String>,
    State(library): State<SharedLibrary>,
    Json(args): Json<HashMap<String, String>>,
) -> Result<String, String> {
    let lib = library.read().unwrap();
    let prompt = lib.get(&name).map_err(|e| e.to_string())?;
    prompt.render(&args).map_err(|e| e.to_string())
}

let app = Router::new()
    .route("/prompts", get(list_prompts))
    .route("/prompts/:name/render", post(render_prompt))
    .with_state(Arc::new(RwLock::new(library)));
```

### Performance Optimization

The library provides more control over performance:

```rust
use swissarmyhammer::{PromptLibrary, MemoryStorage};
use lru::LruCache;

struct CachedLibrary {
    library: PromptLibrary,
    render_cache: LruCache<(String, HashMap<String, String>), String>,
}

impl CachedLibrary {
    fn render_cached(&mut self, name: &str, args: HashMap<String, String>) -> Result<String> {
        let key = (name.to_string(), args.clone());
        
        if let Some(cached) = self.render_cache.get(&key) {
            return Ok(cached.clone());
        }
        
        let prompt = self.library.get(name)?;
        let rendered = prompt.render(&args)?;
        
        self.render_cache.put(key, rendered.clone());
        Ok(rendered)
    }
}
```

## Testing Your Migration

1. **Unit Tests**: Test individual prompt rendering
2. **Integration Tests**: Test full library functionality
3. **Performance Tests**: Ensure acceptable performance
4. **Compatibility Tests**: Verify prompts work identically

Example test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_compatibility() {
        let library = create_configured_library().unwrap();
        let prompt = library.get("code-review").unwrap();
        
        let mut args = HashMap::new();
        args.insert("code".to_string(), "test".to_string());
        
        let result = prompt.render(&args);
        assert!(result.is_ok());
    }
}
```

## Troubleshooting

### Common Issues

1. **Missing prompts**: Ensure you're loading all directories
2. **Template errors**: The library is stricter about undefined variables
3. **Performance**: Use caching for frequently rendered prompts
4. **Thread safety**: Use `Arc<RwLock<PromptLibrary>>` for sharing

### Getting Help

- Check the [API Reference](api-reference.md)
- See [examples](https://github.com/wballard/swissarmyhammer/tree/main/swissarmyhammer/examples)
- File issues for migration problems