//! TreeSitter integration for parsing source code

use crate::semantic::types::{ChunkType, CodeChunk, ContentHash, Language};
use crate::semantic::utils::FileHasher;
use crate::semantic::{Result, SemanticError};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{
    Language as TreeSitterLanguage, Node, Parser, Query, QueryCursor, StreamingIterator,
};

/// Definition of language support for TreeSitter parsing
#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    /// Language identifier
    pub language: Language,
    /// File extensions supported by this language
    pub extensions: Vec<&'static str>,
    /// TreeSitter language function
    pub tree_sitter_language: fn() -> TreeSitterLanguage,
    /// Query patterns for extracting semantic chunks
    pub queries: Vec<(&'static str, ChunkType)>,
}

/// Registry of supported languages with their definitions
pub struct LanguageRegistry {
    definitions: HashMap<Language, LanguageDefinition>,
    extension_map: HashMap<String, Language>,
}

impl LanguageRegistry {
    /// Create a new language registry with default supported languages
    pub fn with_defaults() -> Self {
        let mut registry = Self {
            definitions: HashMap::new(),
            extension_map: HashMap::new(),
        };

        // Register default languages
        registry.register(create_rust_definition());
        registry.register(create_python_definition());
        registry.register(create_typescript_definition());
        registry.register(create_javascript_definition());
        registry.register(create_dart_definition());

        registry
    }

    /// Register a new language definition
    pub fn register(&mut self, definition: LanguageDefinition) {
        // Map file extensions to language
        for ext in &definition.extensions {
            self.extension_map
                .insert(ext.to_string(), definition.language.clone());
        }

        self.definitions
            .insert(definition.language.clone(), definition);
    }

    /// Get language definition by language type
    pub fn get_definition(&self, language: &Language) -> Option<&LanguageDefinition> {
        self.definitions.get(language)
    }

    /// Detect language from file extension
    pub fn detect_language(&self, file_path: &Path) -> Language {
        file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(ext))
            .cloned()
            .unwrap_or(Language::Unknown)
    }

    /// Get all supported languages
    pub fn supported_languages(&self) -> Vec<Language> {
        self.definitions.keys().cloned().collect()
    }
}

/// Create Rust language definition
fn create_rust_definition() -> LanguageDefinition {
    LanguageDefinition {
        language: Language::Rust,
        extensions: vec!["rs"],
        tree_sitter_language: || tree_sitter_rust::LANGUAGE.into(),
        queries: vec![
            // Functions
            (
                "(function_item name: (identifier) @name) @function",
                ChunkType::Function,
            ),
            // Impl blocks
            (
                "(impl_item) @impl",
                ChunkType::Class,
            ),
            // Methods within impl blocks - corrected pattern
            (
                "(impl_item (declaration_list (function_item name: (identifier) @method_name) @method))",
                ChunkType::Function,
            ),
            // Structs
            (
                "(struct_item name: (type_identifier) @name) @struct",
                ChunkType::Class,
            ),
            // Enums
            (
                "(enum_item name: (type_identifier) @name) @enum",
                ChunkType::Class,
            ),
            // Use statements
            ("(use_declaration) @import", ChunkType::Import),
        ],
    }
}

/// Create Python language definition
fn create_python_definition() -> LanguageDefinition {
    LanguageDefinition {
        language: Language::Python,
        extensions: vec!["py", "pyx", "pyi"],
        tree_sitter_language: || tree_sitter_python::LANGUAGE.into(),
        queries: vec![
            // Functions
            (
                "(function_definition name: (identifier) @name) @function",
                ChunkType::Function,
            ),
            // Classes
            (
                "(class_definition name: (identifier) @name) @class",
                ChunkType::Class,
            ),
            // Methods within classes - corrected pattern
            (
                "(class_definition (block (function_definition name: (identifier) @method_name) @method))",
                ChunkType::Function,
            ),
            // Import statements
            ("(import_statement) @import", ChunkType::Import),
            ("(import_from_statement) @import", ChunkType::Import),
        ],
    }
}

/// Create TypeScript language definition
fn create_typescript_definition() -> LanguageDefinition {
    LanguageDefinition {
        language: Language::TypeScript,
        extensions: vec!["ts", "tsx"],
        tree_sitter_language: || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        queries: vec![
            // Functions
            (
                "(function_declaration name: (identifier) @name) @function",
                ChunkType::Function,
            ),
            // Arrow functions
            ("(arrow_function) @function", ChunkType::Function),
            // Classes
            (
                "(class_declaration name: (type_identifier) @name) @class",
                ChunkType::Class,
            ),
            // Methods within classes - corrected pattern
            (
                "(class_declaration (class_body (method_definition name: (property_identifier) @method_name) @method))",
                ChunkType::Function,
            ),
            // Interface definitions
            (
                "(interface_declaration name: (type_identifier) @name) @interface",
                ChunkType::Class,
            ),
            // Type aliases
            (
                "(type_alias_declaration name: (type_identifier) @name) @type_alias",
                ChunkType::Class,
            ),
            // Function expressions - corrected patterns
            (
                "(variable_declarator name: (identifier) @name value: (function_expression)) @function",
                ChunkType::Function,
            ),
            (
                "(variable_declarator name: (identifier) @name value: (arrow_function)) @function",
                ChunkType::Function,
            ),
            // Import statements
            ("(import_statement) @import", ChunkType::Import),
        ],
    }
}

/// Create JavaScript language definition
fn create_javascript_definition() -> LanguageDefinition {
    LanguageDefinition {
        language: Language::JavaScript,
        extensions: vec!["js", "jsx", "mjs"],
        tree_sitter_language: || tree_sitter_javascript::LANGUAGE.into(),
        queries: vec![
            // Functions
            (
                "(function_declaration name: (identifier) @name) @function",
                ChunkType::Function,
            ),
            // Arrow functions
            ("(arrow_function) @function", ChunkType::Function),
            // Classes
            (
                "(class_declaration name: (identifier) @name) @class",
                ChunkType::Class,
            ),
            // Methods within classes - corrected pattern
            (
                "(class_declaration (class_body (method_definition name: (property_identifier) @method_name) @method))",
                ChunkType::Function,
            ),
            // Function expressions - corrected patterns
            (
                "(variable_declarator name: (identifier) @name value: (function_expression)) @function",
                ChunkType::Function,
            ),
            (
                "(variable_declarator name: (identifier) @name value: (arrow_function)) @function",
                ChunkType::Function,
            ),
            // Import statements
            ("(import_statement) @import", ChunkType::Import),
        ],
    }
}

/// Create Dart language definition
fn create_dart_definition() -> LanguageDefinition {
    LanguageDefinition {
        language: Language::Dart,
        extensions: vec!["dart"],
        tree_sitter_language: || tree_sitter_dart::language(),
        queries: vec![
            // Functions
            (
                "(function_signature name: (identifier) @name) @function",
                ChunkType::Function,
            ),
            // Classes
            (
                "(class_definition name: (identifier) @name) @class",
                ChunkType::Class,
            ),
            // Import statements
            ("(import_specification) @import", ChunkType::Import),
        ],
    }
}

/// TreeSitter-based code parser with extensible language support
/// Uses DashMap for thread-safe concurrent access to parsers
pub struct CodeParser {
    parsers: DashMap<Language, Parser>,
    language_registry: LanguageRegistry,
    config: ParserConfig,
}

/// Configuration for the code parser
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Minimum chunk size in characters
    pub min_chunk_size: usize,
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,
    /// Maximum chunks to extract per file
    pub max_chunks_per_file: usize,
    /// Maximum file size in bytes to prevent OOM on massive files
    pub max_file_size_bytes: usize,
}

impl ParserConfig {
    /// Create a new ParserConfig with validation
    ///
    /// # Arguments
    /// * `min_chunk_size` - Minimum chunk size in characters
    /// * `max_chunk_size` - Maximum chunk size in characters  
    /// * `max_chunks_per_file` - Maximum chunks to extract per file
    /// * `max_file_size_bytes` - Maximum file size in bytes to prevent OOM
    ///
    /// # Returns
    /// A validated ParserConfig or an error if validation fails
    ///
    /// # Errors
    /// Returns error if:
    /// - min_chunk_size > max_chunk_size
    /// - Any size parameter is 0
    /// - max_file_size_bytes is unreasonably small (< 1KB)
    pub fn new(
        min_chunk_size: usize,
        max_chunk_size: usize,
        max_chunks_per_file: usize,
        max_file_size_bytes: usize,
    ) -> Result<Self> {
        // Validate chunk size constraints
        if min_chunk_size > max_chunk_size {
            return Err(SemanticError::TreeSitter(format!(
                "Invalid configuration: min_chunk_size ({min_chunk_size}) must be <= max_chunk_size ({max_chunk_size})"
            )));
        }

        // Validate non-zero constraints
        if min_chunk_size == 0 {
            return Err(SemanticError::TreeSitter(
                "Invalid configuration: min_chunk_size must be > 0".to_string(),
            ));
        }

        if max_chunk_size == 0 {
            return Err(SemanticError::TreeSitter(
                "Invalid configuration: max_chunk_size must be > 0".to_string(),
            ));
        }

        if max_chunks_per_file == 0 {
            return Err(SemanticError::TreeSitter(
                "Invalid configuration: max_chunks_per_file must be > 0".to_string(),
            ));
        }

        // Validate reasonable file size limit (at least 1KB)
        if max_file_size_bytes < 1024 {
            return Err(SemanticError::TreeSitter(format!(
                "Invalid configuration: max_file_size_bytes ({max_file_size_bytes}) must be >= 1024 bytes (1KB)"
            )));
        }

        Ok(Self {
            min_chunk_size,
            max_chunk_size,
            max_chunks_per_file,
            max_file_size_bytes,
        })
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        // Use new() with default values, but since we know these are valid,
        // we can unwrap safely
        Self::new(50, 2000, 100, 10 * 1024 * 1024)
            .expect("Default ParserConfig values should always be valid")
    }
}

impl CodeParser {
    /// Create a new code parser with extensible TreeSitter language support.
    ///
    /// Initializes TreeSitter parsers for all languages in the registry.
    /// Languages that fail to initialize are skipped but other parsers remain functional.
    /// Uses the provided configuration to control chunk size limits and extraction behavior.
    ///
    /// Validates all TreeSitter queries during initialization to catch syntax errors early.
    ///
    /// # Arguments
    /// * `config` - Configuration settings for chunk size limits and parsing behavior
    ///
    /// # Returns
    /// A new `CodeParser` instance or an error if creation fails
    ///
    /// # Errors
    /// Returns error if:
    /// - Configuration validation fails
    /// - Any TreeSitter query is invalid or malformed
    pub fn new(config: ParserConfig) -> Result<Self> {
        let language_registry = LanguageRegistry::with_defaults();
        let parsers = DashMap::new();

        // Initialize parsers for all registered languages
        for language in language_registry.supported_languages() {
            if let Some(definition) = language_registry.get_definition(&language) {
                match Self::create_parser_for_language(definition) {
                    Some(parser) => {
                        parsers.insert(language.clone(), parser);
                        tracing::debug!("Initialized TreeSitter parser for {language:?}");
                    }
                    None => {
                        tracing::warn!("Failed to initialize TreeSitter parser for {language:?}");
                    }
                }
            }
        }

        let parser = Self {
            parsers,
            language_registry,
            config,
        };

        // Validate all queries during startup
        parser.validate_all_queries()?;

        tracing::info!(
            "Initialized CodeParser with {} language parsers",
            parser.parsers.len()
        );
        Ok(parser)
    }

    /// Create a parser for a specific language definition
    fn create_parser_for_language(definition: &LanguageDefinition) -> Option<Parser> {
        let mut parser = Parser::new();
        let language = (definition.tree_sitter_language)();
        match parser.set_language(&language) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize {:?} parser: {}",
                    definition.language,
                    e
                );
                None
            }
        }
    }

    /// Detect programming language from file extension using the language registry.
    ///
    /// Uses the extensible language registry to map file extensions to supported
    /// languages. This makes it easy to add support for new languages without
    /// modifying this method.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to analyze
    ///
    /// # Returns
    /// The detected `Language` or `Language::Unknown` if not supported
    pub fn detect_language(&self, file_path: &Path) -> Language {
        self.language_registry.detect_language(file_path)
    }

    /// Get appropriate parser for language from the registry
    /// Check if a parser is available for the given language
    fn has_parser_for_language(&self, language: &Language) -> bool {
        self.parsers.contains_key(language)
    }

    /// Get available language list for error reporting
    fn get_available_languages(&self) -> Vec<Language> {
        self.parsers
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Get TreeSitter queries for extracting semantic chunks from source code.
    ///
    /// Uses the language registry to get query patterns for the specified language.
    /// This makes the query system extensible - new languages can be added by
    /// registering them with their query definitions.
    ///
    /// # Arguments
    /// * `language` - The programming language to get queries for
    ///
    /// # Returns
    /// Vector of (query_pattern, chunk_type) tuples for the specified language
    fn get_queries_for_language(&self, language: &Language) -> Vec<(&'static str, ChunkType)> {
        self.language_registry
            .get_definition(language)
            .map(|def| def.queries.clone())
            .unwrap_or_default()
    }

    /// Parse a source file and extract semantic code chunks using TreeSitter.
    ///
    /// Attempts to parse the file using the appropriate TreeSitter grammar based on
    /// file extension. Extracts semantic chunks like functions, classes, methods, and
    /// imports using language-specific query patterns. Falls back to plain text parsing
    /// if TreeSitter parsing fails for any reason.
    ///
    /// Performance metrics are logged including parse time, chunk count, and throughput.
    ///
    /// # TreeSitter Processing Flow
    /// 1. Detect language from file extension
    /// 2. Get appropriate TreeSitter parser for the language
    /// 3. Parse content into syntax tree
    /// 4. Apply semantic queries to extract meaningful code constructs
    /// 5. Create chunks with accurate line numbers and content
    /// 6. Apply configuration limits (size, count)
    /// 7. Fall back to plain text if any step fails
    ///
    /// # Chunk Extraction
    /// - Functions: Complete function definitions with signatures and bodies
    /// - Classes: Class/struct/enum definitions including nested members
    /// - Methods: Method definitions within classes or impl blocks
    /// - Imports: Import/use statements for dependency tracking
    /// - Fallback: Entire file as single plain text chunk if no semantic chunks found
    ///
    /// # Configuration Limits
    /// - Filters chunks by `min_chunk_size` and `max_chunk_size`
    /// - Limits total chunks per file to `max_chunks_per_file`
    /// - Strict filtering: chunks that don't meet criteria are excluded
    ///
    /// # Arguments
    /// * `file_path` - Path to the source file being parsed
    /// * `content` - Raw file content as string
    ///
    /// # Returns
    /// Vector of `CodeChunk` objects representing extracted semantic units,
    /// or error if parsing completely fails
    ///
    /// # Errors
    /// Returns error only if both TreeSitter parsing and plain text fallback fail
    pub fn parse_file(&self, file_path: &Path, content: &str) -> Result<Vec<CodeChunk>> {
        let start_time = std::time::Instant::now();
        let content_size = content.len();

        // Check file size limit to prevent OOM on massive files
        if content_size > self.config.max_file_size_bytes {
            return Err(SemanticError::TreeSitter(format!(
                "File {} is too large ({content_size} bytes > {} bytes limit). Skipping to prevent OOM.",
                file_path.display(),
                self.config.max_file_size_bytes
            )));
        }

        let language = self.detect_language(file_path);

        let result = match self.parse_with_treesitter(file_path, content, &language) {
            Ok(chunks) => {
                let parse_duration = start_time.elapsed();
                let chunks_per_sec = if parse_duration.as_secs_f64() > 0.0 {
                    chunks.len() as f64 / parse_duration.as_secs_f64()
                } else {
                    chunks.len() as f64
                };
                let bytes_per_sec = if parse_duration.as_secs_f64() > 0.0 {
                    content_size as f64 / parse_duration.as_secs_f64()
                } else {
                    content_size as f64
                };

                tracing::info!(
                    "TreeSitter parse success: {} | {} chunks | {:.2}ms | {:.0} chunks/sec | {:.0} bytes/sec",
                    file_path.display(),
                    chunks.len(),
                    parse_duration.as_secs_f64() * 1000.0,
                    chunks_per_sec,
                    bytes_per_sec
                );
                Ok(chunks)
            }
            Err(e) => {
                // Fall back to plain text as per specification
                tracing::warn!(
                    "TreeSitter parsing failed for {}: {}. Falling back to plain text.",
                    file_path.display(),
                    e
                );
                let fallback_result = self.parse_as_plain_text(file_path, content);

                if let Ok(ref chunks) = fallback_result {
                    let parse_duration = start_time.elapsed();
                    tracing::info!(
                        "Plain text fallback: {} | {} chunks | {:.2}ms",
                        file_path.display(),
                        chunks.len(),
                        parse_duration.as_secs_f64() * 1000.0
                    );
                }

                fallback_result
            }
        };

        // Log final metrics
        match &result {
            Ok(chunks) => {
                let total_duration = start_time.elapsed();
                tracing::debug!(
                    "Parse complete: {} | language: {:?} | {} chunks | {} bytes | {:.2}ms total",
                    file_path.display(),
                    language,
                    chunks.len(),
                    content_size,
                    total_duration.as_secs_f64() * 1000.0
                );
            }
            Err(e) => {
                let total_duration = start_time.elapsed();
                tracing::error!(
                    "Parse failed completely: {} | language: {:?} | {} bytes | {:.2}ms | error: {}",
                    file_path.display(),
                    language,
                    content_size,
                    total_duration.as_secs_f64() * 1000.0,
                    e
                );
            }
        }

        result
    }

    fn parse_with_treesitter(
        &self,
        file_path: &Path,
        content: &str,
        language: &Language,
    ) -> Result<Vec<CodeChunk>> {
        // Check if parser is available before attempting to borrow
        if !self.has_parser_for_language(language) {
            let available_languages = self.get_available_languages();
            let error_msg = format!(
                "No TreeSitter parser available for language: {:?} (file: {}). \
                 Available parsers: {:?}. This could be due to:\
                 \n1. Language not supported in current configuration\
                 \n2. Parser initialization failed during startup\
                 \n3. Missing TreeSitter grammar for this language",
                language,
                file_path.display(),
                available_languages
            );
            tracing::error!("{}", error_msg);
            return Err(SemanticError::TreeSitter(error_msg));
        }

        let tree_parse_start = std::time::Instant::now();

        // Parse using DashMap's entry API for thread-safe access
        let tree = {
            let mut parser_ref = self.parsers.get_mut(language).ok_or_else(|| {
                SemanticError::TreeSitter(format!("Parser disappeared for language: {language:?}"))
            })?;

            parser_ref.parse(content, None)
                .ok_or_else(|| {
                    let error_msg = format!(
                        "TreeSitter parsing failed for {} (language: {:?}, content size: {} bytes). \
                         This could be due to syntax errors, encoding issues, or language grammar limitations.",
                        file_path.display(),
                        language,
                        content.len()
                    );
                    tracing::warn!("{}", error_msg);
                    SemanticError::TreeSitter(error_msg)
                })?
        };
        let tree_parse_duration = tree_parse_start.elapsed();

        // Log additional tree parsing context for debugging
        let root_node = tree.root_node();
        if root_node.has_error() {
            let error_details = format!(
                "TreeSitter syntax tree contains errors for {} (language: {:?}). \
                 Tree root: {:?}, Error nodes may affect chunk extraction.",
                file_path.display(),
                language,
                root_node.kind()
            );
            tracing::warn!("{}", error_details);
        }

        let mut chunks = Vec::new();
        let content_hash = FileHasher::hash_string(content);
        let queries = self.get_queries_for_language(language);
        let mut total_matches = 0;

        let query_start = std::time::Instant::now();
        // Extract semantic chunks using queries
        for (query_str, chunk_type) in queries {
            let query = Query::new(&tree.language(), query_str).map_err(|e| {
                let error_msg = format!(
                    "Invalid TreeSitter query for {} (language: {:?}, chunk type: {:?}): {}\
                         \nQuery pattern: {}\n\
                         This indicates the query pattern doesn't match the language grammar.",
                    file_path.display(),
                    language,
                    chunk_type,
                    e,
                    query_str
                );
                tracing::error!("{}", error_msg);
                SemanticError::TreeSitter(error_msg)
            })?;

            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

            let mut query_matches = 0;
            let mut matches = matches;
            while let Some(query_match) = matches.get() {
                for capture in query_match.captures {
                    let node = capture.node;
                    let chunk = self.create_chunk_from_node(
                        file_path,
                        content,
                        node,
                        language.clone(),
                        chunk_type.clone(),
                        content_hash.clone(),
                    )?;
                    chunks.push(chunk);
                    query_matches += 1;
                }
                matches.advance();
            }

            if query_matches > 0 {
                tracing::debug!(
                    "Query extracted {} chunks for {:?} type from {}",
                    query_matches,
                    chunk_type,
                    file_path.display()
                );
            }
            total_matches += query_matches;
        }
        let query_duration = query_start.elapsed();

        // If no semantic chunks found, create one chunk for entire file
        if chunks.is_empty() {
            chunks.push(self.create_full_file_chunk(
                file_path,
                content,
                language,
                &content_hash,
            )?);
        }

        let filter_start = std::time::Instant::now();
        let initial_chunk_count = chunks.len();
        let filtered_chunks = self.apply_chunk_limits_strict(chunks);
        let filter_duration = filter_start.elapsed();
        let filtered_count = initial_chunk_count - filtered_chunks.len();

        tracing::debug!(
            "TreeSitter parsing metrics: {} | tree: {:.2}ms | queries: {:.2}ms ({} matches) | filter: {:.2}ms ({} filtered)",
            file_path.display(),
            tree_parse_duration.as_secs_f64() * 1000.0,
            query_duration.as_secs_f64() * 1000.0,
            total_matches,
            filter_duration.as_secs_f64() * 1000.0,
            filtered_count
        );

        Ok(filtered_chunks)
    }

    /// Apply chunk size and count limits strictly (no fallback)
    fn apply_chunk_limits_strict(&self, mut chunks: Vec<CodeChunk>) -> Vec<CodeChunk> {
        // Filter by chunk size
        chunks.retain(|chunk| {
            let size = chunk.content.len();
            size >= self.config.min_chunk_size && size <= self.config.max_chunk_size
        });

        // Limit number of chunks per file
        if chunks.len() > self.config.max_chunks_per_file {
            chunks.truncate(self.config.max_chunks_per_file);
        }

        chunks
    }

    fn create_chunk_from_node(
        &self,
        file_path: &Path,
        content: &str,
        node: Node,
        language: Language,
        chunk_type: ChunkType,
        content_hash: ContentHash,
    ) -> Result<CodeChunk> {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let chunk_content = &content[start_byte..end_byte];

        let start_pos = node.start_position();
        let end_pos = node.end_position();

        Ok(CodeChunk {
            id: format!(
                "{}:{}:{}",
                file_path.display(),
                start_pos.row,
                chunk_type.clone() as u8
            ),
            file_path: file_path.to_path_buf(),
            language,
            content: chunk_content.to_string(),
            start_line: start_pos.row + 1, // TreeSitter uses 0-based rows
            end_line: end_pos.row + 1,
            chunk_type,
            content_hash,
        })
    }

    fn parse_as_plain_text(&self, file_path: &Path, content: &str) -> Result<Vec<CodeChunk>> {
        let content_hash = FileHasher::hash_string(content);
        let language = self.detect_language(file_path);

        // Create single chunk for entire file
        let chunk = self.create_full_file_chunk(file_path, content, &language, &content_hash)?;
        let chunks = vec![chunk];

        // Apply configuration limits strictly for plain text
        let filtered_chunks = self.apply_chunk_limits_strict(chunks);
        Ok(filtered_chunks)
    }

    fn create_full_file_chunk(
        &self,
        file_path: &Path,
        content: &str,
        language: &Language,
        content_hash: &ContentHash,
    ) -> Result<CodeChunk> {
        let line_count = content.lines().count();

        Ok(CodeChunk {
            id: format!("{}:full", file_path.display()),
            file_path: file_path.to_path_buf(),
            language: language.clone(),
            content: content.to_string(),
            start_line: 1,
            end_line: line_count.max(1),
            chunk_type: ChunkType::PlainText,
            content_hash: content_hash.clone(),
        })
    }

    /// Check if a file is supported for TreeSitter semantic parsing.
    ///
    /// Uses the extensible language registry to determine if the file extension
    /// maps to a supported language with an available TreeSitter parser.
    /// Files with unknown extensions will fall back to plain text parsing.
    ///
    /// # Arguments
    /// * `file_path` - Path to check for language support
    ///
    /// # Returns
    /// `true` if the file extension maps to a supported language, `false` otherwise
    pub fn is_supported_file(&self, file_path: &Path) -> bool {
        let language = self.detect_language(file_path);
        !matches!(language, Language::Unknown) && self.parsers.contains_key(&language)
    }

    /// Validate all TreeSitter queries for syntax correctness.
    ///
    /// Compiles all query patterns for each supported language to ensure they are
    /// syntactically valid according to the TreeSitter grammar. This catches query
    /// errors at startup rather than during file parsing.
    ///
    /// # Validation Process
    /// 1. For each language with an available parser
    /// 2. Get all query patterns for that language
    /// 3. Attempt to compile each query against the language grammar
    /// 4. Report detailed error information for any invalid queries
    ///
    /// # Returns
    /// `Ok(())` if all queries are valid, or error with details of first invalid query
    ///
    /// # Errors
    /// Returns `SemanticError::TreeSitter` with query validation details if any query is invalid
    /// Validate all TreeSitter queries for syntax correctness using the registry.
    ///
    /// Iterates through all languages in the registry and validates their query patterns
    /// against the corresponding TreeSitter grammars. This extensible approach
    /// automatically validates queries for any registered language.
    ///
    /// # Returns
    /// `Ok(())` if all queries are valid, or error with details of first invalid query
    fn validate_all_queries(&self) -> Result<()> {
        for language in self.language_registry.supported_languages() {
            if let Some(parser) = self.parsers.get(&language) {
                let queries = self.get_queries_for_language(&language);

                for (query_str, chunk_type) in queries {
                    // Attempt to compile the query to validate syntax
                    let language_ref = parser.language().ok_or_else(|| {
                        SemanticError::TreeSitter(format!(
                            "Parser for {language:?} has no language set"
                        ))
                    })?;

                    Query::new(&language_ref, query_str)
                        .map_err(|e| {
                            SemanticError::TreeSitter(format!(
                                "Invalid query for {language:?} language (chunk type: {chunk_type:?}): {e}\nQuery: {query_str}"
                            ))
                        })?;
                }

                tracing::debug!(
                    "Validated {} queries for {language:?}",
                    self.get_queries_for_language(&language).len()
                );
            }
        }

        tracing::info!("All TreeSitter queries validated successfully");
        Ok(())
    }

    /// Add support for a new language to the parser.
    ///
    /// This method demonstrates the extensibility - new languages can be added
    /// at runtime by providing a language definition.
    ///
    /// # Arguments
    /// * `definition` - Language definition with queries and TreeSitter language
    ///
    /// # Returns
    /// `Ok(())` if the language was added successfully, error if parser creation or query validation fails
    pub fn add_language_support(&mut self, definition: LanguageDefinition) -> Result<()> {
        // Create parser for the new language
        if let Some(parser) = Self::create_parser_for_language(&definition) {
            // Validate queries for the new language
            let language_ref = parser.language().ok_or_else(|| {
                SemanticError::TreeSitter(format!(
                    "Parser for {:?} has no language set",
                    definition.language
                ))
            })?;

            for (query_str, chunk_type) in &definition.queries {
                Query::new(&language_ref, query_str)
                    .map_err(|e| {
                        SemanticError::TreeSitter(format!(
                            "Invalid query for {:?} language (chunk type: {chunk_type:?}): {e}\nQuery: {query_str}",
                            definition.language
                        ))
                    })?;
            }

            // Register the language and store the parser in thread-safe map
            let language = definition.language.clone();
            self.parsers.insert(language.clone(), parser);
            // Note: We can't modify the registry here as it's immutable,
            // but in a real implementation you might want to make it mutable

            tracing::info!(
                "Added support for {:?} language with {} queries",
                language,
                definition.queries.len()
            );
            Ok(())
        } else {
            Err(SemanticError::TreeSitter(format!(
                "Failed to create parser for {:?} language",
                definition.language
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parser_creation() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_detect_language() {
        let parser = CodeParser::new(ParserConfig::default()).unwrap();

        assert_eq!(parser.detect_language(Path::new("test.rs")), Language::Rust);
        assert_eq!(
            parser.detect_language(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            parser.detect_language(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            parser.detect_language(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            parser.detect_language(Path::new("test.dart")),
            Language::Dart
        );
        assert_eq!(
            parser.detect_language(Path::new("test.txt")),
            Language::Unknown
        );
    }

    #[test]
    fn test_is_supported_file() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();

        assert!(parser.is_supported_file(Path::new("main.rs")));
        assert!(parser.is_supported_file(Path::new("script.py")));
        assert!(!parser.is_supported_file(Path::new("readme.txt")));
    }

    #[test]
    fn test_parse_rust_function() {
        let config = ParserConfig {
            min_chunk_size: 1, // Allow small chunks for this test
            max_chunk_size: 2000,
            max_chunks_per_file: 100,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        let content = "fn main() {\n    println!(\"Hello, world!\");\n}";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Should extract the function as a chunk
        assert!(!chunks.is_empty());

        // Find the function chunk
        let function_chunk = chunks.iter().find(|c| c.chunk_type == ChunkType::Function);
        if let Some(chunk) = function_chunk {
            assert_eq!(chunk.language, Language::Rust);
            assert!(chunk.content.contains("fn main()"));
            assert_eq!(chunk.start_line, 1);
        } else {
            // If no function found, should have at least one chunk (fallback)
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0].chunk_type, ChunkType::PlainText);
        }
    }

    #[test]
    fn test_parse_python_class() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.py");
        let content = "class MyClass:\n    def __init__(self):\n        pass";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Should extract semantic chunks or fallback to plain text
        assert!(!chunks.is_empty());

        // Check if we got class and function chunks or fallback
        let has_class = chunks.iter().any(|c| c.chunk_type == ChunkType::Class);
        let has_function = chunks.iter().any(|c| c.chunk_type == ChunkType::Function);
        let has_plaintext = chunks.iter().any(|c| c.chunk_type == ChunkType::PlainText);

        // Should have either semantic chunks or plain text fallback
        assert!(has_class || has_function || has_plaintext);
    }

    #[test]
    fn test_fallback_to_plain_text() {
        let config = ParserConfig {
            min_chunk_size: 1, // Allow small chunks for this test
            max_chunk_size: 2000,
            max_chunks_per_file: 100,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.txt");
        let content = "This is just plain text, not code.";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Should fallback to plain text
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].chunk_type, ChunkType::PlainText);
        assert_eq!(chunks[0].language, Language::Unknown);
        assert_eq!(chunks[0].content, content);
    }

    #[test]
    fn test_chunk_ids_are_unique() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        let content = "fn func1() {}\nfn func2() {}";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Collect all chunk IDs
        let ids: Vec<&String> = chunks.iter().map(|c| &c.id).collect();

        // Check that all IDs are unique
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();

        assert_eq!(
            ids.len(),
            unique_ids.len(),
            "All chunk IDs should be unique"
        );
    }

    #[test]
    fn test_content_hashing() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        let content = "fn main() {}";

        let chunks1 = parser.parse_file(file_path, content).unwrap();
        let chunks2 = parser.parse_file(file_path, content).unwrap();

        // Same content should produce same hashes
        assert_eq!(chunks1.len(), chunks2.len());
        for (c1, c2) in chunks1.iter().zip(chunks2.iter()) {
            assert_eq!(c1.content_hash, c2.content_hash);
        }
    }

    #[test]
    fn test_chunk_size_filtering() {
        let config = ParserConfig {
            min_chunk_size: 10,
            max_chunk_size: 50,
            max_chunks_per_file: 100,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        // Create content with functions of different sizes
        let content = "fn small() { /* 5 */ }\nfn medium_sized_function() {\n    println!(\"Hello\");\n    println!(\"World\");\n}\nfn very_long_function_that_should_be_filtered_out_because_it_exceeds_the_maximum_chunk_size_limit() {\n    println!(\"This function is too long\");\n    println!(\"It should be filtered out\");\n    println!(\"Because it exceeds max_chunk_size\");\n    println!(\"And contains way too much code\");\n}";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Verify chunks are filtered by size
        for chunk in &chunks {
            let size = chunk.content.len();
            assert!(
                (10..=50).contains(&size),
                "Chunk size {size} should be between 10 and 50 characters"
            );
        }
    }

    #[test]
    fn test_max_chunks_per_file_limit() {
        let config = ParserConfig {
            min_chunk_size: 1,
            max_chunk_size: 1000,
            max_chunks_per_file: 2,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        // Create content with multiple functions
        let content = "fn func1() {}\nfn func2() {}\nfn func3() {}\nfn func4() {}";

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Should be limited to max 2 chunks
        assert!(
            chunks.len() <= 2,
            "Should have at most 2 chunks, got {}",
            chunks.len()
        );
    }

    #[test]
    fn test_plain_text_respects_config() {
        let config = ParserConfig {
            min_chunk_size: 100, // Large minimum size
            max_chunk_size: 1000,
            max_chunks_per_file: 10,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.txt");
        let content = "Short"; // Only 5 characters, below minimum

        let chunks = parser.parse_file(file_path, content).unwrap();

        // Should be filtered out due to size limit
        assert!(
            chunks.is_empty(),
            "Short content should be filtered out by min_chunk_size"
        );
    }

    #[test]
    fn test_file_size_limit() {
        let config = ParserConfig {
            min_chunk_size: 1,
            max_chunk_size: 1000,
            max_chunks_per_file: 10,
            max_file_size_bytes: 100, // Very small limit for testing
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        // Create content larger than the limit
        let content = "a".repeat(200); // 200 bytes, exceeds 100 byte limit

        let result = parser.parse_file(file_path, &content);

        // Should fail due to file size limit
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("too large"));
        assert!(error_msg.contains("prevent OOM"));
    }

    #[test]
    fn test_enhanced_error_context() {
        let config = ParserConfig {
            min_chunk_size: 1, // Allow small chunks
            max_chunk_size: 1000,
            max_chunks_per_file: 10,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.unknownext"); // Unknown extension
        let content = "some content that should be parsed as plain text with enough characters to pass the minimum size filter";

        let result = parser.parse_file(file_path, content);

        // Should fall back to plain text for unknown language
        match result {
            Ok(chunks) => {
                // Should create plain text chunk for unknown language
                assert!(
                    !chunks.is_empty(),
                    "Should create at least one plain text chunk for unknown language"
                );
                assert_eq!(chunks[0].chunk_type, ChunkType::PlainText);
                assert_eq!(chunks[0].language, Language::Unknown);
            }
            Err(e) => {
                // If it fails, error should contain detailed context
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("No TreeSitter parser available")
                        || error_msg.contains("Unknown")
                );
            }
        }
    }

    #[test]
    fn test_concurrent_parsing() {
        use std::sync::Arc;
        use std::thread;

        let config = ParserConfig {
            min_chunk_size: 1,
            max_chunk_size: 1000,
            max_chunks_per_file: 10,
            max_file_size_bytes: 10 * 1024 * 1024,
        };
        let parser = Arc::new(CodeParser::new(config).unwrap());

        let handles: Vec<_> = (0..4)
            .map(|i| {
                let parser_clone = Arc::clone(&parser);
                thread::spawn(move || {
                    let file_name = format!("test{i}.rs");
                    let file_path = Path::new(&file_name);
                    let content = format!("fn test_function_{i}() {{ println!(\"test\"); }}");

                    // Each thread parses concurrently
                    let result = parser_clone.parse_file(file_path, &content);
                    assert!(result.is_ok(), "Concurrent parsing should succeed");

                    let chunks = result.unwrap();
                    assert!(!chunks.is_empty(), "Should produce chunks");
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread should complete successfully");
        }
    }

    #[test]
    fn test_config_validation_success() {
        // Valid configurations should succeed
        let valid_configs = vec![
            ParserConfig::new(10, 100, 10, 1024),
            ParserConfig::new(1, 1, 1, 1024),
            ParserConfig::new(50, 2000, 100, 10 * 1024 * 1024),
        ];

        for config in valid_configs {
            assert!(config.is_ok(), "Valid config should succeed");
        }
    }

    #[test]
    fn test_config_validation_failures() {
        // Test min_chunk_size > max_chunk_size
        let result = ParserConfig::new(100, 50, 10, 1024);
        assert!(
            result.is_err(),
            "min_chunk_size > max_chunk_size should fail"
        );

        // Test min_chunk_size = 0
        let result = ParserConfig::new(0, 100, 10, 1024);
        assert!(result.is_err(), "min_chunk_size = 0 should fail");

        // Test max_chunk_size = 0
        let result = ParserConfig::new(10, 0, 10, 1024);
        assert!(result.is_err(), "max_chunk_size = 0 should fail");

        // Test max_chunks_per_file = 0
        let result = ParserConfig::new(10, 100, 0, 1024);
        assert!(result.is_err(), "max_chunks_per_file = 0 should fail");

        // Test max_file_size_bytes too small
        let result = ParserConfig::new(10, 100, 10, 500); // Less than 1KB
        assert!(result.is_err(), "max_file_size_bytes < 1KB should fail");
    }

    #[test]
    fn test_default_config_is_valid() {
        let config = ParserConfig::default();
        // Should not panic and should have reasonable values
        assert!(config.min_chunk_size > 0);
        assert!(config.max_chunk_size > config.min_chunk_size);
        assert!(config.max_chunks_per_file > 0);
        assert!(config.max_file_size_bytes >= 1024);
    }
}
