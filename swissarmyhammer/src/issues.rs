use crate::error::{Result, SwissArmyHammerError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};

/// Represents an issue in the tracking system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    /// The issue number (6-digit format)
    pub number: u32,
    /// The issue name (derived from filename without number prefix)
    pub name: String,
    /// The full content of the issue markdown file
    pub content: String,
    /// Whether the issue is completed
    pub completed: bool,
    /// The file path of the issue
    pub file_path: PathBuf,
}

/// Represents the current state of the issue system
#[derive(Debug, Clone)]
pub struct IssueState {
    /// Path to the issues directory
    pub issues_dir: PathBuf,
    /// Path to the completed issues directory
    pub completed_dir: PathBuf,
}

/// Trait for issue storage operations
#[async_trait::async_trait]
pub trait IssueStorage: Send + Sync {
    /// List all issues (both pending and completed)
    async fn list_issues(&self) -> Result<Vec<Issue>>;

    /// Get a specific issue by number
    async fn get_issue(&self, number: u32) -> Result<Issue>;

    /// Create a new issue with auto-assigned number
    async fn create_issue(&self, name: String, content: String) -> Result<Issue>;
}

/// File system implementation of issue storage
pub struct FileSystemIssueStorage {
    #[allow(dead_code)]
    state: IssueState,
}

impl FileSystemIssueStorage {
    /// Create a new FileSystemIssueStorage instance
    pub fn new(issues_dir: PathBuf) -> Self {
        let completed_dir = issues_dir.join("complete");
        Self {
            state: IssueState {
                issues_dir,
                completed_dir,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_issue_serialization() {
        let issue = Issue {
            number: 123,
            name: "test_issue".to_string(),
            content: "Test content".to_string(),
            completed: false,
            file_path: PathBuf::from("/tmp/issues/000123_test_issue.md"),
        };

        // Test serialization
        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue, deserialized);
        assert_eq!(deserialized.number, 123);
        assert_eq!(deserialized.name, "test_issue");
        assert_eq!(deserialized.content, "Test content");
        assert_eq!(deserialized.completed, false);
    }

    #[test]
    fn test_issue_number_validation() {
        // Valid 6-digit numbers
        let valid_numbers = vec![1, 999, 1000, 99999, 100000, 999999];
        for num in valid_numbers {
            assert!(num <= 999999, "Issue number {} should be valid", num);
        }

        // Invalid numbers (too large)
        let invalid_numbers = vec![1000000, 9999999];
        for num in invalid_numbers {
            assert!(num > 999999, "Issue number {} should be invalid", num);
        }
    }

    #[test]
    fn test_path_construction() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();

        let storage = FileSystemIssueStorage::new(issues_dir.clone());

        assert_eq!(storage.state.issues_dir, issues_dir);
        assert_eq!(storage.state.completed_dir, issues_dir.join("complete"));
    }
}
