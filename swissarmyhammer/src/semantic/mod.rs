//! Semantic search functionality using vector embeddings and TreeSitter parsing
//!
//! This module provides semantic search capabilities for source code files.
//! It uses mistral.rs for embeddings, DuckDB for vector storage, and TreeSitter
//! for parsing various programming languages.

use thiserror::Error;

pub mod embedding;
pub mod indexer;
pub mod parser;
pub mod searcher;
pub mod storage;
pub mod types;
pub mod utils;

// Integration tests
#[cfg(test)]
pub mod tests;

/// Semantic search specific errors
#[derive(Error, Debug)]
pub enum SemanticError {
    /// Database operation failed
    #[error("Database error: {0}")]
    Database(String),

    /// Embedding generation failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// File system operation failed
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    /// TreeSitter parsing failed
    #[error("TreeSitter parsing error: {0}")]
    TreeSitter(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization or deserialization failed
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// ONNX Runtime error
    #[error("ONNX Runtime error: {0}")]
    OnnxRuntime(#[from] ort::Error),

    /// Index operation failed
    #[error("Index error: {0}")]
    Index(String),

    /// Search operation failed
    #[error("Search error: {0}")]
    Search(String),
}

/// Result type for semantic search operations
pub type Result<T> = std::result::Result<T, SemanticError>;

pub use embedding::*;
pub use indexer::*;
pub use parser::*;
pub use searcher::*;
pub use storage::*;
pub use types::*;
pub use utils::*;

// Re-export for convenience
pub use SemanticError as Error;
