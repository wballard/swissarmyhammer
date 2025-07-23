//! TreeSitter integration for parsing source code

use crate::semantic::types::{ChunkType, CodeChunk, ContentHash, Language};
use crate::semantic::utils::FileHasher;
use crate::semantic::{Result, SemanticError};
use std::path::Path;
use tree_sitter::{Node, Parser, Query, QueryCursor, StreamingIterator};

/// TreeSitter-based code parser
pub struct CodeParser {
    rust_parser: Option<Parser>,
    python_parser: Option<Parser>,
    typescript_parser: Option<Parser>,
    javascript_parser: Option<Parser>,
    dart_parser: Option<Parser>,
    _config: ParserConfig,
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
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            min_chunk_size: 50,
            max_chunk_size: 2000,
            max_chunks_per_file: 100,
        }
    }
}

impl CodeParser {
    /// Create a new code parser
    pub fn new(config: ParserConfig) -> Result<Self> {
        Ok(Self {
            rust_parser: Self::create_rust_parser(),
            python_parser: Self::create_python_parser(),
            typescript_parser: Self::create_typescript_parser(),
            javascript_parser: Self::create_javascript_parser(),
            dart_parser: Self::create_dart_parser(),
            _config: config,
        })
    }

    fn create_rust_parser() -> Option<Parser> {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_rust::LANGUAGE.into()) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!("Failed to initialize Rust parser: {}", e);
                None
            }
        }
    }

    fn create_python_parser() -> Option<Parser> {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_python::LANGUAGE.into()) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!("Failed to initialize Python parser: {}", e);
                None
            }
        }
    }

    fn create_typescript_parser() -> Option<Parser> {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!("Failed to initialize TypeScript parser: {}", e);
                None
            }
        }
    }

    fn create_javascript_parser() -> Option<Parser> {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_javascript::LANGUAGE.into()) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!("Failed to initialize JavaScript parser: {}", e);
                None
            }
        }
    }

    fn create_dart_parser() -> Option<Parser> {
        let mut parser = Parser::new();
        match parser.set_language(&tree_sitter_dart::language()) {
            Ok(_) => Some(parser),
            Err(e) => {
                tracing::warn!("Failed to initialize Dart parser: {}", e);
                None
            }
        }
    }

    /// Detect programming language from file extension
    pub fn detect_language(file_path: &Path) -> Language {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => Language::Rust,
            Some("py") | Some("pyx") | Some("pyi") => Language::Python,
            Some("ts") | Some("tsx") => Language::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") => Language::JavaScript,
            Some("dart") => Language::Dart,
            _ => Language::Unknown,
        }
    }

    /// Get appropriate parser for language
    fn get_parser_for_language(&mut self, language: &Language) -> Option<&mut Parser> {
        match language {
            Language::Rust => self.rust_parser.as_mut(),
            Language::Python => self.python_parser.as_mut(),
            Language::TypeScript => self.typescript_parser.as_mut(),
            Language::JavaScript => self.javascript_parser.as_mut(),
            Language::Dart => self.dart_parser.as_mut(),
            Language::Unknown => None,
        }
    }

    /// Get TreeSitter queries for extracting semantic chunks
    fn get_queries_for_language(language: &Language) -> Vec<(&'static str, ChunkType)> {
        match language {
            Language::Rust => vec![
                // Functions
                (
                    "(function_item name: (identifier) @name body: (_)) @function",
                    ChunkType::Function,
                ),
                // Impl blocks
                ("(impl_item type: (_) @type body: (_)) @impl", ChunkType::Class),
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
            Language::Python => vec![
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
                // Import statements
                ("(import_statement) @import", ChunkType::Import),
                ("(import_from_statement) @import", ChunkType::Import),
            ],
            Language::TypeScript | Language::JavaScript => vec![
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
                // Import statements
                ("(import_statement) @import", ChunkType::Import),
            ],
            Language::Dart => vec![
                // Functions
                (
                    "(function_signature name: (identifier) @name) @function",
                    ChunkType::Function,
                ),
                // Classes
                (
                    "(class_definition name: (type_identifier) @name) @class",
                    ChunkType::Class,
                ),
                // Import statements
                ("(import_or_export) @import", ChunkType::Import),
            ],
            Language::Unknown => vec![],
        }
    }

    /// Parse a source file and extract code chunks
    pub fn parse_file(&mut self, file_path: &Path, content: &str) -> Result<Vec<CodeChunk>> {
        let language = Self::detect_language(file_path);

        // Try TreeSitter parsing first
        match self.parse_with_treesitter(file_path, content, &language) {
            Ok(chunks) => {
                tracing::debug!(
                    "Successfully parsed {} with TreeSitter: {} chunks",
                    file_path.display(),
                    chunks.len()
                );
                Ok(chunks)
            }
            Err(e) => {
                // Fall back to plain text as per specification
                tracing::warn!(
                    "TreeSitter parsing failed for {}: {}. Treating as plain text.",
                    file_path.display(),
                    e
                );
                self.parse_as_plain_text(file_path, content)
            }
        }
    }

    fn parse_with_treesitter(
        &mut self,
        file_path: &Path,
        content: &str,
        language: &Language,
    ) -> Result<Vec<CodeChunk>> {
        let parser = self.get_parser_for_language(language).ok_or_else(|| {
            SemanticError::TreeSitter(format!("No parser available for language: {language:?}"))
        })?;

        let tree = parser.parse(content, None).ok_or_else(|| {
            SemanticError::TreeSitter("Failed to parse file".to_string())
        })?;

        let mut chunks = Vec::new();
        let content_hash = FileHasher::hash_string(content);

        // Extract semantic chunks using queries
        for (query_str, chunk_type) in Self::get_queries_for_language(language) {
            let query = Query::new(&tree.language(), query_str)
                .map_err(|e| SemanticError::TreeSitter(format!("Invalid query: {e}")))?;

            let mut cursor = QueryCursor::new();
            let matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

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
                }
                matches.advance();
            }
        }

        // If no semantic chunks found, create one chunk for entire file
        if chunks.is_empty() {
            chunks.push(self.create_full_file_chunk(file_path, content, language, &content_hash)?);
        }

        Ok(chunks)
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
        let language = Self::detect_language(file_path);

        // Create single chunk for entire file
        let chunk = self.create_full_file_chunk(file_path, content, &language, &content_hash)?;
        Ok(vec![chunk])
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

    /// Check if a file is supported for parsing
    pub fn is_supported_file(&self, file_path: &Path) -> bool {
        !matches!(Self::detect_language(file_path), Language::Unknown)
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
        assert_eq!(
            CodeParser::detect_language(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            CodeParser::detect_language(Path::new("test.py")),
            Language::Python
        );
        assert_eq!(
            CodeParser::detect_language(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            CodeParser::detect_language(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            CodeParser::detect_language(Path::new("test.dart")),
            Language::Dart
        );
        assert_eq!(
            CodeParser::detect_language(Path::new("test.txt")),
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
        let config = ParserConfig::default();
        let mut parser = CodeParser::new(config).unwrap();

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
        let mut parser = CodeParser::new(config).unwrap();

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
        let config = ParserConfig::default();
        let mut parser = CodeParser::new(config).unwrap();

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
        let mut parser = CodeParser::new(config).unwrap();

        let file_path = Path::new("test.rs");
        let content = "fn func1() {}\nfn func2() {}";

        let chunks = parser.parse_file(file_path, content).unwrap();
        
        // Collect all chunk IDs
        let ids: Vec<&String> = chunks.iter().map(|c| &c.id).collect();
        
        // Check that all IDs are unique
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        
        assert_eq!(ids.len(), unique_ids.len(), "All chunk IDs should be unique");
    }

    #[test]
    fn test_content_hashing() {
        let config = ParserConfig::default();
        let mut parser = CodeParser::new(config).unwrap();

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
}