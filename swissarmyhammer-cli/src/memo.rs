use crate::cli::MemoCommands;
use crate::mcp_integration::{response_formatting, CliToolContext};
use serde_json::json;
use std::io::{self, Read};

pub async fn handle_memo_command(command: MemoCommands) -> Result<(), Box<dyn std::error::Error>> {
    let context = CliToolContext::new().await?;

    match command {
        MemoCommands::Create { title, content } => {
            create_memo(&context, title, content).await?;
        }
        MemoCommands::List => {
            list_memos(&context).await?;
        }
        MemoCommands::Get { id } => {
            get_memo(&context, &id).await?;
        }
        MemoCommands::Update { id, content } => {
            update_memo(&context, &id, content).await?;
        }
        MemoCommands::Delete { id } => {
            delete_memo(&context, &id).await?;
        }
        MemoCommands::Search { query } => {
            search_memos(&context, &query).await?;
        }
        MemoCommands::Context => {
            get_context(&context).await?;
        }
    }

    Ok(())
}

async fn create_memo(
    context: &CliToolContext,
    title: String,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_input(content)?;

    let args = context.create_arguments(vec![("title", json!(title)), ("content", json!(content))]);

    let result = context.execute_tool("memo_create", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn list_memos(context: &CliToolContext) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("memo_list", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn get_memo(context: &CliToolContext, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![("id", json!(id))]);
    let result = context.execute_tool("memo_get", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn update_memo(
    context: &CliToolContext,
    id: &str,
    content: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = get_content_input(content)?;

    let args = context.create_arguments(vec![("id", json!(id)), ("content", json!(content))]);

    let result = context.execute_tool("memo_update", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn delete_memo(context: &CliToolContext, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![("id", json!(id))]);
    let result = context.execute_tool("memo_delete", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn search_memos(
    context: &CliToolContext,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![("query", json!(query))]);
    let result = context.execute_tool("memo_search", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
    Ok(())
}

async fn get_context(context: &CliToolContext) -> Result<(), Box<dyn std::error::Error>> {
    let args = context.create_arguments(vec![]);
    let result = context.execute_tool("memo_get_all_context", args).await?;

    println!("{}", response_formatting::format_success_response(&result));
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
