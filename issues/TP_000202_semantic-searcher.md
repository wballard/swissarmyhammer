# TP_000202: Semantic Searcher Implementation

## Goal
Implement the semantic search functionality that queries the vector database to find code chunks similar to a given query.

## Context
This component provides the query interface for the semantic search system, converting text queries into embeddings and finding similar code chunks using vector similarity search.

## Tasks

### 1. Create SemanticSearcher in `semantic/searcher.rs`

```rust
use crate::semantic::{
    Result, SemanticError, VectorStorage, EmbeddingEngine, 
    SemanticSearchResult, SearchQuery, CodeChunk, Language
};
use std::collections::HashMap;

pub struct SemanticSearcher {
    storage: VectorStorage,
    embedding_engine: EmbeddingEngine,
}

impl SemanticSearcher {
    pub async fn new(storage: VectorStorage) -> Result<Self> {
        let embedding_engine = EmbeddingEngine::new().await?;
        
        Ok(Self {
            storage,
            embedding_engine,
        })
    }
    
    pub async fn with_embedding_engine(
        storage: VectorStorage,
        embedding_engine: EmbeddingEngine,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            embedding_engine,
        })
    }
}
```

### 2. Core Search Functionality

```rust
impl SemanticSearcher {
    /// Perform semantic search with a text query
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SemanticSearchResult>> {
        tracing::debug!("Performing semantic search for: '{}'", query.text);
        
        // Generate embedding for the query
        let query_embedding = self.embedding_engine.embed_text(&query.text).await?;
        
        // Find similar embeddings in the database
        let similar_chunk_ids = self.storage.similarity_search(
            &query_embedding,
            query.limit,
            query.similarity_threshold,
        )?;
        
        if similar_chunk_ids.is_empty() {
            tracing::info!("No results found for query: '{}'", query.text);
            return Ok(Vec::new());
        }
        
        // Retrieve chunk details and create search results
        let mut results = Vec::new();
        for (chunk_id, similarity_score) in similar_chunk_ids {
            if let Some(chunk) = self.storage.get_chunk(&chunk_id)? {
                // Apply language filter if specified
                if let Some(ref language_filter) = query.language_filter {
                    if chunk.language != *language_filter {
                        continue;
                    }
                }
                
                let excerpt = self.create_excerpt(&chunk, &query.text);
                
                results.push(SemanticSearchResult {
                    chunk,
                    similarity_score,
                    excerpt,
                });
            } else {
                tracing::warn!("Chunk not found in database: {}", chunk_id);
            }
        }
        
        // Sort by similarity score (highest first)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());
        
        tracing::info!("Found {} results for query", results.len());
        Ok(results)
    }
    
    /// Search with simple text query using default parameters
    pub async fn search_simple(&self, query_text: &str, limit: usize) -> Result<Vec<SemanticSearchResult>> {
        let query = SearchQuery {
            text: query_text.to_string(),
            limit,
            similarity_threshold: 0.5, // Default threshold
            language_filter: None,
        };
        
        self.search(&query).await
    }
}
```

### 3. Advanced Search Features

```rust
impl SemanticSearcher {
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
    pub async fn find_similar_code(&self, chunk: &CodeChunk, limit: usize) -> Result<Vec<SemanticSearchResult>> {
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
}
```

### 4. Result Processing and Excerpts

```rust
impl SemanticSearcher {
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
    
    fn create_excerpt_around_match(&self, content: &str, match_pos: usize, max_length: usize) -> String {
        let start = match_pos.saturating_sub(max_length / 2);
        let end = (match_pos + max_length / 2).min(content.len());
        
        let excerpt = &content[start..end];
        
        // Clean up excerpt to avoid breaking in middle of words
        let cleaned = self.clean_excerpt(excerpt, start > 0, end < content.len());
        
        cleaned
    }
    
    fn create_excerpt_from_start(&self, content: &str, max_length: usize, context_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let selected_lines = lines.iter().take(context_lines).cloned().collect::<Vec<_>>();
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
            result = format!("...{}", result);
        }
        if has_suffix {
            result = format!("{}...", result);
        }
        
        result
    }
}
```

### 5. Search Statistics and Debugging

```rust
impl SemanticSearcher {
    /// Get search statistics for debugging
    pub async fn get_search_stats(&self) -> Result<SearchStats> {
        let index_stats = self.storage.get_index_stats()?;
        let model_info = self.embedding_engine.model_info();
        
        Ok(SearchStats {
            total_files: index_stats.file_count,
            total_chunks: index_stats.chunk_count,
            total_embeddings: index_stats.embedding_count,
            model_info,
        })
    }
    
    /// Explain search results with detailed scoring information
    pub async fn explain_search(
        &self,
        query: &SearchQuery,
    ) -> Result<SearchExplanation> {
        let query_embedding = self.embedding_engine.embed_text(&query.text).await?;
        
        // Get detailed similarity results
        let similar_results = self.storage.similarity_search_with_details(
            &query_embedding,
            query.limit,
            0.0, // Get all results for explanation
        )?;
        
        let mut explanations = Vec::new();
        for (chunk_id, similarity_score, embedding) in similar_results {
            if let Some(chunk) = self.storage.get_chunk(&chunk_id)? {
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

#[derive(Debug, Clone)]
pub struct SearchStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub total_embeddings: usize,
    pub model_info: crate::semantic::EmbeddingModelInfo,
}

#[derive(Debug)]
pub struct SearchExplanation {
    pub query_text: String,
    pub query_embedding_norm: f32,
    pub threshold: f32,
    pub total_candidates: usize,
    pub results: Vec<ResultExplanation>,
}

#[derive(Debug)]
pub struct ResultExplanation {
    pub chunk_id: String,
    pub similarity_score: f32,
    pub language: Language,
    pub chunk_type: crate::semantic::ChunkType,
    pub content_preview: String,
    pub above_threshold: bool,
}
```

## Acceptance Criteria
- [ ] SemanticSearcher performs accurate vector similarity search
- [ ] Search results are ranked by similarity score
- [ ] Language filtering works correctly
- [ ] Excerpt generation provides meaningful context
- [ ] Advanced search features (similar code, multi-query) work
- [ ] Search statistics provide useful debugging information
- [ ] Performance is reasonable for typical query loads
- [ ] Error handling manages edge cases gracefully

## Architecture Notes
- Combines embedding generation with vector similarity search
- Excerpt generation provides context around matches
- Multiple search modes support different use cases
- Statistics and explanation features aid debugging
- Deduplication prevents duplicate results in multi-query search

## Testing Strategy
- Test semantic search accuracy with known similar code
- Test language filtering with mixed codebases
- Test excerpt generation with various content types
- Performance testing with large indices
- Edge case testing with empty results, malformed queries

## Proposed Solution

After analyzing the existing codebase, I found there's already a basic `SemanticSearcher` implementation in `swissarmyhammer/src/semantic/searcher.rs`. However, the current implementation has a different API and lacks the advanced features specified in this issue.

### Current State Analysis
- Existing `SemanticSearcher` uses `SearchOptions` instead of `SearchQuery`
- Basic search functionality exists but without excerpt generation
- Missing advanced features like multi-query search, similar code search, and statistics
- Missing the debugging and explanation features
- Different API signatures than specified

### Implementation Plan
1. **Replace existing implementation** with the comprehensive version specified in the issue
2. **Maintain backward compatibility** where possible by keeping useful existing test infrastructure
3. **Use Test-Driven Development** to ensure all acceptance criteria are met
4. **Leverage existing components**: `VectorStorage`, `EmbeddingEngine`, and type definitions are already available
5. **Add missing types**: `SearchStats`, `SearchExplanation`, `ResultExplanation` need to be added to `types.rs`
6. **Implement comprehensive error handling** following the existing `SemanticError` patterns

### Key Integration Points
- Use existing `VectorStorage.search_similar()` method (currently in-memory fallback)
- Use existing `EmbeddingEngine.embed_text()` for query embedding generation
- Use existing `SearchQuery` type from `types.rs`
- Follow existing error handling patterns with `crate::semantic::Result<T>`

### Testing Strategy
- Replace existing basic tests with comprehensive test coverage
- Test all search modes: basic, language-filtered, similar code, multi-query
- Test excerpt generation with various content types
- Test error conditions and edge cases
- Use existing test infrastructure (`create_test_searcher`, `create_test_chunk`)

## Next Steps
After completion, proceed to TP_000203_cli-integration to implement the command-line interface.