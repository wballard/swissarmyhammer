use crate::error::{Result, SwissArmyHammerError};
use crate::memoranda::{Memo, MemoId};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// State configuration for memo storage
///
/// Contains the directory path where memo files are stored.
/// This struct encapsulates the filesystem location for memo persistence.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use swissarmyhammer::memoranda::MemoState;
///
/// let state = MemoState {
///     memos_dir: PathBuf::from("/path/to/memos"),
/// };
/// ```
pub struct MemoState {
    /// The directory where memo files are stored as JSON
    pub memos_dir: PathBuf,
}

/// Trait for memo storage operations
///
/// Defines the interface for memo storage backends, allowing different
/// storage implementations (filesystem, database, etc.) while maintaining
/// a consistent API for memo operations.
///
/// All operations are asynchronous to support high-performance storage backends
/// and concurrent access patterns.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::{MemoStorage, FileSystemMemoStorage, MemoId};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let storage = FileSystemMemoStorage::new_default()?;
///
/// // Create a memo
/// let memo = storage.create_memo(
///     "Meeting Notes".to_string(),
///     "Discussed project roadmap".to_string()
/// ).await?;
///
/// // Retrieve it
/// let retrieved = storage.get_memo(&memo.id).await?;
/// assert_eq!(memo.id, retrieved.id);
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait MemoStorage: Send + Sync {
    /// Create a new memo with the given title and content
    ///
    /// Generates a unique ULID identifier and timestamps automatically.
    /// The memo is persisted to storage before returning.
    ///
    /// # Arguments
    ///
    /// * `title` - The title for the new memo
    /// * `content` - The content for the new memo
    ///
    /// # Returns
    ///
    /// * `Result<Memo>` - The created memo with generated ID and timestamps
    ///
    /// # Errors
    ///
    /// Returns an error if the memo cannot be persisted to storage.
    async fn create_memo(&self, title: String, content: String) -> Result<Memo>;

    /// Retrieve a memo by its unique identifier
    ///
    /// # Arguments
    ///
    /// * `id` - The unique ULID identifier of the memo to retrieve
    ///
    /// # Returns
    ///
    /// * `Result<Memo>` - The memo if found
    ///
    /// # Errors
    ///
    /// Returns `MemoNotFound` error if no memo exists with the given ID.
    async fn get_memo(&self, id: &MemoId) -> Result<Memo>;

    /// Update the content of an existing memo
    ///
    /// Updates the memo's content and refreshes the `updated_at` timestamp.
    /// The title and ID remain unchanged.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the memo to update
    /// * `content` - The new content to replace the existing content
    ///
    /// # Returns
    ///
    /// * `Result<Memo>` - The updated memo with new content and timestamp
    ///
    /// # Errors
    ///
    /// Returns `MemoNotFound` error if no memo exists with the given ID.
    async fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo>;

    /// Delete a memo by its unique identifier
    ///
    /// Permanently removes the memo from storage. This operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the memo to delete
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success if the memo was deleted
    ///
    /// # Errors
    ///
    /// Returns `MemoNotFound` error if no memo exists with the given ID.
    async fn delete_memo(&self, id: &MemoId) -> Result<()>;

    /// List all memos in storage
    ///
    /// Returns all memos currently stored, regardless of creation time or content.
    /// The order of returned memos is implementation-specific.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Memo>>` - All memos in storage, or empty vec if none exist
    ///
    /// # Errors
    ///
    /// Returns an error if the storage backend cannot be accessed.
    async fn list_memos(&self) -> Result<Vec<Memo>>;

    /// Search memos by title and content
    ///
    /// Performs case-insensitive full-text search across memo titles and content.
    /// Returns all memos that contain the query string in either field.
    ///
    /// # Arguments
    ///
    /// * `query` - The search term to match against memo titles and content
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Memo>>` - Memos matching the search query, or empty vec if none match
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn search_example(storage: &impl swissarmyhammer::memoranda::MemoStorage) -> swissarmyhammer::error::Result<()> {
    /// // Search for memos containing "project"
    /// let results = storage.search_memos("project").await?;
    /// println!("Found {} memos containing 'project'", results.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn search_memos(&self, query: &str) -> Result<Vec<Memo>>;
}

/// Filesystem-based implementation of memo storage
///
/// Stores memos as JSON files in a directory structure, with each memo
/// saved as a separate file named by its ULID. Provides atomic operations
/// and concurrent access safety through internal locking.
///
/// # Storage Format
///
/// - Each memo is stored as `{ulid}.json` in the memos directory
/// - JSON files contain the complete memo structure with metadata
/// - Directory is created automatically if it doesn't exist
///
/// # Thread Safety
///
/// Uses internal mutex for creation operations to prevent race conditions
/// when multiple concurrent operations attempt to create memos.
///
/// # Examples
///
/// ```rust
/// use swissarmyhammer::memoranda::FileSystemMemoStorage;
/// use std::path::PathBuf;
///
/// // Use default location (~/.swissarmyhammer/memos)
/// let storage = FileSystemMemoStorage::new_default()?;
///
/// // Use custom location
/// let custom_storage = FileSystemMemoStorage::new(PathBuf::from("/tmp/memos"));
/// # Ok::<(), swissarmyhammer::error::SwissArmyHammerError>(())
/// ```
pub struct FileSystemMemoStorage {
    /// Configuration state including storage directory path
    state: MemoState,
    /// Mutex to ensure thread-safe memo creation and prevent race conditions
    creation_lock: Mutex<()>,
}

impl FileSystemMemoStorage {
    /// Create a new filesystem storage with the default memo directory
    ///
    /// Uses the `SWISSARMYHAMMER_MEMOS_DIR` environment variable if set,
    /// otherwise defaults to `.swissarmyhammer/memos` in the current directory.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - New storage instance or error if directory access fails
    ///
    /// # Environment Variables
    ///
    /// * `SWISSARMYHAMMER_MEMOS_DIR` - Custom directory for memo storage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::FileSystemMemoStorage;
    ///
    /// let storage = FileSystemMemoStorage::new_default()?;
    /// # Ok::<(), swissarmyhammer::error::SwissArmyHammerError>(())
    /// ```
    pub fn new_default() -> Result<Self> {
        let memos_dir = if let Ok(custom_path) = std::env::var("SWISSARMYHAMMER_MEMOS_DIR") {
            PathBuf::from(custom_path)
        } else {
            std::env::current_dir()?
                .join(".swissarmyhammer")
                .join("memos")
        };
        Ok(Self::new(memos_dir))
    }

    /// Create a new filesystem storage with a specific memo directory
    ///
    /// # Arguments
    ///
    /// * `memos_dir` - The directory path where memo files will be stored
    ///
    /// # Returns
    ///
    /// * `Self` - New storage instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use swissarmyhammer::memoranda::FileSystemMemoStorage;
    /// use std::path::PathBuf;
    ///
    /// let storage = FileSystemMemoStorage::new(PathBuf::from("/tmp/my-memos"));
    /// ```
    pub fn new(memos_dir: PathBuf) -> Self {
        Self {
            state: MemoState { memos_dir },
            creation_lock: Mutex::new(()),
        }
    }

    /// Ensure the memo directory exists, creating it if necessary
    ///
    /// Creates the full directory path including any parent directories
    /// that don't exist. This is called automatically before file operations.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or filesystem error
    async fn ensure_directory_exists(&self) -> Result<()> {
        if !self.state.memos_dir.exists() {
            tokio::fs::create_dir_all(&self.state.memos_dir).await?;
        }
        Ok(())
    }

    /// Get the filesystem path for a memo with the given ID
    ///
    /// # Arguments
    ///
    /// * `id` - The memo ID to generate a path for
    ///
    /// # Returns
    ///
    /// * `PathBuf` - The full path where the memo file should be stored
    fn get_memo_path(&self, id: &MemoId) -> PathBuf {
        self.state.memos_dir.join(format!("{}.json", id.as_str()))
    }

    /// Load and deserialize a memo from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path to the memo JSON file
    ///
    /// # Returns
    ///
    /// * `Result<Memo>` - The deserialized memo or error if file cannot be read/parsed
    async fn load_memo_from_file(&self, path: &PathBuf) -> Result<Memo> {
        let content = tokio::fs::read_to_string(path).await?;
        let memo: Memo = serde_json::from_str(&content)?;
        Ok(memo)
    }

    /// Serialize and save a memo to a JSON file
    ///
    /// Creates the directory if it doesn't exist, then writes the memo
    /// as pretty-printed JSON to the appropriate file.
    ///
    /// # Arguments
    ///
    /// * `memo` - The memo to serialize and save
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or error if file cannot be written
    async fn save_memo_to_file(&self, memo: &Memo) -> Result<()> {
        self.ensure_directory_exists().await?;

        let path = self.get_memo_path(&memo.id);
        let content = serde_json::to_string_pretty(memo)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Create a new memo file atomically to prevent race conditions
    ///
    /// Uses `create_new(true)` to ensure the file doesn't already exist,
    /// preventing accidental overwrites of existing memos. This is important
    /// for concurrent operations where multiple threads might attempt to
    /// create memos with the same ID.
    ///
    /// # Arguments
    ///
    /// * `memo` - The memo to create and save
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Success or error if file already exists or cannot be created
    ///
    /// # Errors
    ///
    /// Returns `MemoAlreadyExists` if a memo with the same ID already exists.
    async fn create_memo_file_atomically(&self, memo: &Memo) -> Result<()> {
        self.ensure_directory_exists().await?;

        let path = self.get_memo_path(&memo.id);
        let content = serde_json::to_string_pretty(memo)?;

        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    SwissArmyHammerError::MemoAlreadyExists(memo.id.as_str().to_string())
                } else {
                    SwissArmyHammerError::from(e)
                }
            })?;

        file.write_all(content.as_bytes()).await?;
        file.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl MemoStorage for FileSystemMemoStorage {
    async fn create_memo(&self, title: String, content: String) -> Result<Memo> {
        let _lock = self.creation_lock.lock().await;

        let memo = Memo::new(title, content);
        self.create_memo_file_atomically(&memo).await?;
        Ok(memo)
    }

    async fn get_memo(&self, id: &MemoId) -> Result<Memo> {
        let path = self.get_memo_path(id);
        if !path.exists() {
            return Err(SwissArmyHammerError::MemoNotFound(id.as_str().to_string()));
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
            return Err(SwissArmyHammerError::MemoNotFound(id.as_str().to_string()));
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
                    Err(e) => {
                        tracing::warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to load memo file, skipping"
                        );
                        continue;
                    }
                }
            }
        }

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
        assert_eq!(updated_memo.title, "Update Test");
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

        // Check that all created memos are present, regardless of order
        let memo_ids: std::collections::HashSet<&MemoId> = memos.iter().map(|m| &m.id).collect();
        let expected_ids: std::collections::HashSet<&MemoId> =
            [&memo1.id, &memo2.id, &memo3.id].into_iter().collect();
        assert_eq!(memo_ids, expected_ids);
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

        let rust_results = storage.search_memos("Rust").await.unwrap();
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].title, "Rust Programming");

        let programming_results = storage.search_memos("programming").await.unwrap();
        assert_eq!(programming_results.len(), 2);

        let js_results = storage.search_memos("javascript").await.unwrap();
        assert_eq!(js_results.len(), 1);
        assert_eq!(js_results[0].title, "JavaScript Basics");

        let empty_results = storage.search_memos("nonexistent").await.unwrap();
        assert_eq!(empty_results.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_creation() {
        let (storage, _temp_dir) = create_test_storage();

        let tasks = (0..10).map(|i| {
            let storage_ref = &storage;
            async move {
                storage_ref
                    .create_memo(format!("Title {i}"), format!("Content {i}"))
                    .await
            }
        });

        let results = futures::future::try_join_all(tasks).await.unwrap();
        assert_eq!(results.len(), 10);

        let mut ids: Vec<_> = results.iter().map(|memo| &memo.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 10);
    }
}
