//! Core data structures for semantic search functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

/// Status of a file's change detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeStatus {
    /// File has changed since last indexing
    Changed {
        /// The new content hash for the changed file
        new_hash: ContentHash,
        /// Whether this file was previously indexed
        exists_in_index: bool,
    },
    /// File hasn't changed since last indexing
    Unchanged {
        /// The content hash of the unchanged file
        hash: ContentHash,
    },
}

/// Report of file change detection results
#[derive(Debug)]
pub struct FileChangeReport {
    /// Files that have changed and exist in index
    pub changed_files: std::collections::HashMap<PathBuf, ContentHash>,
    /// Files that haven't changed
    pub unchanged_files: std::collections::HashMap<PathBuf, ContentHash>,
    /// Files that are new (not in index)
    pub new_files: std::collections::HashMap<PathBuf, ContentHash>,
    /// Files that had errors during processing
    pub errors: std::collections::HashMap<PathBuf, crate::error::SwissArmyHammerError>,
}

/// Statistics about the vector storage index
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexStats {
    /// Number of indexed files
    pub file_count: usize,
    /// Number of code chunks
    pub chunk_count: usize,
    /// Number of embeddings
    pub embedding_count: usize,
}

impl FileChangeReport {
    /// Create a new empty change report
    pub fn new() -> Self {
        Self {
            changed_files: std::collections::HashMap::new(),
            unchanged_files: std::collections::HashMap::new(),
            new_files: std::collections::HashMap::new(),
            errors: std::collections::HashMap::new(),
        }
    }

    /// Add a file status to the report
    pub fn add_file_status(&mut self, path: PathBuf, status: FileChangeStatus) {
        match status {
            FileChangeStatus::Changed {
                new_hash,
                exists_in_index,
            } => {
                if exists_in_index {
                    self.changed_files.insert(path, new_hash);
                } else {
                    self.new_files.insert(path, new_hash);
                }
            }
            FileChangeStatus::Unchanged { hash } => {
                self.unchanged_files.insert(path, hash);
            }
        }
    }

    /// Add an error for a file
    pub fn add_error(&mut self, path: PathBuf, error: crate::error::SwissArmyHammerError) {
        self.errors.insert(path, error);
    }

    /// Get files that need indexing (changed + new)
    pub fn files_needing_indexing(&self) -> impl Iterator<Item = &PathBuf> {
        self.changed_files.keys().chain(self.new_files.keys())
    }

    /// Get total number of files processed
    pub fn total_files(&self) -> usize {
        self.changed_files.len()
            + self.unchanged_files.len()
            + self.new_files.len()
            + self.errors.len()
    }

    /// Get a summary string of the report
    pub fn summary(&self) -> String {
        format!(
            "Files: {} new, {} changed, {} unchanged, {} errors",
            self.new_files.len(),
            self.changed_files.len(),
            self.unchanged_files.len(),
            self.errors.len()
        )
    }
}

impl Default for FileChangeReport {
    fn default() -> Self {
        Self::new()
    }
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
        assert_eq!(
            config.database_path,
            PathBuf::from(".swissarmyhammer/semantic.db")
        );
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

    #[test]
    fn test_file_change_status() {
        let hash = ContentHash("abc123".to_string());

        let changed_status = FileChangeStatus::Changed {
            new_hash: hash.clone(),
            exists_in_index: true,
        };

        let unchanged_status = FileChangeStatus::Unchanged { hash: hash.clone() };

        match changed_status {
            FileChangeStatus::Changed {
                new_hash,
                exists_in_index,
            } => {
                assert_eq!(new_hash, hash);
                assert!(exists_in_index);
            }
            _ => panic!("Expected Changed status"),
        }

        match unchanged_status {
            FileChangeStatus::Unchanged { hash: h } => {
                assert_eq!(h, hash);
            }
            _ => panic!("Expected Unchanged status"),
        }
    }

    #[test]
    fn test_file_change_report_new() {
        let report = FileChangeReport::new();
        assert_eq!(report.changed_files.len(), 0);
        assert_eq!(report.unchanged_files.len(), 0);
        assert_eq!(report.new_files.len(), 0);
        assert_eq!(report.errors.len(), 0);
        assert_eq!(report.total_files(), 0);
    }

    #[test]
    fn test_file_change_report_add_file_status() {
        let mut report = FileChangeReport::new();
        let path1 = PathBuf::from("test1.rs");
        let path2 = PathBuf::from("test2.rs");
        let path3 = PathBuf::from("test3.rs");
        let hash = ContentHash("abc123".to_string());

        // Test adding changed file (exists in index)
        report.add_file_status(
            path1.clone(),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: true,
            },
        );
        assert_eq!(report.changed_files.len(), 1);
        assert!(report.changed_files.contains_key(&path1));

        // Test adding new file (doesn't exist in index)
        report.add_file_status(
            path2.clone(),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: false,
            },
        );
        assert_eq!(report.new_files.len(), 1);
        assert!(report.new_files.contains_key(&path2));

        // Test adding unchanged file
        report.add_file_status(
            path3.clone(),
            FileChangeStatus::Unchanged { hash: hash.clone() },
        );
        assert_eq!(report.unchanged_files.len(), 1);
        assert!(report.unchanged_files.contains_key(&path3));

        assert_eq!(report.total_files(), 3);
    }

    #[test]
    fn test_file_change_report_add_error() {
        let mut report = FileChangeReport::new();
        let path = PathBuf::from("error_file.rs");
        let error = crate::error::SwissArmyHammerError::Config("Test error".to_string());

        report.add_error(path.clone(), error);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors.contains_key(&path));
        assert_eq!(report.total_files(), 1);
    }

    #[test]
    fn test_file_change_report_files_needing_indexing() {
        let mut report = FileChangeReport::new();
        let changed_path = PathBuf::from("changed.rs");
        let new_path = PathBuf::from("new.rs");
        let unchanged_path = PathBuf::from("unchanged.rs");
        let hash = ContentHash("abc123".to_string());

        report.add_file_status(
            changed_path.clone(),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: true,
            },
        );

        report.add_file_status(
            new_path.clone(),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: false,
            },
        );

        report.add_file_status(
            unchanged_path.clone(),
            FileChangeStatus::Unchanged { hash: hash.clone() },
        );

        let needing_indexing: Vec<&PathBuf> = report.files_needing_indexing().collect();
        assert_eq!(needing_indexing.len(), 2);
        assert!(needing_indexing.contains(&&changed_path));
        assert!(needing_indexing.contains(&&new_path));
        assert!(!needing_indexing.contains(&&unchanged_path));
    }

    #[test]
    fn test_file_change_report_summary() {
        let mut report = FileChangeReport::new();
        let hash = ContentHash("abc123".to_string());

        // Add one of each type
        report.add_file_status(
            PathBuf::from("new.rs"),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: false,
            },
        );

        report.add_file_status(
            PathBuf::from("changed.rs"),
            FileChangeStatus::Changed {
                new_hash: hash.clone(),
                exists_in_index: true,
            },
        );

        report.add_file_status(
            PathBuf::from("unchanged.rs"),
            FileChangeStatus::Unchanged { hash: hash.clone() },
        );

        report.add_error(
            PathBuf::from("error.rs"),
            crate::error::SwissArmyHammerError::Config("Test error".to_string()),
        );

        let summary = report.summary();
        assert_eq!(summary, "Files: 1 new, 1 changed, 1 unchanged, 1 errors");
    }

    #[test]
    fn test_index_stats() {
        let stats = IndexStats {
            file_count: 10,
            chunk_count: 100,
            embedding_count: 100,
        };

        assert_eq!(stats.file_count, 10);
        assert_eq!(stats.chunk_count, 100);
        assert_eq!(stats.embedding_count, 100);
    }
}
