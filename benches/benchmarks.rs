use criterion::{black_box, criterion_group, criterion_main, Criterion};
use swissarmyhammer::prompts::{PromptLoader, Prompt};
use swissarmyhammer::template::TemplateEngine;
use std::collections::HashMap;
use std::process::Command;
use std::time::Instant;
use serde_json::Value;

fn benchmark_prompt_loading(c: &mut Criterion) {
    c.bench_function("load all prompts", |b| {
        b.iter(|| {
            let mut loader = PromptLoader::new();
            loader.load_all()
        });
    });
}

fn benchmark_template_processing(c: &mut Criterion) {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, your score is {{score}} and you are {{status}}";
    let mut args = HashMap::new();
    args.insert("name".to_string(), Value::String("World".to_string()));
    args.insert("score".to_string(), Value::Number(42.into()));
    args.insert("status".to_string(), Value::String("active".to_string()));
    
    c.bench_function("template substitution", |b| {
        b.iter(|| {
            engine.process(black_box(template), black_box(&args))
        });
    });
}

fn benchmark_prompt_creation(c: &mut Criterion) {
    c.bench_function("create prompt", |b| {
        b.iter(|| {
            Prompt::new(
                black_box("test".to_string()),
                black_box("This is a test prompt".to_string()),
                black_box("test.md".to_string()),
            )
        });
    });
}

fn benchmark_prompt_storage(c: &mut Criterion) {
    let storage = swissarmyhammer::prompts::PromptStorage::new();
    let prompt = Prompt::new(
        "test".to_string(),
        "Test content".to_string(),
        "test.md".to_string(),
    );
    
    // Insert benchmark
    c.bench_function("prompt storage insert", |b| {
        b.iter(|| {
            storage.insert(black_box("test".to_string()), black_box(prompt.clone()));
        });
    });
    
    // Lookup benchmark
    storage.insert("lookup_test".to_string(), prompt.clone());
    c.bench_function("prompt storage get", |b| {
        b.iter(|| {
            storage.get(black_box("lookup_test"))
        });
    });
}

fn benchmark_template_validation(c: &mut Criterion) {
    let engine = TemplateEngine::new();
    let template = "Hello {{name}}, your score is {{score}}";
    let mut args = HashMap::new();
    args.insert("name".to_string(), Value::String("Test".to_string()));
    args.insert("score".to_string(), Value::Number(100.into()));
    
    let expected_args = vec![
        swissarmyhammer::template::TemplateArgument {
            name: "name".to_string(),
            description: Some("User name".to_string()),
            required: true,
            default_value: None,
        },
        swissarmyhammer::template::TemplateArgument {
            name: "score".to_string(),
            description: Some("User score".to_string()),
            required: true,
            default_value: None,
        },
    ];
    
    c.bench_function("template validation", |b| {
        b.iter(|| {
            engine.process_with_validation(
                black_box(template), 
                black_box(&args),
                black_box(&expected_args)
            )
        });
    });
}

fn benchmark_cli_startup_time(c: &mut Criterion) {
    // Build the CLI binary first in release mode for accurate measurement
    let output = Command::new("cargo")
        .args(["build", "--release", "--bin", "swissarmyhammer"])
        .output()
        .expect("Failed to build release binary");
    
    if !output.status.success() {
        panic!("Failed to build release binary: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Get the binary path
    let binary_path = "./target/release/swissarmyhammer";
    
    c.bench_function("CLI startup time (--help)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _i in 0..iters {
                let _output = Command::new(black_box(binary_path))
                    .arg("--help")
                    .output()
                    .expect("Failed to run CLI binary");
            }
            start.elapsed()
        });
    });
    
    c.bench_function("CLI startup time (list)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _i in 0..iters {
                let _output = Command::new(black_box(binary_path))
                    .arg("list")
                    .output()
                    .expect("Failed to run CLI binary");
            }
            start.elapsed()
        });
    });
}

fn benchmark_cli_vs_other_tools(c: &mut Criterion) {
    // Benchmark against common fast Rust CLI tools for comparison
    let tools = vec![
        ("swissarmyhammer", "./target/release/swissarmyhammer", "--help"),
        ("cargo", "cargo", "--help"),
        ("git", "git", "--help"),
        ("rg", "rg", "--help"),  // ripgrep
        ("fd", "fd", "--help"),  // fd-find
    ];
    
    let mut group = c.benchmark_group("CLI startup comparison");
    
    for (name, binary, args) in tools {
        // Check if tool exists before benchmarking
        if let Ok(_) = Command::new(binary).arg("--version").output() {
            group.bench_function(format!("{} startup", name), |b| {
                b.iter_custom(|iters| {
                    let start = Instant::now();
                    for _i in 0..iters {
                        let _output = Command::new(black_box(binary))
                            .arg(black_box(args))
                            .output();
                    }
                    start.elapsed()
                });
            });
        }
    }
    
    group.finish();
}

fn benchmark_mcp_startup_time(c: &mut Criterion) {
    // Benchmark MCP server startup specifically (simulate real usage)
    let binary_path = "./target/release/swissarmyhammer";
    
    c.bench_function("MCP server startup (serve command)", |b| {
        b.iter_custom(|iters| {
            let start = Instant::now();
            for _i in 0..iters {
                // Use timeout to avoid hanging on serve command
                let _output = Command::new(black_box(binary_path))
                    .arg("serve")
                    .env("TIMEOUT", "1") // Signal quick exit for benchmarking
                    .output();
            }
            start.elapsed()
        });
    });
}

criterion_group!(
    benches, 
    benchmark_prompt_loading,
    benchmark_template_processing,
    benchmark_prompt_creation,
    benchmark_prompt_storage,
    benchmark_template_validation,
    benchmark_cli_startup_time,
    benchmark_cli_vs_other_tools,
    benchmark_mcp_startup_time
);
criterion_main!(benches);