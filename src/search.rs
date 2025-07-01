use anyhow::{Context, Result};
use colored::*;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use is_terminal::IsTerminal;
use regex::Regex;
use serde_json;
use std::io;
use tabled::{
    settings::{object::Rows, Alignment, Color, Modify, Style},
    Table, Tabled,
};
use tantivy::{
    collector::TopDocs, doc, query::QueryParser, schema::*, IndexBuilder, TantivyDocument,
};

use crate::cli::{OutputFormat, PromptSource};
use crate::prompts::{PromptLoader, PromptStorage};

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
    pub description: String,
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

pub fn run_search_command(
    query: String,
    fields: Option<Vec<String>>,
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
    // Load all prompts from all sources
    let storage = PromptStorage::new();
    let mut loader = PromptLoader::new();
    loader.storage = storage.clone();
    loader.load_all()?;

    // Collect prompt information
    let mut prompts = Vec::new();
    for (name, prompt) in storage.iter() {
        let source_str = match &prompt.source {
            crate::prompts::PromptSource::BuiltIn => "builtin",
            crate::prompts::PromptSource::User => "user",
            crate::prompts::PromptSource::Local => "local",
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

        prompts.push((
            name.clone(),
            prompt.title.clone(),
            prompt.description.clone(),
            prompt.content.clone(),
            source_str.to_string(),
            arguments,
        ));
    }

    // Perform search based on mode
    let results = if regex {
        search_with_regex(&query, &prompts, &fields, case_sensitive)?
    } else if fuzzy {
        search_with_fuzzy(&query, &prompts, &fields, case_sensitive)?
    } else {
        search_with_tantivy(&query, &prompts, &fields, case_sensitive)?
    };

    // Apply limit
    let limited_results = if let Some(limit) = limit {
        results.into_iter().take(limit).collect()
    } else {
        results
    };

    // Output results
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&limited_results)?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&limited_results)?;
            print!("{}", yaml);
        }
        OutputFormat::Table => {
            display_search_results(&limited_results, full, highlight)?;
        }
    }

    Ok(())
}

fn search_with_tantivy(
    query: &str,
    prompts: &[(
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Vec<SearchArgument>,
    )],
    fields: &Option<Vec<String>>,
    _case_sensitive: bool,
) -> Result<Vec<SearchResult>> {
    // Create in-memory index
    let mut schema_builder = Schema::builder();
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let description_field = schema_builder.add_text_field("description", TEXT | STORED);
    let content_field = schema_builder.add_text_field("content", TEXT | STORED);
    let arguments_field = schema_builder.add_text_field("arguments", TEXT | STORED);
    let source_field = schema_builder.add_text_field("source", TEXT | STORED);
    let schema = schema_builder.build();

    let index = IndexBuilder::new()
        .schema(schema.clone())
        .create_in_ram()
        .context("Failed to create search index")?;

    let mut index_writer = index.writer(50_000_000)?;

    // Index all prompts
    for (name, title, description, content, source, arguments) in prompts {
        let arguments_text = arguments
            .iter()
            .map(|arg| format!("{}: {}", arg.name, arg.description))
            .collect::<Vec<_>>()
            .join(" ");

        let mut doc = TantivyDocument::default();
        doc.add_text(name_field, name);
        doc.add_text(title_field, title.as_deref().unwrap_or(""));
        doc.add_text(description_field, description.as_deref().unwrap_or(""));
        doc.add_text(content_field, content);
        doc.add_text(arguments_field, &arguments_text);
        doc.add_text(source_field, source);

        index_writer.add_document(doc)?;
    }

    index_writer.commit()?;

    let reader = index.reader()?;
    let searcher = reader.searcher();

    // Determine which fields to search
    let search_fields = if let Some(ref fields) = fields {
        let mut tantivy_fields = Vec::new();
        for field_name in fields {
            match field_name.as_str() {
                "name" => tantivy_fields.push(name_field),
                "title" => tantivy_fields.push(title_field),
                "description" => tantivy_fields.push(description_field),
                "content" => tantivy_fields.push(content_field),
                "arguments" => tantivy_fields.push(arguments_field),
                _ => {}
            }
        }
        tantivy_fields
    } else {
        vec![
            name_field,
            title_field,
            description_field,
            content_field,
            arguments_field,
        ]
    };

    let query_parser = QueryParser::for_index(&index, search_fields);
    let tantivy_query = query_parser.parse_query(query)?;

    let top_docs = searcher.search(&tantivy_query, &TopDocs::with_limit(100))?;

    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        let name = retrieved_doc
            .get_first(name_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let title = retrieved_doc
            .get_first(title_field)
            .and_then(|v| v.as_str())
            .filter(|s: &&str| !s.is_empty())
            .map(|s| s.to_string());
        let description = retrieved_doc
            .get_first(description_field)
            .and_then(|v| v.as_str())
            .filter(|s: &&str| !s.is_empty())
            .map(|s| s.to_string());
        let source = retrieved_doc
            .get_first(source_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Find the corresponding prompt to get arguments
        let arguments = prompts
            .iter()
            .find(|(prompt_name, _, _, _, _, _)| prompt_name == &name)
            .map(|(_, _, _, _, _, args)| args.clone())
            .unwrap_or_default();

        let excerpt = generate_excerpt(query, &description, &title);

        results.push(SearchResult {
            name,
            title,
            description,
            source,
            score,
            excerpt,
            arguments,
        });
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Ok(results)
}

fn search_with_fuzzy(
    query: &str,
    prompts: &[(
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Vec<SearchArgument>,
    )],
    fields: &Option<Vec<String>>,
    case_sensitive: bool,
) -> Result<Vec<SearchResult>> {
    let matcher = SkimMatcherV2::default();
    let mut results = Vec::new();

    for (name, title, description, content, source, arguments) in prompts {
        let mut best_score = None;
        let mut best_excerpt = None;

        // Determine which fields to search
        let search_targets = if let Some(ref fields) = fields {
            let mut targets = Vec::new();
            for field_name in fields {
                match field_name.as_str() {
                    "name" => targets.push((name.clone(), "name")),
                    "title" => {
                        if let Some(ref t) = title {
                            targets.push((t.clone(), "title"));
                        }
                    }
                    "description" => {
                        if let Some(ref d) = description {
                            targets.push((d.clone(), "description"));
                        }
                    }
                    "content" => targets.push((content.clone(), "content")),
                    "arguments" => {
                        let args_text = arguments
                            .iter()
                            .map(|arg| format!("{}: {}", arg.name, arg.description))
                            .collect::<Vec<_>>()
                            .join(" ");
                        targets.push((args_text, "arguments"));
                    }
                    _ => {}
                }
            }
            targets
        } else {
            let mut targets = vec![
                (name.clone(), "name"),
                (content.clone(), "content"),
            ];
            if let Some(ref t) = title {
                targets.push((t.clone(), "title"));
            }
            if let Some(ref d) = description {
                targets.push((d.clone(), "description"));
            }
            let args_text = arguments
                .iter()
                .map(|arg| format!("{}: {}", arg.name, arg.description))
                .collect::<Vec<_>>()
                .join(" ");
            targets.push((args_text, "arguments"));
            targets
        };

        for (text, _field_type) in search_targets {
            let search_text = if case_sensitive { text.clone() } else { text.to_lowercase() };
            let search_query = if case_sensitive { query.to_string() } else { query.to_lowercase() };

            if let Some(score) = matcher.fuzzy_match(&search_text, &search_query) {
                if best_score.is_none() || score > best_score.unwrap() {
                    best_score = Some(score);
                    best_excerpt = generate_excerpt(query, &Some(text.clone()), &None);
                }
            }
        }

        if let Some(score) = best_score {
            results.push(SearchResult {
                name: name.clone(),
                title: title.clone(),
                description: description.clone(),
                source: source.clone(),
                score: score as f32,
                excerpt: best_excerpt,
                arguments: arguments.clone(),
            });
        }
    }

    // Sort by score (descending)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Ok(results)
}

fn search_with_regex(
    query: &str,
    prompts: &[(
        String,
        Option<String>,
        Option<String>,
        String,
        String,
        Vec<SearchArgument>,
    )],
    fields: &Option<Vec<String>>,
    case_sensitive: bool,
) -> Result<Vec<SearchResult>> {
    let regex_flags = if case_sensitive { "" } else { "(?i)" };
    let pattern = format!("{}{}", regex_flags, query);
    let regex = Regex::new(&pattern)
        .with_context(|| format!("Invalid regular expression: {}", query))?;

    let mut results = Vec::new();

    for (name, title, description, content, source, arguments) in prompts {
        let mut matches = false;
        let mut match_excerpt = None;

        // Determine which fields to search
        if let Some(ref fields) = fields {
            for field_name in fields {
                let (found, excerpt) = match field_name.as_str() {
                    "name" => (regex.is_match(name), Some(name.clone())),
                    "title" => {
                        if let Some(ref t) = title {
                            (regex.is_match(t), Some(t.clone()))
                        } else {
                            (false, None)
                        }
                    }
                    "description" => {
                        if let Some(ref d) = description {
                            (regex.is_match(d), Some(d.clone()))
                        } else {
                            (false, None)
                        }
                    }
                    "content" => (regex.is_match(content), Some(content.clone())),
                    "arguments" => {
                        let args_text = arguments
                            .iter()
                            .map(|arg| format!("{}: {}", arg.name, arg.description))
                            .collect::<Vec<_>>()
                            .join(" ");
                        (regex.is_match(&args_text), Some(args_text))
                    }
                    _ => (false, None),
                };

                if found {
                    matches = true;
                    if match_excerpt.is_none() {
                        match_excerpt = excerpt;
                    }
                }
            }
        } else {
            // Search all fields by default
            if regex.is_match(name) {
                matches = true;
                match_excerpt = Some(name.clone());
            } else if let Some(ref t) = title {
                if regex.is_match(t) {
                    matches = true;
                    match_excerpt = Some(t.clone());
                }
            } else if let Some(ref d) = description {
                if regex.is_match(d) {
                    matches = true;
                    match_excerpt = Some(d.clone());
                }
            } else if regex.is_match(content) {
                matches = true;
                match_excerpt = Some(content.clone());
            } else {
                let args_text = arguments
                    .iter()
                    .map(|arg| format!("{}: {}", arg.name, arg.description))
                    .collect::<Vec<_>>()
                    .join(" ");
                if regex.is_match(&args_text) {
                    matches = true;
                    match_excerpt = Some(args_text);
                }
            }
        }

        if matches {
            results.push(SearchResult {
                name: name.clone(),
                title: title.clone(),
                description: description.clone(),
                source: source.clone(),
                score: 1.0, // All regex matches have equal score
                excerpt: match_excerpt,
                arguments: arguments.clone(),
            });
        }
    }

    Ok(results)
}

fn generate_excerpt(
    query: &str,
    description: &Option<String>,
    title: &Option<String>,
) -> Option<String> {
    let text = description
        .as_ref()
        .or(title.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("");

    if text.is_empty() {
        return None;
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    if let Some(pos) = text_lower.find(&query_lower) {
        let start = if pos > 30 { pos - 30 } else { 0 };
        let end = if pos + query.len() + 30 < text.len() {
            pos + query.len() + 30
        } else {
            text.len()
        };

        let excerpt = &text[start..end];
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };

        Some(format!("{}{}{}", prefix, excerpt, suffix))
    } else {
        // Fallback to first 60 characters
        if text.len() > 60 {
            Some(format!("{}...", &text[..57]))
        } else {
            Some(text.to_string())
        }
    }
}

fn display_search_results(
    results: &[SearchResult],
    full: bool,
    _highlight: bool,
) -> Result<()> {
    if results.is_empty() {
        println!("No prompts found matching the search criteria.");
        return Ok(());
    }

    let is_tty = io::stderr().is_terminal();

    if full {
        // Display full details for each result
        for (i, result) in results.iter().enumerate() {
            if i > 0 {
                println!();
            }

            if is_tty {
                println!("{}", format!("{}. {}", i + 1, result.name).bold().cyan());
                if let Some(ref title) = result.title {
                    println!("   {}: {}", "Title".yellow(), title);
                }
                if let Some(ref description) = result.description {
                    println!("   {}: {}", "Description".yellow(), description);
                }
                println!("   {}: {}", "Source".yellow(), result.source);
                println!("   {}: {:.2}", "Score".yellow(), result.score);
                if !result.arguments.is_empty() {
                    println!("   {}:", "Arguments".yellow());
                    for arg in &result.arguments {
                        let req_marker = if arg.required { " (required)" } else { "" };
                        println!("     • {}{}: {}", arg.name, req_marker, arg.description);
                        if let Some(ref default) = arg.default {
                            println!("       Default: {}", default.dimmed());
                        }
                    }
                }
            } else {
                println!("{}. {}", i + 1, result.name);
                if let Some(ref title) = result.title {
                    println!("   Title: {}", title);
                }
                if let Some(ref description) = result.description {
                    println!("   Description: {}", description);
                }
                println!("   Source: {}", result.source);
                println!("   Score: {:.2}", result.score);
                if !result.arguments.is_empty() {
                    println!("   Arguments:");
                    for arg in &result.arguments {
                        let req_marker = if arg.required { " (required)" } else { "" };
                        println!("     • {}{}: {}", arg.name, req_marker, arg.description);
                        if let Some(ref default) = arg.default {
                            println!("       Default: {}", default);
                        }
                    }
                }
            }
        }
    } else {
        // Display compact table
        let rows: Vec<SearchResultRow> = results
            .iter()
            .map(|result| {
                let title = result.title.as_deref().unwrap_or("");
                let excerpt = result
                    .excerpt
                    .as_deref()
                    .unwrap_or(result.description.as_deref().unwrap_or(""))
                    .chars()
                    .take(50)
                    .collect::<String>();
                let excerpt = if excerpt.len() == 50 {
                    format!("{}...", excerpt)
                } else {
                    excerpt
                };

                SearchResultRow {
                    name: result.name.clone(),
                    title: title.to_string(),
                    excerpt,
                    source: result.source.clone(),
                    score: format!("{:.2}", result.score),
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
            println!("{}", "Legend:".bright_white());
            println!("  {} Built-in prompts", "●".green());
            println!("  {} User prompts (~/.swissarmyhammer/prompts/)", "●".blue());
            println!("  {} Local prompts (./prompts/)", "●".yellow());
            println!("  Use {} to see full details", "--full".cyan());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_excerpt() {
        let description = Some("This is a test prompt for debugging code".to_string());
        let title = Some("Debug Helper".to_string());

        let excerpt = generate_excerpt("test", &description, &title);
        assert!(excerpt.is_some());
        assert!(excerpt.unwrap().contains("test"));
    }

    #[test]
    fn test_generate_excerpt_with_long_text() {
        let long_text = Some("This is a very long description that should be truncated when generating an excerpt for display purposes in the search results".to_string());
        let title = None;

        let excerpt = generate_excerpt("long", &long_text, &title);
        assert!(excerpt.is_some());
        let result = excerpt.unwrap();
        assert!(result.contains("long"));
        assert!(result.len() < long_text.as_ref().unwrap().len());
    }

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            name: "test-prompt".to_string(),
            title: Some("Test Prompt".to_string()),
            description: Some("A test prompt".to_string()),
            source: "builtin".to_string(),
            score: 0.85,
            excerpt: Some("A test prompt for testing".to_string()),
            arguments: vec![],
        };

        assert_eq!(result.name, "test-prompt");
        assert_eq!(result.score, 0.85);
        assert!(result.excerpt.is_some());
    }
}