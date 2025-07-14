use anyhow::Result;
use colored::*;
use is_terminal::IsTerminal;
use std::io;
use tabled::{
    settings::{object::Rows, Alignment, Color, Modify, Style},
    Table, Tabled,
};

use crate::cli::{OutputFormat, PromptSource, PromptSourceArg};
use swissarmyhammer::{
    prelude::{AdvancedSearchEngine, AdvancedSearchOptions},
    PromptFilter, PromptLibrary, PromptResolver,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub source: String,
    pub score: f32,
    pub excerpt: Option<String>,
    pub arguments: Vec<SearchArgument>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Tabled)]
struct SearchResultRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Excerpt")]
    excerpt: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Score")]
    score: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run_search_command(
    query: String,
    _fields: Option<Vec<String>>,
    regex: bool,
    fuzzy: bool,
    case_sensitive: bool,
    source_filter: Option<PromptSourceArg>,
    has_arg: Option<String>,
    no_args: bool,
    full: bool,
    format: OutputFormat,
    highlight: bool,
    limit: Option<usize>,
) -> Result<()> {
    // Load all prompts from all sources
    let mut library = PromptLibrary::new();
    let mut resolver = PromptResolver::new();
    resolver.load_all_prompts(&mut library)?;

    // Get all prompts
    let all_prompts = library.list()?;

    // Create advanced search options
    let search_options = AdvancedSearchOptions {
        regex,
        fuzzy,
        case_sensitive,
        highlight,
        limit,
    };

    // Create filter based on CLI options
    let mut filter = PromptFilter::new();

    // Apply source filter
    if let Some(ref src_filter) = source_filter {
        let library_source: PromptSource = src_filter.clone().into();
        filter = filter.with_source(library_source);
    }

    // Apply argument filters
    if let Some(arg_name) = has_arg {
        filter = filter.with_has_arg(arg_name);
    }

    if no_args {
        filter = filter.with_no_args(true);
    }

    // Perform search using advanced search engine
    let search_engine = AdvancedSearchEngine::new()?;
    let search_results = search_engine.search(
        &query,
        &all_prompts,
        &search_options,
        Some(&filter),
        &resolver.prompt_sources,
    )?;

    // Convert results to CLI format
    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|result| {
            // Get the source from the resolver
            let prompt_source = match resolver.prompt_sources.get(&result.prompt.name) {
                Some(PromptSource::Builtin) => PromptSource::Builtin,
                Some(PromptSource::User) => PromptSource::User,
                Some(PromptSource::Local) => PromptSource::Local,
                Some(PromptSource::Dynamic) => PromptSource::Dynamic,
                None => PromptSource::Dynamic,
            };
            let source_str = prompt_source.to_string();

            let arguments = result
                .prompt
                .arguments
                .iter()
                .map(|arg| SearchArgument {
                    name: arg.name.clone(),
                    description: arg.description.clone(),
                    required: arg.required,
                    default: arg.default.clone(),
                })
                .collect();

            SearchResult {
                name: result.prompt.name.clone(),
                title: None, // No title field in new API
                description: result.prompt.description.clone(),
                source: source_str,
                score: result.score,
                excerpt: result.excerpt,
                arguments,
            }
        })
        .collect();

    // Output results
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&results)?;
            print!("{}", yaml);
        }
        OutputFormat::Table => {
            display_table(&results, full)?;
        }
    }

    Ok(())
}

fn display_table(results: &[SearchResult], full: bool) -> Result<()> {
    if results.is_empty() {
        println!("No prompts found matching the search criteria.");
        return Ok(());
    }

    let is_tty = io::stdout().is_terminal();

    let rows: Vec<SearchResultRow> = results
        .iter()
        .map(|result| {
            let title = result.title.as_deref().unwrap_or("");
            let excerpt = if full {
                result.excerpt.as_deref().unwrap_or("")
            } else {
                // Truncate long excerpts for table display
                let exc = result.excerpt.as_deref().unwrap_or("");
                if exc.len() > 50 {
                    &format!("{}...", &exc[..47])
                } else {
                    exc
                }
            };

            SearchResultRow {
                name: result.name.clone(),
                title: title.to_string(),
                excerpt: excerpt.to_string(),
                source: result.source.clone(),
                score: format!("{:.1}", result.score),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::modern());

    if is_tty {
        // Add colors for better readability in terminal
        table.with(Modify::new(Rows::single(0)).with(Color::FG_BRIGHT_CYAN));

        // Color code sources
        for (i, result) in results.iter().enumerate() {
            let row_index = i + 1; // +1 because row 0 is header
            match result.source.as_str() {
                "builtin" => {
                    table.with(Modify::new(Rows::single(row_index)).with(Color::FG_GREEN));
                }
                "user" => {
                    table.with(Modify::new(Rows::single(row_index)).with(Color::FG_BLUE));
                }
                "local" => {
                    table.with(Modify::new(Rows::single(row_index)).with(Color::FG_YELLOW));
                }
                _ => {}
            }
        }
    }

    table.with(Modify::new(Rows::new(1..)).with(Alignment::left()));

    println!("{}", table);

    if is_tty && !results.is_empty() {
        println!();
        println!("{} results found", results.len());
    }

    Ok(())
}

pub fn generate_excerpt(content: &str, query: &str, highlight: bool) -> Option<String> {
    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();

    if let Some(pos) = content_lower.find(&query_lower) {
        let start = pos.saturating_sub(30);
        let end = (pos + query.len() + 30).min(content.len());

        let excerpt = &content[start..end];

        if highlight {
            let highlighted = excerpt.replace(query, &format!("{}", query.bright_yellow()));
            Some(format!("...{}...", highlighted))
        } else {
            Some(format!("...{}...", excerpt))
        }
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn generate_excerpt_with_long_text(content: &str, query: &str, max_length: usize) -> String {
    let excerpt = generate_excerpt(content, query, false).unwrap_or_default();
    if excerpt.len() > max_length {
        format!("{}...", &excerpt[..max_length.saturating_sub(3)])
    } else {
        excerpt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_excerpt() {
        let content = "This is a test content with some keywords in it";
        let query = "keywords";

        let excerpt = generate_excerpt(content, query, false);
        assert!(excerpt.is_some());
        assert!(excerpt.unwrap().contains("keywords"));
    }

    #[test]
    fn test_generate_excerpt_with_long_text() {
        let content = "This is a very long test content with some keywords that we want to find and excerpt properly";
        let query = "keywords";

        let excerpt = generate_excerpt_with_long_text(content, query, 50);
        assert!(excerpt.len() <= 50);
        assert!(excerpt.contains("..."));
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            name: "test-prompt".to_string(),
            title: Some("Test Prompt".to_string()),
            description: Some("A test prompt".to_string()),
            source: "local".to_string(),
            score: 100.0,
            excerpt: None,
            arguments: vec![],
        };

        assert_eq!(result.name, "test-prompt");
        assert_eq!(result.score, 100.0);
    }
}
