//! Search functionality for prompts

use crate::{Prompt, Result, SwissArmyHammerError};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::QueryParser,
    schema::{Schema, Field, STORED, TEXT, Value},
    Index, IndexWriter,
};
use std::path::Path;

/// Search result with relevance score
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The prompt
    pub prompt: Prompt,
    /// Relevance score (higher is better)
    pub score: f32,
}

/// Search engine for prompts
pub struct SearchEngine {
    index: Index,
    writer: IndexWriter,
    name_field: Field,
    description_field: Field,
    category_field: Field,
    tags_field: Field,
    template_field: Field,
    fuzzy_matcher: SkimMatcherV2,
}

impl SearchEngine {
    /// Create a new search engine with in-memory index
    pub fn new() -> Result<Self> {
        let mut schema_builder = Schema::builder();
        
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let category_field = schema_builder.add_text_field("category", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let template_field = schema_builder.add_text_field("template", TEXT);
        
        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);
        
        let writer = index.writer(50_000_000)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        Ok(Self {
            index,
            writer,
            name_field,
            description_field,
            category_field,
            tags_field,
            template_field,
            fuzzy_matcher: SkimMatcherV2::default(),
        })
    }
    
    /// Create a new search engine with persistent index
    pub fn with_directory(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        std::fs::create_dir_all(path)?;
        
        let mut schema_builder = Schema::builder();
        
        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let category_field = schema_builder.add_text_field("category", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let template_field = schema_builder.add_text_field("template", TEXT);
        
        let schema = schema_builder.build();
        
        let directory = MmapDirectory::open(path)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        let index = Index::open_or_create(directory, schema)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        let writer = index.writer(50_000_000)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        Ok(Self {
            index,
            writer,
            name_field,
            description_field,
            category_field,
            tags_field,
            template_field,
            fuzzy_matcher: SkimMatcherV2::default(),
        })
    }
    
    /// Index a prompt
    pub fn index_prompt(&mut self, prompt: &Prompt) -> Result<()> {
        let mut document = doc!();
        
        document.add_text(self.name_field, &prompt.name);
        
        if let Some(description) = &prompt.description {
            document.add_text(self.description_field, description);
        }
        
        if let Some(category) = &prompt.category {
            document.add_text(self.category_field, category);
        }
        
        if !prompt.tags.is_empty() {
            document.add_text(self.tags_field, prompt.tags.join(" "));
        }
        
        document.add_text(self.template_field, &prompt.template);
        
        self.writer.add_document(document)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        Ok(())
    }
    
    /// Index multiple prompts
    pub fn index_prompts(&mut self, prompts: &[Prompt]) -> Result<()> {
        for prompt in prompts {
            self.index_prompt(prompt)?;
        }
        
        self.commit()?;
        Ok(())
    }
    
    /// Commit changes to the index
    pub fn commit(&mut self) -> Result<()> {
        self.writer.commit()
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        Ok(())
    }
    
    /// Search for prompts using full-text search
    pub fn search(&self, query: &str, prompts: &[Prompt]) -> Result<Vec<SearchResult>> {
        let reader = self.index
            .reader()
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        let searcher = reader.searcher();
        
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.name_field,
                self.description_field,
                self.category_field,
                self.tags_field,
                self.template_field,
            ],
        );
        
        let query = query_parser
            .parse_query(query)
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(100))
            .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
        
        let mut results = Vec::new();
        
        for (score, doc_address) in top_docs {
            let doc: tantivy::TantivyDocument = searcher.doc(doc_address)
                .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
            
            if let Some(name_value) = doc.get_first(self.name_field) {
                if let Some(name) = name_value.as_str() {
                    // Find the corresponding prompt
                    if let Some(prompt) = prompts.iter().find(|p| p.name == name) {
                        results.push(SearchResult {
                            prompt: prompt.clone(),
                            score,
                        });
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    /// Search using fuzzy matching
    pub fn fuzzy_search(&self, query: &str, prompts: &[Prompt]) -> Vec<SearchResult> {
        let mut results = Vec::new();
        
        for prompt in prompts {
            let mut best_score = 0;
            
            // Score against name
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(&prompt.name, query) {
                best_score = best_score.max(score);
            }
            
            // Score against description
            if let Some(description) = &prompt.description {
                if let Some(score) = self.fuzzy_matcher.fuzzy_match(description, query) {
                    best_score = best_score.max(score / 2); // Weight description less
                }
            }
            
            // Score against category
            if let Some(category) = &prompt.category {
                if let Some(score) = self.fuzzy_matcher.fuzzy_match(category, query) {
                    best_score = best_score.max(score / 2);
                }
            }
            
            // Score against tags
            for tag in &prompt.tags {
                if let Some(score) = self.fuzzy_matcher.fuzzy_match(tag, query) {
                    best_score = best_score.max(score / 2);
                }
            }
            
            if best_score > 0 {
                results.push(SearchResult {
                    prompt: prompt.clone(),
                    score: best_score as f32,
                });
            }
        }
        
        // Sort by score (highest first)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        results
    }
    
    /// Combined search using both full-text and fuzzy matching
    pub fn hybrid_search(&self, query: &str, prompts: &[Prompt]) -> Result<Vec<SearchResult>> {
        let mut results = std::collections::HashMap::new();
        
        // Get full-text search results
        let text_results = self.search(query, prompts)?;
        for result in text_results {
            results.insert(result.prompt.name.clone(), result);
        }
        
        // Get fuzzy search results
        let fuzzy_results = self.fuzzy_search(query, prompts);
        for result in fuzzy_results {
            results.entry(result.prompt.name.clone())
                .and_modify(|e| e.score = e.score.max(result.score))
                .or_insert(result);
        }
        
        let mut final_results: Vec<SearchResult> = results.into_values().collect();
        final_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(final_results)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create search engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new().unwrap();
        assert!(engine.index.schema().fields().count() > 0);
    }
    
    #[test]
    fn test_fuzzy_search() {
        let engine = SearchEngine::new().unwrap();
        
        let prompts = vec![
            Prompt::new("code-review", "Review code")
                .with_description("A prompt for reviewing code"),
            Prompt::new("bug-fix", "Fix bugs")
                .with_description("A prompt for fixing bugs"),
        ];
        
        let results = engine.fuzzy_search("cod", &prompts);
        assert!(!results.is_empty());
        assert_eq!(results[0].prompt.name, "code-review");
    }
}