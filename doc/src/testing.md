# Testing

This guide covers testing practices and strategies for SwissArmyHammer development.

## Overview

SwissArmyHammer uses a comprehensive testing approach:
- **Unit tests** - Test individual components
- **Integration tests** - Test component interactions
- **End-to-end tests** - Test complete workflows
- **Property tests** - Test with generated inputs
- **Benchmark tests** - Test performance

## Test Organization

```
swissarmyhammer/
├── src/
│   └── *.rs                  # Unit tests in source files
├── tests/
│   ├── integration/          # Integration test files
│   ├── common/              # Shared test utilities
│   └── fixtures/            # Test data files
├── benches/                 # Benchmark tests
└── examples/                # Example code (also tested)
```

## Unit Testing

### Basic Unit Tests

Place unit tests in the same file as the code:

```rust
// src/prompts/prompt.rs

pub struct Prompt {
    pub name: String,
    pub title: String,
    pub content: String,
}

impl Prompt {
    pub fn parse(content: &str) -> Result<Self> {
        // Implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_prompt() {
        let content = r#"---
name: test
title: Test Prompt
---
Content here"#;

        let prompt = Prompt::parse(content).unwrap();
        assert_eq!(prompt.name, "test");
        assert_eq!(prompt.title, "Test Prompt");
        assert!(prompt.content.contains("Content here"));
    }

    #[test]
    fn test_parse_missing_name() {
        let content = r#"---
title: Test Prompt
---
Content"#;

        let result = Prompt::parse(content);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }
}
```

### Testing Private Functions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Test private functions by making them pub(crate) in test mode
    #[test]
    fn test_private_helper() {
        // Can access private functions within the module
        let result = validate_prompt_name("test-name");
        assert!(result);
    }
}
```

### Mock Dependencies

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;

    #[automock]
    trait FileSystem {
        fn read_file(&self, path: &Path) -> io::Result<String>;
    }

    #[test]
    fn test_with_mock_filesystem() {
        let mut mock = MockFileSystem::new();
        mock.expect_read_file()
            .returning(|_| Ok("file content".to_string()));

        let result = process_with_fs(&mock, "test.md");
        assert!(result.is_ok());
    }
}
```

## Integration Testing

### Basic Integration Test

Create files in `tests/integration/`:

```rust
// tests/integration/prompt_loading.rs

use swissarmyhammer::{PromptManager, Config};
use tempfile::tempdir;
use std::fs;

#[test]
fn test_load_prompts_from_directory() {
    // Create temporary directory
    let temp_dir = tempdir().unwrap();
    let prompt_path = temp_dir.path().join("test.md");
    
    // Write test prompt
    fs::write(&prompt_path, r#"---
name: test-prompt
title: Test Prompt
---
Test content"#).unwrap();

    // Test loading
    let mut config = Config::default();
    config.prompt_directories.push(temp_dir.path().to_path_buf());
    
    let manager = PromptManager::with_config(config).unwrap();
    manager.load_prompts().unwrap();
    
    // Verify
    let prompt = manager.get_prompt("test-prompt").unwrap();
    assert_eq!(prompt.title, "Test Prompt");
}
```

### Testing MCP Server

```rust
// tests/integration/mcp_server.rs

use swissarmyhammer::mcp::{MCPServer, MCPRequest, MCPResponse};
use serde_json::json;

#[tokio::test]
async fn test_mcp_initialize() {
    let server = MCPServer::new();
    
    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        method: "initialize".to_string(),
        params: json!({}),
        id: Some(json!(1)),
    };
    
    let response = server.handle_request(request).await.unwrap();
    
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.result.is_some());
    assert!(response.result.unwrap()["serverInfo"]["name"]
        .as_str()
        .unwrap()
        .contains("swissarmyhammer"));
}

#[tokio::test]
async fn test_mcp_list_prompts() {
    let server = setup_test_server().await;
    
    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        method: "prompts/list".to_string(),
        params: json!({}),
        id: Some(json!(2)),
    };
    
    let response = server.handle_request(request).await.unwrap();
    let prompts = &response.result.unwrap()["prompts"];
    
    assert!(prompts.is_array());
    assert!(!prompts.as_array().unwrap().is_empty());
}
```

### Testing CLI Commands

```rust
// tests/integration/cli_commands.rs

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available prompts:"));
}

#[test]
fn test_serve_command_help() {
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    
    cmd.arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the MCP server"));
}

#[test]
fn test_export_import_workflow() {
    let temp_dir = tempdir().unwrap();
    let export_path = temp_dir.path().join("export.tar.gz");
    
    // Export
    Command::cargo_bin("swissarmyhammer").unwrap()
        .arg("export")
        .arg(&export_path)
        .assert()
        .success();
    
    // Import
    Command::cargo_bin("swissarmyhammer").unwrap()
        .arg("import")
        .arg(&export_path)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("Would import"));
}
```

## Property Testing

### Using Proptest

```rust
// src/validation.rs

use proptest::prelude::*;

fn is_valid_prompt_name(name: &str) -> bool {
    !name.is_empty() 
        && name.chars().all(|c| c.is_alphanumeric() || c == '-')
        && name.chars().next().unwrap().is_alphabetic()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_valid_names_accepted(name in "[a-z][a-z0-9-]{0,50}") {
            assert!(is_valid_prompt_name(&name));
        }

        #[test]
        fn test_invalid_names_rejected(name in "[^a-z].*|.*[^a-z0-9-].*") {
            // Names starting with non-letter or containing invalid chars
            if !name.chars().next().unwrap().is_alphabetic() 
                || name.chars().any(|c| !c.is_alphanumeric() && c != '-') {
                assert!(!is_valid_prompt_name(&name));
            }
        }
    }
}
```

### Testing Template Rendering

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_template_escaping(
        user_input in any::<String>(),
        template in "Hello {{name}}!"
    ) {
        let mut args = HashMap::new();
        args.insert("name", &user_input);
        
        let result = render_template(&template, &args).unwrap();
        
        // Should not contain raw HTML
        if user_input.contains('<') {
            assert!(!result.contains('<'));
        }
    }
}
```

## Testing Async Code

### Basic Async Tests

```rust
#[tokio::test]
async fn test_async_prompt_loading() {
    let manager = PromptManager::new();
    
    let result = manager.load_prompts_async().await;
    assert!(result.is_ok());
    
    let prompts = manager.list_prompts().await;
    assert!(!prompts.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_concurrent_access() {
    let manager = Arc::new(PromptManager::new());
    
    let handle1 = {
        let mgr = Arc::clone(&manager);
        tokio::spawn(async move {
            mgr.get_prompt("test1").await
        })
    };
    
    let handle2 = {
        let mgr = Arc::clone(&manager);
        tokio::spawn(async move {
            mgr.get_prompt("test2").await
        })
    };
    
    let (result1, result2) = tokio::join!(handle1, handle2);
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}
```

### Testing Timeouts

```rust
#[tokio::test]
async fn test_operation_timeout() {
    let manager = PromptManager::new();
    
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        manager.slow_operation()
    ).await;
    
    assert!(result.is_ok(), "Operation should complete within timeout");
}
```

## Test Fixtures

### Using Test Data

Create reusable test data in `tests/fixtures/`:

```rust
// tests/common/mod.rs

use std::path::PathBuf;

pub fn test_prompt_content() -> &'static str {
    r#"---
name: test-prompt
title: Test Prompt
description: A prompt for testing
arguments:
  - name: input
    description: Test input
    required: true
---
Process this input: {{input}}"#
}

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixtures_dir().join(name))
        .expect("Failed to load fixture")
}
```

### Test Builders

```rust
// tests/common/builders.rs

pub struct PromptBuilder {
    name: String,
    title: String,
    content: String,
    arguments: Vec<ArgumentSpec>,
}

impl PromptBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            title: format!("{} Title", name),
            content: "Default content".to_string(),
            arguments: vec![],
        }
    }
    
    pub fn with_argument(mut self, name: &str, required: bool) -> Self {
        self.arguments.push(ArgumentSpec {
            name: name.to_string(),
            required,
            ..Default::default()
        });
        self
    }
    
    pub fn build(self) -> String {
        // Generate YAML front matter and content
        format!(r#"---
name: {}
title: {}
arguments:
{}
---
{}"#, self.name, self.title, 
            self.arguments.iter()
                .map(|a| format!("  - name: {}\n    required: {}", a.name, a.required))
                .collect::<Vec<_>>()
                .join("\n"),
            self.content)
    }
}

// Usage in tests
#[test]
fn test_with_builder() {
    let prompt_content = PromptBuilder::new("test")
        .with_argument("input", true)
        .with_argument("format", false)
        .build();
    
    let prompt = Prompt::parse(&prompt_content).unwrap();
    assert_eq!(prompt.arguments.len(), 2);
}
```

## Performance Testing

### Benchmarks

Create benchmarks in `benches/`:

```rust
// benches/prompt_loading.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use swissarmyhammer::PromptManager;

fn benchmark_prompt_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt_loading");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let temp_dir = create_test_prompts(size);
                b.iter(|| {
                    let manager = PromptManager::new();
                    manager.add_directory(temp_dir.path());
                    manager.load_prompts()
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_template_rendering(c: &mut Criterion) {
    c.bench_function("render_simple_template", |b| {
        let template = "Hello {{name}}, welcome to {{place}}!";
        let mut args = HashMap::new();
        args.insert("name", "Alice");
        args.insert("place", "Wonderland");
        
        b.iter(|| {
            black_box(render_template(template, &args))
        });
    });
}

criterion_group!(benches, benchmark_prompt_loading, benchmark_template_rendering);
criterion_main!(benches);
```

### Profiling Tests

```rust
#[test]
#[ignore] // Run with cargo test -- --ignored
fn profile_large_prompt_set() {
    let temp_dir = create_test_prompts(10000);
    
    let start = Instant::now();
    let manager = PromptManager::new();
    manager.add_directory(temp_dir.path());
    manager.load_prompts().unwrap();
    let duration = start.elapsed();
    
    println!("Loaded 10000 prompts in {:?}", duration);
    assert!(duration < Duration::from_secs(5), "Loading too slow");
}
```

## Test Coverage

### Generating Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# With specific features
cargo tarpaulin --features "experimental" --out Lcov

# Exclude test code from coverage
cargo tarpaulin --exclude-files "*/tests/*" --exclude-files "*/benches/*"
```

### Coverage Configuration

`.tarpaulin.toml`:

```toml
[default]
exclude-files = ["*/tests/*", "*/benches/*", "*/examples/*"]
ignored = false
timeout = "600s"
features = "all"

[report]
out = ["Html", "Lcov"]
output-dir = "coverage"
```

## Test Utilities

### Custom Assertions

```rust
// tests/common/assertions.rs

pub trait PromptAssertions {
    fn assert_valid_prompt(&self);
    fn assert_has_argument(&self, name: &str);
    fn assert_renders_with(&self, args: &HashMap<String, String>);
}

impl PromptAssertions for Prompt {
    fn assert_valid_prompt(&self) {
        assert!(!self.name.is_empty(), "Prompt name is empty");
        assert!(!self.title.is_empty(), "Prompt title is empty");
        assert!(is_valid_prompt_name(&self.name), "Invalid prompt name");
    }
    
    fn assert_has_argument(&self, name: &str) {
        assert!(
            self.arguments.iter().any(|a| a.name == name),
            "Prompt missing expected argument: {}", name
        );
    }
    
    fn assert_renders_with(&self, args: &HashMap<String, String>) {
        let result = self.render(args);
        assert!(result.is_ok(), "Failed to render: {:?}", result.err());
        assert!(!result.unwrap().is_empty(), "Rendered output is empty");
    }
}
```

### Test Helpers

```rust
// tests/common/helpers.rs

use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .init();
    });
}

pub fn with_test_env<F>(vars: Vec<(&str, &str)>, test: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let _guards: Vec<_> = vars.into_iter()
        .map(|(k, v)| {
            env::set_var(k, v);
            defer::defer(move || env::remove_var(k))
        })
        .collect();
    
    test();
}

// Usage
#[test]
fn test_with_env_vars() {
    with_test_env(vec![
        ("SWISSARMYHAMMER_DEBUG", "true"),
        ("SWISSARMYHAMMER_PORT", "9999"),
    ], || {
        let config = Config::from_env();
        assert!(config.debug);
        assert_eq!(config.port, 9999);
    });
}
```

## Debugging Tests

### Debug Output

```rust
#[test]
fn test_with_debug_output() {
    init_test_logging();
    
    log::debug!("Starting test");
    
    let result = some_operation();
    
    // Print debug info on failure
    if result.is_err() {
        eprintln!("Operation failed: {:?}", result);
        eprintln!("Current state: {:?}", get_debug_state());
    }
    
    assert!(result.is_ok());
}
```

### Test Isolation

```rust
#[test]
fn test_isolated_state() {
    // Use a unique test ID to avoid conflicts
    let test_id = uuid::Uuid::new_v4();
    let test_dir = temp_dir().join(format!("test-{}", test_id));
    
    // Ensure cleanup even on panic
    let _guard = defer::defer(|| {
        let _ = fs::remove_dir_all(&test_dir);
    });
    
    // Run test with isolated state
    run_test_in_dir(&test_dir);
}
```

## CI Testing

### GitHub Actions Test Matrix

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
        features: ["", "all", "experimental"]
    
    runs-on: ${{ matrix.os }}
    
    steps:
    - uses: actions/checkout@v3
    
    - uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
    
    - name: Test
      run: cargo test --features "${{ matrix.features }}"
      
    - name: Test Examples
      run: cargo test --examples
      
    - name: Doc Tests
      run: cargo test --doc
```

## Best Practices

### 1. Test Organization

- Keep unit tests with the code
- Use integration tests for workflows
- Group related tests
- Share common utilities

### 2. Test Naming

```rust
#[test]
fn test_parse_valid_prompt() { }       // Clear what's being tested

#[test]
fn test_render_with_missing_arg() { }  // Clear expected outcome

#[test]
fn test_concurrent_access_safety() { } // Clear test scenario
```

### 3. Test Independence

- Each test should be independent
- Use temporary directories
- Clean up resources
- Don't rely on test order

### 4. Test Coverage

- Aim for >80% coverage
- Test edge cases
- Test error paths
- Test concurrent scenarios

### 5. Performance

- Keep tests fast (<100ms each)
- Use `#[ignore]` for slow tests
- Run slow tests in CI only
- Mock expensive operations

## Next Steps

- Read [Development Setup](./development.md) for environment setup
- See [Contributing](./contributing.md) for contribution guidelines
- Check [CI/CD](./ci-cd.md) for automated testing
- Review [Benchmarking](./benchmarking.md) for performance testing