/// In-memory cache for issue data with TTL and LRU eviction
pub mod cache;
/// Cached storage implementation combining filesystem storage with in-memory cache
pub mod cached_storage;
/// Filesystem-based issue storage implementation
pub mod filesystem;
/// Performance metrics collection and analysis
pub mod metrics;
/// Storage wrapper that collects performance metrics for all operations
pub mod instrumented_storage;

// Re-export main types from the filesystem module
pub use filesystem::{
    Issue, IssueNumber, IssueStorage, FileSystemIssueStorage, IssueState,
    format_issue_number, parse_issue_number, parse_issue_filename,
    create_safe_filename, sanitize_issue_name, validate_issue_name,
    is_issue_file
};

// Export cache types
pub use cache::{IssueCache, CacheEntry, CacheStats};

// Export cached storage types
pub use cached_storage::CachedIssueStorage;

// Export metrics types
pub use metrics::{PerformanceMetrics, Operation, MetricsSnapshot};

// Export instrumented storage types
pub use instrumented_storage::InstrumentedIssueStorage;