use crate::error::{Result, SwissArmyHammerError};
use crate::memoranda::{Memo, MemoId};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub struct MemoState {
    pub memos_dir: PathBuf,
}

#[async_trait]
pub trait MemoStorage: Send + Sync {
    async fn create_memo(&self, title: String, content: String) -> Result<Memo>;

    async fn get_memo(&self, id: &MemoId) -> Result<Memo>;

    async fn update_memo(&self, id: &MemoId, content: String) -> Result<Memo>;

    async fn delete_memo(&self, id: &MemoId) -> Result<()>;

    async fn list_memos(&self) -> Result<Vec<Memo>>;

    async fn search_memos(&self, query: &str) -> Result<Vec<Memo>>;
}

pub struct FileSystemMemoStorage {
    state: MemoState,
    creation_lock: Mutex<()>,
}

impl FileSystemMemoStorage {
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

    pub fn new(memos_dir: PathBuf) -> Self {
        Self {
            state: MemoState { memos_dir },
            creation_lock: Mutex::new(()),
        }
    }

    async fn ensure_directory_exists(&self) -> Result<()> {
        if !self.state.memos_dir.exists() {
            tokio::fs::create_dir_all(&self.state.memos_dir).await?;
        }
        Ok(())
    }

    fn get_memo_path(&self, id: &MemoId) -> PathBuf {
        self.state.memos_dir.join(format!("{}.json", id.as_str()))
    }

    async fn load_memo_from_file(&self, path: &PathBuf) -> Result<Memo> {
        let content = tokio::fs::read_to_string(path).await?;
        let memo: Memo = serde_json::from_str(&content)?;
        Ok(memo)
    }

    async fn save_memo_to_file(&self, memo: &Memo) -> Result<()> {
        self.ensure_directory_exists().await?;

        let path = self.get_memo_path(&memo.id);
        let content = serde_json::to_string_pretty(memo)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

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
        let expected_ids: std::collections::HashSet<&MemoId> = [&memo1.id, &memo2.id, &memo3.id].into_iter().collect();
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
