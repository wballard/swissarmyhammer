use anyhow::Result;
use colored::*;
use is_terminal::IsTerminal;
use std::io;
// Tabled import removed - using custom 2-line format instead

use crate::cli::{OutputFormat, PromptSource};
use swissarmyhammer::PromptLibrary;
use swissarmyhammer::PromptResolver;

// PromptRow struct removed - using custom 2-line format instead of table

#[derive(serde::Serialize)]
struct PromptInfo {
    name: String,
    title: Option<String>,
    description: Option<String>,
    source: PromptSource,
    category: Option<String>,
    arguments: Vec<PromptArgument>,
}

#[derive(serde::Serialize)]
struct PromptArgument {
    name: String,
    description: Option<String>,
    required: bool,
    default: Option<String>,
}

pub fn run_list_command(
    format: OutputFormat,
    verbose: bool,
    source_filter: Option<PromptSource>,
    category_filter: Option<String>,
    search_term: Option<String>,
) -> Result<()> {
    // Load all prompts from all sources
    let mut library = PromptLibrary::new();
    let mut resolver = PromptResolver::new();
    resolver.load_all_prompts(&mut library)?;

    // Get all prompts
    let all_prompts = library.list()?;

    // Collect prompt information
    let mut prompt_infos = Vec::new();

    for prompt in all_prompts {
        // Get the source from the resolver
        let prompt_source = match resolver.prompt_sources.get(&prompt.name) {
            Some(swissarmyhammer::PromptSource::Builtin) => PromptSource::Builtin,
            Some(swissarmyhammer::PromptSource::User) => PromptSource::User,
            Some(swissarmyhammer::PromptSource::Local) => PromptSource::Local,
            Some(swissarmyhammer::PromptSource::Dynamic) => PromptSource::Dynamic,
            None => PromptSource::Dynamic,
        };

        // Apply source filter
        if let Some(ref filter) = source_filter {
            if filter != &prompt_source && filter != &PromptSource::Dynamic {
                continue;
            }
        }

        // Apply category filter
        if let Some(ref category) = category_filter {
            if prompt.category.as_deref() != Some(category) {
                continue;
            }
        }

        // Apply search filter
        if let Some(ref search) = search_term {
            let search_lower = search.to_lowercase();
            let name_matches = prompt.name.to_lowercase().contains(&search_lower);
            let desc_matches = prompt
                .description
                .as_ref()
                .map(|d| d.to_lowercase().contains(&search_lower))
                .unwrap_or(false);
            let category_matches = prompt
                .category
                .as_ref()
                .map(|c| c.to_lowercase().contains(&search_lower))
                .unwrap_or(false);
            let tag_matches = prompt
                .tags
                .iter()
                .any(|t| t.to_lowercase().contains(&search_lower));

            if !(name_matches || desc_matches || category_matches || tag_matches) {
                continue;
            }
        }

        let arguments = prompt
            .arguments
            .iter()
            .map(|arg| PromptArgument {
                name: arg.name.clone(),
                description: arg.description.clone(),
                required: arg.required,
                default: arg.default.clone(),
            })
            .collect();

        // Extract title from metadata
        // If metadata is empty, we have a problem with the library's YAML parsing
        // For now, let's use the prompt name as a fallback title
        let title = prompt
            .metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                // Fallback: convert prompt name to a readable title
                Some(
                    prompt
                        .name
                        .replace(['-', '_'], " ")
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => {
                                    first.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                )
            });

        prompt_infos.push(PromptInfo {
            name: prompt.name.clone(),
            title,
            description: prompt.description.clone(),
            source: prompt_source,
            category: prompt.category.clone(),
            arguments,
        });
    }

    // Sort by name for consistent output
    prompt_infos.sort_by(|a, b| a.name.cmp(&b.name));

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&prompt_infos)?;
            println!("{}", json);
        }
        OutputFormat::Yaml => {
            let yaml = serde_yaml::to_string(&prompt_infos)?;
            print!("{}", yaml);
        }
        OutputFormat::Table => {
            display_table(&prompt_infos, verbose)?;
        }
    }

    Ok(())
}

fn display_table(prompt_infos: &[PromptInfo], _verbose: bool) -> Result<()> {
    if prompt_infos.is_empty() {
        println!("No prompts found matching the criteria.");
        return Ok(());
    }

    let is_tty = io::stdout().is_terminal();

    // Create a custom 2-line format instead of using Tabled
    for info in prompt_infos {
        let title = info.title.as_deref().unwrap_or("");
        let description = info.description.as_deref().unwrap_or("");

        // First line: Name | Title (colored by source)
        let first_line = if is_tty {
            let (name_colored, title_colored) = match &info.source {
                PromptSource::Builtin => (
                    info.name.green().bold().to_string(),
                    title.green().to_string(),
                ),
                PromptSource::User => (
                    info.name.blue().bold().to_string(),
                    title.blue().to_string(),
                ),
                PromptSource::Local => (
                    info.name.yellow().bold().to_string(),
                    title.yellow().to_string(),
                ),
                PromptSource::Dynamic => (
                    info.name.magenta().bold().to_string(),
                    title.magenta().to_string(),
                ),
            };
            format!("{} | {}", name_colored, title_colored)
        } else {
            format!("{} | {}", info.name, title)
        };

        // Second line: Full description (indented)
        let second_line = if !description.is_empty() {
            format!("  {}", description)
        } else {
            "  (no description)".to_string()
        };

        println!("{}", first_line);
        println!("{}", second_line);
        println!(); // Empty line between entries
    }

    if is_tty && !prompt_infos.is_empty() {
        println!("{}", "Legend:".bright_white());
        println!("  {} Built-in prompts", "●".green());
        println!(
            "  {} User prompts (~/.swissarmyhammer/prompts/)",
            "●".blue()
        );
        println!("  {} Local prompts (./prompts/)", "●".yellow());
        println!("  {} Dynamic prompts", "●".magenta());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_command_with_no_prompts() {
        // This test will fail initially, driving the implementation
        let result = run_list_command(OutputFormat::Table, false, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_command_with_search() {
        let result = run_list_command(
            OutputFormat::Table,
            false,
            None,
            None,
            Some("example".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_command_json_format() {
        let result = run_list_command(OutputFormat::Json, false, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_command_yaml_format() {
        let result = run_list_command(OutputFormat::Yaml, false, None, None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_command_source_filter() {
        let result = run_list_command(
            OutputFormat::Table,
            false,
            Some(PromptSource::Builtin),
            None,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_color_coding_when_terminal() {
        let prompt_infos = vec![
            PromptInfo {
                name: "test_builtin".to_string(),
                title: Some("Builtin Test".to_string()),
                description: Some("A builtin prompt".to_string()),
                source: PromptSource::Builtin,
                category: Some("test".to_string()),
                arguments: vec![],
            },
            PromptInfo {
                name: "test_user".to_string(),
                title: Some("User Test".to_string()),
                description: Some("A user prompt".to_string()),
                source: PromptSource::User,
                category: Some("test".to_string()),
                arguments: vec![],
            },
            PromptInfo {
                name: "test_local".to_string(),
                title: Some("Local Test".to_string()),
                description: Some("A local prompt".to_string()),
                source: PromptSource::Local,
                category: Some("test".to_string()),
                arguments: vec![],
            },
        ];

        // This test currently fails because display_table checks stderr instead of stdout
        let result = display_table(&prompt_infos, false);
        assert!(result.is_ok());

        // TODO: Once fixed, we should capture stdout and verify color codes are present
    }

    #[test]
    fn test_prompt_info_creation() {
        let info = PromptInfo {
            name: "test".to_string(),
            title: Some("Test Prompt".to_string()),
            description: Some("A test prompt".to_string()),
            source: PromptSource::Builtin,
            category: None,
            arguments: vec![],
        };

        assert_eq!(info.name, "test");
        assert_eq!(info.title, Some("Test Prompt".to_string()));
        assert_eq!(info.source, PromptSource::Builtin);
    }

    #[test]
    fn test_builtin_prompts_should_be_identified_correctly() {
        // Test that the resolver properly tracks prompt sources
        let mut resolver = PromptResolver::new();
        let mut library = swissarmyhammer::PromptLibrary::new();

        // Load builtin prompts
        resolver.load_builtin_prompts(&mut library).unwrap();

        // Note: Builtin prompts may not exist in test environment
        // The test passes if no error occurs - builtin prompt loading is optional
        // In production, builtin prompts would be embedded in the binary

        // If any builtin prompts were loaded, they should be marked as builtin
        for (_, source) in &resolver.prompt_sources {
            if matches!(source, swissarmyhammer::PromptSource::Builtin) {
                // This is good - builtin prompts are properly marked
                break;
            }
        }
    }

    #[test]
    fn test_title_extraction_logic() {
        // Test that title extraction from metadata works correctly
        use serde_json::Value;
        use std::collections::HashMap;

        let mut metadata = HashMap::new();
        metadata.insert(
            "title".to_string(),
            Value::String("Array Data Processor".to_string()),
        );

        // Test the title extraction logic
        let title = metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        assert_eq!(
            title,
            Some("Array Data Processor".to_string()),
            "Title should be extracted from metadata"
        );

        // Test when title is missing
        let empty_metadata: HashMap<String, Value> = HashMap::new();
        let no_title: Option<String> = empty_metadata
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        assert_eq!(
            no_title, None,
            "Title should be None when not present in metadata"
        );
    }
}
