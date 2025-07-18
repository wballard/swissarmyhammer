/// In-memory cache for issue data with TTL and LRU eviction
pub mod cache;
/// Cached storage implementation combining filesystem storage with in-memory cache
pub mod cached_storage;
/// Filesystem-based issue storage implementation
pub mod filesystem;
/// Storage wrapper that collects performance metrics for all operations
pub mod instrumented_storage;
/// Performance metrics collection and analysis
pub mod metrics;

// Re-export main types from the filesystem module
pub use filesystem::{
    create_safe_filename, format_issue_number, is_issue_file, parse_issue_filename,
    parse_issue_number, sanitize_issue_name, validate_issue_name, FileSystemIssueStorage, Issue,
    IssueNumber, IssueState, IssueStorage,
};

// Export cache types
pub use cache::{CacheEntry, CacheStats, IssueCache};

// Export cached storage types
pub use cached_storage::CachedIssueStorage;

// Export metrics types
pub use metrics::{MetricsSnapshot, Operation, PerformanceMetrics};

// Export instrumented storage types
pub use instrumented_storage::InstrumentedIssueStorage;
