//! Prompt command implementation for managing prompts

use crate::cli::PromptSubcommand;
use crate::{list, search, test, validate};
use swissarmyhammer::Result;

/// Main entry point for prompt command
pub async fn run_prompt_command(subcommand: PromptSubcommand) -> Result<i32> {
    match subcommand {
        PromptSubcommand::List {
            format,
            verbose,
            source,
            category,
            search,
        } => Ok(
            list::run_list_command(format, verbose, source, category, search)
                .map(|_| 0)
                .unwrap_or_else(|e| {
                    eprintln!("List error: {}", e);
                    1
                }),
        ),
        PromptSubcommand::Validate {
            quiet,
            format,
            workflow_dirs,
        } => Ok(
            validate::run_validate_command(quiet, format, workflow_dirs).unwrap_or_else(|e| {
                eprintln!("Validation error: {}", e);
                2
            }),
        ),
        PromptSubcommand::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        } => {
            let mut runner = test::TestRunner::new();
            Ok(runner
                .run(&prompt_name, &file, &arguments, raw, copy, &save, debug)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("Test error: {}", e);
                    1
                }))
        }
        PromptSubcommand::Search {
            query,
            r#in,
            regex,
            fuzzy,
            case_sensitive,
            source,
            has_arg,
            no_args,
            full,
            format,
            highlight,
            limit,
        } => Ok(search::run_search_command(
            query,
            r#in,
            regex,
            fuzzy,
            case_sensitive,
            source,
            has_arg,
            no_args,
            full,
            format,
            highlight,
            limit,
        )
        .map(|_| 0)
        .unwrap_or_else(|e| {
            eprintln!("Search error: {}", e);
            1
        })),
    }
}
