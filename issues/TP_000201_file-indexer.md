# TP_000201: File Indexer Implementation

## Goal
Implement the complete file indexing pipeline that orchestrates parsing, embedding, and storage.

## Context
This component brings together all previous components (file hashing, TreeSitter parsing, embedding generation, and DuckDB storage) into a complete indexing system that can process files and globs efficiently.

## Tasks

### 1. Create FileIndexer in `semantic/indexer.rs`

```rust
use crate::semantic::{
    Result, SemanticError, VectorStorage, EmbeddingEngine, CodeParser, 
    FileChangeTracker, FileHasher, CodeChunk, IndexedFile, Language, FileId, ContentHash
};
use std::path::{Path, PathBuf};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use chrono::Utc;

pub struct FileIndexer {
    storage: VectorStorage,
    embedding_engine: EmbeddingEngine,
    parser: CodeParser,
    change_tracker: FileChangeTracker,
}

impl FileIndexer {
    pub async fn new(storage: VectorStorage) -> Result<Self> {
        let embedding_engine = EmbeddingEngine::new().await?;
        let parser = CodeParser::new()?;
        let change_tracker = FileChangeTracker::new(storage.clone());
        
        Ok(Self {
            storage,
            embedding_engine,
            parser,
            change_tracker,
        })
    }
    
    pub async fn with_custom_embedding_engine(
        storage: VectorStorage,
        embedding_engine: EmbeddingEngine,
    ) -> Result<Self> {
        let parser = CodeParser::new()?;
        let change_tracker = FileChangeTracker::new(storage.clone());
        
        Ok(Self {
            storage,
            embedding_engine,
            parser,
            change_tracker,
        })
    }
}
```

### 2. Glob Pattern Processing

```rust
impl FileIndexer {
    /// Index files matching a glob pattern
    pub async fn index_glob(&mut self, pattern: &str, force_reindex: bool) -> Result<IndexingReport> {
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
    
    fn expand_glob_pattern(&self, pattern: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        for entry in glob(pattern).map_err(|e| {
            SemanticError::Config(format!("Invalid glob pattern '{}': {}", pattern, e))
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
    
    fn is_supported_file(&self, path: &Path) -> bool {
        let language = CodeParser::detect_language(path);
        !matches!(language, Language::Unknown)
    }
    
    async fn filter_changed_files(&self, paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let change_report = self.change_tracker.check_files_for_changes(paths).await?;
        
        tracing::info!("{}", change_report.summary()); 
        
        Ok(change_report.files_needing_indexing().cloned().collect())
    }
}
```

### 3. Core Indexing Logic

```rust
impl FileIndexer {
    /// Index a list of files
    pub async fn index_files(
        &mut self, 
        file_paths: Vec<PathBuf>, 
        force_reindex: bool
    ) -> Result<IndexingReport> {
        let mut report = IndexingReport::new();
        
        // Setup progress bar
        let progress = ProgressBar::new(file_paths.len() as u64);
        progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-")
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
                    report.add_error(file_path, e);
                }
            }
            
            progress.inc(1);
        }
        
        progress.finish_with_message("Indexing complete");
        
        tracing::info!("Indexing report: {}", report.summary());
        Ok(report)
    }
    
    async fn index_single_file(&mut self, file_path: &Path, force_reindex: bool) -> Result<SingleFileReport> {
        let mut report = SingleFileReport::new(file_path.to_path_buf());
        
        // Remove existing data if force re-indexing
        if force_reindex {
            self.storage.remove_file(file_path)?;
        }
        
        // Parse file into chunks
        let chunks = self.parser.parse_file(file_path)?;
        report.chunks_parsed = chunks.len();
        
        if chunks.is_empty() {
            tracing::warn!("No chunks extracted from file: {}", file_path.display());
            return Ok(report);
        }
        
        // Generate embeddings for chunks
        let embeddings = self.embedding_engine.embed_chunks_batch(&chunks).await?;
        report.embeddings_generated = embeddings.len();
        
        // Store chunks and embeddings
        for chunk in &chunks {
            self.storage.store_chunk(chunk)?;
        }
        
        for embedding in &embeddings {
            self.storage.store_embedding(embedding)?;
        }
        
        // Store file metadata
        let file_metadata = self.create_file_metadata(file_path, &chunks)?;
        self.storage.store_indexed_file(&file_metadata)?;
        
        report.success = true;
        Ok(report)
    }
    
    fn create_file_metadata(&self, file_path: &Path, chunks: &[CodeChunk]) -> Result<IndexedFile> {
        let language = CodeParser::detect_language(file_path);
        let content_hash = FileHasher::hash_file(file_path)?;
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
}
```

### 4. Reporting Structures

```rust
#[derive(Debug, Clone)]
pub struct IndexingReport {
    pub files_processed: usize,
    pub files_successful: usize,
    pub files_failed: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub errors: Vec<(PathBuf, SemanticError)>,
    pub duration: std::time::Duration,
}

impl IndexingReport {
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
    
    pub fn empty() -> Self {
        Self::new()
    }
    
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
    
    pub fn add_error(&mut self, file_path: PathBuf, error: SemanticError) {
        self.files_processed += 1;
        self.files_failed += 1;
        self.errors.push((file_path, error));
    }
    
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

#[derive(Debug)]
struct SingleFileReport {
    file_path: PathBuf,
    success: bool,
    chunks_parsed: usize,
    embeddings_generated: usize,
}

impl SingleFileReport {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            success: false,
            chunks_parsed: 0,
            embeddings_generated: 0,
        }
    }
}
```

### 5. Batch and Incremental Indexing

```rust
impl FileIndexer {
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
            tracing::info!("Processing batch {} with {} files", batch_num + 1, batch.len());
            
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
```

## Acceptance Criteria
- [ ] FileIndexer successfully orchestrates all components
- [ ] Glob pattern expansion works for complex patterns
- [ ] Change detection prevents unnecessary re-indexing
- [ ] Progress reporting provides clear feedback
- [ ] Error handling allows partial failures without stopping
- [ ] Batch processing manages memory usage
- [ ] Force re-indexing option works correctly
- [ ] Performance is reasonable for large codebases

## Architecture Notes
- Orchestrates all previous components into complete pipeline
- Progress bars provide user feedback during long operations
- Batch processing prevents memory issues with large codebases
- Error handling is permissive - individual file failures don't stop entire operation
- Change detection optimizes performance by skipping unchanged files

## Testing Strategy
- Test with various glob patterns
- Test incremental vs full indexing
- Test error handling with malformed files
- Performance testing with large codebases
- Memory usage testing with batch processing

## Next Steps
After completion, proceed to TP_000202_semantic-searcher to implement the query/search functionality.

## Proposed Solution

After analyzing the current codebase, I found that most of the FileIndexer functionality is already implemented! The existing `semantic/indexer.rs` contains:

✅ **Already Implemented:**
- FileIndexer struct with storage, embedding_engine, parser, change_tracker
- `new()` and `with_custom_embedding_engine()` constructors  
- `index_glob()` method with glob pattern expansion
- `expand_glob_pattern()` and `is_supported_file()` methods
- Change detection via `filter_changed_files()` 
- `index_files_with_report()` with progress bars
- `index_single_file_with_report()` with comprehensive reporting
- `create_file_metadata()` with proper IndexedFile creation
- `index_files_in_batches()` for memory management
- `incremental_index()` and `full_reindex()` convenience methods
- IndexingReport and SingleFileReport structures with all required fields and methods
- Comprehensive test coverage

✅ **Supporting Components Available:**
- FileHasher in utils.rs with hash_file() method
- FileChangeTracker in utils.rs with check_files_for_changes() method  
- VectorStorage with store_chunk(), store_embedding(), store_indexed_file() methods
- EmbeddingEngine with embed_chunks_batch() method
- CodeParser with parse_file() and detect_language() methods

🔍 **Missing Components to Complete Issue:**
1. Update constructor to match exact issue specification (async `new()` method)
2. Add missing `embed_chunks_batch()` method to EmbeddingEngine
3. Ensure SemanticError is properly aliased to Error  
4. Add any missing storage methods (remove_file, etc.)
5. Run tests to verify everything works correctly

**Implementation Plan:**
1. Check and update EmbeddingEngine to support `embed_chunks_batch()` method
2. Ensure VectorStorage has all required methods (remove_file, etc.)
3. Update constructor signatures to match issue specification exactly  
4. Add comprehensive testing to verify the complete pipeline works
5. Test with real files and glob patterns to ensure integration works

The architecture is sound and most code is implemented correctly following the coding standards. Main focus will be on ensuring API compatibility and testing the complete pipeline.