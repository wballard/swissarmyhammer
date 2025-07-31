//! CLI Performance Benchmarks
//!
//! Benchmarks for detecting performance regressions in CLI-MCP integration.
//! These benchmarks measure the performance of CLI commands that use MCP tools.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

/// Setup function to create a standardized benchmark environment
fn setup_benchmark_environment() -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path().to_path_buf();

    // Create issues directory
    let issues_dir = temp_path.join("issues");
    std::fs::create_dir_all(&issues_dir).expect("Failed to create issues directory");

    // Create sample issues for benchmarking
    for i in 1..=10 {
        std::fs::write(
            issues_dir.join(format!("BENCH_{:03}_issue.md", i)),
            format!(
                r#"# Benchmark Issue {}

This is benchmark issue number {} for performance testing.

## Details
- Priority: Medium
- Type: Performance Test
- Created: 2024-01-01
- Iteration: {}

## Description
This issue exists solely for benchmarking CLI performance.
It contains sufficient content to make operations realistic.
"#,
                i, i, i
            ),
        )
        .expect("Failed to create benchmark issue");
    }

    // Create .swissarmyhammer directory
    let swissarmyhammer_dir = temp_path.join(".swissarmyhammer");
    std::fs::create_dir_all(&swissarmyhammer_dir).expect("Failed to create .swissarmyhammer directory");

    // Create source files for search benchmarking
    let src_dir = temp_path.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src directory");

    for i in 1..=5 {
        std::fs::write(
            src_dir.join(format!("benchmark_{}.rs", i)),
            format!(
                r#"
//! Benchmark source file {}

use std::error::Error;

/// Benchmark function {}
pub fn benchmark_function_{}() -> Result<String, Box<dyn Error>> {{
    println!("Running benchmark function {}", {});
    Ok(format!("Benchmark {} completed", {}))
}}

/// Error handling for benchmark {}
pub fn handle_benchmark_error_{}(error: &str) -> Result<(), String> {{
    eprintln!("Benchmark error {}: {{}}", {}, error);
    Err(format!("Benchmark error {} handled", {}))
}}

/// Performance critical function {}
pub fn performance_critical_{}() {{
    for i in 0..1000 {{
        let _ = i * 2 + 1;
    }}
}}
"#,
                i, i, i, i, i, i, i, i, i, i, i, i
            ),
        )
        .expect("Failed to create benchmark source file");
    }

    // Initialize git repository for issue operations
    Command::new("git")
        .args(["init"])
        .current_dir(&temp_path)
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.name", "Benchmark User"])
        .current_dir(&temp_path)
        .output()
        .expect("Failed to set git user name");

    Command::new("git")
        .args(["config", "user.email", "benchmark@example.com"])
        .current_dir(&temp_path)
        .output()
        .expect("Failed to set git user email");

    (temp_dir, temp_path)
}

/// Benchmark issue operations
fn bench_issue_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("issue_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark issue list command
    group.bench_function("issue_list", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--bin", "swissarmyhammer", "--", "issue", "list"])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run issue list command");
            black_box(output.status.success())
        })
    });

    // Benchmark issue creation
    group.bench_function("issue_create", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "issue",
                    "create",
                    &format!("bench_issue_{}", counter),
                    "--content",
                    "Benchmark issue content",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run issue create command");
            black_box(output.status.success())
        })
    });

    // Benchmark issue show
    group.bench_function("issue_show", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        b.iter(|| {
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "issue",
                    "show",
                    "BENCH_001_issue",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run issue show command");
            black_box(output.status.code())
        })
    });

    group.finish();
}

/// Benchmark memo operations
fn bench_memo_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memo_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark memo creation
    group.bench_function("memo_create", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "memo",
                    "create",
                    &format!("Benchmark Memo {}", counter),
                    "--content",
                    "This is benchmark memo content for performance testing.",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run memo create command");
            black_box(output.status.success())
        })
    });

    // Benchmark memo listing
    group.bench_function("memo_list", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        // Pre-create some memos
        for i in 1..=5 {
            Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "memo",
                    "create",
                    &format!("Pre-created Memo {}", i),
                    "--content",
                    &format!("Content for memo {}", i),
                ])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to pre-create memo");
        }

        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--bin", "swissarmyhammer", "--", "memo", "list"])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run memo list command");
            black_box(output.status.success())
        })
    });

    group.finish();
}

/// Benchmark search operations
fn bench_search_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_operations");
    group.measurement_time(Duration::from_secs(15));

    // Benchmark search indexing
    group.bench_function("search_index", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        b.iter(|| {
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "search",
                    "index",
                    "src/**/*.rs",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run search index command");
            black_box(output.status.success())
        })
    });

    // Benchmark search querying (after indexing)
    group.bench_function("search_query", |b| {
        let (_temp_dir, temp_path) = setup_benchmark_environment();
        
        // Pre-index files
        Command::new("cargo")
            .args([
                "run",
                "--bin",
                "swissarmyhammer",
                "--",
                "search",
                "index",
                "src/**/*.rs",
            ])
            .current_dir(&temp_path)
            .output()
            .expect("Failed to pre-index files");

        b.iter(|| {
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "search",
                    "query",
                    "benchmark function",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run search query command");
            black_box(output.status.success())
        })
    });

    group.finish();
}

/// Benchmark CLI startup time
fn bench_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_startup");
    group.measurement_time(Duration::from_secs(5));

    // Benchmark help command (minimal operation)
    group.bench_function("help_command", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--bin", "swissarmyhammer", "--", "--help"])
                .output()
                .expect("Failed to run help command");
            black_box(output.status.code())
        })
    });

    // Benchmark version command
    group.bench_function("version_command", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--bin", "swissarmyhammer", "--", "--version"])
                .output()
                .expect("Failed to run version command");
            black_box(output.status.code())
        })
    });

    group.finish();
}

/// Benchmark different output formats
fn bench_output_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_formats");
    group.measurement_time(Duration::from_secs(10));

    let (_temp_dir, temp_path) = setup_benchmark_environment();

    let formats = ["table", "json"];
    for format in &formats {
        group.bench_with_input(
            BenchmarkId::new("issue_list_format", format),
            format,
            |b, format| {
                b.iter(|| {
                    let output = Command::new("cargo")
                        .args([
                            "run",
                            "--bin",
                            "swissarmyhammer",
                            "--",
                            "issue",
                            "list",
                            "--format",
                            format,
                        ])
                        .current_dir(black_box(&temp_path))
                        .output()
                        .expect("Failed to run formatted issue list command");
                    black_box(output.status.success())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark with different data sizes
fn bench_data_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_scaling");
    group.measurement_time(Duration::from_secs(15));

    let sizes = [10, 50, 100];
    for size in &sizes {
        group.bench_with_input(
            BenchmarkId::new("issue_list_scaling", size),
            size,
            |b, &size| {
                // Create temporary environment with specified number of issues
                let temp_dir = TempDir::new().expect("Failed to create temp directory");
                let temp_path = temp_dir.path().to_path_buf();
                let issues_dir = temp_path.join("issues");
                std::fs::create_dir_all(&issues_dir).expect("Failed to create issues directory");

                // Create the specified number of issues
                for i in 1..=size {
                    std::fs::write(
                        issues_dir.join(format!("SCALE_{:03}_issue.md", i)),
                        format!(
                            r#"# Scaling Issue {}

This is scaling issue number {} for performance testing with {} total issues.

## Content
This issue contains realistic content to test performance at scale.
The goal is to measure how performance changes with data size.
"#,
                            i, i, size
                        ),
                    )
                    .expect("Failed to create scaling issue");
                }

                b.iter(|| {
                    let output = Command::new("cargo")
                        .args(["run", "--bin", "swissarmyhammer", "--", "issue", "list"])
                        .current_dir(black_box(&temp_path))
                        .output()
                        .expect("Failed to run scaled issue list command");
                    black_box(output.status.success())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(5));

    let (_temp_dir, temp_path) = setup_benchmark_environment();

    // Benchmark non-existent issue error
    group.bench_function("nonexistent_issue_error", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "swissarmyhammer",
                    "--",
                    "issue",
                    "show",
                    "nonexistent_issue",
                ])
                .current_dir(black_box(&temp_path))
                .output()
                .expect("Failed to run nonexistent issue command");
            black_box(output.status.code())
        })
    });

    // Benchmark invalid command error
    group.bench_function("invalid_command_error", |b| {
        b.iter(|| {
            let output = Command::new("cargo")
                .args(["run", "--bin", "swissarmyhammer", "--", "invalid", "command"])
                .output()
                .expect("Failed to run invalid command");
            black_box(output.status.code())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_issue_operations,
    bench_memo_operations,
    bench_search_operations,
    bench_cli_startup,
    bench_output_formats,
    bench_data_scaling,
    bench_error_handling
);
criterion_main!(benches);