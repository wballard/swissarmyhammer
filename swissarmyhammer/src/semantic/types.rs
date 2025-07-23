//! Core data structures for semantic search functionality

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported programming languages for TreeSitter parsing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Dart,
}

/// A code chunk extracted from a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    /// Unique identifier for this chunk
    pub id: String,
    /// Path to the source file
    pub file_path: PathBuf,
    /// The actual code content
    pub content: String,
    /// Language of the code
    pub language: Language,
    /// Line number where this chunk starts
    pub start_line: usize,
    /// Line number where this chunk ends
    pub end_line: usize,
    /// MD5 hash of the content
    pub content_hash: String,
    /// Vector embedding of the content
    pub embedding: Option<Vec<f32>>,
}

/// Search result containing a code chunk and its relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching code chunk
    pub chunk: CodeChunk,
    /// Similarity score (0.0 to 1.0)
    pub score: f32,
}

/// Configuration for the semantic search system
#[derive(Debug, Clone)]
pub struct SemanticConfig {
    /// Path to the DuckDB database file
    pub database_path: PathBuf,
    /// Maximum number of chunks to extract per file
    pub max_chunks_per_file: usize,
    /// Minimum chunk size in characters
    pub min_chunk_size: usize,
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from(".swissarmyhammer/semantic.db"),
            max_chunks_per_file: 100,
            min_chunk_size: 50,
            max_chunk_size: 2000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_serialization() {
        let lang = Language::Rust;
        let serialized = serde_json::to_string(&lang).unwrap();
        let deserialized: Language = serde_json::from_str(&serialized).unwrap();
        assert_eq!(lang, deserialized);
    }

    #[test]
    fn test_semantic_config_default() {
        let config = SemanticConfig::default();
        assert_eq!(config.database_path, PathBuf::from(".swissarmyhammer/semantic.db"));
        assert_eq!(config.max_chunks_per_file, 100);
        assert_eq!(config.min_chunk_size, 50);
        assert_eq!(config.max_chunk_size, 2000);
    }
}