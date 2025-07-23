//! Semantic search functionality using vector embeddings and TreeSitter parsing
//!
//! This module provides semantic search capabilities for source code files.
//! It uses mistral.rs for embeddings, DuckDB for vector storage, and TreeSitter
//! for parsing various programming languages.

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

pub use embedding::*;
pub use indexer::*;
pub use parser::*;
pub use searcher::*;
pub use storage::*;
pub use types::*;
pub use utils::*;