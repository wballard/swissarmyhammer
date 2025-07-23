//! Query and search logic for semantic search

use crate::error::Result;
use crate::semantic::{EmbeddingService, SemanticSearchResult, VectorStorage};

/// Semantic searcher for querying indexed code
pub struct SemanticSearcher {
    embedding_service: EmbeddingService,
    storage: VectorStorage,
}

/// Options for search operations
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Maximum number of results to return
    pub limit: usize,
    /// Minimum similarity score threshold (0.0 to 1.0)
    pub min_score: f32,
    /// Filter by programming language
    pub language_filter: Option<crate::semantic::Language>,
    /// Filter by file path pattern
    pub file_filter: Option<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 10,
            min_score: 0.0,
            language_filter: None,
            file_filter: None,
        }
    }
}

impl SemanticSearcher {
    /// Create a new semantic searcher
    pub fn new(embedding_service: EmbeddingService, storage: VectorStorage) -> Self {
        Self {
            embedding_service,
            storage,
        }
    }

    /// Search for code chunks semantically similar to the query
    pub fn search(
        &self,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SemanticSearchResult>> {
        // Generate embedding for the query
        let query_embedding = self.embedding_service.embed_text(query)?;

        // Search for similar chunks in storage
        let mut results =
            self.storage
                .search_similar(&query_embedding, options.limit * 2, options.min_score)?;

        // Apply filters
        results = self.apply_filters(results, options);

        // Sort by score (highest first) and limit results
        results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(options.limit);

        Ok(results)
    }

    /// Search for code chunks with multiple query terms
    pub fn search_multi(
        &self,
        queries: &[&str],
        options: &SearchOptions,
    ) -> Result<Vec<SemanticSearchResult>> {
        let mut all_results = Vec::new();

        for query in queries {
            let results = self.search(query, options)?;
            all_results.extend(results);
        }

        // Remove duplicates and merge scores
        let mut merged_results = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for result in all_results {
            if seen_ids.insert(result.chunk.id.clone()) {
                merged_results.push(result);
            }
        }

        // Sort and limit
        merged_results.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        merged_results.truncate(options.limit);

        Ok(merged_results)
    }

    /// Get search suggestions based on indexed content
    pub fn get_suggestions(&self, _partial_query: &str, _limit: usize) -> Result<Vec<String>> {
        // TODO: Implement search suggestions
        Ok(vec![])
    }

    /// Apply filters to search results
    fn apply_filters(
        &self,
        mut results: Vec<SemanticSearchResult>,
        options: &SearchOptions,
    ) -> Vec<SemanticSearchResult> {
        // Filter by minimum score
        results.retain(|result| result.similarity_score >= options.min_score);

        // Filter by language
        if let Some(ref language) = options.language_filter {
            results.retain(|result| &result.chunk.language == language);
        }

        // Filter by file pattern
        if let Some(ref pattern) = options.file_filter {
            results.retain(|result| result.chunk.file_path.to_string_lossy().contains(pattern));
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        ChunkType, CodeChunk, ContentHash, Language, SemanticConfig, VectorStorage,
    };
    use std::path::PathBuf;

    fn create_test_searcher() -> Result<SemanticSearcher> {
        let config = SemanticConfig::default();
        let embedding_service = EmbeddingService::new()?;
        let storage = VectorStorage::new(config)?;
        Ok(SemanticSearcher::new(embedding_service, storage))
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

    #[test]
    fn test_searcher_creation() {
        let searcher = create_test_searcher();
        assert!(searcher.is_ok());
    }

    #[test]
    fn test_search_empty_results() {
        let searcher = create_test_searcher().unwrap();
        let options = SearchOptions::default();

        let results = searcher.search("fn main", &options);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
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
    fn test_apply_filters_min_score() {
        let searcher = create_test_searcher().unwrap();
        let chunk = create_test_chunk();

        let results = vec![
            SemanticSearchResult {
                chunk: chunk.clone(),
                similarity_score: 0.8,
                excerpt: "test excerpt".to_string(),
            },
            SemanticSearchResult {
                chunk: chunk.clone(),
                similarity_score: 0.3,
                excerpt: "test excerpt".to_string(),
            },
            SemanticSearchResult {
                chunk,
                similarity_score: 0.1,
                excerpt: "test excerpt".to_string(),
            },
        ];

        let options = SearchOptions {
            min_score: 0.5,
            ..Default::default()
        };

        let filtered = searcher.apply_filters(results, &options);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].similarity_score, 0.8);
    }

    #[test]
    fn test_apply_filters_language() {
        let searcher = create_test_searcher().unwrap();
        let mut rust_chunk = create_test_chunk();
        rust_chunk.language = Language::Rust;

        let mut python_chunk = create_test_chunk();
        python_chunk.language = Language::Python;
        python_chunk.id = "test-chunk-2".to_string();

        let results = vec![
            SemanticSearchResult {
                chunk: rust_chunk,
                similarity_score: 0.8,
                excerpt: "test excerpt".to_string(),
            },
            SemanticSearchResult {
                chunk: python_chunk,
                similarity_score: 0.9,
                excerpt: "test excerpt".to_string(),
            },
        ];

        let options = SearchOptions {
            language_filter: Some(Language::Rust),
            ..Default::default()
        };

        let filtered = searcher.apply_filters(results, &options);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].chunk.language, Language::Rust);
    }
}
