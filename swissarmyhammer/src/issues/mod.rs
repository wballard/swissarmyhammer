//! Issue management and tracking system
//!
//! This module provides a comprehensive issue tracking system that stores issues as markdown
//! files in a git repository. It's designed to be lightweight yet powerful, with features
//! like automatic numbering, git integration, and performance monitoring.
//!
//! ## Features
//!
//! - **Markdown-based Storage**: Issues are stored as markdown files with automatic numbering
//! - **Git Integration**: Automatic branch creation and management for issue workflows
//! - **Performance Monitoring**: Built-in metrics collection and caching for large projects
//! - **Flexible Storage**: Multiple storage backends with caching and instrumentation
//!
//! ## Basic Usage
//!
//! ```rust
//! use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new issue storage
//! let storage = FileSystemIssueStorage::new_default()?;
//!
//! // Create an issue
//! let issue = storage.create_issue(
//!     "fix_login_bug".to_string(),
//!     "# Login Bug\n\nUsers cannot log in with special characters.".to_string()
//! ).await?;
//!
//! println!("Created issue #{:06} - {}", issue.number, issue.name);
//!
//! // List all issues
//! let issues = storage.list_issues().await?;
//! println!("Found {} issues", issues.len());
//!
//! // Mark as complete
//! let completed = storage.mark_complete(issue.number.into()).await?;
//! println!("Issue completed and moved to: {}", completed.file_path.display());
//! # Ok(())
//! # }
//! ```
//!
//! ## Issue Lifecycle
//!
//! ```rust
//! use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
//! use swissarmyhammer::git::GitOperations;
//! use swissarmyhammer::config::Config;
//!
//! # async fn workflow_example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileSystemIssueStorage::new_default()?;
//! let config = Config::new();
//! let git_ops = GitOperations::new(&config)?;
//!
//! // 1. Create issue
//! let issue = storage.create_issue("new_feature".to_string(), "# New Feature\n\nDescription".to_string()).await?;
//!
//! // 2. Create work branch
//! let branch_name = git_ops.create_work_branch(&format!("{:06}_{}", issue.number, issue.name))?;
//!
//! // 3. Work on the issue...
//! // 4. Update issue with progress
//! let updated = storage.update_issue(issue.number.into(), "# New Feature\n\nDescription\n\n## Progress\n\nCompleted basic structure".to_string()).await?;
//!
//! // 5. Mark complete
//! let completed = storage.mark_complete(issue.number.into()).await?;
//!
//! // 6. Merge branch
//! git_ops.merge_issue_branch(&format!("{:06}_{}", issue.number, issue.name))?;
//! # Ok(())
//! # }
//! ```

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
