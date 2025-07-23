//! Core data structures for semantic search functionality

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Unique identifier for indexed files
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub String);

/// Hash of file content for change detection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentHash(pub String);

/// Programming language detected for a file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// Rust programming language
    Rust,
    /// Python programming language
    Python,
    /// TypeScript programming language
    TypeScript,
    /// JavaScript programming language
    JavaScript,
    /// Dart programming language
    Dart,
    /// Unknown or unsupported language
    Unknown,
}

/// Type of code chunk based on TreeSitter parsing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkType {
    /// Function definition
    Function,
    /// Class definition
    Class,
    /// Module or namespace
    Module,
    /// Import or include statement
    Import,
    /// Plain text for files that fail TreeSitter parsing
    PlainText,
}

/// A chunk of code extracted from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    /// Unique identifier for this chunk
    pub id: String,
    /// Path to the source file
    pub file_path: PathBuf,
    /// Programming language of the file
    pub language: Language,
    /// The actual code content
    pub content: String,
    /// Line number where this chunk starts
    pub start_line: usize,
    /// Line number where this chunk ends
    pub end_line: usize,
    /// Type of code chunk
    pub chunk_type: ChunkType,
    /// Hash of the content for change detection
    pub content_hash: ContentHash,
}

/// Vector embedding for a code chunk
#[derive(Debug, Clone)]
pub struct Embedding {
    /// ID of the chunk this embedding represents
    pub chunk_id: String,
    /// 384-dimensional vector for nomic-embed-code
    pub vector: Vec<f32>,
}

/// Indexed file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    /// Unique identifier for the file
    pub file_id: FileId,
    /// Path to the file
    pub path: PathBuf,
    /// Detected programming language
    pub language: Language,
    /// Hash of the file content
    pub content_hash: ContentHash,
    /// Number of chunks extracted from this file
    pub chunk_count: usize,
    /// Timestamp when the file was indexed
    pub indexed_at: DateTime<Utc>,
}

/// Search result with similarity score
#[derive(Debug, Clone)]
pub struct SemanticSearchResult {
    /// The matching code chunk
    pub chunk: CodeChunk,
    /// Similarity score (0.0 to 1.0)
    pub similarity_score: f32,
    /// Excerpt of the matching content
    pub excerpt: String,
}

/// Search query parameters
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// The search text
    pub text: String,
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity threshold
    pub similarity_threshold: f32,
    /// Optional language filter
    pub language_filter: Option<Language>,
}

/// Configuration for the semantic search system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConfig {
    /// Path to the DuckDB database file
    pub database_path: PathBuf,
    /// Embedding model name
    pub embedding_model: String,
    /// Maximum chunk size in characters
    pub chunk_size: usize,
    /// Chunk overlap in characters
    pub chunk_overlap: usize,
    /// Minimum similarity threshold for search results
    pub similarity_threshold: f32,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from(".swissarmyhammer/semantic.db"),
            embedding_model: "nomic-ai/nomic-embed-code".to_string(),
            chunk_size: 512,
            chunk_overlap: 64,
            similarity_threshold: 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_id() {
        let id1 = FileId("test_file".to_string());
        let id2 = FileId("test_file".to_string());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_content_hash() {
        let hash1 = ContentHash("abc123".to_string());
        let hash2 = ContentHash("abc123".to_string());
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_language_serialization() {
        let lang = Language::Rust;
        let serialized = serde_json::to_string(&lang).unwrap();
        let deserialized: Language = serde_json::from_str(&serialized).unwrap();
        assert_eq!(lang, deserialized);
    }

    #[test]
    fn test_chunk_type_serialization() {
        let chunk_type = ChunkType::Function;
        let serialized = serde_json::to_string(&chunk_type).unwrap();
        let deserialized: ChunkType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(chunk_type, deserialized);
    }

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk {
            id: "test_chunk".to_string(),
            file_path: PathBuf::from("test.rs"),
            language: Language::Rust,
            content: "fn main() {}".to_string(),
            start_line: 1,
            end_line: 1,
            chunk_type: ChunkType::Function,
            content_hash: ContentHash("hash123".to_string()),
        };
        
        assert_eq!(chunk.id, "test_chunk");
        assert_eq!(chunk.language, Language::Rust);
        assert_eq!(chunk.chunk_type, ChunkType::Function);
    }

    #[test]
    fn test_embedding_creation() {
        let embedding = Embedding {
            chunk_id: "test_chunk".to_string(),
            vector: vec![0.1, 0.2, 0.3],
        };
        
        assert_eq!(embedding.chunk_id, "test_chunk");
        assert_eq!(embedding.vector.len(), 3);
    }

    #[test]
    fn test_indexed_file_creation() {
        let indexed_file = IndexedFile {
            file_id: FileId("file123".to_string()),
            path: PathBuf::from("src/main.rs"),
            language: Language::Rust,
            content_hash: ContentHash("abc123".to_string()),
            chunk_count: 5,
            indexed_at: Utc::now(),
        };
        
        assert_eq!(indexed_file.chunk_count, 5);
        assert_eq!(indexed_file.language, Language::Rust);
    }

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery {
            text: "function implementation".to_string(),
            limit: 10,
            similarity_threshold: 0.8,
            language_filter: Some(Language::Rust),
        };
        
        assert_eq!(query.limit, 10);
        assert_eq!(query.similarity_threshold, 0.8);
        assert_eq!(query.language_filter, Some(Language::Rust));
    }

    #[test]
    fn test_semantic_config_default() {
        let config = SemanticConfig::default();
        assert_eq!(config.database_path, PathBuf::from(".swissarmyhammer/semantic.db"));
        assert_eq!(config.embedding_model, "nomic-ai/nomic-embed-code");
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.chunk_overlap, 64);
        assert_eq!(config.similarity_threshold, 0.7);
    }

    #[test]
    fn test_semantic_config_serialization() {
        let config = SemanticConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SemanticConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.embedding_model, deserialized.embedding_model);
        assert_eq!(config.chunk_size, deserialized.chunk_size);
    }
}