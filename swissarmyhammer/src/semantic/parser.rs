//! TreeSitter integration for parsing source code

use crate::error::Result;
use crate::semantic::types::{CodeChunk, Language};
use std::path::Path;

/// TreeSitter-based code parser
pub struct CodeParser {
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
        Ok(Self { _config: config })
    }

    /// Parse a source file and extract code chunks
    pub fn parse_file(&self, file_path: &Path, content: &str) -> Result<Vec<CodeChunk>> {
        let language = self.detect_language(file_path)?;
        
        // TODO: Implement actual TreeSitter parsing
        // For now, create a simple chunk from the entire content
        let chunk = CodeChunk {
            id: format!("{}:{}", file_path.display(), 1),
            file_path: file_path.to_path_buf(),
            content: content.to_string(),
            language,
            start_line: 1,
            end_line: content.lines().count(),
            content_hash: format!("{:x}", md5::compute(content.as_bytes())),
            embedding: None,
        };

        Ok(vec![chunk])
    }

    /// Detect the programming language from file extension
    fn detect_language(&self, file_path: &Path) -> Result<Language> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| crate::error::SwissArmyHammerError::Other(
                format!("Could not determine file extension for: {}", file_path.display())
            ))?;

        match extension {
            "rs" => Ok(Language::Rust),
            "py" => Ok(Language::Python),
            "ts" => Ok(Language::TypeScript),
            "js" => Ok(Language::JavaScript),
            "dart" => Ok(Language::Dart),
            _ => Err(crate::error::SwissArmyHammerError::Other(
                format!("Unsupported file extension: {}", extension)
            )),
        }
    }

    /// Check if a file is supported for parsing
    pub fn is_supported_file(&self, file_path: &Path) -> bool {
        self.detect_language(file_path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parser_creation() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_detect_language() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();

        assert_eq!(parser.detect_language(Path::new("test.rs")).unwrap(), Language::Rust);
        assert_eq!(parser.detect_language(Path::new("test.py")).unwrap(), Language::Python);
        assert_eq!(parser.detect_language(Path::new("test.ts")).unwrap(), Language::TypeScript);
        assert_eq!(parser.detect_language(Path::new("test.js")).unwrap(), Language::JavaScript);
        assert_eq!(parser.detect_language(Path::new("test.dart")).unwrap(), Language::Dart);
        
        assert!(parser.detect_language(Path::new("test.txt")).is_err());
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
    fn test_parse_file() {
        let config = ParserConfig::default();
        let parser = CodeParser::new(config).unwrap();
        
        let file_path = Path::new("test.rs");
        let content = "fn main() {\n    println!(\"Hello, world!\");\n}";
        
        let chunks = parser.parse_file(file_path, content);
        assert!(chunks.is_ok());
        
        let chunks = chunks.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].language, Language::Rust);
        assert_eq!(chunks[0].content, content);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 3);
    }
}