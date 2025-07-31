use crate::cli::IssueCommands;
use crate::mcp_integration::{response_formatting, CliToolContext};
use colored::*;
use serde_json::json;
use std::io::{self, Read};
use swissarmyhammer::issues::IssueStorage;

// Removed NAMELESS_ISSUE_NAME constant as it's no longer used

fn format_issue_status(completed: bool) -> colored::ColoredString {
    if completed {
        "✅ Completed".green()
    } else {
        "🔄 Active".yellow()
    }
}

pub async fn handle_issue_command(
    command: IssueCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = CliToolContext::new().await?;

    match command {
        IssueCommands::Create {
            name,
            content,
            file,
        } => {
            create_issue(&context, name, content, file).await?;
        }
        IssueCommands::List {
            completed,
            active,
            format,
        } => {
            list_issues(&context, completed, active, format).await?;
        }
        IssueCommands::Show { name, raw } => {
            show_issue(&context, &name, raw).await?;
        }
        IssueCommands::Update {
            name,
            content,
            file,
            append,
        } => {
            update_issue(&context, &name, content, file, append).await?;
        }
        IssueCommands::Complete { name } => {
            complete_issue(&context, &name).await?;
        }
        IssueCommands::Work { name } => {
            work_issue(&context, &name).await?;
        }
        IssueCommands::Merge { name, keep_branch } => {
            merge_issue(&context, &name, keep_branch).await?;
        }
        IssueCommands::Current => {
            show_current_issue(&context).await?;
        }
        IssueCommands::Status => {
            show_status(&context).await?;
        }
        IssueCommands::Next => {
            show_next_issue(&context).await?;
        }
    }

    Ok(())
}

async fn create_issue(
    context: &CliToolContext,
    name: Option<String>,
    content: Option<String>,
    file: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_from_args(content, file)?;
    // Ensure content is not empty for MCP tool compatibility
    let content = if content.is_empty() {
        "# Issue\n\nDescribe the issue here.".to_string()
    } else {
        content
    };

    let args = if let Some(issue_name) = name {
        context.create_arguments(vec![
            ("name", json!(issue_name)),
            ("content", json!(content)),
        ])
    } else {
        // For nameless issues, don't pass a name argument
        context.create_arguments(vec![("content", json!(content))])
    };

    let result = context.execute_tool("issue_create", args).await?;
    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn list_issues(
    _context: &CliToolContext,
    show_completed: bool,
    show_active: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // For now, create direct storage access since there's no MCP list tool
    // This maintains existing functionality while we transition
    let storage = swissarmyhammer::issues::FileSystemIssueStorage::new_default()?;
    let all_issues = storage.list_issues().await?;

    let issues: Vec<_> = all_issues
        .into_iter()
        .filter(|issue| {
            if show_completed && show_active {
                true // show all
            } else if show_completed {
                issue.completed
            } else if show_active {
                !issue.completed
            } else {
                true // default: show all
            }
        })
        .collect();

    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&issues)?);
        }
        "markdown" => {
            print_issues_markdown(&issues).await?;
        }
        _ => {
            print_issues_table(&issues).await?;
        }
    }

    Ok(())
}

async fn print_issues_table(
    issues: &[swissarmyhammer::issues::Issue],
) -> Result<(), Box<dyn std::error::Error>> {
    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    let active_issues: Vec<_> = issues.iter().filter(|i| !i.completed).collect();
    let completed_issues: Vec<_> = issues.iter().filter(|i| i.completed).collect();

    let total_issues = issues.len();
    let completed_count = completed_issues.len();
    let active_count = active_issues.len();
    let completion_percentage = if total_issues > 0 {
        (completed_count * 100) / total_issues
    } else {
        0
    };

    println!("📊 Issues: {total_issues} total");
    println!("✅ Completed: {completed_count} ({completion_percentage}%)");
    println!("🔄 Active: {active_count}");

    if active_count > 0 {
        println!();
        println!("{}", "Active Issues:".bold());
        for issue in active_issues {
            println!("  🔄 {}", issue.name);
        }
    }

    if completed_count > 0 {
        println!();
        println!("{}", "Recently Completed:".bold());
        let mut sorted_completed = completed_issues;
        sorted_completed.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        for issue in sorted_completed.iter().take(5) {
            println!("  ✅ {}", issue.name);
        }
    }

    Ok(())
}

async fn print_issues_markdown(
    issues: &[swissarmyhammer::issues::Issue],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("# Issues");
    println!();

    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }

    for issue in issues {
        let status = if issue.completed { "✅" } else { "🔄" };
        println!("## {} - {}", status, issue.name);
        println!();
        println!(
            "- **Status**: {}",
            if issue.completed {
                "Completed"
            } else {
                "Active"
            }
        );
        println!("- **Created**: {}", issue.created_at.format("%Y-%m-%d"));
        println!("- **File**: {}", issue.file_path.display());
        println!();
        if !issue.content.is_empty() {
            println!("### Content");
            println!();
            println!("{}", issue.content);
            println!();
        }
        println!("---");
        println!();
    }

    Ok(())
}

async fn show_issue(
    _context: &CliToolContext,
    name: &str,
    raw: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // For now, create direct storage access since there's no MCP show tool
    // This maintains existing functionality while we transition
    let storage = swissarmyhammer::issues::FileSystemIssueStorage::new_default()?;
    let issues = storage.list_issues().await?;
    let issue = issues
        .into_iter()
        .find(|i| i.name == name)
        .ok_or_else(|| format!("Issue '{name}' not found"))?;

    if raw {
        println!("{}", issue.content);
    } else {
        let status = format_issue_status(issue.completed);

        println!("{} Issue: {}", status, issue.name.as_str().bold());
        println!("📁 File: {}", issue.file_path.display());
        println!(
            "📅 Created: {}",
            issue.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!();
        println!("{}", issue.content);
    }

    Ok(())
}

async fn update_issue(
    context: &CliToolContext,
    name: &str,
    content: Option<String>,
    file: Option<std::path::PathBuf>,
    append: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_content = get_content_from_args(content, file)?;

    let args = context.create_arguments(vec![
        ("name", json!(name)),
        ("content", json!(new_content)),
        ("append", json!(append)),
    ]);

    let result = context.execute_tool("issue_update", args).await?;
    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn complete_issue(
    context: &CliToolContext,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![("name", json!(name))]);
    let result = context.execute_tool("issue_mark_complete", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn work_issue(
    context: &CliToolContext,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![("name", json!(name))]);
    let result = context.execute_tool("issue_work", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn merge_issue(
    context: &CliToolContext,
    name: &str,
    keep_branch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![
        ("name", json!(name)),
        ("delete_branch", json!(!keep_branch)),
    ]);

    let result = context.execute_tool("issue_merge", args).await?;
    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn show_current_issue(context: &CliToolContext) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("issue_current", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn show_status(context: &CliToolContext) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("issue_all_complete", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

fn get_content_from_args(
    content: Option<String>,
    file: Option<std::path::PathBuf>,
) -> Result<String, Box<dyn std::error::Error>> {
    match (content, file) {
        (Some(content), None) => {
            if content == "-" {
                // Read from stdin
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                Ok(buffer.trim().to_string())
            } else {
                Ok(content)
            }
        }
        (None, Some(path)) => {
            let content = std::fs::read_to_string(path)?;
            Ok(content.trim().to_string())
        }
        (Some(_), Some(_)) => Err("Cannot specify both --content and --file options".into()),
        (None, None) => {
            // Allow empty content for nameless issues
            Ok(String::new())
        }
    }
}

async fn show_next_issue(context: &CliToolContext) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("issue_next", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}
