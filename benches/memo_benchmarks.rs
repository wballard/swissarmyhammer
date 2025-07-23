//! Performance benchmarks for memoranda functionality

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use swissarmyhammer::memoranda::{MarkdownMemoStorage, MemoStorage};
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Create a test storage with temporary directory
fn create_test_storage() -> (MarkdownMemoStorage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = MarkdownMemoStorage::new(temp_dir.path().join("memos"));
    (storage, temp_dir)
}

/// Benchmark basic memo creation
fn bench_memo_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memo_creation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (storage, _temp_dir) = create_test_storage();
                let result = storage
                    .create_memo(
                        "Benchmark Test".to_string(),
                        "This is test content for benchmarking".to_string(),
                    )
                    .await
                    .unwrap();
                black_box(result);
            })
        })
    });
}

/// Benchmark memo retrieval
fn bench_memo_retrieval(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memo_retrieval", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (storage, _temp_dir) = create_test_storage();
                // Create a memo first
                let memo = storage
                    .create_memo(
                        "Test Memo".to_string(),
                        "Content for retrieval test".to_string(),
                    )
                    .await
                    .unwrap();

                // Now retrieve it
                let result = storage.get_memo(&memo.id).await.unwrap();
                black_box(result);
            })
        })
    });
}

/// Benchmark memo search
fn bench_memo_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memo_search", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (storage, _temp_dir) = create_test_storage();
                // Create some test memos
                for i in 0..10 {
                    storage
                        .create_memo(
                            format!("Test Memo {i}"),
                            format!("Content about projects and meetings {i}"),
                        )
                        .await
                        .unwrap();
                }

                // Search for them
                let results = storage.search_memos("projects").await.unwrap();
                black_box(results);
            })
        })
    });
}

/// Benchmark memo listing
fn bench_memo_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memo_list", |b| {
        b.iter(|| {
            rt.block_on(async {
                let (storage, _temp_dir) = create_test_storage();
                // Create some test memos
                for i in 0..10 {
                    storage
                        .create_memo(
                            format!("Test Memo {i}"),
                            format!("Content for listing test {i}"),
                        )
                        .await
                        .unwrap();
                }

                // List them
                let results = storage.list_memos().await.unwrap();
                black_box(results);
            })
        })
    });
}

criterion_group!(
    memo_benches,
    bench_memo_creation,
    bench_memo_retrieval,
    bench_memo_search,
    bench_memo_list
);
criterion_main!(memo_benches);
