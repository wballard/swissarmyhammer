use crate::cli::MemoCommands;
use colored::*;
use std::io::{self, Read};
use swissarmyhammer::memoranda::{FileSystemMemoStorage, MemoId, MemoStorage};

// Preview length constants
const LIST_PREVIEW_LENGTH: usize = 100;
const SEARCH_PREVIEW_LENGTH: usize = 150;

/// Format content preview with specified maximum length
fn format_content_preview(content: &str, max_length: usize) -> String {
    let preview = if content.len() > max_length {
        format!("{}...", &content[..max_length])
    } else {
        content.to_string()
    };
    preview.replace('\n', " ")
}

pub async fn handle_memo_command(command: MemoCommands) -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileSystemMemoStorage::new_default()?;

    match command {
        MemoCommands::Create { title, content } => {
            create_memo(storage, title, content).await?;
        }
        MemoCommands::List => {
            list_memos(storage).await?;
        }
        MemoCommands::Get { id } => {
            get_memo(storage, &id).await?;
        }
        MemoCommands::Update { id, content } => {
            update_memo(storage, &id, content).await?;
        }
        MemoCommands::Delete { id } => {
            delete_memo(storage, &id).await?;
        }
        MemoCommands::Search { query } => {
            search_memos(storage, &query).await?;
        }
        MemoCommands::Context => {
            get_context(storage).await?;
        }
    }

    Ok(())
}

async fn create_memo(
    storage: FileSystemMemoStorage,
    title: String,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_input(content)?;

    let memo = storage.create_memo(title, content).await?;

    println!("{} Created memo: {}", "✅".green(), memo.title.bold());

    println!("🆔 ID: {}", memo.id.as_str().blue());

    println!(
        "📅 Created: {}",
        memo.created_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .dimmed()
    );

    Ok(())
}

async fn list_memos(storage: FileSystemMemoStorage) -> Result<(), Box<dyn std::error::Error>> {
    let memos = storage.list_memos().await?;

    if memos.is_empty() {
        println!("{} No memos found", "ℹ️".blue());
        return Ok(());
    }

    println!(
        "{} Found {} memo{}",
        "📝".green(),
        memos.len().to_string().bold(),
        if memos.len() == 1 { "" } else { "s" }
    );
    println!();

    // Sort by creation time, newest first
    let mut sorted_memos = memos;
    sorted_memos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for memo in sorted_memos {
        println!("{} {}", "🆔".dimmed(), memo.id.as_str().blue());
        println!("{} {}", "📄".dimmed(), memo.title.bold());
        println!(
            "{} {}",
            "📅".dimmed(),
            memo.created_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()
                .dimmed()
        );

        // Show a preview of content
        let preview = format_content_preview(&memo.content, LIST_PREVIEW_LENGTH);
        println!("{} {}", "💬".dimmed(), preview.dimmed());
        println!();
    }

    Ok(())
}

async fn get_memo(
    storage: FileSystemMemoStorage,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let memo_id = MemoId::from_string(id.to_string())?;
    let memo = storage.get_memo(&memo_id).await?;

    println!("{} Memo: {}", "📝".green(), memo.title.bold());

    println!("{} ID: {}", "🆔".dimmed(), memo.id.as_str().blue());

    println!(
        "{} Created: {}",
        "📅".dimmed(),
        memo.created_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .dimmed()
    );

    println!(
        "{} Updated: {}",
        "🔄".dimmed(),
        memo.updated_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .dimmed()
    );

    println!();
    println!("Content:");
    println!("{}", memo.content);

    Ok(())
}

async fn update_memo(
    storage: FileSystemMemoStorage,
    id: &str,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let memo_id = MemoId::from_string(id.to_string())?;

    let content = get_content_input(content)?;

    let updated_memo = storage.update_memo(&memo_id, content).await?;

    println!(
        "{} Updated memo: {}",
        "✅".green(),
        updated_memo.title.bold()
    );

    println!("🆔 ID: {}", updated_memo.id.as_str().blue());

    println!(
        "🔄 Updated: {}",
        updated_memo
            .updated_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string()
            .dimmed()
    );

    Ok(())
}

async fn delete_memo(
    storage: FileSystemMemoStorage,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let memo_id = MemoId::from_string(id.to_string())?;

    // Get memo first to show what we're deleting
    let memo = storage.get_memo(&memo_id).await?;

    storage.delete_memo(&memo_id).await?;

    println!("{} Deleted memo: {}", "🗑️".red(), memo.title.bold());

    println!("🆔 ID: {}", memo.id.as_str().blue());

    Ok(())
}

async fn search_memos(
    storage: FileSystemMemoStorage,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let results = storage.search_memos(query).await?;

    if results.is_empty() {
        println!(
            "{} No memos found matching \"{}\"",
            "🔍".blue(),
            query.yellow()
        );
        return Ok(());
    }

    println!(
        "{} Found {} memo{} matching \"{}\"",
        "🔍".green(),
        results.len().to_string().bold(),
        if results.len() == 1 { "" } else { "s" },
        query.yellow()
    );
    println!();

    // Sort by creation time, newest first
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for memo in sorted_results {
        println!("{} {}", "🆔".dimmed(), memo.id.as_str().blue());
        println!("{} {}", "📄".dimmed(), memo.title.bold());
        println!(
            "{} {}",
            "📅".dimmed(),
            memo.created_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()
                .dimmed()
        );

        // Show a highlighted preview of content
        let preview = format_content_preview(&memo.content, SEARCH_PREVIEW_LENGTH);

        // Simple highlighting - replace query with colored version
        let highlighted_preview = preview.replace(query, &query.yellow().to_string());
        println!("{} {}", "💬".dimmed(), highlighted_preview.dimmed());
        println!();
    }

    Ok(())
}

async fn get_context(storage: FileSystemMemoStorage) -> Result<(), Box<dyn std::error::Error>> {
    let memos = storage.list_memos().await?;

    if memos.is_empty() {
        println!("No memos available for context.");
        return Ok(());
    }

    println!("# Memoranda Context");
    println!();

    // Sort by creation time, newest first
    let mut sorted_memos = memos;
    sorted_memos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for memo in sorted_memos {
        println!("## {} (ID: {})", memo.title, memo.id.as_str());
        println!();
        println!(
            "Created: {}",
            memo.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!(
            "Updated: {}",
            memo.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!();
        println!("{}", memo.content);
        println!();
        println!("---");
        println!();
    }

    Ok(())
}

/// Represents different sources of content input
enum ContentInput {
    Direct(String),
    Stdin,
    Interactive,
}

/// Get content from various input sources
fn get_content_input(content: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let input_type = match content {
        Some(c) if c == "-" => ContentInput::Stdin,
        Some(c) => ContentInput::Direct(c),
        None => ContentInput::Interactive,
    };

    match input_type {
        ContentInput::Direct(content) => Ok(content),
        ContentInput::Stdin => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer.trim().to_string())
        }
        ContentInput::Interactive => {
            println!("📝 Enter memo content:");
            println!("   💡 Type or paste your content, then press Ctrl+D (or Cmd+D on Mac) when finished");
            println!("   💡 You can enter multiple lines - just keep typing and press Enter for new lines");
            println!("   💡 To cancel, press Ctrl+C");
            println!();
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer.trim().to_string())
        }
    }
}
