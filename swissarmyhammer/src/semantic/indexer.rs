//! File indexing logic for semantic search

use crate::error::Result;
use crate::semantic::{CodeParser, Embedding, EmbeddingEngine, VectorStorage};
use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

/// File indexer that processes source files for semantic search
pub struct FileIndexer {
    parser: CodeParser,
    embedding_service: EmbeddingEngine,
    storage: VectorStorage,
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
    /// Create a new file indexer with TreeSitter parsing capabilities.
    ///
    /// Combines a TreeSitter-based code parser, embedding service, and vector storage
    /// to create a complete semantic search indexing pipeline.
    ///
    /// # Components
    /// - `parser`: TreeSitter-based parser that extracts semantic chunks from source code
    /// - `embedding_service`: Service for generating vector embeddings from text chunks
    /// - `storage`: Vector database for storing code chunks and their embeddings
    ///
    /// # Arguments
    /// * `parser` - Configured `CodeParser` with TreeSitter support for target languages
    /// * `embedding_service` - Service for generating vector embeddings from code chunks
    /// * `storage` - Vector storage backend for persisting chunks and embeddings
    ///
    /// # Returns
    /// A new `FileIndexer` ready to process source files
    pub fn new(
        parser: CodeParser,
        embedding_service: EmbeddingEngine,
        storage: VectorStorage,
    ) -> Self {
        Self {
            parser,
            embedding_service,
            storage,
        }
    }

    /// Index source files using TreeSitter parsing and semantic embeddings.
    ///
    /// Recursively walks the directory tree from `root_path`, processes supported source files
    /// with TreeSitter parsing to extract semantic chunks, generates vector embeddings,
    /// and stores everything in the vector database for semantic search.
    ///
    /// # Processing Pipeline
    /// 1. **File Discovery**: Walk directory tree, filter by glob patterns and file types
    /// 2. **TreeSitter Parsing**: Extract semantic chunks (functions, classes, methods) from source
    /// 3. **Embedding Generation**: Create vector embeddings for each code chunk
    /// 4. **Storage**: Persist chunks and embeddings in vector database
    /// 5. **Statistics**: Track processed, skipped, and failed files
    ///
    /// # Supported Languages
    /// Only files with TreeSitter parser support are processed:
    /// - Rust (`.rs`)
    /// - Python (`.py`, `.pyx`, `.pyi`)
    /// - TypeScript (`.ts`, `.tsx`)
    /// - JavaScript (`.js`, `.jsx`, `.mjs`)
    /// - Dart (`.dart`)
    ///
    /// # Filtering and Limits
    /// - **Glob Patterns**: Include only files matching optional glob pattern
    /// - **Change Detection**: Skip files already indexed (unless `force: true`)
    /// - **File Limits**: Stop after processing `max_files` if specified
    /// - **Chunk Filtering**: Apply parser configuration limits for chunk size and count
    ///
    /// # Error Handling
    /// Individual file failures are logged but don't stop the indexing process.
    /// Failed files are counted in statistics but don't cause the operation to fail.
    ///
    /// # Arguments
    /// * `root_path` - Root directory to start recursive file discovery
    /// * `options` - Configuration for glob patterns, file limits, and force re-indexing
    ///
    /// # Returns
    /// `IndexingStats` with counts of processed, skipped, and failed files plus total chunks
    ///
    /// # Errors
    /// Returns error only if directory walking fails or database operations fail
    pub async fn index_files(
        &mut self,
        root_path: &Path,
        options: &IndexingOptions,
    ) -> Result<IndexingStats> {
        let mut stats = IndexingStats::default();

        for entry in WalkDir::new(root_path).into_iter() {
            let entry = entry.map_err(|e| {
                crate::error::SwissArmyHammerError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Walk error: {e}"),
                ))
            })?;

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
            match self.index_single_file(path).await {
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

    /// Index a single file with performance metrics.
    ///
    /// Tracks timing for file reading, parsing, embedding generation, and storage operations.
    async fn index_single_file(&mut self, file_path: &Path) -> Result<usize> {
        let total_start = std::time::Instant::now();

        // Read file content
        let read_start = std::time::Instant::now();
        let content =
            std::fs::read_to_string(file_path).map_err(crate::error::SwissArmyHammerError::Io)?;
        let read_duration = read_start.elapsed();
        let file_size = content.len();

        // Parse into chunks
        let parse_start = std::time::Instant::now();
        let chunks = self.parser.parse_file(file_path, &content)?;
        let parse_duration = parse_start.elapsed();

        // Generate embeddings for chunks
        let embed_start = std::time::Instant::now();
        let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embedding_vectors = self.embedding_service.embed_batch(&chunk_texts).await?;
        let embed_duration = embed_start.elapsed();

        // Create embedding objects
        let embeddings: Vec<Embedding> = chunks
            .iter()
            .zip(embedding_vectors)
            .map(|(chunk, vector)| Embedding {
                chunk_id: chunk.id.clone(),
                vector,
            })
            .collect();

        // Store chunks and embeddings
        let store_start = std::time::Instant::now();
        let chunk_count = chunks.len();
        for chunk in chunks {
            self.storage.store_chunk(&chunk)?;
        }

        for embedding in embeddings {
            self.storage.store_embedding(&embedding)?;
        }
        let store_duration = store_start.elapsed();

        let total_duration = total_start.elapsed();

        // Log comprehensive indexing metrics
        tracing::info!(
            "Indexed file: {} | {} bytes | {} chunks | total: {:.2}ms | read: {:.2}ms | parse: {:.2}ms | embed: {:.2}ms | store: {:.2}ms",
            file_path.display(),
            file_size,
            chunk_count,
            total_duration.as_secs_f64() * 1000.0,
            read_duration.as_secs_f64() * 1000.0,
            parse_duration.as_secs_f64() * 1000.0,
            embed_duration.as_secs_f64() * 1000.0,
            store_duration.as_secs_f64() * 1000.0
        );

        Ok(chunk_count)
    }

    /// Check if a path matches a glob pattern
    fn matches_glob(&self, path: &Path, pattern: &str) -> bool {
        match self.glob_to_regex(pattern) {
            Ok(regex) => {
                // If pattern contains directory separators, match against full path
                // Otherwise, match against just the filename
                let match_str = if pattern.contains('/') || pattern.contains('\\') {
                    path.to_string_lossy()
                } else {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .into()
                };
                regex.is_match(&match_str)
            }
            Err(e) => {
                tracing::warn!("Invalid glob pattern '{}': {}", pattern, e);
                false
            }
        }
    }

    /// Convert a glob pattern to a regex pattern
    fn glob_to_regex(&self, pattern: &str) -> std::result::Result<Regex, regex::Error> {
        let mut regex_pattern = String::new();
        let mut chars = pattern.chars().peekable();

        // Start with anchor to match from beginning
        regex_pattern.push('^');

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    // Check for ** (match across directory separators)
                    if chars.peek() == Some(&'*') {
                        chars.next(); // consume second *
                        regex_pattern.push_str(".*");
                    } else {
                        // Single * matches everything except directory separator
                        regex_pattern.push_str("[^/\\\\]*");
                    }
                }
                '?' => {
                    // ? matches exactly one character except directory separator
                    regex_pattern.push_str("[^/\\\\]");
                }
                '[' => {
                    // Character class - pass through but escape regex special chars inside
                    regex_pattern.push('[');
                    let mut in_class = true;
                    while let Some(class_ch) = chars.next() {
                        match class_ch {
                            ']' => {
                                regex_pattern.push(']');
                                in_class = false;
                                break;
                            }
                            '\\' => {
                                // Escape the next character
                                regex_pattern.push_str("\\\\");
                                if let Some(escaped) = chars.next() {
                                    regex_pattern.push(escaped);
                                }
                            }
                            _ => {
                                // Regular character in class
                                regex_pattern.push(class_ch);
                            }
                        }
                    }
                    if in_class {
                        // Unclosed bracket - treat as literal
                        regex_pattern.clear();
                        regex_pattern.push_str(&format!("^{}.*", regex::escape(pattern)));
                        break;
                    }
                }
                '{' => {
                    // Brace expansion {a,b,c} -> (a|b|c)
                    regex_pattern.push('(');
                    let mut alternatives = Vec::new();
                    let mut current_alt = String::new();
                    let mut depth = 1;

                    #[allow(clippy::while_let_on_iterator)]
                    while let Some(brace_ch) = chars.next() {
                        match brace_ch {
                            '{' => {
                                depth += 1;
                                current_alt.push(brace_ch);
                            }
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    alternatives.push(current_alt);
                                    break;
                                } else {
                                    current_alt.push(brace_ch);
                                }
                            }
                            ',' if depth == 1 => {
                                alternatives.push(current_alt);
                                current_alt = String::new();
                            }
                            _ => {
                                current_alt.push(brace_ch);
                            }
                        }
                    }

                    if depth != 0 {
                        // Unclosed brace - treat as literal
                        regex_pattern.clear();
                        regex_pattern.push_str(&format!("^{}.*", regex::escape(pattern)));
                        break;
                    }

                    // Join alternatives with |
                    let escaped_alts: Vec<String> =
                        alternatives.iter().map(|alt| regex::escape(alt)).collect();
                    regex_pattern.push_str(&escaped_alts.join("|"));
                    regex_pattern.push(')');
                }
                '\\' => {
                    // Escape the next character
                    if let Some(escaped) = chars.next() {
                        regex_pattern.push_str(&regex::escape(&escaped.to_string()));
                    } else {
                        regex_pattern.push_str("\\\\");
                    }
                }
                _ => {
                    // Regular character - escape regex special chars
                    regex_pattern.push_str(&regex::escape(&ch.to_string()));
                }
            }
        }

        // End with anchor to match to end
        regex_pattern.push('$');

        Regex::new(&regex_pattern)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ParserConfig, SemanticConfig, VectorStorage};
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_indexer() -> Result<(FileIndexer, TempDir)> {
        let temp_dir = TempDir::new().map_err(crate::error::SwissArmyHammerError::Io)?;
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
        let parser = CodeParser::new(parser_config)?;
        let embedding_service = EmbeddingEngine::new().await?;
        let storage = VectorStorage::new(config)?;

        let indexer = FileIndexer::new(parser, embedding_service, storage);
        Ok((indexer, temp_dir))
    }

    #[tokio::test]
    async fn test_indexer_creation() {
        let result = create_test_indexer().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_index_empty_directory() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();
        let options = IndexingOptions::default();

        let stats = indexer.index_files(temp_dir.path(), &options).await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.processed_files, 0);
        assert_eq!(stats.skipped_files, 0);
        assert_eq!(stats.failed_files, 0);
    }

    #[tokio::test]
    async fn test_index_single_rust_file() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create a test Rust file
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();

        let options = IndexingOptions::default();
        let stats = indexer.index_files(temp_dir.path(), &options).await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.processed_files, 1);
        assert_eq!(stats.total_chunks, 1);
    }

    #[tokio::test]
    async fn test_glob_matching() {
        let (indexer, _temp_dir) = create_test_indexer().await.unwrap();

        // Test basic wildcard matching
        assert!(indexer.matches_glob(Path::new("test.rs"), "*.rs"));
        assert!(indexer.matches_glob(Path::new("test.py"), "*.py"));
        assert!(!indexer.matches_glob(Path::new("test.rs"), "*.py"));

        // Test directory matching
        assert!(indexer.matches_glob(Path::new("src/main.rs"), "src/*.rs"));
        assert!(indexer.matches_glob(Path::new("src/lib.rs"), "src/*.rs"));
        assert!(!indexer.matches_glob(Path::new("tests/main.rs"), "src/*.rs"));

        // Test recursive matching with **
        assert!(indexer.matches_glob(Path::new("src/main.rs"), "**/*.rs"));
        assert!(indexer.matches_glob(Path::new("src/utils/helper.rs"), "**/*.rs"));
        assert!(indexer.matches_glob(Path::new("tests/integration/test.rs"), "**/*.rs"));

        // Test question mark matching
        assert!(indexer.matches_glob(Path::new("test1.rs"), "test?.rs"));
        assert!(indexer.matches_glob(Path::new("testa.rs"), "test?.rs"));
        assert!(!indexer.matches_glob(Path::new("test12.rs"), "test?.rs"));

        // Test character class matching
        assert!(indexer.matches_glob(Path::new("test1.rs"), "test[123].rs"));
        assert!(indexer.matches_glob(Path::new("test2.rs"), "test[123].rs"));
        assert!(!indexer.matches_glob(Path::new("test4.rs"), "test[123].rs"));

        // Test brace expansion
        assert!(indexer.matches_glob(Path::new("test.rs"), "*.{rs,py}"));
        assert!(indexer.matches_glob(Path::new("test.py"), "*.{rs,py}"));
        assert!(!indexer.matches_glob(Path::new("test.js"), "*.{rs,py}"));

        // Test escaping
        assert!(indexer.matches_glob(Path::new("test*.rs"), "test\\*.rs"));
        assert!(!indexer.matches_glob(Path::new("testx.rs"), "test\\*.rs"));
    }

    #[tokio::test]
    async fn test_index_with_glob_pattern() {
        let (mut indexer, temp_dir) = create_test_indexer().await.unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
        fs::write(temp_dir.path().join("test.py"), "def main(): pass").unwrap();
        fs::write(temp_dir.path().join("test.js"), "function main() {}").unwrap();

        // Index only Rust files
        let options = IndexingOptions {
            glob_pattern: Some("*.rs".to_string()),
            ..Default::default()
        };

        let stats = indexer.index_files(temp_dir.path(), &options).await.unwrap();
        assert_eq!(stats.processed_files, 1); // Only test.rs should be processed

        // Create a fresh indexer for the second test to avoid "already indexed" issues
        let (mut indexer2, _) = create_test_indexer().await.unwrap();

        // Index only Python files (JS files are not supported by parser)
        let options = IndexingOptions {
            glob_pattern: Some("*.py".to_string()),
            ..Default::default()
        };

        let stats = indexer2.index_files(temp_dir.path(), &options).await.unwrap();
        assert_eq!(stats.processed_files, 1); // Only test.py should be processed
    }
}
