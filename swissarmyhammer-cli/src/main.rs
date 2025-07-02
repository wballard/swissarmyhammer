use std::process;
mod cli;
mod completions;
mod doctor;
mod export;
mod import;
mod list;
mod mcp;
mod search;
mod signal_handler;
mod test;
mod validate;

use cli::{Cli, Commands, ExportFormat, ImportStrategy, OutputFormat, PromptSource, ValidateFormat};
use mcp::MCPServer;
use tokio::sync::oneshot;
use tracing::Level;

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    // Configure logging based on verbosity flags
    let log_level = if cli.quiet {
        Level::ERROR
    } else if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(log_level)
        .init();

    let exit_code = match cli.command {
        Some(Commands::Serve) => {
            tracing::info!("Starting MCP server");
            run_server().await
        }
        Some(Commands::Doctor) => {
            tracing::info!("Running diagnostics");
            run_doctor()
        }
        Some(Commands::List { format, verbose, source, category, search }) => {
            tracing::info!("Listing prompts");
            run_list(format, verbose, source, category, search)
        }
        Some(Commands::Validate { path, all, quiet, format }) => {
            tracing::info!("Validating prompts");
            run_validate(path, all, quiet, format)
        }
        Some(Commands::Test { prompt_name, file, arguments, raw, copy, save, debug }) => {
            tracing::info!("Testing prompt");
            run_test(&Commands::Test { 
                prompt_name: prompt_name.clone(), 
                file: file.clone(), 
                arguments: arguments.clone(), 
                raw, 
                copy, 
                save: save.clone(), 
                debug 
            }).await
        }
        Some(Commands::Search { query, r#in, regex, fuzzy, case_sensitive, source, has_arg, no_args, full, format, highlight, limit }) => {
            tracing::info!("Searching prompts");
            run_search(query, r#in, regex, fuzzy, case_sensitive, source, has_arg, no_args, full, format, highlight, limit)
        }
        Some(Commands::Export { prompt_name, all, category, source, format, output, metadata, exclude }) => {
            tracing::info!("Exporting prompts");
            run_export(prompt_name, all, category, source, format, output, metadata, exclude).await
        }
        Some(Commands::Import { source, dry_run, strategy, target, no_validate, no_backup, verbose }) => {
            tracing::info!("Importing prompts");
            run_import(source, dry_run, strategy, target, !no_validate, !no_backup, verbose).await
        }
        Some(Commands::Completion { shell }) => {
            tracing::info!("Generating completion for {:?}", shell);
            run_completions(shell)
        }
        None => {
            // No subcommand provided
            if Cli::is_tty() {
                // Running in terminal, show setup instructions
                Cli::show_setup_instructions();
                0
            } else {
                // Not in terminal (likely stdio), default to serve mode
                tracing::info!("No subcommand specified, defaulting to serve mode via stdio");
                run_server().await
            }
        }
    };

    process::exit(exit_code);
}

async fn run_server() -> i32 {
    let server = MCPServer::new();

    // Set up shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Set up signal handlers
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");

        tracing::info!("Shutdown signal received");
        let _ = shutdown_tx.send(());
    });

    match server.run(shutdown_rx).await {
        Ok(_) => {
            tracing::info!("MCP server exited successfully");
            0
        }
        Err(e) => {
            tracing::error!("MCP server error: {}", e);
            1
        }
    }
}

fn run_doctor() -> i32 {
    use doctor::Doctor;
    
    let mut doctor = Doctor::new();
    match doctor.run_diagnostics() {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Doctor error: {}", e);
            2
        }
    }
}

fn run_list(
    format: OutputFormat,
    verbose: bool,
    source: Option<PromptSource>,
    category: Option<String>,
    search: Option<String>,
) -> i32 {
    use list;
    
    match list::run_list_command(format, verbose, source, category, search) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("List error: {}", e);
            1
        }
    }
}

fn run_validate(
    path: Option<String>,
    all: bool,
    quiet: bool,
    format: ValidateFormat,
) -> i32 {
    use validate;
    
    match validate::run_validate_command(path, all, quiet, format) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Validation error: {}", e);
            2
        }
    }
}

async fn run_test(command: &Commands) -> i32 {
    use test::TestRunner;
    
    let mut runner = TestRunner::new();
    match runner.run(command).await {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Test error: {}", e);
            1
        }
    }
}

fn run_search(
    query: String,
    fields: Option<Vec<String>>,
    regex: bool,
    fuzzy: bool,
    case_sensitive: bool,
    source: Option<PromptSource>,
    has_arg: Option<String>,
    no_args: bool,
    full: bool,
    format: OutputFormat,
    highlight: bool,
    limit: Option<usize>,
) -> i32 {
    use search;
    
    match search::run_search_command(
        query, fields, regex, fuzzy, case_sensitive, source, 
        has_arg, no_args, full, format, highlight, limit
    ) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Search error: {}", e);
            1
        }
    }
}

async fn run_export(
    prompt_name: Option<String>,
    all: bool,
    category: Option<String>,
    source: Option<PromptSource>,
    format: ExportFormat,
    output: Option<String>,
    metadata: bool,
    exclude: Vec<String>,
) -> i32 {
    use export;
    
    match export::run_export_command(prompt_name, all, category, source, format, output, metadata, exclude).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Export error: {}", e);
            1
        }
    }
}

async fn run_import(
    source: String,
    dry_run: bool,
    strategy: ImportStrategy,
    target: Option<String>,
    validate: bool,
    backup: bool,
    verbose: bool,
) -> i32 {
    use import;
    
    match import::run_import_command(source, dry_run, strategy, target, validate, backup, verbose).await {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Import error: {}", e);
            1
        }
    }
}

fn run_completions(shell: clap_complete::Shell) -> i32 {
    use completions;
    
    match completions::print_completion(shell) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Completion error: {}", e);
            1
        }
    }
}
