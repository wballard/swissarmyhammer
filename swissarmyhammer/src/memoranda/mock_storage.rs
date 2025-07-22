//! Mock in-memory storage implementation for testing
//!
//! This module provides a `MockMemoStorage` implementation that stores all memos in memory.
//! It's designed for testing other components that depend on `MemoStorage` without requiring
//! actual file system operations.
//!
//! # Features
//!
//! - Full `MemoStorage` trait implementation
//! - In-memory storage using HashMap
//! - Thread-safe with RwLock for concurrent access
//! - Configurable behavior for testing error conditions
//! - Realistic search and context generation functionality
//! - Support for all advanced search options
//!
//! # Usage
//!
//! ```ignore
//! use swissarmyhammer::memoranda::mock_storage::MockMemoStorage;
//! use swissarmyhammer::memoranda::MemoStorage;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let storage = MockMemoStorage::new();
//!     
//!     let memo = storage.create_memo(
//!         "Test Memo".to_string(),
//!         "This is a test memo".to_string()
//!     ).await?;
//!     
//!     let retrieved = storage.get_memo(&memo.id).await?;
//!     assert_eq!(retrieved.title, "Test Memo");
//!     
//!     Ok(())
//! }
//! ```

use crate::error::{Result, SwissArmyHammerError};
use crate::memoranda::{
    AdvancedMemoSearchEngine, ContextOptions, Memo, MemoId, MemoStorage, SearchOptions,
    SearchResult,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock in-memory storage implementation for testing
///
/// This implementation stores all memos in a HashMap in memory, making it suitable
/// for unit tests and integration tests that need predictable, fast storage without
/// file system dependencies.
#[derive(Debug, Clone)]
pub struct MockMemoStorage {
    /// In-memory storage for memos, keyed by memo ID
    storage: Arc<RwLock<HashMap<MemoId, Memo>>>,

    /// Configuration for controlling mock behavior during tests
    config: Arc<RwLock<MockStorageConfig>>,
}

/// Configuration options for controlling mock storage behavior during tests
#[derive(Debug, Clone)]
pub struct MockStorageConfig {
    /// Whether to simulate storage failures for create operations
    pub fail_create: bool,

    /// Whether to simulate storage failures for get operations
    pub fail_get: bool,

    /// Whether to simulate storage failures for update operations
    pub fail_update: bool,

    /// Whether to simulate storage failures for delete operations
    pub fail_delete: bool,

    /// Whether to simulate storage failures for list operations
    pub fail_list: bool,

    /// Whether to simulate storage failures for search operations
    pub fail_search: bool,

    /// Whether to simulate storage failures for context operations
    pub fail_context: bool,

    /// Maximum number of memos to store (for testing capacity limits)
    pub max_memos: Option<usize>,

    /// Simulate slow operations by adding delay (in milliseconds)
    pub operation_delay_ms: Option<u64>,
}

impl Default for MockStorageConfig {
    fn default() -> Self {
        Self {
            fail_create: false,
            fail_get: false,
            fail_update: false,
            fail_delete: false,
            fail_list: false,
            fail_search: false,
            fail_context: false,
            max_memos: None,
            operation_delay_ms: None,
        }
    }
}

impl MockMemoStorage {
    /// Create a new mock storage instance with default configuration
    ///
    /// The storage starts empty and uses default behavior (no simulated failures).
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(MockStorageConfig::default())),
        }
    }

    /// Create a new mock storage instance with custom configuration
    ///
    /// This allows testing specific error conditions and behaviors.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for mock behavior during tests
    pub fn new_with_config(config: MockStorageConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Update the mock storage configuration
    ///
    /// This allows changing behavior during a test, such as enabling failures
    /// after some operations have succeeded.
    ///
    /// # Arguments
    ///
    /// * `config` - New configuration to apply
    pub async fn set_config(&self, config: MockStorageConfig) {
        let mut current_config = self.config.write().await;
        *current_config = config;
    }

    /// Get the current number of stored memos
    ///
    /// This is useful for testing and verification.
    pub async fn memo_count(&self) -> usize {
        self.storage.read().await.len()
    }

    /// Clear all stored memos
    ///
    /// This is useful for test cleanup and resetting state between tests.
    pub async fn clear(&self) {
        self.storage.write().await.clear();
    }

    /// Check if the storage is empty
    pub async fn is_empty(&self) -> bool {
        self.storage.read().await.is_empty()
    }

    /// Get all memo IDs currently stored
    ///
    /// This is useful for testing and debugging.
    pub async fn get_all_memo_ids(&self) -> Vec<MemoId> {
        self.storage.read().await.keys().cloned().collect()
    }

    /// Simulate a delay based on configuration
    async fn simulate_delay(&self) {
        let config = self.config.read().await;
        if let Some(delay_ms) = config.operation_delay_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }

    /// Check if storage is at capacity limit
    async fn check_capacity_limit(&self) -> Result<()> {
        let config = self.config.read().await;
        if let Some(max_memos) = config.max_memos {
            let storage = self.storage.read().await;
            if storage.len() >= max_memos {
                return Err(SwissArmyHammerError::Storage(
                    "Mock storage at capacity limit".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for MockMemoStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl MemoStorage for MockMemoStorage {
    async fn create_memo(&self, title: String, content: String) -> Result<Memo> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_create {
            return Err(SwissArmyHammerError::Storage(
                "Simulated create failure".to_string(),
            ));
        }
        drop(config); // Release lock early

        // Check capacity limit
        self.check_capacity_limit().await?;

        // Create new memo
        let memo = Memo::new(title, content);

        // Store in memory
        let mut storage = self.storage.write().await;
        storage.insert(memo.id.clone(), memo.clone());
        drop(storage);

        Ok(memo)
    }

    async fn get_memo(&self, id: &MemoId) -> Result<Memo> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_get {
            return Err(SwissArmyHammerError::Storage(
                "Simulated get failure".to_string(),
            ));
        }
        drop(config);

        // Retrieve from storage
        let storage = self.storage.read().await;
        match storage.get(id) {
            Some(memo) => Ok(memo.clone()),
            None => Err(SwissArmyHammerError::MemoNotFound(id.as_str().to_string())),
        }
    }

    async fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_update {
            return Err(SwissArmyHammerError::Storage(
                "Simulated update failure".to_string(),
            ));
        }
        drop(config);

        // Update in storage
        let mut storage = self.storage.write().await;
        match storage.get_mut(id) {
            Some(memo) => {
                memo.content = content;
                memo.updated_at = chrono::Utc::now();
                Ok(memo.clone())
            }
            None => Err(SwissArmyHammerError::MemoNotFound(id.as_str().to_string())),
        }
    }

    async fn delete_memo(&self, id: &MemoId) -> Result<()> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_delete {
            return Err(SwissArmyHammerError::Storage(
                "Simulated delete failure".to_string(),
            ));
        }
        drop(config);

        // Remove from storage
        let mut storage = self.storage.write().await;
        match storage.remove(id) {
            Some(_) => Ok(()),
            None => Err(SwissArmyHammerError::MemoNotFound(id.as_str().to_string())),
        }
    }

    async fn list_memos(&self) -> Result<Vec<Memo>> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_list {
            return Err(SwissArmyHammerError::Storage(
                "Simulated list failure".to_string(),
            ));
        }
        drop(config);

        // Return all memos sorted by creation time (newest first)
        let storage = self.storage.read().await;
        let mut memos: Vec<Memo> = storage.values().cloned().collect();
        memos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(memos)
    }

    async fn search_memos(&self, query: &str) -> Result<Vec<Memo>> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_search {
            return Err(SwissArmyHammerError::Storage(
                "Simulated search failure".to_string(),
            ));
        }
        drop(config);

        // Simple case-insensitive substring search
        let storage = self.storage.read().await;
        let query_lower = query.to_lowercase();

        let mut matching_memos: Vec<Memo> = storage
            .values()
            .filter(|memo| {
                memo.title.to_lowercase().contains(&query_lower)
                    || memo.content.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();

        // Sort by creation time (newest first)
        matching_memos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(matching_memos)
    }

    async fn search_memos_advanced(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_search {
            return Err(SwissArmyHammerError::Storage(
                "Simulated advanced search failure".to_string(),
            ));
        }
        drop(config);

        // For mock implementation, use in-memory search engine
        let search_engine = AdvancedMemoSearchEngine::new_in_memory().await?;

        // Get all memos and index them
        let all_memos = self.list_memos().await?;
        if !all_memos.is_empty() {
            search_engine.index_memos(&all_memos).await?;
        }

        // Perform search
        let results = search_engine.search(query, options, &all_memos).await?;
        Ok(results)
    }

    async fn get_all_context(&self, options: &ContextOptions) -> Result<String> {
        self.simulate_delay().await;

        // Check for simulated failure
        let config = self.config.read().await;
        if config.fail_context {
            return Err(SwissArmyHammerError::Storage(
                "Simulated context failure".to_string(),
            ));
        }
        drop(config);

        // Get all memos sorted by creation time (newest first)
        let memos = self.list_memos().await?;

        if memos.is_empty() {
            return Ok(String::new());
        }

        let mut context_parts = Vec::new();
        let mut total_tokens = 0;

        for memo in memos {
            let mut memo_context = String::new();

            if options.include_metadata {
                memo_context.push_str(&format!("# {} (ID: {})\n\n", memo.title, memo.id.as_str()));
                memo_context.push_str(&format!(
                    "Created: {}\n",
                    memo.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));
                memo_context.push_str(&format!(
                    "Updated: {}\n\n",
                    memo.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
                ));
            }

            memo_context.push_str(&memo.content);

            // Simple token counting (approximate)
            let memo_tokens = memo_context.split_whitespace().count();

            // Check token limit
            if let Some(max_tokens) = options.max_tokens {
                if total_tokens + memo_tokens > max_tokens {
                    // Truncate this memo's content to fit within limit
                    let remaining_tokens = max_tokens - total_tokens;
                    if remaining_tokens > 0 {
                        let words: Vec<&str> = memo_context.split_whitespace().collect();
                        let truncated_words = &words[..remaining_tokens.min(words.len())];
                        context_parts.push(truncated_words.join(" "));
                    }
                    break;
                }
            }

            context_parts.push(memo_context);
            total_tokens += memo_tokens;
        }

        Ok(context_parts.join(&format!("\n{}\n", options.delimiter)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_storage_basic_operations() {
        let storage = MockMemoStorage::new();

        // Test create
        let memo = storage
            .create_memo("Test Title".to_string(), "Test Content".to_string())
            .await
            .unwrap();

        assert_eq!(memo.title, "Test Title");
        assert_eq!(memo.content, "Test Content");
        assert_eq!(storage.memo_count().await, 1);

        // Test get
        let retrieved = storage.get_memo(&memo.id).await.unwrap();
        assert_eq!(retrieved, memo);

        // Test update
        let updated = storage
            .update_memo(&memo.id, "Updated Content".to_string())
            .await
            .unwrap();
        assert_eq!(updated.content, "Updated Content");
        assert_ne!(updated.updated_at, memo.updated_at);

        // Test list
        let memos = storage.list_memos().await.unwrap();
        assert_eq!(memos.len(), 1);
        assert_eq!(memos[0].content, "Updated Content");

        // Test delete
        storage.delete_memo(&memo.id).await.unwrap();
        assert_eq!(storage.memo_count().await, 0);

        // Test get after delete
        let result = storage.get_memo(&memo.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_storage_search_functionality() {
        let storage = MockMemoStorage::new();

        // Create test memos
        storage
            .create_memo(
                "Rust Programming".to_string(),
                "Learning Rust language".to_string(),
            )
            .await
            .unwrap();
        storage
            .create_memo(
                "Python Guide".to_string(),
                "Python programming tutorial".to_string(),
            )
            .await
            .unwrap();
        storage
            .create_memo(
                "JavaScript Basics".to_string(),
                "Introduction to JS".to_string(),
            )
            .await
            .unwrap();

        // Test basic search
        let rust_results = storage.search_memos("Rust").await.unwrap();
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].title, "Rust Programming");

        let programming_results = storage.search_memos("programming").await.unwrap();
        assert_eq!(programming_results.len(), 2);

        // Test advanced search
        let options = SearchOptions::default();
        let advanced_results = storage
            .search_memos_advanced("rust", &options)
            .await
            .unwrap();
        assert_eq!(advanced_results.len(), 1);
        assert!(advanced_results[0].relevance_score > 0.0);
    }

    #[tokio::test]
    async fn test_mock_storage_context_generation() {
        let storage = MockMemoStorage::new();

        // Test empty context
        let empty_context = storage
            .get_all_context(&ContextOptions::default())
            .await
            .unwrap();
        assert!(empty_context.is_empty());

        // Create memos
        storage
            .create_memo("First".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        storage
            .create_memo("Second".to_string(), "Content 2".to_string())
            .await
            .unwrap();

        // Test context generation
        let context = storage
            .get_all_context(&ContextOptions::default())
            .await
            .unwrap();

        assert!(context.contains("First"));
        assert!(context.contains("Second"));
        assert!(context.contains("Content 1"));
        assert!(context.contains("Content 2"));
        assert!(context.contains("---")); // Default delimiter
    }

    #[tokio::test]
    async fn test_mock_storage_error_simulation() {
        let config = MockStorageConfig {
            fail_create: true,
            ..Default::default()
        };
        let storage = MockMemoStorage::new_with_config(config);

        // Test simulated create failure
        let result = storage
            .create_memo("Test".to_string(), "Content".to_string())
            .await;
        assert!(result.is_err());

        // Change config to allow create but fail get
        let new_config = MockStorageConfig {
            fail_create: false,
            fail_get: true,
            ..Default::default()
        };
        storage.set_config(new_config).await;

        // Create should work now
        let memo = storage
            .create_memo("Test".to_string(), "Content".to_string())
            .await
            .unwrap();

        // Get should fail
        let result = storage.get_memo(&memo.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_storage_capacity_limit() {
        let config = MockStorageConfig {
            max_memos: Some(2),
            ..Default::default()
        };
        let storage = MockMemoStorage::new_with_config(config);

        // Create up to capacity
        storage
            .create_memo("First".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        storage
            .create_memo("Second".to_string(), "Content 2".to_string())
            .await
            .unwrap();

        // Third should fail
        let result = storage
            .create_memo("Third".to_string(), "Content 3".to_string())
            .await;
        assert!(result.is_err());
        assert_eq!(storage.memo_count().await, 2);
    }

    #[tokio::test]
    async fn test_mock_storage_utility_methods() {
        let storage = MockMemoStorage::new();

        assert!(storage.is_empty().await);
        assert_eq!(storage.memo_count().await, 0);
        assert_eq!(storage.get_all_memo_ids().await.len(), 0);

        let memo = storage
            .create_memo("Test".to_string(), "Content".to_string())
            .await
            .unwrap();

        assert!(!storage.is_empty().await);
        assert_eq!(storage.memo_count().await, 1);
        assert_eq!(storage.get_all_memo_ids().await, vec![memo.id]);

        storage.clear().await;

        assert!(storage.is_empty().await);
        assert_eq!(storage.memo_count().await, 0);
    }
}
