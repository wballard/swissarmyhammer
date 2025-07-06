//! Example showing search functionality

use swissarmyhammer::{Prompt, PromptLibrary};

use swissarmyhammer::search::SearchEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a library with various prompts
    let mut library = PromptLibrary::new();

    // Add diverse prompts for searching
    let prompts = vec![
        Prompt::new(
            "rust-optimization",
            "Help me optimize this Rust code for performance",
        )
        .with_description("Get suggestions for optimizing Rust code")
        .with_category("rust")
        .with_tags(vec![
            "rust".to_string(),
            "performance".to_string(),
            "optimization".to_string(),
        ]),
        Prompt::new("python-debug", "Debug this Python code and find the issue")
            .with_description("Assist with debugging Python applications")
            .with_category("python")
            .with_tags(vec![
                "python".to_string(),
                "debugging".to_string(),
                "troubleshooting".to_string(),
            ]),
        Prompt::new("code-documentation", "Generate documentation for this code")
            .with_description("Create comprehensive documentation")
            .with_category("documentation")
            .with_tags(vec![
                "docs".to_string(),
                "comments".to_string(),
                "documentation".to_string(),
            ]),
        Prompt::new("api-design", "Design a REST API for this use case")
            .with_description("Help design RESTful APIs")
            .with_category("architecture")
            .with_tags(vec![
                "api".to_string(),
                "rest".to_string(),
                "design".to_string(),
            ]),
        Prompt::new(
            "sql-optimization",
            "Optimize this SQL query for better performance",
        )
        .with_description("Improve SQL query performance")
        .with_category("database")
        .with_tags(vec![
            "sql".to_string(),
            "database".to_string(),
            "optimization".to_string(),
        ]),
        Prompt::new(
            "react-component",
            "Create a React component with these requirements",
        )
        .with_description("Generate React components")
        .with_category("frontend")
        .with_tags(vec![
            "react".to_string(),
            "javascript".to_string(),
            "component".to_string(),
        ]),
    ];

    for prompt in prompts {
        library.add(prompt)?;
    }

    // Basic search using library's built-in search
    println!("Basic search results for 'optimization':");
    let results = library.search("optimization")?;
    for prompt in results {
        println!(
            "  - {}: {}",
            prompt.name,
            prompt.description.as_deref().unwrap_or("No description")
        );
    }

    {
        // Advanced search using SearchEngine
        let mut search_engine = SearchEngine::new()?;

        // Get all prompts from library
        let all_prompts = library.list()?;

        // Index all prompts
        search_engine.index_prompts(&all_prompts)?;

        println!("\nFull-text search results for 'performance':");
        let results = search_engine.search("performance", &all_prompts)?;
        for result in results.iter().take(5) {
            println!(
                "  - {} (score: {:.2}): {}",
                result.prompt.name,
                result.score,
                result
                    .prompt
                    .description
                    .as_deref()
                    .unwrap_or("No description")
            );
        }

        println!("\nFuzzy search results for 'optim' (partial match):");
        let results = search_engine.fuzzy_search("optim", &all_prompts);
        for result in results.iter().take(5) {
            println!(
                "  - {} (score: {:.2}): {}",
                result.prompt.name,
                result.score,
                result
                    .prompt
                    .description
                    .as_deref()
                    .unwrap_or("No description")
            );
        }

        println!("\nHybrid search (combining full-text and fuzzy) for 'react':");
        let results = search_engine.hybrid_search("react", &all_prompts)?;
        for result in results.iter().take(5) {
            println!(
                "  - {} (score: {:.2}): {}",
                result.prompt.name,
                result.score,
                result
                    .prompt
                    .description
                    .as_deref()
                    .unwrap_or("No description")
            );
        }

        // Search by category
        println!("\nSearch by tag 'database':");
        let results = library.search("database")?;
        for prompt in results {
            println!(
                "  - {}: {}",
                prompt.name,
                prompt.description.as_deref().unwrap_or("No description")
            );
        }
    }


    Ok(())
}
