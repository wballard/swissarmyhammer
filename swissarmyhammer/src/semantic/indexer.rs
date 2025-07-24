//! File indexing logic for semantic search

use crate::semantic::{
    CodeChunk, CodeParser, EmbeddingEngine, FileHasher, FileId, IndexedFile,
    ParserConfig, Result, SemanticError, VectorStorage,
};
use chrono::Utc;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};

/// File indexer that processes source files for semantic search
pub struct FileIndexer {
    storage: VectorStorage,
    embedding_engine: EmbeddingEngine,
    parser: CodeParser,
}

/// Options for indexing operations
#[derive(Debug, Clone, Default)]
pub struct IndexingOptions {
    /// Force re-indexing of already indexed files
    pub force: bool,
    /// Glob pattern for files to include
    pub glob_pattern: Option<String>,
    /// Maximum number of files to process
    pub max_files: Option<usize>,
}

impl FileIndexer {
    /// Create a new FileIndexer with default configuration
    ///
    /// # Arguments
    /// * `storage` - Vector storage backend for persisting chunks and embeddings
    ///
    /// # Returns
    /// A new FileIndexer instance with default embedding engine and parser configuration
    pub async fn new(storage: VectorStorage) -> Result<Self> {
        let embedding_engine = EmbeddingEngine::new().await?;
        let parser = CodeParser::new(Default::default())?;

        Ok(Self {
            storage,
            embedding_engine,
            parser,
        })
    }

    /// Create a FileIndexer with a custom embedding engine
    ///
    /// # Arguments
    /// * `storage` - Vector storage backend for persisting chunks and embeddings
    /// * `embedding_engine` - Pre-configured embedding engine to use for generating embeddings
    ///
    /// # Returns
    /// A new FileIndexer instance with the provided embedding engine and default parser configuration
    pub async fn with_custom_embedding_engine(
        storage: VectorStorage,
        embedding_engine: EmbeddingEngine,
    ) -> Result<Self> {
        let parser = CodeParser::new(Default::default())?;

        Ok(Self {
            storage,
            embedding_engine,
            parser,
        })
    }

    /// Create a FileIndexer with custom embedding engine and parser configuration
    ///
    /// # Arguments
    /// * `storage` - Vector storage backend for persisting chunks and embeddings
    /// * `embedding_engine` - Pre-configured embedding engine to use for generating embeddings
    /// * `parser_config` - Custom parser configuration for code chunk extraction
    ///
    /// # Returns
    /// A new FileIndexer instance with all custom components configured
    pub async fn with_custom_config(
        storage: VectorStorage,
        embedding_engine: EmbeddingEngine,
        parser_config: ParserConfig,
    ) -> Result<Self> {
        let parser = CodeParser::new(parser_config)?;

        Ok(Self {
            storage,
            embedding_engine,
            parser,
        })
    }

    /// Index files matching a glob pattern (new API from issue specification)
    pub async fn index_glob(
        &mut self,
        pattern: &str,
        force_reindex: bool,
    ) -> Result<IndexingReport> {
        tracing::info!("Starting indexing with pattern: {}", pattern);

        // Expand glob pattern to file paths
        let file_paths = self.expand_glob_pattern(pattern)?;

        if file_paths.is_empty() {
            tracing::warn!("No files found matching pattern: {}", pattern);
            return Ok(IndexingReport::empty());
        }

        tracing::info!("Found {} files matching pattern", file_paths.len());

        // Filter files based on change detection unless forced
        let files_to_process = if force_reindex {
            file_paths
        } else {
            self.filter_changed_files(file_paths).await?
        };

        if files_to_process.is_empty() {
            tracing::info!("No files need re-indexing");
            return Ok(IndexingReport::empty());
        }

        // Process files
        self.index_files(files_to_process, force_reindex).await
    }

    /// Expand glob pattern to list of file paths
    fn expand_glob_pattern(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        for entry in glob(pattern).map_err(|e| {
            SemanticError::Config(format!("Invalid glob pattern '{pattern}': {e}"))
        })? {
            match entry {
                Ok(path) if path.is_file() => {
                    // Filter supported file types
                    if self.is_supported_file(&path) {
                        paths.push(path);
                    }
                }
                Ok(_) => {
                    // Skip directories
                }
                Err(e) => {
                    tracing::warn!("Error processing glob entry: {}", e);
                }
            }
        }

        Ok(paths)
    }

    /// Check if a file is supported for indexing
    fn is_supported_file(&self, path: &Path) -> bool {
        self.parser.is_supported_file(path)
    }

    /// Filter files based on change detection
    async fn filter_changed_files(&self, paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        // Check files for changes directly using main storage to avoid clone issues
        let mut files_needing_indexing = Vec::new();
        
        for path in paths {
            // Calculate current hash
            let current_hash = FileHasher::hash_file(&path)
                .map_err(|e| SemanticError::Index(e.to_string()))?;
            
            // Check if file needs re-indexing using main storage
            let needs_reindexing = self.storage.needs_reindexing(&path, &current_hash)
                .map_err(|e| SemanticError::Index(e.to_string()))?;
            
            if needs_reindexing {
                files_needing_indexing.push(path);
            }
        }
        
        tracing::info!("Found {} files needing indexing", files_needing_indexing.len());

        Ok(files_needing_indexing)
    }

    /// Index a list of files
    pub async fn index_files(
        &mut self,
        file_paths: Vec<PathBuf>,
        force_reindex: bool,
    ) -> Result<IndexingReport> {
        let mut report = IndexingReport::new();
        let start_time = std::time::Instant::now();

        // Setup progress bar
        let progress = ProgressBar::new(file_paths.len() as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("##-"),
        );

        for file_path in file_paths {
            progress.set_message(format!("Processing {}", file_path.display()));

            match self.index_single_file(&file_path, force_reindex).await {
                Ok(file_report) => {
                    report.merge(file_report);
                    tracing::debug!("Successfully indexed: {}", file_path.display());
                }
                Err(e) => {
                    tracing::error!("Failed to index {}: {}", file_path.display(), e);
                    report.add_error(file_path, crate::error::SwissArmyHammerError::Semantic(e));
                }
            }

            progress.inc(1);
        }

        progress.finish_with_message("Indexing complete");

        report.duration = start_time.elapsed();
        tracing::info!("Indexing report: {}", report.summary());
        Ok(report)
    }

    async fn index_single_file(
        &mut self,
        file_path: &Path,
        force_reindex: bool,
    ) -> Result<SingleFileReport> {
        let mut report = SingleFileReport::new(file_path.to_path_buf());

        // Remove existing data if force re-indexing
        if force_reindex {
            self.storage
                .remove_file(file_path)
                .map_err(|e| SemanticError::Index(e.to_string()))?;
        }

        // Parse file into chunks
        let content = std::fs::read_to_string(file_path).map_err(SemanticError::FileSystem)?;
        let chunks = self.parser.parse_file(file_path, &content)?;
        report.chunks_parsed = chunks.len();

        if chunks.is_empty() {
            tracing::warn!("No chunks extracted from file: {}", file_path.display());
            report.success = true; // Not an error, just no content
            return Ok(report);
        }

        // Generate embeddings for chunks
        let embeddings = self.embedding_engine.embed_chunks_batch(&chunks).await?;
        report.embeddings_generated = embeddings.len();

        // Store chunks and embeddings
        for chunk in &chunks {
            self.storage
                .store_chunk(chunk)
                .map_err(|e| SemanticError::Index(e.to_string()))?;
        }

        for embedding in &embeddings {
            self.storage
                .store_embedding(embedding)
                .map_err(|e| SemanticError::Index(e.to_string()))?;
        }

        // Store file metadata
        let file_metadata = self.create_file_metadata(file_path, &chunks)?;
        self.storage
            .store_indexed_file(&file_metadata)
            .map_err(|e| SemanticError::Index(e.to_string()))?;

        report.success = true;
        Ok(report)
    }

    /// Create file metadata for storage
    fn create_file_metadata(&self, file_path: &Path, chunks: &[CodeChunk]) -> Result<IndexedFile> {
        let language = self.parser.detect_language(file_path);
        let content_hash = FileHasher::hash_file(file_path).map_err(|e| {
            SemanticError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        let file_id = FileId(file_path.to_string_lossy().to_string());

        Ok(IndexedFile {
            file_id,
            path: file_path.to_path_buf(),
            language,
            content_hash,
            chunk_count: chunks.len(),
            indexed_at: Utc::now(),
        })
    }

    /// Index files in smaller batches to manage memory usage
    pub async fn index_files_in_batches(
        &mut self,
        file_paths: Vec<PathBuf>,
        batch_size: usize,
        force_reindex: bool,
    ) -> Result<IndexingReport> {
        let mut overall_report = IndexingReport::new();
        let start_time = std::time::Instant::now();

        for (batch_num, batch) in file_paths.chunks(batch_size).enumerate() {
            tracing::info!(
                "Processing batch {} with {} files",
                batch_num + 1,
                batch.len()
            );

            let batch_report = self.index_files(batch.to_vec(), force_reindex).await?;

            // Merge reports
            overall_report.files_processed += batch_report.files_processed;
            overall_report.files_successful += batch_report.files_successful;
            overall_report.files_failed += batch_report.files_failed;
            overall_report.total_chunks += batch_report.total_chunks;
            overall_report.total_embeddings += batch_report.total_embeddings;
            overall_report.errors.extend(batch_report.errors);

            // Optional: garbage collection between batches
            if batch_num % 10 == 0 {
                tracing::debug!("Running garbage collection after batch {}", batch_num + 1);
                // Force garbage collection to manage memory
                std::hint::black_box(());
            }
        }

        overall_report.duration = start_time.elapsed();
        Ok(overall_report)
    }

    /// Re-index only files that have changed
    pub async fn incremental_index(&mut self, pattern: &str) -> Result<IndexingReport> {
        self.index_glob(pattern, false).await
    }

    /// Force re-index all files matching pattern
    pub async fn full_reindex(&mut self, pattern: &str) -> Result<IndexingReport> {
        self.index_glob(pattern, true).await
    }

}

/// Statistics from an indexing operation
#[derive(Debug, Clone, Default)]
pub struct IndexingStats {
    /// Number of files that were successfully processed and indexed
    pub processed_files: usize,
    /// Number of files that were skipped (e.g., no changes detected)
    pub skipped_files: usize,
    /// Number of files that failed to process due to errors
    pub failed_files: usize,
    /// Total number of code chunks generated from processed files
    pub total_chunks: usize,
}

/// Enhanced reporting structure for indexing operations
#[derive(Debug)]
pub struct IndexingReport {
    /// Total number of files processed
    pub files_processed: usize,
    /// Number of files successfully indexed
    pub files_successful: usize,
    /// Number of files that failed to index
    pub files_failed: usize,
    /// Total number of code chunks generated
    pub total_chunks: usize,
    /// Total number of embeddings generated
    pub total_embeddings: usize,
    /// List of errors encountered during indexing
    pub errors: Vec<(PathBuf, crate::error::SwissArmyHammerError)>,
    /// Total duration of the indexing operation
    pub duration: std::time::Duration,
}

impl IndexingReport {
    /// Create a new empty indexing report
    pub fn new() -> Self {
        Self {
            files_processed: 0,
            files_successful: 0,
            files_failed: 0,
            total_chunks: 0,
            total_embeddings: 0,
            errors: Vec::new(),
            duration: std::time::Duration::from_secs(0),
        }
    }

    /// Create an empty indexing report (alias for `new`)
    pub fn empty() -> Self {
        Self::new()
    }

    /// Merge a single file report into this overall report
    pub fn merge(&mut self, other: SingleFileReport) {
        self.files_processed += 1;
        if other.success {
            self.files_successful += 1;
        } else {
            self.files_failed += 1;
        }
        self.total_chunks += other.chunks_parsed;
        self.total_embeddings += other.embeddings_generated;
    }

    /// Add an error for a failed file
    pub fn add_error(&mut self, file_path: PathBuf, error: crate::error::SwissArmyHammerError) {
        self.files_processed += 1;
        self.files_failed += 1;
        self.errors.push((file_path, error));
    }

    /// Get a summary string of the indexing results
    pub fn summary(&self) -> String {
        format!(
            "Processed {} files ({} successful, {} failed), {} chunks, {} embeddings",
            self.files_processed,
            self.files_successful,
            self.files_failed,
            self.total_chunks,
            self.total_embeddings
        )
    }
}

impl Default for IndexingReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Report for a single file indexing operation
#[derive(Debug)]
pub struct SingleFileReport {
    #[allow(dead_code)]
    file_path: PathBuf,
    success: bool,
    chunks_parsed: usize,
    embeddings_generated: usize,
}

impl SingleFileReport {
    /// Create a new single file report for the given file path
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            success: false,
            chunks_parsed: 0,
            embeddings_generated: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ParserConfig, SemanticConfig, VectorStorage};
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_indexer() -> Result<(FileIndexer, TempDir)> {
        let temp_dir = TempDir::new().map_err(SemanticError::FileSystem)?;
        let db_name = format!("test_{}.db", std::process::id());
        let config = SemanticConfig {
            database_path: temp_dir.path().join(db_name),
            ..Default::default()
        };

        // Use permissive parser config for tests to avoid chunk size filtering
        let parser_config = ParserConfig {
            min_chunk_size: 1,
            max_chunk_size: 10000,
            max_chunks_per_file: 1000,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let embedding_service = EmbeddingEngine::new_for_testing().await?;
        let storage =
            VectorStorage::new(config).map_err(|e| SemanticError::Index(e.to_string()))?;

        let indexer =
            FileIndexer::with_custom_config(storage, embedding_service, parser_config).await?;
        Ok((indexer, temp_dir))
    }

    #[tokio::test]
    async fn test_indexer_creation() {
        let result = create_test_indexer().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_index_empty_directory() {
        let (mut indexer, _temp_dir) = create_test_indexer().await.unwrap();

        let report = indexer.index_files(vec![], false).await;
        assert!(report.is_ok());

        let report = report.unwrap();
        assert_eq!(report.files_processed, 0);
        assert_eq!(report.files_successful, 0);
        assert_eq!(report.files_failed, 0);
    }

    #[tokio::test]
    async fn test_index_single_rust_file() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create a test Rust file
        let test_file = temp_dir.path().join("test.rs");
        let content = "fn main() { println!(\"Hello, world!\"); }";
        fs::write(&test_file, content).unwrap();

        // Test indexing

        let report = indexer.index_files(vec![test_file], false).await;
        assert!(report.is_ok());

        let report = report.unwrap();
        assert_eq!(report.files_processed, 1);
        assert_eq!(report.total_chunks, 1);
    }


    #[tokio::test]
    async fn test_new_index_glob_api() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create test files
        std::fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        std::fs::write(temp_dir.path().join("lib.rs"), "pub fn hello() {}").unwrap();

        // Test glob pattern with new API
        let pattern = format!("{}/*.rs", temp_dir.path().display());
        let report = indexer.index_glob(&pattern, false).await.unwrap();

        assert_eq!(report.files_successful, 2);
        assert_eq!(report.files_failed, 0);
        assert!(report.total_chunks > 0);
    }

    #[tokio::test]
    async fn test_incremental_vs_full_reindex() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create test file
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();
        let pattern = format!("{}/*.rs", temp_dir.path().display());

        // Initial index
        let report1 = indexer.incremental_index(&pattern).await.unwrap();
        assert_eq!(report1.files_successful, 1);

        // Incremental index should find no changes
        let report2 = indexer.incremental_index(&pattern).await.unwrap();
        assert_eq!(report2.files_successful, 0); // No changes detected - fixed change tracking bug

        // Force reindex should reindex everything
        let report3 = indexer.full_reindex(&pattern).await.unwrap();
        assert_eq!(report3.files_successful, 1); // Forced reindex
    }

    #[tokio::test]
    async fn test_index_with_glob_pattern() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.py"), "def main(): pass").unwrap();

        // Test indexing specific files with new API
        let rust_file = temp_dir.path().join("test.rs");
        let python_file = temp_dir.path().join("test.py");

        // Index only Rust file
        let report = indexer.index_files(vec![rust_file], false).await.unwrap();
        assert_eq!(report.files_processed, 1); // Only test.rs should be processed

        // Create a fresh indexer for the second test to avoid "already indexed" issues
        let (mut indexer2, _) = create_test_indexer().await.unwrap();

        // Index only Python file
        let report = indexer2
            .index_files(vec![python_file], false)
            .await
            .unwrap();
        assert_eq!(report.files_processed, 1); // Only test.py should be processed
    }
}
