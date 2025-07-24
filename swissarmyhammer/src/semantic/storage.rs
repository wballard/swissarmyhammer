//! DuckDB vector storage for semantic search
//!
//! This module provides a DuckDB-based vector storage implementation for code chunks
//! and their embeddings. It supports efficient vector similarity search using cosine
//! similarity and manages the database schema for semantic search operations.
//!
//! **Note**: This is currently implemented as an in-memory fallback while DuckDB
//! integration is being developed. Most storage operations are stubbed with TODO
//! comments and only basic file metadata is stored in memory.

use crate::error::{Result, SwissArmyHammerError};
use crate::semantic::{
    types::{
        CodeChunk, ContentHash, Embedding, IndexStats, IndexedFile, Language, SemanticSearchResult,
    },
    utils::SemanticUtils,
    SemanticConfig,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Vector storage for code chunks and embeddings
///
/// **CURRENT STATUS**: Using in-memory HashMap fallback
/// **ROADMAP**: Full DuckDB implementation planned
/// **LIMITATIONS**: No persistence, limited search capabilities
///
/// This implementation provides basic storage operations using in-memory data structures
/// while DuckDB integration is being developed. Most methods contain TODO comments marking
/// where DuckDB operations will be implemented. Only file metadata tracking and basic
/// similarity search are fully functional.
pub struct VectorStorage {
    db_path: PathBuf,
    _config: SemanticConfig,
    /// In-memory storage for file metadata (temporary fallback until DuckDB is implemented)
    indexed_files: Arc<Mutex<HashMap<PathBuf, (ContentHash, IndexedFile)>>>,
    /// In-memory storage for code chunks (temporary fallback until DuckDB is implemented)
    chunks: Arc<Mutex<HashMap<String, CodeChunk>>>,
    /// In-memory storage for embeddings (temporary fallback until DuckDB is implemented)
    embeddings: Arc<Mutex<HashMap<String, Embedding>>>,
}

impl Clone for VectorStorage {
    fn clone(&self) -> Self {
        Self {
            db_path: self.db_path.clone(),
            _config: self._config.clone(),
            indexed_files: Arc::clone(&self.indexed_files),
            chunks: Arc::clone(&self.chunks),
            embeddings: Arc::clone(&self.embeddings),
        }
    }
}

impl VectorStorage {
    /// Create a new vector storage instance
    pub fn new(config: SemanticConfig) -> Result<Self> {
        let db_path = config.database_path.clone();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(SwissArmyHammerError::Io)?;
        }

        Ok(Self {
            db_path,
            _config: config,
            indexed_files: Arc::new(Mutex::new(HashMap::new())),
            chunks: Arc::new(Mutex::new(HashMap::new())),
            embeddings: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Initialize the database schema
    pub fn initialize(&self) -> Result<()> {
        // TODO: Implement DuckDB schema initialization when DuckDB is available
        // For now, just log that we're using in-memory storage
        tracing::info!(
            "Initializing vector storage at: {} (using in-memory fallback)",
            self.db_path.display()
        );
        Ok(())
    }

    /// Store indexed file metadata
    pub fn store_indexed_file(&self, file: &IndexedFile) -> Result<()> {
        // TODO: Implement DuckDB file metadata storage
        // INSERT OR REPLACE INTO indexed_files
        // (file_id, path, language, content_hash, chunk_count, indexed_at)
        // VALUES (?, ?, ?, ?, ?, ?)

        tracing::debug!("Storing indexed file: {}", file.path.display());
        // For now, use in-memory storage
        let mut indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!("Failed to lock indexed_files for storage: {e}"))
        })?;

        indexed_files.insert(file.path.clone(), (file.content_hash.clone(), file.clone()));

        Ok(())
    }

    /// Store a code chunk
    pub fn store_chunk(&self, chunk: &CodeChunk) -> Result<()> {
        // Store chunk in in-memory storage (fallback until DuckDB is implemented)
        let mut chunks = self.chunks.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
        })?;

        chunks.insert(chunk.id.clone(), chunk.clone());
        tracing::debug!("Stored chunk: {}", chunk.id);
        Ok(())
    }

    /// Store an embedding for a code chunk
    pub fn store_embedding(&self, embedding: &Embedding) -> Result<()> {
        // Store embedding in in-memory storage (fallback until DuckDB is implemented)
        let mut embeddings = self.embeddings.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire embeddings lock".to_string())
        })?;

        embeddings.insert(embedding.chunk_id.clone(), embedding.clone());
        tracing::debug!("Stored embedding for chunk: {}", embedding.chunk_id);
        Ok(())
    }

    /// Search for similar chunks using vector similarity
    pub fn similarity_search(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<SemanticSearchResult>> {
        // In-memory similarity search using cosine similarity (fallback until DuckDB is implemented)
        tracing::debug!(
            "Searching for similar embeddings: query_dim={}, limit={}, threshold={}",
            query_embedding.len(),
            limit,
            threshold
        );

        let embeddings = self.embeddings.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire embeddings lock".to_string())
        })?;

        let chunks = self.chunks.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
        })?;

        let mut results = Vec::new();

        // Calculate similarity for each embedding
        for (chunk_id, embedding) in embeddings.iter() {
            let similarity = SemanticUtils::cosine_similarity(query_embedding, &embedding.vector);

            if similarity >= threshold {
                if let Some(chunk) = chunks.get(chunk_id) {
                    results.push(SemanticSearchResult {
                        chunk: chunk.clone(),
                        similarity_score: similarity,
                        excerpt: chunk.content.clone(),
                    });
                }
            }
        }

        // Sort by similarity (descending) and limit results
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        tracing::debug!("Found {} similar chunks", results.len());
        Ok(results)
    }

    /// Search for similar chunks with detailed embedding information for debugging
    pub fn similarity_search_with_details(
        &self,
        query_embedding: &[f32],
        limit: usize,
        threshold: f32,
    ) -> Result<Vec<(String, f32, Vec<f32>)>> {
        // In-memory similarity search with detailed results (fallback until DuckDB is implemented)
        tracing::debug!(
            "Searching for similar embeddings with details: query_dim={}, limit={}, threshold={}",
            query_embedding.len(),
            limit,
            threshold
        );

        let embeddings = self.embeddings.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire embeddings lock".to_string())
        })?;

        let mut results = Vec::new();

        // Calculate similarity for each embedding
        for (chunk_id, embedding) in embeddings.iter() {
            let similarity = SemanticUtils::cosine_similarity(query_embedding, &embedding.vector);

            if similarity >= threshold {
                results.push((chunk_id.clone(), similarity, embedding.vector.clone()));
            }
        }

        // Sort by similarity (descending) and limit results
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        tracing::debug!("Found {} similar chunks with details", results.len());
        Ok(results)
    }

    /// Get chunk by ID
    pub fn get_chunk(&self, chunk_id: &str) -> Result<Option<CodeChunk>> {
        // TODO: Implement DuckDB chunk retrieval
        // SELECT chunk_id, file_id, content, start_line, end_line, chunk_type, language, content_hash
        // FROM code_chunks WHERE chunk_id = ?

        tracing::debug!("Getting chunk: {}", chunk_id);

        // Use in-memory storage (fallback until DuckDB is implemented)
        let chunks = self.chunks.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
        })?;

        Ok(chunks.get(chunk_id).cloned())
    }

    /// Get all chunks for a file
    pub fn get_file_chunks(&self, file_path: &Path) -> Result<Vec<CodeChunk>> {
        // Get chunks from in-memory storage (fallback until DuckDB is implemented)
        let chunks = self.chunks.lock().map_err(|_| {
            SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
        })?;

        let file_chunks: Vec<CodeChunk> = chunks
            .values()
            .filter(|chunk| chunk.file_path == file_path)
            .cloned()
            .collect();

        tracing::debug!(
            "Found {} chunks for file: {}",
            file_chunks.len(),
            file_path.display()
        );
        Ok(file_chunks)
    }

    /// Check if file needs re-indexing based on content hash
    pub fn needs_reindexing(&self, file_path: &Path, current_hash: &ContentHash) -> Result<bool> {
        tracing::debug!("Checking if file needs reindexing: {}", file_path.display());

        // Get the stored hash for this file
        match self.get_file_hash(file_path)? {
            Some(stored_hash) => {
                // File exists in index - compare hashes
                let needs_reindexing = stored_hash != *current_hash;
                if needs_reindexing {
                    tracing::debug!("File {} has changed (hash mismatch)", file_path.display());
                } else {
                    tracing::debug!("File {} unchanged (hash match)", file_path.display());
                }
                Ok(needs_reindexing)
            }
            None => {
                // File not in index - needs indexing
                tracing::debug!("File {} not in index - needs indexing", file_path.display());
                Ok(true)
            }
        }
    }

    /// Check if a file has been indexed
    pub fn is_file_indexed(&self, file_path: &Path) -> Result<bool> {
        // TODO: Implement DuckDB file indexing check
        // SELECT 1 FROM indexed_files WHERE path = ? LIMIT 1

        tracing::debug!("Checking if file is indexed: {}", file_path.display());

        // Check in-memory storage for now
        let indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock indexed_files for file check: {e}"
            ))
        })?;
        Ok(indexed_files.contains_key(file_path))
    }

    /// Remove all data for a file (for re-indexing)
    pub fn remove_file(&self, file_path: &Path) -> Result<()> {
        // Remove from in-memory storage (fallback until DuckDB is implemented)
        tracing::debug!("Removing file: {}", file_path.display());

        // Get chunks for this file to find chunk IDs
        let chunks_to_remove: Vec<String> = {
            let chunks = self.chunks.lock().map_err(|_| {
                SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
            })?;

            chunks
                .values()
                .filter(|chunk| chunk.file_path == file_path)
                .map(|chunk| chunk.id.clone())
                .collect()
        };

        // Remove embeddings for these chunks
        {
            let mut embeddings = self.embeddings.lock().map_err(|_| {
                SwissArmyHammerError::Storage("Failed to acquire embeddings lock".to_string())
            })?;

            for chunk_id in &chunks_to_remove {
                embeddings.remove(chunk_id);
            }
        }

        // Remove chunks for this file
        {
            let mut chunks = self.chunks.lock().map_err(|_| {
                SwissArmyHammerError::Storage("Failed to acquire chunks lock".to_string())
            })?;

            for chunk_id in &chunks_to_remove {
                chunks.remove(chunk_id);
            }
        }

        // Remove indexed file metadata
        {
            let mut indexed_files = self.indexed_files.lock().map_err(|_| {
                SwissArmyHammerError::Storage("Failed to acquire indexed_files lock".to_string())
            })?;

            indexed_files.remove(file_path);
        }

        tracing::debug!(
            "Removed {} chunks for file: {}",
            chunks_to_remove.len(),
            file_path.display()
        );
        Ok(())
    }

    /// Get statistics about the stored data
    pub fn get_stats(&self) -> Result<StorageStats> {
        // TODO: Implement DuckDB statistics gathering
        // SELECT COUNT(*) FROM code_chunks;
        // SELECT COUNT(DISTINCT file_id) FROM code_chunks;

        Ok(StorageStats {
            total_chunks: 0,
            total_files: 0,
            total_embeddings: 0,
            database_size_bytes: 0,
        })
    }

    /// Get chunks by language
    pub fn get_chunks_by_language(&self, language: &Language) -> Result<Vec<CodeChunk>> {
        // TODO: Implement DuckDB language filtering
        // SELECT chunk_id, file_id, content, start_line, end_line, chunk_type, language, content_hash
        // FROM code_chunks WHERE language = ?

        tracing::debug!("Getting chunks by language: {:?}", language);
        Ok(vec![])
    }

    /// Perform database maintenance (vacuum, analyze, etc.)
    pub fn maintenance(&self) -> Result<()> {
        // TODO: Implement DuckDB maintenance
        // VACUUM;
        // ANALYZE;

        tracing::info!("Performing database maintenance");
        Ok(())
    }

    /// Check if file exists in index
    pub fn file_exists(&self, file_path: &Path) -> Result<bool> {
        let indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock indexed_files for file existence check: {e}"
            ))
        })?;

        Ok(indexed_files.contains_key(file_path))
    }

    /// Get file hash from index
    pub fn get_file_hash(&self, file_path: &Path) -> Result<Option<ContentHash>> {
        let indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock indexed_files for hash retrieval: {e}"
            ))
        })?;

        Ok(indexed_files.get(file_path).map(|(hash, _)| hash.clone()))
    }

    /// Get statistics about indexed files
    pub fn get_index_stats(&self) -> Result<IndexStats> {
        let indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock indexed_files for statistics retrieval: {e}"
            ))
        })?;

        let chunks = self.chunks.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock chunks for statistics retrieval: {e}"
            ))
        })?;

        let embeddings = self.embeddings.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock embeddings for statistics retrieval: {e}"
            ))
        })?;

        let file_count = indexed_files.len();
        let chunk_count = chunks.len();
        let embedding_count = embeddings.len();

        Ok(IndexStats {
            file_count,
            chunk_count,
            embedding_count,
        })
    }

    /// Store indexed file (for internal use in testing/development)
    pub fn store_indexed_file_internal(&self, file: &IndexedFile) -> Result<()> {
        let mut indexed_files = self.indexed_files.lock().map_err(|e| {
            SwissArmyHammerError::Config(format!(
                "Failed to lock indexed_files for file storage: {e}"
            ))
        })?;

        indexed_files.insert(file.path.clone(), (file.content_hash.clone(), file.clone()));
        Ok(())
    }
}

/// Statistics about the vector storage
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StorageStats {
    /// Total number of code chunks stored
    pub total_chunks: usize,
    /// Total number of indexed files
    pub total_files: usize,
    /// Total number of embeddings stored
    pub total_embeddings: usize,
    /// Database file size in bytes
    pub database_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::types::{ChunkType, FileId};
    use chrono::Utc;

    fn create_test_config() -> SemanticConfig {
        SemanticConfig {
            database_path: PathBuf::from("/tmp/test_semantic.db"),
            embedding_model: "test-model".to_string(),
            chunk_size: 512,
            chunk_overlap: 64,
            similarity_threshold: 0.7,
            excerpt_length: 200,
            context_lines: 2,
            simple_search_threshold: 0.5,
            code_similarity_threshold: 0.7,
            content_preview_length: 100,
        }
    }

    fn create_test_chunk() -> CodeChunk {
        CodeChunk {
            id: "test-chunk-1".to_string(),
            file_path: PathBuf::from("test.rs"),
            language: Language::Rust,
            content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
            start_line: 1,
            end_line: 1,
            chunk_type: ChunkType::Function,
            content_hash: ContentHash("test-hash".to_string()),
        }
    }

    fn create_test_embedding() -> Embedding {
        let mut vector = vec![0.1; 384]; // 384-dimensional vector
        vector[0] = 0.1;
        vector[1] = 0.2;
        vector[2] = 0.3;

        Embedding {
            chunk_id: "test-chunk-1".to_string(),
            vector,
        }
    }

    fn create_test_indexed_file() -> IndexedFile {
        IndexedFile {
            file_id: FileId("test-file-1".to_string()),
            path: PathBuf::from("test.rs"),
            language: Language::Rust,
            content_hash: ContentHash("file-hash".to_string()),
            chunk_count: 1,
            indexed_at: Utc::now(),
        }
    }

    #[test]
    fn test_vector_storage_creation() {
        let config = create_test_config();
        let storage = VectorStorage::new(config);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_initialize() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        assert!(storage.initialize().is_ok());
    }

    #[test]
    fn test_store_chunk() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let chunk = create_test_chunk();
        assert!(storage.store_chunk(&chunk).is_ok());
    }

    #[test]
    fn test_store_embedding() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let embedding = create_test_embedding();
        assert!(storage.store_embedding(&embedding).is_ok());
    }

    #[test]
    fn test_store_indexed_file() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let indexed_file = create_test_indexed_file();
        assert!(storage.store_indexed_file(&indexed_file).is_ok());
    }

    #[test]
    fn test_empty_search() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let query = vec![0.1; 384]; // 384-dimensional query vector
        let results = storage.similarity_search(&query, 10, 0.5);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_get_chunk() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let result = storage.get_chunk("test-chunk-1");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_needs_reindexing() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let path = Path::new("test.rs");
        let hash = ContentHash("test-hash".to_string());
        let result = storage.needs_reindexing(path, &hash);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should always return true for placeholder
    }

    #[test]
    fn test_is_file_indexed() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let path = Path::new("test.rs");
        let result = storage.is_file_indexed(path);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should always return false for placeholder
    }

    #[test]
    fn test_remove_file() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let path = Path::new("test.rs");
        assert!(storage.remove_file(path).is_ok());
    }

    #[test]
    fn test_get_stats() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let stats = storage.get_stats();
        assert!(stats.is_ok());
        let stats = stats.unwrap();
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_embeddings, 0);
    }

    #[test]
    fn test_get_chunks_by_language() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let chunks = storage.get_chunks_by_language(&Language::Rust);
        assert!(chunks.is_ok());
        assert_eq!(chunks.unwrap().len(), 0);
    }

    #[test]
    fn test_maintenance() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        assert!(storage.maintenance().is_ok());
    }

    #[test]
    fn test_storage_stats_default() {
        let stats = StorageStats::default();
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_embeddings, 0);
        assert_eq!(stats.database_size_bytes, 0);
    }

    #[test]
    fn test_get_file_chunks() {
        let config = create_test_config();
        let storage = VectorStorage::new(config).unwrap();
        let path = Path::new("test.rs");
        let chunks = storage.get_file_chunks(path);
        assert!(chunks.is_ok());
        assert_eq!(chunks.unwrap().len(), 0);
    }
}
