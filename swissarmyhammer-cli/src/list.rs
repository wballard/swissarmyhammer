use anyhow::Result;
use colored::*;
use is_terminal::IsTerminal;
use std::io;
use tabled::{
    settings::{object::Rows, Alignment, Color, Modify, Style},
    Table, Tabled,
};

use crate::cli::{OutputFormat, PromptSource};
use swissarmyhammer::PromptLibrary;

#[derive(Tabled)]
struct PromptRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Arguments")]
    arguments: String,
}

#[derive(serde::Serialize)]
struct PromptInfo {
    name: String,
    title: Option<String>,
    description: Option<String>,
    source: String,
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

    // Collect prompt information
    let mut prompt_infos = Vec::new();

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

        prompt_infos.push(PromptInfo {
            name: prompt.name.clone(),
            title: None, // Title field not in new API
            description: prompt.description.clone(),
            source: source_str.to_string(),
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

fn display_table(prompt_infos: &[PromptInfo], verbose: bool) -> Result<()> {
    if prompt_infos.is_empty() {
        println!("No prompts found matching the criteria.");
        return Ok(());
    }

    let is_tty = io::stderr().is_terminal();

    let rows: Vec<PromptRow> = prompt_infos
        .iter()
        .map(|info| {
            let title = info.title.as_deref().unwrap_or("");
            let description = if verbose {
                info.description.as_deref().unwrap_or("")
            } else {
                // Truncate long descriptions for table display
                let desc = info.description.as_deref().unwrap_or("");
                if desc.len() > 50 {
                    &format!("{}...", &desc[..47])
                } else {
                    desc
                }
            };

            let arguments = if verbose {
                info.arguments
                    .iter()
                    .map(|arg| {
                        if arg.required {
                            format!("{}*", arg.name)
                        } else {
                            arg.name.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                format!("{}", info.arguments.len())
            };

            PromptRow {
                name: info.name.clone(),
                title: title.to_string(),
                description: description.to_string(),
                source: info.source.clone(),
                arguments,
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::modern());

    if is_tty {
        // Add colors for better readability in terminal
        table.with(Modify::new(Rows::single(0)).with(Color::FG_BRIGHT_CYAN));

        // Color code sources
        for (i, info) in prompt_infos.iter().enumerate() {
            let row_index = i + 1; // +1 because row 0 is header
            match info.source.as_str() {
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

    if is_tty && !prompt_infos.is_empty() {
        println!();
        println!("{}", "Legend:".bright_white());
        println!("  {} Built-in prompts", "●".green());
        println!(
            "  {} User prompts (~/.swissarmyhammer/prompts/)",
            "●".blue()
        );
        println!("  {} Local prompts (./prompts/)", "●".yellow());
        if verbose {
            println!("  {} Required argument", "*".red());
        } else {
            println!("  Use {} to see full details", "--verbose".cyan());
        }
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
    fn test_prompt_row_creation() {
        let row = PromptRow {
            name: "test".to_string(),
            title: "Test Prompt".to_string(),
            description: "A test prompt".to_string(),
            source: "builtin".to_string(),
            arguments: "1".to_string(),
        };

        assert_eq!(row.name, "test");
        assert_eq!(row.source, "builtin");
    }
}
