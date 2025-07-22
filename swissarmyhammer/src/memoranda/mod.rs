use crate::error::{Result, SwissArmyHammerError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub mod storage;
pub use storage::{FileSystemMemoStorage, MemoState, MemoStorage};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MemoId(String);

impl MemoId {
    pub fn new() -> Self {
        Self(Ulid::new().to_string())
    }

    pub fn from_string(id: String) -> Result<Self> {
        let _ulid =
            Ulid::from_string(&id).map_err(|_| SwissArmyHammerError::invalid_memo_id(&id))?;
        Ok(Self(id))
    }

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memo {
    pub id: MemoId,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Memo {
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

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateMemoRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateMemoRequest {
    pub id: MemoId,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosRequest {
    pub query: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMemosResponse {
    pub memos: Vec<Memo>,
    pub total_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetMemoRequest {
    pub id: MemoId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteMemoRequest {
    pub id: MemoId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListMemosResponse {
    pub memos: Vec<Memo>,
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
