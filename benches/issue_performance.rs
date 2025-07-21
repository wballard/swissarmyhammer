use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;
use tokio::time::Duration;

use swissarmyhammer::issues::{
    CachedIssueStorage, FileSystemIssueStorage, InstrumentedIssueStorage, Issue, IssueCache,
    IssueNumber, IssueStorage, Operation, PerformanceMetrics,
};
use swissarmyhammer::mcp::types::IssueName;

fn create_test_issue(number: u32, name: &str) -> Issue {
    Issue {
        number: IssueNumber::new(number).unwrap(),
        name: IssueName::from_filesystem(name.to_string()).unwrap(),
        content: format!("Test content for issue {number}"),
        completed: false,
        file_path: PathBuf::from(format!("test_{number}.md")),
        created_at: Utc::now(),
    }
}

fn create_test_issues(count: usize) -> Vec<Issue> {
    (1..=count)
        .map(|i| create_test_issue(i as u32, &format!("test_issue_{i}")))
        .collect()
}

fn setup_fs_storage() -> (FileSystemIssueStorage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let issues_dir = temp_dir.path().join("issues");
    let storage = FileSystemIssueStorage::new(issues_dir).unwrap();
    (storage, temp_dir)
}

fn benchmark_cache_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_operations");

    // Test different cache sizes
    let cache_sizes = vec![100, 1000, 10000];
    let issue_counts = vec![10, 100, 1000];

    for cache_size in cache_sizes {
        for issue_count in &issue_counts {
            let cache = IssueCache::new(Duration::from_secs(300), cache_size);
            let issues = create_test_issues(*issue_count);

            group.bench_with_input(
                BenchmarkId::new(
                    "cache_put",
                    format!("cache{cache_size}_issues{issue_count}"),
                ),
                &(*issue_count, cache_size),
                |b, _| {
                    b.iter(|| {
                        for issue in &issues {
                            cache.put(black_box(issue.clone()));
                        }
                    });
                },
            );

            // Pre-populate cache for get benchmarks
            for issue in &issues {
                cache.put(issue.clone());
            }

            group.bench_with_input(
                BenchmarkId::new(
                    "cache_get_hit",
                    format!("cache{cache_size}_issues{issue_count}"),
                ),
                &(*issue_count, cache_size),
                |b, _| {
                    b.iter(|| {
                        for i in 1..=*issue_count {
                            let _issue = cache.get(black_box(i as u32));
                        }
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(
                    "cache_get_miss",
                    format!("cache{cache_size}_issues{issue_count}"),
                ),
                &(*issue_count, cache_size),
                |b, _| {
                    b.iter(|| {
                        for i in (*issue_count + 1)..=(*issue_count * 2) {
                            let _issue = cache.get(black_box(i as u32));
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_cache_ttl_behavior(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_ttl_behavior");

    let ttl_values = vec![
        Duration::from_millis(10),
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(10),
    ];

    for ttl in ttl_values {
        let cache = IssueCache::new(ttl, 1000);
        let issues = create_test_issues(100);

        group.bench_with_input(
            BenchmarkId::new("cache_with_ttl", format!("{}ms", ttl.as_millis())),
            &ttl,
            |b, _| {
                b.iter(|| {
                    // Put issues
                    for issue in &issues {
                        cache.put(black_box(issue.clone()));
                    }

                    // Get issues immediately (should hit)
                    for i in 1..=100 {
                        let _issue = cache.get(black_box(i as u32));
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_cache_lru_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_lru_eviction");

    let cache_sizes = [10, 50, 100];
    let issue_counts = [20, 100, 200]; // More issues than cache size to force eviction

    for (cache_size, issue_count) in cache_sizes.iter().zip(issue_counts.iter()) {
        let cache = IssueCache::new(Duration::from_secs(300), *cache_size);
        let issues = create_test_issues(*issue_count);

        group.bench_with_input(
            BenchmarkId::new(
                "lru_eviction",
                format!("cache{cache_size}_issues{issue_count}"),
            ),
            &(*cache_size, *issue_count),
            |b, _| {
                b.iter(|| {
                    // Fill cache beyond capacity to trigger eviction
                    for issue in &issues {
                        cache.put(black_box(issue.clone()));
                    }

                    // Access some issues to test LRU behavior
                    for i in 1..=(*cache_size / 2) {
                        let _issue = cache.get(black_box(i as u32));
                    }
                });
            },
        );
    }

    group.finish();
}

fn benchmark_cached_storage_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cached_storage_operations");
    let rt = Runtime::new().unwrap();

    let operation_counts = vec![10, 50, 100];

    for count in operation_counts {
        group.bench_with_input(
            BenchmarkId::new("cached_create", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let (fs_storage, _temp) = setup_fs_storage();
                        let cached_storage = CachedIssueStorage::new(Box::new(fs_storage));

                        for i in 1..=count {
                            let _issue = cached_storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("cached_get_performance", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let (fs_storage, _temp) = setup_fs_storage();
                        let cached_storage = CachedIssueStorage::new(Box::new(fs_storage));

                        // Pre-create issues
                        for i in 1..=count {
                            let _issue = cached_storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }

                        // First access - cache miss
                        for i in 1..=count {
                            let issue_name =
                                IssueName::from_filesystem(format!("test_{i}")).unwrap();
                            let _issue = cached_storage.get_issue(&issue_name).await.unwrap();
                        }

                        // Second access - cache hit
                        for i in 1..=count {
                            let issue_name =
                                IssueName::from_filesystem(format!("test_{i}")).unwrap();
                            let _issue = cached_storage.get_issue(&issue_name).await.unwrap();
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    let rt = Runtime::new().unwrap();

    let batch_sizes = vec![5, 10, 20, 50];

    for batch_size in batch_sizes {
        // Compare individual operations vs batch operations
        group.bench_with_input(
            BenchmarkId::new("individual_creates", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (storage, _temp) = setup_fs_storage();

                        for i in 1..=batch_size {
                            let _issue = storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("batch_creates", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (storage, _temp) = setup_fs_storage();

                        let batch_data: Vec<(String, String)> = (1..=batch_size)
                            .map(|i| (format!("test_{i}"), format!("Content {i}")))
                            .collect();

                        let _issues = storage.create_issues_batch(batch_data).await.unwrap();
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("individual_gets", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (storage, _temp) = setup_fs_storage();

                        // Pre-create issues
                        for i in 1..=batch_size {
                            let _issue = storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }

                        // Individual gets
                        for i in 1..=batch_size {
                            let issue_name =
                                IssueName::from_filesystem(format!("test_{i}")).unwrap();
                            let _issue = storage.get_issue(&issue_name).await.unwrap();
                        }
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("batch_gets", batch_size),
            &batch_size,
            |b, &batch_size| {
                b.iter(|| {
                    rt.block_on(async {
                        let (storage, _temp) = setup_fs_storage();

                        // Pre-create issues
                        for i in 1..=batch_size {
                            let _issue = storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }

                        // Batch get
                        let names: Vec<IssueName> = (1..=batch_size)
                            .map(|i| IssueName::from_filesystem(format!("test_{i}")).unwrap())
                            .collect();
                        let name_refs: Vec<&IssueName> = names.iter().collect();
                        let _issues = storage.get_issues_batch(name_refs).await.unwrap();
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_metrics_collection(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_collection");

    let operation_counts = vec![100, 1000, 10000];

    for count in operation_counts {
        group.bench_with_input(
            BenchmarkId::new("metrics_recording", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    let metrics = PerformanceMetrics::new();

                    for i in 0..count {
                        let operation = match i % 5 {
                            0 => Operation::Create,
                            1 => Operation::Read,
                            2 => Operation::Update,
                            3 => Operation::Delete,
                            _ => Operation::List,
                        };

                        metrics.record_operation(
                            black_box(operation),
                            black_box(Duration::from_micros(100 + (i % 1000) as u64)),
                        );
                    }

                    let _stats = metrics.get_stats();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_instrumented_storage(c: &mut Criterion) {
    let mut group = c.benchmark_group("instrumented_storage");
    let rt = Runtime::new().unwrap();

    let operation_counts = vec![10, 50, 100];

    for count in operation_counts {
        group.bench_with_input(
            BenchmarkId::new("instrumented_operations", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let (fs_storage, _temp) = setup_fs_storage();
                        let instrumented_storage =
                            InstrumentedIssueStorage::new(Box::new(fs_storage));

                        // Create issues
                        for i in 1..=count {
                            let _issue = instrumented_storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }

                        // Read issues
                        for i in 1..=count {
                            let issue_name =
                                IssueName::from_filesystem(format!("test_{i}")).unwrap();
                            let _issue = instrumented_storage.get_issue(&issue_name).await.unwrap();
                        }

                        // Update issues
                        for i in 1..=count {
                            let issue_name =
                                IssueName::from_filesystem(format!("test_{i}")).unwrap();
                            let _issue = instrumented_storage
                                .update_issue(&issue_name, format!("Updated content {i}"))
                                .await
                                .unwrap();
                        }

                        // List issues
                        let _issues = instrumented_storage.list_issues().await.unwrap();

                        // Get metrics
                        let _snapshot = instrumented_storage.get_metrics_snapshot();
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_combined_cache_and_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_cache_metrics");
    let rt = Runtime::new().unwrap();

    let operation_counts = vec![10, 50, 100];

    for count in operation_counts {
        group.bench_with_input(
            BenchmarkId::new("cache_plus_metrics", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let (fs_storage, _temp) = setup_fs_storage();
                        let cached_storage = CachedIssueStorage::new(Box::new(fs_storage));
                        let instrumented_cached_storage =
                            InstrumentedIssueStorage::new(Box::new(cached_storage));

                        // Create issues
                        for i in 1..=count {
                            let _issue = instrumented_cached_storage
                                .create_issue(format!("test_{i}"), format!("Content {i}"))
                                .await
                                .unwrap();
                        }

                        // Read issues multiple times to test cache performance
                        for _ in 0..3 {
                            for i in 1..=count {
                                let issue_name =
                                    IssueName::from_filesystem(format!("test_{i}")).unwrap();
                                let _issue = instrumented_cached_storage
                                    .get_issue(&issue_name)
                                    .await
                                    .unwrap();
                            }
                        }

                        // Get metrics
                        let _snapshot = instrumented_cached_storage.get_metrics_snapshot();
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    let concurrent_counts = vec![2, 5, 10];

    for concurrent in concurrent_counts {
        group.bench_with_input(
            BenchmarkId::new("concurrent_cache_access", concurrent),
            &concurrent,
            |b, &concurrent| {
                b.iter(|| {
                    rt.block_on(async {
                        let cache = Arc::new(IssueCache::new(Duration::from_secs(300), 1000));
                        let issues = create_test_issues(100);

                        // Pre-populate cache
                        for issue in &issues {
                            cache.put(issue.clone());
                        }

                        // Concurrent access
                        let mut handles = Vec::new();
                        for _ in 0..concurrent {
                            let cache_clone = cache.clone();
                            let handle = tokio::spawn(async move {
                                for i in 1..=100 {
                                    let _issue = cache_clone.get(i as u32);
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("concurrent_metrics_recording", concurrent),
            &concurrent,
            |b, &concurrent| {
                b.iter(|| {
                    rt.block_on(async {
                        let metrics = Arc::new(PerformanceMetrics::new());

                        let mut handles = Vec::new();
                        for _ in 0..concurrent {
                            let metrics_clone = metrics.clone();
                            let handle = tokio::spawn(async move {
                                for i in 0..100 {
                                    let operation = match i % 5 {
                                        0 => Operation::Create,
                                        1 => Operation::Read,
                                        2 => Operation::Update,
                                        3 => Operation::Delete,
                                        _ => Operation::List,
                                    };

                                    metrics_clone.record_operation(
                                        operation,
                                        Duration::from_micros(100 + (i % 1000) as u64),
                                    );
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }

                        let _stats = metrics.get_stats();
                    });
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    issue_performance_benches,
    benchmark_cache_operations,
    benchmark_cache_ttl_behavior,
    benchmark_cache_lru_eviction,
    benchmark_cached_storage_operations,
    benchmark_batch_operations,
    benchmark_metrics_collection,
    benchmark_instrumented_storage,
    benchmark_combined_cache_and_metrics,
    benchmark_concurrent_operations
);

criterion_main!(issue_performance_benches);
