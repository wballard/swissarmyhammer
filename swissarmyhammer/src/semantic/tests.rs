//! Integration tests for semantic search module

#[cfg(test)]
mod tests {
    use crate::semantic::*;
    use tempfile::TempDir;

    #[test]
    fn test_semantic_module_imports() {
        // Test that all module exports are accessible
        let _config = SemanticConfig::default();
        let _language = Language::Rust;
        
        // Verify we can create core components
        let parser_config = ParserConfig::default();
        assert!(CodeParser::new(parser_config).is_ok());
        
        assert!(EmbeddingService::new().is_ok());
        
        let config = SemanticConfig::default();
        assert!(VectorStorage::new(config).is_ok());
    }

    #[test]
    fn test_language_enum() {
        // Test Language enum serialization/deserialization
        let languages = vec![
            Language::Rust,
            Language::Python, 
            Language::TypeScript,
            Language::JavaScript,
            Language::Dart,
        ];
        
        for lang in languages {
            let serialized = serde_json::to_string(&lang).unwrap();
            let deserialized: Language = serde_json::from_str(&serialized).unwrap();
            assert_eq!(lang, deserialized);
        }
    }

    #[test]
    fn test_semantic_config_default() {
        let config = SemanticConfig::default();
        assert!(config.database_path.to_string_lossy().contains("semantic.db"));
        assert_eq!(config.max_chunks_per_file, 100);
        assert_eq!(config.min_chunk_size, 50);
        assert_eq!(config.max_chunk_size, 2000);
    }

    #[test]
    fn test_parser_config_default() {
        let config = ParserConfig::default();
        assert_eq!(config.min_chunk_size, 50);
        assert_eq!(config.max_chunk_size, 2000);
        assert_eq!(config.max_chunks_per_file, 100);
    }

    #[test]
    fn test_indexing_options_default() {
        let options = IndexingOptions::default();
        assert!(!options.force);
        assert!(options.glob_pattern.is_none());
        assert!(options.max_files.is_none());
    }

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 10);
        assert_eq!(options.min_score, 0.0);
        assert!(options.language_filter.is_none());
        assert!(options.file_filter.is_none());
    }

    #[test] 
    fn test_semantic_utils() {
        // Test text normalization
        let input = "  fn main() {  \n\n  println!(\"hello\");  \n  }  \n\n";
        let normalized = SemanticUtils::normalize_text(input);
        assert!(!normalized.contains("  "));
        assert!(normalized.contains("fn main()"));
        assert!(normalized.contains("println!"));
        
        // Test cosine similarity
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = SemanticUtils::cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 1e-6);
        
        // Test chunk ID generation
        let path = std::path::Path::new("test.rs");
        let id = SemanticUtils::generate_chunk_id(path, 1, 5);
        assert_eq!(id, "test.rs:1:5");
    }

    #[test]
    fn test_file_extensions_for_languages() {
        let rust_exts = SemanticUtils::get_file_extensions_for_language(&Language::Rust);
        assert!(rust_exts.contains(&"rs"));
        
        let python_exts = SemanticUtils::get_file_extensions_for_language(&Language::Python);
        assert!(python_exts.contains(&"py"));
        
        let ts_exts = SemanticUtils::get_file_extensions_for_language(&Language::TypeScript);
        assert!(ts_exts.contains(&"ts"));
    }

    #[test]
    fn test_should_index_file() {
        // Should index normal source files
        assert!(SemanticUtils::should_index_file(std::path::Path::new("src/main.rs")));
        assert!(SemanticUtils::should_index_file(std::path::Path::new("lib/utils.py")));
        
        // Should not index hidden files or build directories
        assert!(!SemanticUtils::should_index_file(std::path::Path::new(".hidden/file.rs")));
        assert!(!SemanticUtils::should_index_file(std::path::Path::new("target/debug/main")));
        assert!(!SemanticUtils::should_index_file(std::path::Path::new("node_modules/package/index.js")));
    }

    #[test]
    fn test_code_chunk_creation() {
        use std::path::PathBuf;
        
        let chunk = CodeChunk {
            id: "test-1".to_string(),
            file_path: PathBuf::from("test.rs"),
            content: "fn main() {}".to_string(),
            language: Language::Rust,
            start_line: 1,
            end_line: 1,
            content_hash: "abc123".to_string(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
        };
        
        assert_eq!(chunk.id, "test-1");
        assert_eq!(chunk.language, Language::Rust);
        assert!(chunk.embedding.is_some());
        assert_eq!(chunk.embedding.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_search_result_creation() {
        use std::path::PathBuf;
        
        let chunk = CodeChunk {
            id: "test-1".to_string(),
            file_path: PathBuf::from("test.rs"),
            content: "fn main() {}".to_string(),
            language: Language::Rust,
            start_line: 1,
            end_line: 1,
            content_hash: "abc123".to_string(),
            embedding: None,
        };
        
        let result = SearchResult {
            chunk,
            score: 0.95,
        };
        
        assert_eq!(result.score, 0.95);
        assert_eq!(result.chunk.id, "test-1");
    }

    #[test]
    fn test_indexing_stats_default() {
        let stats = IndexingStats::default();
        assert_eq!(stats.processed_files, 0);
        assert_eq!(stats.skipped_files, 0);
        assert_eq!(stats.failed_files, 0);
        assert_eq!(stats.total_chunks, 0);
    }
}