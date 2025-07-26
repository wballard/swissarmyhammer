// THIS IS A SCRATCH FILE - Test to reproduce the search failure issue

#[cfg(test)]
mod reproduction_tests {
    use swissarmyhammer::semantic::{SearchQuery, SemanticConfig, SemanticSearcher, VectorStorage};

    #[tokio::test]
    async fn reproduce_search_failure() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize semantic search components exactly like the failing command
    let config = SemanticConfig::default();
    let storage = VectorStorage::new(config.clone())?;
    storage.initialize()?;

    let searcher = SemanticSearcher::new(storage, config).await?;

    // Perform search with the same parameters as the failing command
    let search_query = SearchQuery {
        text: "duckdb".to_string(),
        limit: 10,
        similarity_threshold: 0.5,
        language_filter: None,
    };

    println!("Attempting search...");
    match searcher.search(&search_query).await {
        Ok(results) => {
            println!("Search succeeded! Found {} results", results.len());
        }
        Err(e) => {
            println!("Search failed with error: {}", e);
            println!("Error debug: {:?}", e);
        }
    }

    Ok(())
    }
}