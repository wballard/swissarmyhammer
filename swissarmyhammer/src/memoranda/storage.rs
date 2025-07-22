//! Filesystem storage backend for memoranda
//!
//! This module provides a filesystem-based storage implementation for memoranda,
//! following the same patterns as the issues storage system.

use crate::error::{Result, SwissArmyHammerError};
use crate::memoranda::{Memo, MemoId};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::sync::Mutex;

/// Storage state configuration
pub struct MemoState {
    /// Path to the memos directory
    pub memos_dir: PathBuf,
}

/// Trait for memo storage operations
#[async_trait]
pub trait MemoStorage: Send + Sync {
    /// Create a new memo
    async fn create_memo(&self, title: String, content: String) -> Result<Memo>;

    /// Get a specific memo by ID
    async fn get_memo(&self, id: &MemoId) -> Result<Memo>;

    /// Update an existing memo's content
    async fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo>;

    /// Delete a memo by ID
    async fn delete_memo(&self, id: &MemoId) -> Result<()>;

    /// List all memos
    async fn list_memos(&self) -> Result<Vec<Memo>>;

    /// Search memos by query string (basic string matching)
    async fn search_memos(&self, query: &str) -> Result<Vec<Memo>>;
}

/// Filesystem-based memo storage implementation
pub struct FileSystemMemoStorage {
    state: MemoState,
    /// Mutex to ensure thread-safe memo creation and prevent race conditions
    creation_lock: Mutex<()>,
}

impl FileSystemMemoStorage {
    /// Create new storage with default directory (.swissarmyhammer/memos)
    pub fn new_default() -> Result<Self> {
        let memos_dir = std::env::current_dir()?
            .join(".swissarmyhammer")
            .join("memos");
        Ok(Self::new(memos_dir))
    }

    /// Create new storage with custom directory
    pub fn new(memos_dir: PathBuf) -> Self {
        Self {
            state: MemoState { memos_dir },
            creation_lock: Mutex::new(()),
        }
    }

    /// Ensure the memos directory exists
    async fn ensure_directory_exists(&self) -> Result<()> {
        if !self.state.memos_dir.exists() {
            tokio::fs::create_dir_all(&self.state.memos_dir).await?;
        }
        Ok(())
    }

    /// Get the filepath for a memo
    fn get_memo_path(&self, id: &MemoId) -> PathBuf {
        self.state.memos_dir.join(format!("{}.json", id.as_str()))
    }

    /// Load a memo from disk
    async fn load_memo_from_file(&self, path: &PathBuf) -> Result<Memo> {
        let content = tokio::fs::read_to_string(path).await?;
        let memo: Memo = serde_json::from_str(&content)?;
        Ok(memo)
    }

    /// Save a memo to disk
    async fn save_memo_to_file(&self, memo: &Memo) -> Result<()> {
        self.ensure_directory_exists().await?;

        let path = self.get_memo_path(&memo.id);
        let content = serde_json::to_string_pretty(memo)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}

#[async_trait]
impl MemoStorage for FileSystemMemoStorage {
    async fn create_memo(&self, title: String, content: String) -> Result<Memo> {
        let _lock = self.creation_lock.lock().await;

        let memo = Memo::new(title, content);

        // Check if memo already exists
        let path = self.get_memo_path(&memo.id);
        if path.exists() {
            return Err(SwissArmyHammerError::MemoAlreadyExists(memo.id.to_string()));
        }

        self.save_memo_to_file(&memo).await?;
        Ok(memo)
    }

    async fn get_memo(&self, id: &MemoId) -> Result<Memo> {
        let path = self.get_memo_path(id);
        if !path.exists() {
            return Err(SwissArmyHammerError::MemoNotFound(id.to_string()));
        }

        self.load_memo_from_file(&path).await
    }

    async fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo> {
        let mut memo = self.get_memo(id).await?;
        memo.update_content(content);
        self.save_memo_to_file(&memo).await?;
        Ok(memo)
    }

    async fn delete_memo(&self, id: &MemoId) -> Result<()> {
        let path = self.get_memo_path(id);
        if !path.exists() {
            return Err(SwissArmyHammerError::MemoNotFound(id.to_string()));
        }

        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn list_memos(&self) -> Result<Vec<Memo>> {
        if !self.state.memos_dir.exists() {
            return Ok(Vec::new());
        }

        let mut memos = Vec::new();
        let mut dir_entries = tokio::fs::read_dir(&self.state.memos_dir).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                match self.load_memo_from_file(&path).await {
                    Ok(memo) => memos.push(memo),
                    Err(_) => {
                        // Skip corrupted files, continuing with others
                        continue;
                    }
                }
            }
        }

        // Sort by created_at for consistent ordering
        memos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(memos)
    }

    async fn search_memos(&self, query: &str) -> Result<Vec<Memo>> {
        let all_memos = self.list_memos().await?;
        let query_lower = query.to_lowercase();

        let matching_memos: Vec<Memo> = all_memos
            .into_iter()
            .filter(|memo| {
                memo.title.to_lowercase().contains(&query_lower)
                    || memo.content.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(matching_memos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (FileSystemMemoStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileSystemMemoStorage::new(temp_dir.path().join("memos"));
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_create_memo() {
        let (storage, _temp_dir) = create_test_storage();

        let memo = storage
            .create_memo("Test Title".to_string(), "Test Content".to_string())
            .await
            .unwrap();

        assert_eq!(memo.title, "Test Title");
        assert_eq!(memo.content, "Test Content");
        assert!(!memo.id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_get_memo() {
        let (storage, _temp_dir) = create_test_storage();

        let created_memo = storage
            .create_memo("Get Test".to_string(), "Get Content".to_string())
            .await
            .unwrap();

        let retrieved_memo = storage.get_memo(&created_memo.id).await.unwrap();
        assert_eq!(created_memo, retrieved_memo);
    }

    #[tokio::test]
    async fn test_get_nonexistent_memo() {
        let (storage, _temp_dir) = create_test_storage();

        let fake_id = MemoId::new();
        let result = storage.get_memo(&fake_id).await;

        assert!(result.is_err());
        match result {
            Err(SwissArmyHammerError::MemoNotFound(_)) => {}
            _ => panic!("Expected MemoNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_memo() {
        let (storage, _temp_dir) = create_test_storage();

        let created_memo = storage
            .create_memo("Update Test".to_string(), "Original Content".to_string())
            .await
            .unwrap();

        let updated_memo = storage
            .update_memo(&created_memo.id, "Updated Content".to_string())
            .await
            .unwrap();

        assert_eq!(updated_memo.content, "Updated Content");
        assert_eq!(updated_memo.title, "Update Test"); // Title should remain the same
        assert_ne!(updated_memo.updated_at, created_memo.updated_at);
    }

    #[tokio::test]
    async fn test_delete_memo() {
        let (storage, _temp_dir) = create_test_storage();

        let created_memo = storage
            .create_memo("Delete Test".to_string(), "Delete Content".to_string())
            .await
            .unwrap();

        // Verify memo exists
        storage.get_memo(&created_memo.id).await.unwrap();

        // Delete memo
        storage.delete_memo(&created_memo.id).await.unwrap();

        // Verify memo no longer exists
        let result = storage.get_memo(&created_memo.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_memos() {
        let (storage, _temp_dir) = create_test_storage();

        // Create multiple memos
        let memo1 = storage
            .create_memo("Title 1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        let memo2 = storage
            .create_memo("Title 2".to_string(), "Content 2".to_string())
            .await
            .unwrap();
        let memo3 = storage
            .create_memo("Title 3".to_string(), "Content 3".to_string())
            .await
            .unwrap();

        let memos = storage.list_memos().await.unwrap();
        assert_eq!(memos.len(), 3);

        // Should be sorted by created_at
        assert_eq!(memos[0].id, memo1.id);
        assert_eq!(memos[1].id, memo2.id);
        assert_eq!(memos[2].id, memo3.id);
    }

    #[tokio::test]
    async fn test_list_memos_empty() {
        let (storage, _temp_dir) = create_test_storage();

        let memos = storage.list_memos().await.unwrap();
        assert_eq!(memos.len(), 0);
    }

    #[tokio::test]
    async fn test_search_memos() {
        let (storage, _temp_dir) = create_test_storage();

        // Create memos with different content
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

        // Search by title
        let rust_results = storage.search_memos("Rust").await.unwrap();
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].title, "Rust Programming");

        // Search by content
        let programming_results = storage.search_memos("programming").await.unwrap();
        assert_eq!(programming_results.len(), 2); // Should match "programming" in content

        // Search case insensitive
        let js_results = storage.search_memos("javascript").await.unwrap();
        assert_eq!(js_results.len(), 1);
        assert_eq!(js_results[0].title, "JavaScript Basics");

        // Search with no results
        let empty_results = storage.search_memos("nonexistent").await.unwrap();
        assert_eq!(empty_results.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_creation() {
        let (storage, _temp_dir) = create_test_storage();

        // Create multiple memos concurrently
        let tasks = (0..10).map(|i| {
            let storage_ref = &storage;
            async move {
                storage_ref
                    .create_memo(format!("Title {}", i), format!("Content {}", i))
                    .await
            }
        });

        let results = futures::future::try_join_all(tasks).await.unwrap();
        assert_eq!(results.len(), 10);

        // All should have unique IDs
        let mut ids: Vec<_> = results.iter().map(|memo| &memo.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10); // All IDs should be unique
    }
}
