# TP_000198: File Hashing and Change Detection

## Goal
Implement MD5-based file content hashing for smart re-indexing to avoid re-embedding files that haven't changed.

## Context
The specification requires "smart" indexing with MD5 content hashing to avoid re-embedding files that haven't changed. This is a critical performance optimization for large codebases.

## Specification Requirements
- Use MD5 content hashing to avoid re-embedding unchanged files
- Smart indexing: if a file changed, replace any indexed chunks of that file in the index

## Tasks

### 1. Create FileHasher utility in `semantic/utils.rs`

```rust
use crate::semantic::{Result, SemanticError, ContentHash};
use std::path::Path;
use std::fs;
use md5::{Md5, Digest};

pub struct FileHasher;

impl FileHasher {
    /// Calculate MD5 hash of file content
    pub fn hash_file(path: impl AsRef<Path>) -> Result<ContentHash> {
        let path = path.as_ref();
        let content = fs::read(path)?;
        let hash = Self::hash_content(&content);
        Ok(hash)
    }
    
    /// Calculate MD5 hash of content bytes
    pub fn hash_content(content: &[u8]) -> ContentHash {
        let mut hasher = Md5::new();
        hasher.update(content);
        let result = hasher.finalize();
        ContentHash(format!("{:x}", result))
    }
    
    /// Calculate hash of string content (for testing)
    pub fn hash_string(content: &str) -> ContentHash {
        Self::hash_content(content.as_bytes())
    }
}
```

### 2. Create FileChangeTracker in `semantic/utils.rs`

```rust
use crate::semantic::{VectorStorage, Result, ContentHash};
use std::path::Path;
use std::collections::HashMap;

pub struct FileChangeTracker {
    storage: VectorStorage,
}

impl FileChangeTracker {
    pub fn new(storage: VectorStorage) -> Self {
        Self { storage }
    }
    
    /// Check multiple files for changes and return those that need re-indexing
    pub fn check_files_for_changes<P: AsRef<Path>>(
        &self, 
        file_paths: impl IntoIterator<Item = P>
    ) -> Result<FileChangeReport> {
        let mut report = FileChangeReport::new();
        
        for path in file_paths {
            let path = path.as_ref();
            match self.check_single_file(path) {
                Ok(status) => {
                    report.add_file_status(path.to_path_buf(), status);
                }
                Err(e) => {
                    tracing::warn!("Failed to check file {}: {}", path.display(), e);
                    report.add_error(path.to_path_buf(), e);
                }
            }
        }
        
        Ok(report)
    }
    
    fn check_single_file(&self, path: &Path) -> Result<FileChangeStatus> {
        // Calculate current hash
        let current_hash = FileHasher::hash_file(path)?;
        
        // Check if file needs re-indexing
        let needs_reindexing = self.storage.needs_reindexing(path, &current_hash)?;
        
        Ok(match needs_reindexing {
            true => FileChangeStatus::Changed { 
                new_hash: current_hash,
                exists_in_index: self.storage.file_exists(path)?,
            },
            false => FileChangeStatus::Unchanged { hash: current_hash },
        })
    }
}

#[derive(Debug)]
pub enum FileChangeStatus {
    Changed { 
        new_hash: ContentHash,
        exists_in_index: bool,
    },
    Unchanged { 
        hash: ContentHash 
    },
}

#[derive(Debug)]
pub struct FileChangeReport {
    pub changed_files: HashMap<PathBuf, ContentHash>,
    pub unchanged_files: HashMap<PathBuf, ContentHash>,
    pub new_files: HashMap<PathBuf, ContentHash>,
    pub errors: HashMap<PathBuf, SemanticError>,
}

impl FileChangeReport {
    fn new() -> Self {
        Self {
            changed_files: HashMap::new(),
            unchanged_files: HashMap::new(),
            new_files: HashMap::new(),
            errors: HashMap::new(),
        }
    }
    
    fn add_file_status(&mut self, path: PathBuf, status: FileChangeStatus) {
        match status {
            FileChangeStatus::Changed { new_hash, exists_in_index } => {
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
    
    fn add_error(&mut self, path: PathBuf, error: SemanticError) {
        self.errors.insert(path, error);
    }
    
    pub fn files_needing_indexing(&self) -> impl Iterator<Item = &PathBuf> {
        self.changed_files.keys().chain(self.new_files.keys())
    }
    
    pub fn total_files(&self) -> usize {
        self.changed_files.len() + self.unchanged_files.len() + 
        self.new_files.len() + self.errors.len()
    }
    
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
```

### 3. Add VectorStorage support methods

Add these methods to `VectorStorage` in `semantic/storage.rs`:

```rust
impl VectorStorage {
    /// Check if file exists in index
    pub fn file_exists(&self, file_path: &Path) -> Result<bool> {
        let conn = self.get_connection()?;
        
        let count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM indexed_files WHERE path = ?",
            params![file_path.to_string_lossy()],
            |row| row.get(0)
        )?;
        
        Ok(count > 0)
    }
    
    /// Get file hash from index
    pub fn get_file_hash(&self, file_path: &Path) -> Result<Option<ContentHash>> {
        let conn = self.get_connection()?;
        
        let hash: Option<String> = conn.query_row(
            "SELECT content_hash FROM indexed_files WHERE path = ?",
            params![file_path.to_string_lossy()],
            |row| row.get(0)
        ).optional()?;
        
        Ok(hash.map(ContentHash))
    }
    
    /// Get statistics about indexed files
    pub fn get_index_stats(&self) -> Result<IndexStats> {
        let conn = self.get_connection()?;
        
        let file_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM indexed_files",
            [],
            |row| row.get(0)
        )?;
        
        let chunk_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM code_chunks",
            [],
            |row| row.get(0)
        )?;
        
        let embedding_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM embeddings",
            [],
            |row| row.get(0)
        )?;
        
        Ok(IndexStats {
            file_count: file_count as usize,
            chunk_count: chunk_count as usize,
            embedding_count: embedding_count as usize,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IndexStats {
    pub file_count: usize,
    pub chunk_count: usize,
    pub embedding_count: usize,
}
```

### 4. Batch processing utilities

```rust
pub struct BatchProcessor {
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self { batch_size }
    }
    
    /// Process files in batches to avoid memory issues
    pub fn process_files_in_batches<F, T>(
        &self,
        files: Vec<T>,
        mut processor: F,
    ) -> Result<()>
    where
        F: FnMut(&[T]) -> Result<()>,
        T: Clone,
    {
        for chunk in files.chunks(self.batch_size) {
            processor(chunk)?;
            
            // Optional: add progress logging
            tracing::debug!("Processed batch of {} files", chunk.len());
        }
        
        Ok(())
    }
}
```

## Acceptance Criteria
- [ ] FileHasher correctly calculates MD5 hashes for file content
- [ ] FileChangeTracker efficiently detects which files need re-indexing
- [ ] FileChangeReport provides clear summary of file status
- [ ] VectorStorage supports file existence and hash queries
- [ ] Batch processing handles large numbers of files efficiently
- [ ] Error handling gracefully manages file access issues
- [ ] Performance is optimized for large codebases

## Architecture Notes
- MD5 is used for speed over cryptographic security (content change detection only)
- FileChangeTracker batches database queries for efficiency
- FileChangeReport separates new, changed, and unchanged files for different processing
- Batch processing prevents memory issues with large codebases

## Testing Strategy
- Test MD5 hash consistency across identical file content
- Test change detection with file modifications
- Test batch processing with large file sets
- Test error handling with inaccessible files

## Next Steps
After completion, proceed to TP_000199_treesitter-parser to implement TreeSitter-based code parsing.