//! Memoranda management and storage system
//!
//! This module provides a comprehensive memoranda (memo) system for storing and managing
//! notes and documents. It integrates with the MCP (Model Context Protocol) to provide
//! AI assistants with persistent memory capabilities.
//!
//! ## Features
//!
//! - **ULID-based IDs**: Strong-typed memo IDs using ULIDs for sortable, unique identifiers
//! - **JSON Serialization**: Full serde support for MCP protocol integration
//! - **Type Safety**: Strong typing to prevent ID confusion with other system components
//! - **Async Support**: Designed for async/await patterns throughout the system
//!
//! ## Basic Usage
//!
//! ```rust
//! use swissarmyhammer::memoranda::{Memo, MemoId, CreateMemoRequest};
//! use chrono::Utc;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new memo ID
//! let memo_id = MemoId::new();
//!
//! // Create a memo
//! let memo = Memo {
//!     id: memo_id,
//!     title: "My First Memo".to_string(),
//!     content: "This is the content of my memo.".to_string(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//! };
//!
//! // Create a request for MCP integration
//! let request = CreateMemoRequest {
//!     title: "New Memo".to_string(),
//!     content: "Content here".to_string(),
//! };
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, SwissArmyHammerError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Type-safe wrapper for memo IDs using ULID to prevent confusion with other IDs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoId(String);

impl MemoId {
    /// Generate a new ULID-based memo ID
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    /// Create a memo ID from an existing ULID string
    pub fn from_string(id: String) -> Result<Self> {
        // Validate that this is a proper ULID
        let _ulid =
            Ulid::from_string(&id).map_err(|_| SwissArmyHammerError::invalid_memo_id(&id))?;
        Ok(Self(id))
    }

    /// Get the raw string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for MemoId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for MemoId {
    type Err = SwissArmyHammerError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_string(s.to_string())
    }
}

/// Core memo structure representing a stored memorandum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memo {
    /// Unique identifier for the memo
    pub id: MemoId,
    /// Title of the memo
    pub title: String,
    /// Content of the memo
    pub content: String,
    /// When the memo was created
    pub created_at: DateTime<Utc>,
    /// When the memo was last updated
    pub updated_at: DateTime<Utc>,
}

impl Memo {
    /// Create a new memo with the current timestamp
    pub fn new(title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: MemoId::new(),
            title,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the memo content and timestamp
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Update the memo title and timestamp
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }
}

/// Request type for creating a new memo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMemoRequest {
    /// Title for the new memo
    pub title: String,
    /// Initial content for the memo
    pub content: String,
}

/// Request type for updating an existing memo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateMemoRequest {
    /// ID of the memo to update
    pub id: MemoId,
    /// New content for the memo
    pub content: String,
}

/// Request type for searching memos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosRequest {
    /// Search query string
    pub query: String,
}

/// Response type containing search results
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosResponse {
    /// List of memos matching the search query
    pub memos: Vec<Memo>,
    /// Number of total results found
    pub total_count: usize,
}

/// Request type for getting a specific memo by ID
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMemoRequest {
    /// ID of the memo to retrieve
    pub id: MemoId,
}

/// Request type for deleting a memo
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteMemoRequest {
    /// ID of the memo to delete
    pub id: MemoId,
}

/// Response type for listing all memos
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMemosResponse {
    /// List of all memos
    pub memos: Vec<Memo>,
    /// Total number of memos
    pub total_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_id_generation() {
        let id1 = MemoId::new();
        let id2 = MemoId::new();

        // IDs should be unique
        assert_ne!(id1, id2);

        // IDs should be valid ULID strings
        assert!(id1.as_str().len() == 26); // ULID length
        assert!(id2.as_str().len() == 26);
    }

    #[test]
    fn test_memo_id_from_string() {
        let ulid = Ulid::new();
        let ulid_string = ulid.to_string();

        let memo_id = MemoId::from_string(ulid_string.clone()).unwrap();
        assert_eq!(memo_id.as_str(), &ulid_string);
    }

    #[test]
    fn test_memo_id_invalid_string() {
        let result = MemoId::from_string("invalid-ulid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_creation() {
        let memo = Memo::new("Test Title".to_string(), "Test Content".to_string());

        assert_eq!(memo.title, "Test Title");
        assert_eq!(memo.content, "Test Content");
        assert!(memo.created_at <= Utc::now());
        assert_eq!(memo.created_at, memo.updated_at);
    }

    #[test]
    fn test_memo_update_content() {
        let mut memo = Memo::new("Title".to_string(), "Original".to_string());
        let original_created_at = memo.created_at;
        let original_updated_at = memo.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        memo.update_content("Updated Content".to_string());

        assert_eq!(memo.content, "Updated Content");
        assert_eq!(memo.created_at, original_created_at); // Should not change
        assert!(memo.updated_at > original_updated_at); // Should be updated
    }

    #[test]
    fn test_memo_update_title() {
        let mut memo = Memo::new("Original Title".to_string(), "Content".to_string());
        let original_created_at = memo.created_at;
        let original_updated_at = memo.updated_at;

        // Small delay to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));

        memo.update_title("New Title".to_string());

        assert_eq!(memo.title, "New Title");
        assert_eq!(memo.created_at, original_created_at); // Should not change
        assert!(memo.updated_at > original_updated_at); // Should be updated
    }

    #[test]
    fn test_memo_serialization() {
        let memo = Memo::new("Test Title".to_string(), "Test Content".to_string());

        // Test JSON serialization
        let json = serde_json::to_string(&memo).unwrap();
        let deserialized: Memo = serde_json::from_str(&json).unwrap();

        assert_eq!(memo, deserialized);
    }

    #[test]
    fn test_request_types_serialization() {
        let create_request = CreateMemoRequest {
            title: "New Memo".to_string(),
            content: "New Content".to_string(),
        };

        let json = serde_json::to_string(&create_request).unwrap();
        let deserialized: CreateMemoRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(create_request, deserialized);
    }

    #[test]
    fn test_search_request_serialization() {
        let search_request = SearchMemosRequest {
            query: "test query".to_string(),
        };

        let json = serde_json::to_string(&search_request).unwrap();
        let deserialized: SearchMemosRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(search_request, deserialized);
    }
}
