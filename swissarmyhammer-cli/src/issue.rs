use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use swissarmyhammer::git::GitOperations;
use crate::cli::IssueCommands;
use colored::*;
use std::io::{self, Read};

pub async fn handle_issue_command(command: IssueCommands) -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileSystemIssueStorage::new_default()?;
    
    match command {
        IssueCommands::Create { name, content, file } => {
            create_issue(storage, name, content, file).await?;
        }
        IssueCommands::List { completed, active, format } => {
            list_issues(storage, completed, active, format).await?;
        }
        IssueCommands::Show { number, raw } => {
            show_issue(storage, number, raw).await?;
        }
        IssueCommands::Update { number, content, file, append } => {
            update_issue(storage, number, content, file, append).await?;
        }
        IssueCommands::Complete { number } => {
            complete_issue(storage, number).await?;
        }
        IssueCommands::Work { number } => {
            work_issue(storage, number).await?;
        }
        IssueCommands::Merge { number, keep_branch } => {
            merge_issue(storage, number, keep_branch).await?;
        }
        IssueCommands::Current => {
            show_current_issue(storage).await?;
        }
        IssueCommands::Status => {
            show_status(storage).await?;
        }
    }
    
    Ok(())
}

async fn create_issue(
    storage: FileSystemIssueStorage,
    name: String,
    content: Option<String>,
    file: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_from_args(content, file)?;
    
    let issue = storage.create_issue(name, content).await?;
    
    println!("{} Created issue #{:06} - {}", 
        "✅".green(), 
        issue.number, 
        issue.name.bold()
    );
    
    println!("📁 File: {}", issue.file_path.display());
    
    Ok(())
}

async fn list_issues(
    storage: FileSystemIssueStorage,
    show_completed: bool,
    show_active: bool,
    format: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let issues = storage.list_issues().await?;
    
    let filtered_issues: Vec<_> = issues.into_iter()
        .filter(|issue| {
            match (show_completed, show_active) {
                (true, false) => issue.completed,
                (false, true) => !issue.completed,
                _ => true, // Show all if both flags or neither flag
            }
        })
        .collect();
    
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&filtered_issues)?;
            println!("{}", json);
        }
        "markdown" => {
            print_issues_markdown(&filtered_issues)?;
        }
        "table" | _ => {
            print_issues_table(&filtered_issues)?;
        }
    }
    
    Ok(())
}

fn print_issues_table(issues: &[swissarmyhammer::issues::Issue]) -> Result<(), Box<dyn std::error::Error>> {
    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }
    
    println!("{}", "Issues".bold().underline());
    println!();
    
    for issue in issues {
        let status = if issue.completed {
            "✅ Completed".green()
        } else {
            "🔄 Active".yellow()
        };
        
        println!("{} #{:06} - {} {}", 
            status,
            issue.number,
            issue.name.bold(),
            format!("({})", issue.created_at.format("%Y-%m-%d")).dimmed()
        );
    }
    
    println!();
    println!("Total: {} issues", issues.len());
    
    Ok(())
}

fn print_issues_markdown(issues: &[swissarmyhammer::issues::Issue]) -> Result<(), Box<dyn std::error::Error>> {
    println!("# Issues");
    println!();
    
    if issues.is_empty() {
        println!("No issues found.");
        return Ok(());
    }
    
    for issue in issues {
        let status = if issue.completed { "✅" } else { "🔄" };
        println!("## {} #{:06} - {}", status, issue.number, issue.name);
        println!();
        println!("- **Status**: {}", if issue.completed { "Completed" } else { "Active" });
        println!("- **Created**: {}", issue.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("- **File**: `{}`", issue.file_path.display());
        println!();
        
        // Show first few lines of content
        let lines: Vec<&str> = issue.content.lines().collect();
        if lines.len() > 3 {
            for line in &lines[..3] {
                println!("{}", line);
            }
            println!("...");
        } else {
            println!("{}", issue.content);
        }
        println!();
    }
    
    Ok(())
}

async fn show_issue(
    storage: FileSystemIssueStorage,
    number: u32,
    raw: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = storage.get_issue(number).await?;
    
    if raw {
        println!("{}", issue.content);
    } else {
        let status = if issue.completed {
            "✅ Completed".green()
        } else {
            "🔄 Active".yellow()
        };
        
        println!("{} Issue #{:06} - {}", status, issue.number, issue.name.bold());
        println!("📁 File: {}", issue.file_path.display());
        println!("📅 Created: {}", issue.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("{}", "Content:".bold());
        println!("{}", issue.content);
    }
    
    Ok(())
}

async fn update_issue(
    storage: FileSystemIssueStorage,
    number: u32,
    content: Option<String>,
    file: Option<std::path::PathBuf>,
    append: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let new_content = get_content_from_args(content, file)?;
    
    let issue = storage.get_issue(number).await?;
    
    let updated_content = if append {
        if issue.content.is_empty() {
            new_content
        } else {
            format!("{}\n\n{}", issue.content, new_content)
        }
    } else {
        new_content
    };
    
    let updated_issue = storage.update_issue(number, updated_content).await?;
    
    println!("{} Updated issue #{:06} - {}", 
        "✅".green(), 
        updated_issue.number, 
        updated_issue.name.bold()
    );
    
    Ok(())
}

async fn complete_issue(
    storage: FileSystemIssueStorage,
    number: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = storage.get_issue(number).await?;
    
    if issue.completed {
        println!("ℹ️ Issue #{:06} - {} is already completed", issue.number, issue.name);
        return Ok(());
    }
    
    let completed_issue = storage.mark_complete(number).await?;
    
    println!("{} Marked issue #{:06} - {} as complete", 
        "✅".green(), 
        completed_issue.number, 
        completed_issue.name.bold()
    );
    
    println!("📁 Moved to: {}", completed_issue.file_path.display());
    
    Ok(())
}

async fn work_issue(
    storage: FileSystemIssueStorage,
    number: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = storage.get_issue(number).await?;
    
    if issue.completed {
        println!("⚠️ Issue #{:06} - {} is already completed", issue.number, issue.name);
        return Ok(());
    }
    
    let git_ops = GitOperations::new()?;
    let branch_identifier = format!("{:06}_{}", issue.number, issue.name);
    let branch_name = git_ops.create_work_branch(&branch_identifier)?;
    
    println!("{} Started working on issue #{:06} - {}", 
        "🔄".yellow(), 
        issue.number, 
        issue.name.bold()
    );
    
    println!("🌿 Branch: {}", branch_name.bold());
    
    Ok(())
}

async fn merge_issue(
    storage: FileSystemIssueStorage,
    number: u32,
    keep_branch: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let issue = storage.get_issue(number).await?;
    
    if !issue.completed {
        println!("⚠️ Issue #{:06} - {} is not completed", issue.number, issue.name);
        println!("Complete the issue first with: swissarmyhammer issue complete {}", number);
        return Ok(());
    }
    
    let git_ops = GitOperations::new()?;
    let branch_identifier = format!("{:06}_{}", issue.number, issue.name);
    
    git_ops.merge_issue_branch(&branch_identifier)?;
    
    if !keep_branch {
        // Delete the branch
        let branch_name = format!("issue/{}", branch_identifier);
        if let Err(e) = git_ops.delete_branch(&branch_name) {
            eprintln!("Warning: Failed to delete branch {}: {}", branch_name, e);
        }
    }
    
    println!("{} Merged issue #{:06} - {} to main", 
        "✅".green(), 
        issue.number, 
        issue.name.bold()
    );
    
    Ok(())
}

async fn show_current_issue(
    storage: FileSystemIssueStorage,
) -> Result<(), Box<dyn std::error::Error>> {
    let git_ops = GitOperations::new()?;
    let current_branch = git_ops.current_branch()?;
    
    if current_branch.starts_with("issue/") {
        let identifier = current_branch.strip_prefix("issue/").unwrap();
        
        // Try to parse issue number from branch
        if let Ok((number, _)) = swissarmyhammer::issues::parse_issue_filename(identifier) {
            match storage.get_issue(number).await {
                Ok(issue) => {
                    let status = if issue.completed {
                        "✅ Completed".green()
                    } else {
                        "🔄 Active".yellow()
                    };
                    
                    println!("{} Current issue: #{:06} - {}", 
                        status, 
                        issue.number, 
                        issue.name.bold()
                    );
                    println!("🌿 Branch: {}", current_branch.bold());
                    println!("📁 File: {}", issue.file_path.display());
                }
                Err(_) => {
                    println!("⚠️ On issue branch '{}' but issue not found", current_branch);
                }
            }
        } else {
            println!("⚠️ On issue branch '{}' but cannot parse issue number", current_branch);
        }
    } else {
        println!("ℹ️ Not currently working on a specific issue");
        println!("🌿 Current branch: {}", current_branch.bold());
    }
    
    Ok(())
}

async fn show_status(
    storage: FileSystemIssueStorage,
) -> Result<(), Box<dyn std::error::Error>> {
    let issues = storage.list_issues().await?;
    
    let total_issues = issues.len();
    let completed_count = issues.iter().filter(|i| i.completed).count();
    let active_count = total_issues - completed_count;
    
    println!("{}", "Project Status".bold().underline());
    println!();
    
    if total_issues == 0 {
        println!("📋 No issues found");
        println!("Create your first issue with: swissarmyhammer issue create <name>");
        return Ok(());
    }
    
    let completion_percentage = if total_issues > 0 {
        (completed_count * 100) / total_issues
    } else {
        0
    };
    
    println!("📊 Issues: {} total", total_issues);
    println!("✅ Completed: {} ({}%)", completed_count, completion_percentage);
    println!("🔄 Active: {}", active_count);
    
    if active_count > 0 {
        println!();
        println!("{}", "Active Issues:".bold());
        for issue in issues.iter().filter(|i| !i.completed) {
            println!("  #{:06} - {}", issue.number, issue.name);
        }
    }
    
    if completed_count > 0 {
        println!();
        println!("{}", "Recently Completed:".bold());
        let mut completed_issues: Vec<_> = issues.iter()
            .filter(|i| i.completed)
            .collect();
        completed_issues.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        for issue in completed_issues.iter().take(5) {
            println!("  #{:06} - {}", issue.number, issue.name);
        }
    }
    
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
                Ok(buffer)
            } else {
                Ok(content)
            }
        }
        (None, Some(file)) => {
            Ok(std::fs::read_to_string(file)?)
        }
        (Some(_), Some(_)) => {
            Err("Cannot specify both --content and --file".into())
        }
        (None, None) => {
            Err("Must specify either --content or --file".into())
        }
    }
}