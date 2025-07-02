use anyhow::Result;
use colored::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use is_terminal::IsTerminal;
use regex::Regex;
use std::io;
use tabled::{
    settings::{object::Rows, Alignment, Color, Modify, Style},
    Table, Tabled,
};

use crate::cli::{OutputFormat, PromptSource};

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
    source_filter: Option<PromptSource>,
    has_arg: Option<String>,
    no_args: bool,
    full: bool,
    format: OutputFormat,
    highlight: bool,
    limit: Option<usize>,
) -> Result<()> {
    use swissarmyhammer::PromptLibrary;

    // Load all prompts from all sources
    let mut library = PromptLibrary::new();

    // Load builtin prompts
    let builtin_dir = dirs::data_dir()
        .map(|d| d.join("swissarmyhammer").join("prompts"))
        .filter(|p| p.exists());

    if let Some(dir) = builtin_dir {
        library.add_directory(&dir)?;
    }

    // Load user prompts
    let user_dir = dirs::home_dir()
        .map(|d| d.join(".prompts"))
        .filter(|p| p.exists());

    if let Some(dir) = user_dir {
        library.add_directory(&dir)?;
    }

    // Load local prompts
    let local_dir = std::path::Path::new("prompts");
    if local_dir.exists() {
        library.add_directory(local_dir)?;
    }

    // Get all prompts
    let all_prompts = library.list()?;

    // Search and filter prompts
    let mut results = Vec::new();

    for prompt in all_prompts {
        // Determine source based on path
        let source_str = if let Some(source_path) = &prompt.source {
            let path_str = source_path.to_string_lossy();
            if path_str.contains(".swissarmyhammer") || path_str.contains("data") {
                "builtin"
            } else if path_str.contains(".prompts") {
                "user"
            } else {
                "local"
            }
        } else {
            "unknown"
        };

        // Apply source filter
        if let Some(ref filter) = source_filter {
            let filter_matches = match filter {
                PromptSource::Builtin => source_str == "builtin",
                PromptSource::User => source_str == "user",
                PromptSource::Local => source_str == "local",
            };
            if !filter_matches {
                continue;
            }
        }

        // Apply argument filters
        if let Some(ref arg_name) = has_arg {
            if !prompt.arguments.iter().any(|arg| arg.name == *arg_name) {
                continue;
            }
        }

        if no_args && !prompt.arguments.is_empty() {
            continue;
        }

        // Perform search
        let mut score = 0.0;
        let mut matched = false;
        let query_lower = if case_sensitive {
            query.clone()
        } else {
            query.to_lowercase()
        };

        if regex {
            let re = Regex::new(&query)?;
            matched = re.is_match(&prompt.name)
                || prompt
                    .description
                    .as_ref()
                    .map(|d| re.is_match(d))
                    .unwrap_or(false)
                || prompt.template.contains(&query);
        } else if fuzzy {
            let matcher = SkimMatcherV2::default();
            if let Some(s) = matcher.fuzzy_match(&prompt.name, &query) {
                score = s as f32;
                matched = true;
            }
            if let Some(desc) = &prompt.description {
                if let Some(s) = matcher.fuzzy_match(desc, &query) {
                    score = score.max(s as f32);
                    matched = true;
                }
            }
        } else {
            // Simple substring search
            let name_check = if case_sensitive {
                &prompt.name
            } else {
                &prompt.name.to_lowercase()
            };
            matched = name_check.contains(&query_lower)
                || prompt
                    .description
                    .as_ref()
                    .map(|d| {
                        if case_sensitive {
                            d.contains(&query)
                        } else {
                            d.to_lowercase().contains(&query_lower)
                        }
                    })
                    .unwrap_or(false)
                || prompt.template.contains(&query);

            if matched {
                score = 100.0;
            }
        }

        if matched {
            let excerpt = if highlight {
                generate_excerpt(&prompt.template, &query, highlight)
            } else {
                None
            };

            let arguments = prompt
                .arguments
                .iter()
                .map(|arg| SearchArgument {
                    name: arg.name.clone(),
                    description: arg.description.clone(),
                    required: arg.required,
                    default: arg.default.clone(),
                })
                .collect();

            results.push(SearchResult {
                name: prompt.name.clone(),
                title: None, // No title field in new API
                description: prompt.description.clone(),
                source: source_str.to_string(),
                score,
                excerpt,
                arguments,
            });
        }
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    // Apply limit
    if let Some(limit) = limit {
        results.truncate(limit);
    }

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

    let is_tty = io::stderr().is_terminal();

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
