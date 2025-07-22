//! Memoranda system for storing and managing structured text memos
//!
//! This module provides a comprehensive memo management system that stores memos with
//! structured metadata, automatic timestamping, and efficient search capabilities.
//! Memos are identified by ULID (Universally Unique Lexicographically Sortable Identifier)
//! for both uniqueness and natural ordering.
//!
//! ## Features
//!
//! - **ULID-based Identifiers**: Unique, sortable identifiers for efficient ordering and retrieval
//! - **Automatic Timestamps**: Creation and update times tracked automatically
//! - **Full-text Search**: Search across memo titles and content
//! - **Structured Storage**: Filesystem-based storage with atomic operations
//! - **Type-safe API**: Strong typing for memo identifiers and validation
//!
//! ## Basic Usage
//!
//! ```rust
//! use swissarmyhammer::memoranda::{MemoStorage, FileSystemMemoStorage, Memo};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new memo storage
//! let storage = FileSystemMemoStorage::new_default()?;
//!
//! // Store the memo
//! let stored_memo = storage.create_memo(
//!     "Meeting Notes".to_string(),
//!     "Discussed project timeline and deliverables.".to_string()
//! ).await?;
//! println!("Created memo with ID: {}", stored_memo.id);
//!
//! // Search memos
//! let results = storage.search_memos("timeline").await?;
//! println!("Found {} matching memos", results.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Memo Management
//!
//! ```rust
//! use swissarmyhammer::memoranda::{MemoStorage, FileSystemMemoStorage, MemoId};
//!
//! # async fn management_example() -> Result<(), Box<dyn std::error::Error>> {
//! let storage = FileSystemMemoStorage::new_default()?;
//!
//! // List all memos (sorted by creation time, newest first)
//! let all_memos = storage.list_memos().await?;
//!
//! // Get a specific memo by ID
//! if let Some(memo) = all_memos.first() {
//!     let retrieved = storage.get_memo(&memo.id).await?;
//!     println!("Retrieved memo: {}", retrieved.title);
//!
//!     // Update memo content
//!     storage.update_memo(&memo.id, "Updated content".to_string()).await?;
//!
//!     // Delete memo when done
//!     storage.delete_memo(&memo.id).await?;
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, SwissArmyHammerError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Storage backends for memo persistence and retrieval
pub mod storage;
pub use storage::{FileSystemMemoStorage, MemoState, MemoStorage};

/// A unique identifier for memos using ULID (Universally Unique Lexicographically Sortable Identifier)
///
/// ULIDs provide both uniqueness and natural ordering, making them ideal for memo identification
/// and chronological sorting. They are 26 characters long and URL-safe.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::MemoId;
///
/// // Generate a new ID
/// let id = MemoId::new();
/// println!("Generated ID: {}", id);
///
/// // Parse from string
/// let id_str = "01ARZ3NDEKTSV4RRFFQ69G5FAV";
/// let parsed_id = MemoId::from_string(id_str.to_string())?;
/// assert_eq!(parsed_id.as_str(), id_str);
/// # Ok::<(), swissarmyhammer::error::SwissArmyHammerError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoId(String);

impl MemoId {
    /// Create a new unique memo identifier using ULID generation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::MemoId;
    ///
    /// let id1 = MemoId::new();
    /// let id2 = MemoId::new();
    /// assert_ne!(id1, id2); // Each ID is unique
    /// assert_eq!(id1.as_str().len(), 26); // ULID is always 26 characters
    /// ```
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    /// Create a memo ID from a string, validating it's a proper ULID format
    ///
    /// # Arguments
    ///
    /// * `id` - A string that should be a valid ULID
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The memo ID if valid, or an error if the format is invalid
    ///
    /// # Errors
    ///
    /// Returns an error if the provided string is not a valid ULID format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::MemoId;
    ///
    /// // Valid ULID
    /// let valid_id = MemoId::from_string("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string())?;
    /// assert_eq!(valid_id.as_str(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");
    ///
    /// // Invalid ULID will return an error
    /// let invalid_result = MemoId::from_string("not-a-ulid".to_string());
    /// assert!(invalid_result.is_err());
    /// # Ok::<(), swissarmyhammer::error::SwissArmyHammerError>(())
    /// ```
    pub fn from_string(id: String) -> Result<Self> {
        let _ulid = Ulid::from_string(&id).map_err(|_| {
            SwissArmyHammerError::Other(format!(
                "Invalid memo ID format: '{id}'. Expected a valid ULID (26 characters, base32 encoded, case-insensitive). Example: 01ARZ3NDEKTSV4RRFFQ69G5FAV"
            ))
        })?;
        Ok(Self(id))
    }

    /// Get the string representation of the memo ID
    ///
    /// # Returns
    ///
    /// * `&str` - The ULID string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::MemoId;
    ///
    /// let id = MemoId::new();
    /// let id_str = id.as_str();
    /// assert_eq!(id_str.len(), 26);
    /// ```
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

impl AsRef<str> for MemoId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A memo containing structured text with metadata
///
/// Memos are the core data structure of the memoranda system, containing a title,
/// content, and automatic timestamping. Each memo has a unique ULID identifier
/// that allows for natural chronological ordering.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::Memo;
///
/// let memo = Memo::new(
///     "Project Notes".to_string(),
///     "Remember to review the API documentation.".to_string()
/// );
///
/// println!("Created memo '{}' with ID: {}", memo.title, memo.id);
/// assert_eq!(memo.created_at, memo.updated_at); // Initially same timestamp
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memo {
    /// Unique identifier for this memo
    pub id: MemoId,
    /// Brief title or subject of the memo
    pub title: String,
    /// The main content/body of the memo
    pub content: String,
    /// When this memo was first created
    pub created_at: DateTime<Utc>,
    /// When this memo was last modified
    pub updated_at: DateTime<Utc>,
}

impl Memo {
    /// Create a new memo with the given title and content
    ///
    /// Automatically generates a unique ULID identifier and sets creation/update timestamps
    /// to the current time.
    ///
    /// # Arguments
    ///
    /// * `title` - The title or subject of the memo
    /// * `content` - The main content/body of the memo
    ///
    /// # Returns
    ///
    /// * `Self` - A new memo instance with unique ID and current timestamps
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::Memo;
    ///
    /// let memo = Memo::new(
    ///     "Daily Standup".to_string(),
    ///     "Completed feature X, working on bug Y today.".to_string()
    /// );
    ///
    /// assert!(!memo.title.is_empty());
    /// assert!(!memo.content.is_empty());
    /// assert_eq!(memo.created_at, memo.updated_at);
    /// ```
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

    /// Update the content of this memo and refresh the updated timestamp
    ///
    /// # Arguments
    ///
    /// * `content` - The new content to replace the existing content
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::Memo;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut memo = Memo::new("Title".to_string(), "Original content".to_string());
    /// let original_updated_at = memo.updated_at;
    ///
    /// // Small delay to ensure timestamp difference
    /// thread::sleep(Duration::from_millis(1));
    ///
    /// memo.update_content("Updated content".to_string());
    /// assert_eq!(memo.content, "Updated content");
    /// assert!(memo.updated_at > original_updated_at);
    /// ```
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    /// Update the title of this memo and refresh the updated timestamp
    ///
    /// # Arguments
    ///
    /// * `title` - The new title to replace the existing title
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::Memo;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let mut memo = Memo::new("Original Title".to_string(), "Content".to_string());
    /// let original_updated_at = memo.updated_at;
    ///
    /// // Small delay to ensure timestamp difference
    /// thread::sleep(Duration::from_millis(1));
    ///
    /// memo.update_title("New Title".to_string());
    /// assert_eq!(memo.title, "New Title");
    /// assert!(memo.updated_at > original_updated_at);
    /// ```
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }
}

/// Request to create a new memo
///
/// Used by MCP tools and API endpoints to specify the title and content
/// for a new memo. The system will automatically generate a unique ID and timestamps.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::CreateMemoRequest;
///
/// let request = CreateMemoRequest {
///     title: "Meeting Notes".to_string(),
///     content: "Discussed quarterly goals and team structure.".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMemoRequest {
    /// The title for the new memo
    pub title: String,
    /// The content for the new memo
    pub content: String,
}

/// Request to update an existing memo's content
///
/// Used to modify the content of a memo identified by its ULID.
/// The title remains unchanged, and the updated_at timestamp is refreshed automatically.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{UpdateMemoRequest, MemoId};
///
/// let request = UpdateMemoRequest {
///     id: MemoId::new(),
///     content: "Updated content with new information.".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateMemoRequest {
    /// The ID of the memo to update
    pub id: MemoId,
    /// The new content to replace the existing content
    pub content: String,
}

/// Request to search memos by content or title
///
/// Performs full-text search across memo titles and content, returning
/// memos that match the query string.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::SearchMemosRequest;
///
/// let request = SearchMemosRequest {
///     query: "project timeline".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosRequest {
    /// The search query to match against memo titles and content
    pub query: String,
}

/// Response containing search results for memo queries
///
/// Returns memos that match the search criteria along with a count
/// of total matches found.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{SearchMemosResponse, Memo};
///
/// let response = SearchMemosResponse {
///     memos: vec![], // Vec of matching memos
///     total_count: 0,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosResponse {
    /// List of memos that match the search query
    pub memos: Vec<Memo>,
    /// Total number of memos found matching the query
    pub total_count: usize,
}

/// Request to retrieve a specific memo by its ID
///
/// Used to fetch a single memo using its unique ULID identifier.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{GetMemoRequest, MemoId};
///
/// let request = GetMemoRequest {
///     id: MemoId::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMemoRequest {
    /// The unique ID of the memo to retrieve
    pub id: MemoId,
}

/// Request to delete a specific memo by its ID
///
/// Used to permanently remove a memo from storage using its unique ULID identifier.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{DeleteMemoRequest, MemoId};
///
/// let request = DeleteMemoRequest {
///     id: MemoId::new(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteMemoRequest {
    /// The unique ID of the memo to delete
    pub id: MemoId,
}

/// Response containing a list of all memos
///
/// Returns all memos in the system, typically ordered by creation time
/// (newest first), along with the total count.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{ListMemosResponse, Memo};
///
/// let response = ListMemosResponse {
///     memos: vec![], // Vec of all memos
///     total_count: 0,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMemosResponse {
    /// List of all memos in the system
    pub memos: Vec<Memo>,
    /// Total number of memos in the system
    pub total_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_id_generation() {
        let id1 = MemoId::new();
        let id2 = MemoId::new();

        assert_ne!(id1, id2);

        assert!(id1.as_str().len() == 26);
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

        std::thread::sleep(std::time::Duration::from_millis(1));

        memo.update_content("Updated Content".to_string());

        assert_eq!(memo.content, "Updated Content");
        assert_eq!(memo.created_at, original_created_at);
        assert!(memo.updated_at > original_updated_at);
    }

    #[test]
    fn test_memo_update_title() {
        let mut memo = Memo::new("Original Title".to_string(), "Content".to_string());
        let original_created_at = memo.created_at;
        let original_updated_at = memo.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(1));

        memo.update_title("New Title".to_string());

        assert_eq!(memo.title, "New Title");
        assert_eq!(memo.created_at, original_created_at);
        assert!(memo.updated_at > original_updated_at);
    }

    #[test]
    fn test_memo_serialization() {
        let memo = Memo::new("Test Title".to_string(), "Test Content".to_string());

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
