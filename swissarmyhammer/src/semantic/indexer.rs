//! File indexing logic for semantic search

use crate::error::Result;
use crate::semantic::{CodeParser, EmbeddingService, ParserConfig, VectorStorage};
use std::path::Path;
use walkdir::WalkDir;

/// File indexer that processes source files for semantic search
pub struct FileIndexer {
    parser: CodeParser,
    embedding_service: EmbeddingService,
    storage: VectorStorage,
}

/// Options for indexing operations
#[derive(Debug, Clone)]
pub struct IndexingOptions {
    /// Force re-indexing of already indexed files
    pub force: bool,
    /// Glob pattern for files to include
    pub glob_pattern: Option<String>,
    /// Maximum number of files to process
    pub max_files: Option<usize>,
}

impl Default for IndexingOptions {
    fn default() -> Self {
        Self {
            force: false,
            glob_pattern: None,
            max_files: None,
        }
    }
}

impl FileIndexer {
    /// Create a new file indexer
    pub fn new(
        parser: CodeParser,
        embedding_service: EmbeddingService,
        storage: VectorStorage,
    ) -> Self {
        Self {
            parser,
            embedding_service,
            storage,
        }
    }

    /// Index files matching the given glob pattern
    pub fn index_files(&self, root_path: &Path, options: &IndexingOptions) -> Result<IndexingStats> {
        let mut stats = IndexingStats::default();

        for entry in WalkDir::new(root_path).into_iter() {
            let entry = entry.map_err(|e| crate::error::SwissArmyHammerError::Io(
                std::io::Error::new(std::io::ErrorKind::Other, format!("Walk error: {}", e))
            ))?;

            let path = entry.path();
            
            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check if file is supported
            if !self.parser.is_supported_file(path) {
                continue;
            }

            // Check glob pattern if specified
            if let Some(pattern) = &options.glob_pattern {
                if !self.matches_glob(path, pattern) {
                    continue;
                }
            }

            // Check if already indexed (unless force is true)
            if !options.force && self.storage.is_file_indexed(path)? {
                stats.skipped_files += 1;
                continue;
            }

            // Check max files limit
            if let Some(max_files) = options.max_files {
                if stats.processed_files >= max_files {
                    break;
                }
            }

            // Process the file
            match self.index_single_file(path) {
                Ok(chunk_count) => {
                    stats.processed_files += 1;
                    stats.total_chunks += chunk_count;
                }
                Err(e) => {
                    stats.failed_files += 1;
                    tracing::warn!("Failed to index file {}: {}", path.display(), e);
                }
            }
        }

        Ok(stats)
    }

    /// Index a single file
    fn index_single_file(&self, file_path: &Path) -> Result<usize> {
        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(crate::error::SwissArmyHammerError::Io)?;

        // Parse into chunks
        let mut chunks = self.parser.parse_file(file_path, &content)?;

        // Generate embeddings for chunks
        let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedding_service.embed_batch(&chunk_texts)?;

        // Add embeddings to chunks
        for (chunk, embedding) in chunks.iter_mut().zip(embeddings.into_iter()) {
            chunk.embedding = Some(embedding);
        }

        // Store chunks
        let chunk_count = chunks.len();
        for chunk in chunks {
            self.storage.store_chunk(&chunk)?;
        }

        Ok(chunk_count)
    }

    /// Check if a path matches a glob pattern
    fn matches_glob(&self, _path: &Path, _pattern: &str) -> bool {
        // TODO: Implement proper glob matching
        true
    }
}

/// Statistics from an indexing operation
#[derive(Debug, Clone, Default)]
pub struct IndexingStats {
    pub processed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub total_chunks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{SemanticConfig, VectorStorage};
    use tempfile::TempDir;
    use std::fs;

    fn create_test_indexer() -> Result<(FileIndexer, TempDir)> {
        let temp_dir = TempDir::new().map_err(crate::error::SwissArmyHammerError::Io)?;
        let config = SemanticConfig {
            database_path: temp_dir.path().join("test.db"),
            ..Default::default()
        };

        let parser = CodeParser::new(ParserConfig::default())?;
        let embedding_service = EmbeddingService::new()?;
        let storage = VectorStorage::new(config)?;

        let indexer = FileIndexer::new(parser, embedding_service, storage);
        Ok((indexer, temp_dir))
    }

    #[test]
    fn test_indexer_creation() {
        let result = create_test_indexer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_empty_directory() {
        let (indexer, temp_dir) = create_test_indexer().unwrap();
        let options = IndexingOptions::default();
        
        let stats = indexer.index_files(temp_dir.path(), &options);
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert_eq!(stats.processed_files, 0);
        assert_eq!(stats.skipped_files, 0);
        assert_eq!(stats.failed_files, 0);
    }

    #[test]
    fn test_index_single_rust_file() {
        let (indexer, temp_dir) = create_test_indexer().unwrap();
        
        // Create a test Rust file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
        
        let options = IndexingOptions::default();
        let stats = indexer.index_files(temp_dir.path(), &options);
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert_eq!(stats.processed_files, 1);
        assert_eq!(stats.total_chunks, 1);
    }
}