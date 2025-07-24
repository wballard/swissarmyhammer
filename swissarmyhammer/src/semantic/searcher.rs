//! Query and search logic for semantic search

use crate::semantic::{
    CodeChunk, EmbeddingEngine, Language, Result, ResultExplanation, SearchExplanation,
    SearchQuery, SearchStats, SemanticSearchResult, VectorStorage,
};
use std::collections::HashMap;

/// Semantic searcher for querying indexed code
pub struct SemanticSearcher {
    storage: VectorStorage,
    embedding_engine: EmbeddingEngine,
}

impl SemanticSearcher {
    /// Create a new semantic searcher
    pub async fn new(storage: VectorStorage) -> Result<Self> {
        let embedding_engine = EmbeddingEngine::new().await?;

        Ok(Self {
            storage,
            embedding_engine,
        })
    }

    /// Create searcher with existing embedding engine
    pub async fn with_embedding_engine(
        storage: VectorStorage,
        embedding_engine: EmbeddingEngine,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            embedding_engine,
        })
    }

    /// Perform semantic search with a text query
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SemanticSearchResult>> {
        tracing::debug!("Performing semantic search for: '{}'", query.text);

        // Generate embedding for the query
        let query_embedding = self.embedding_engine.embed_text(&query.text).await?;

        // Find similar embeddings in the database
        let similar_chunk_ids = self
            .storage
            .search_similar(&query_embedding, query.limit, query.similarity_threshold)
            .map_err(|e| crate::semantic::SemanticError::Search(e.to_string()))?;

        if similar_chunk_ids.is_empty() {
            tracing::info!("No results found for query: '{}'", query.text);
            return Ok(Vec::new());
        }

        // Apply language filter and create excerpts
        let mut results = Vec::new();
        for mut result in similar_chunk_ids {
            // Apply language filter if specified
            if let Some(ref language_filter) = query.language_filter {
                if result.chunk.language != *language_filter {
                    continue;
                }
            }

            // Create excerpt for this result
            result.excerpt = self.create_excerpt(&result.chunk, &query.text);
            results.push(result);
        }

        // Sort by similarity score (highest first) - already sorted by storage, but ensure consistency
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        tracing::info!("Found {} results for query", results.len());
        Ok(results)
    }

    /// Search with simple text query using default parameters
    pub async fn search_simple(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        let query = SearchQuery {
            text: query_text.to_string(),
            limit,
            similarity_threshold: 0.5, // Default threshold
            language_filter: None,
        };

        self.search(&query).await
    }

    /// Search within specific programming languages
    pub async fn search_by_language(
        &self,
        query_text: &str,
        language: Language,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        let query = SearchQuery {
            text: query_text.to_string(),
            limit,
            similarity_threshold: 0.5,
            language_filter: Some(language),
        };

        self.search(&query).await
    }

    /// Search for similar code to a given chunk
    pub async fn find_similar_code(
        &self,
        chunk: &CodeChunk,
        limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        // Use the chunk content as the query
        let query = SearchQuery {
            text: chunk.content.clone(),
            limit: limit + 1, // +1 because the original chunk might be included
            similarity_threshold: 0.7, // Higher threshold for code similarity
            language_filter: None, // Don't filter by language for broader results
        };

        let mut results = self.search(&query).await?;

        // Remove the original chunk from results if present
        results.retain(|result| result.chunk.id != chunk.id);

        // Limit to requested number
        results.truncate(limit);

        Ok(results)
    }

    /// Multi-query search - combine results from multiple related queries
    pub async fn multi_query_search(
        &self,
        queries: &[String],
        limit_per_query: usize,
        overall_limit: usize,
    ) -> Result<Vec<SemanticSearchResult>> {
        let mut all_results = HashMap::new();

        for query_text in queries {
            let results = self.search_simple(query_text, limit_per_query).await?;

            for result in results {
                // Use chunk ID as key to deduplicate
                all_results
                    .entry(result.chunk.id.clone())
                    .and_modify(|existing: &mut SemanticSearchResult| {
                        // Keep the result with higher similarity score
                        if result.similarity_score > existing.similarity_score {
                            *existing = result.clone();
                        }
                    })
                    .or_insert(result);
            }
        }

        // Convert to vector and sort
        let mut final_results: Vec<_> = all_results.into_values().collect();
        final_results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        // Limit results
        final_results.truncate(overall_limit);

        Ok(final_results)
    }

    /// Create an excerpt showing relevant parts of the code
    fn create_excerpt(&self, chunk: &CodeChunk, query: &str) -> String {
        const EXCERPT_LENGTH: usize = 200;
        const CONTEXT_LINES: usize = 2;

        let content = &chunk.content;
        let query_lower = query.to_lowercase();

        // Try to find query terms in the content
        let content_lower = content.to_lowercase();

        if let Some(match_pos) = content_lower.find(&query_lower) {
            // Found direct match - create excerpt around it
            self.create_excerpt_around_match(content, match_pos, EXCERPT_LENGTH)
        } else {
            // No direct match - create excerpt from beginning with context
            self.create_excerpt_from_start(content, EXCERPT_LENGTH, CONTEXT_LINES)
        }
    }

    fn create_excerpt_around_match(
        &self,
        content: &str,
        match_pos: usize,
        max_length: usize,
    ) -> String {
        let start = match_pos.saturating_sub(max_length / 2);
        let end = (match_pos + max_length / 2).min(content.len());

        let excerpt = &content[start..end];

        // Clean up excerpt to avoid breaking in middle of words
        self.clean_excerpt(excerpt, start > 0, end < content.len())
    }

    fn create_excerpt_from_start(
        &self,
        content: &str,
        max_length: usize,
        context_lines: usize,
    ) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let selected_lines = lines
            .iter()
            .take(context_lines)
            .cloned()
            .collect::<Vec<_>>();
        let excerpt = selected_lines.join("\n");

        if excerpt.len() <= max_length {
            excerpt
        } else {
            let truncated = &excerpt[..max_length];
            format!("{}...", truncated.trim_end())
        }
    }

    fn clean_excerpt(&self, excerpt: &str, has_prefix: bool, has_suffix: bool) -> String {
        let mut result = excerpt.trim().to_string();

        // Add ellipsis if truncated
        if has_prefix {
            result = format!("...{result}");
        }
        if has_suffix {
            result = format!("{result}...");
        }

        result
    }

    /// Get search statistics for debugging
    pub async fn get_search_stats(&self) -> Result<SearchStats> {
        let index_stats = self
            .storage
            .get_index_stats()
            .map_err(|e| crate::semantic::SemanticError::Search(e.to_string()))?;
        let model_info = self.embedding_engine.model_info();

        Ok(SearchStats {
            total_files: index_stats.file_count,
            total_chunks: index_stats.chunk_count,
            total_embeddings: index_stats.embedding_count,
            model_info,
        })
    }

    /// Explain search results with detailed scoring information
    pub async fn explain_search(&self, query: &SearchQuery) -> Result<SearchExplanation> {
        let query_embedding = self.embedding_engine.embed_text(&query.text).await?;

        // Get detailed similarity results
        let similar_results = self
            .storage
            .similarity_search_with_details(
                &query_embedding,
                query.limit,
                0.0, // Get all results for explanation
            )
            .map_err(|e| crate::semantic::SemanticError::Search(e.to_string()))?;

        let mut explanations = Vec::new();
        for (chunk_id, similarity_score, _embedding) in similar_results {
            if let Some(chunk) = self
                .storage
                .get_chunk(&chunk_id)
                .map_err(|e| crate::semantic::SemanticError::Search(e.to_string()))?
            {
                explanations.push(ResultExplanation {
                    chunk_id: chunk_id.clone(),
                    similarity_score,
                    language: chunk.language.clone(),
                    chunk_type: chunk.chunk_type.clone(),
                    content_preview: chunk.content.chars().take(100).collect(),
                    above_threshold: similarity_score >= query.similarity_threshold,
                });
            }
        }

        Ok(SearchExplanation {
            query_text: query.text.clone(),
            query_embedding_norm: self.calculate_vector_norm(&query_embedding),
            threshold: query.similarity_threshold,
            total_candidates: explanations.len(),
            results: explanations,
        })
    }

    fn calculate_vector_norm(&self, vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ChunkType, ContentHash, Language, SemanticConfig};
    use std::path::PathBuf;

    async fn create_test_searcher() -> Result<SemanticSearcher> {
        let config = SemanticConfig::default();
        let embedding_engine = EmbeddingEngine::new_for_testing().await?;
        let storage = VectorStorage::new(config)
            .map_err(|e| crate::semantic::SemanticError::Config(e.to_string()))?;
        SemanticSearcher::with_embedding_engine(storage, embedding_engine).await
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

    #[tokio::test]
    async fn test_searcher_creation() {
        let searcher = create_test_searcher().await;
        assert!(searcher.is_ok());
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        let searcher = create_test_searcher().await.unwrap();
        let query = SearchQuery {
            text: "fn main".to_string(),
            limit: 10,
            similarity_threshold: 0.5,
            language_filter: None,
        };

        let results = searcher.search(&query).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_search_simple() {
        let searcher = create_test_searcher().await.unwrap();
        let results = searcher.search_simple("fn main", 10).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_search_by_language() {
        let searcher = create_test_searcher().await.unwrap();
        let results = searcher
            .search_by_language("fn main", Language::Rust, 10)
            .await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_find_similar_code() {
        let searcher = create_test_searcher().await.unwrap();
        let chunk = create_test_chunk();
        let results = searcher.find_similar_code(&chunk, 5).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_multi_query_search() {
        let searcher = create_test_searcher().await.unwrap();
        let queries = vec!["fn main".to_string(), "println".to_string()];
        let results = searcher.multi_query_search(&queries, 5, 10).await;
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_create_excerpt_around_match() {
        let searcher = futures::executor::block_on(create_test_searcher()).unwrap();
        let content = "This is a test function that does something interesting";
        let match_pos = 10; // Position of "test"
        let excerpt = searcher.create_excerpt_around_match(content, match_pos, 20);

        assert!(excerpt.contains("test"));
        assert!(excerpt.len() <= 25); // 20 + ellipsis
    }

    #[test]
    fn test_create_excerpt_from_start() {
        let searcher = futures::executor::block_on(create_test_searcher()).unwrap();
        let content = "line1\nline2\nline3\nline4";
        let excerpt = searcher.create_excerpt_from_start(content, 50, 2);

        assert!(excerpt.contains("line1"));
        assert!(excerpt.contains("line2"));
        assert!(!excerpt.contains("line3")); // Should be limited to 2 lines
    }

    #[test]
    fn test_clean_excerpt() {
        let searcher = futures::executor::block_on(create_test_searcher()).unwrap();
        let excerpt = "  some content  ";

        let cleaned = searcher.clean_excerpt(excerpt, true, true);
        assert_eq!(cleaned, "...some content...");

        let cleaned = searcher.clean_excerpt(excerpt, false, false);
        assert_eq!(cleaned, "some content");
    }

    #[tokio::test]
    async fn test_get_search_stats() {
        let searcher = create_test_searcher().await.unwrap();
        let stats = searcher.get_search_stats().await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.total_embeddings, 0);
    }

    #[tokio::test]
    async fn test_explain_search() {
        let searcher = create_test_searcher().await.unwrap();
        let query = SearchQuery {
            text: "fn main".to_string(),
            limit: 10,
            similarity_threshold: 0.5,
            language_filter: None,
        };

        let explanation = searcher.explain_search(&query).await;
        assert!(explanation.is_ok());

        let explanation = explanation.unwrap();
        assert_eq!(explanation.query_text, "fn main");
        assert_eq!(explanation.threshold, 0.5);
        assert_eq!(explanation.results.len(), 0);
    }

    #[test]
    fn test_calculate_vector_norm() {
        let searcher = futures::executor::block_on(create_test_searcher()).unwrap();
        let vector = vec![3.0, 4.0]; // 3-4-5 triangle
        let norm = searcher.calculate_vector_norm(&vector);
        assert!((norm - 5.0).abs() < 0.001);
    }
}
