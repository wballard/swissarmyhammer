pub mod cache;
pub mod cached_storage;
pub mod filesystem;
pub mod metrics;
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