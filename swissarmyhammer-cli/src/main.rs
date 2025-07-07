use std::process;
mod cli;
mod completions;
mod doctor;
mod list;
// prompt_loader module removed - using SDK's PromptResolver directly
mod search;
mod signal_handler;
mod test;
mod validate;

use clap::CommandFactory;
use cli::{Cli, Commands, OutputFormat, PromptSource, ValidateFormat};

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    // Fast path for help - avoid expensive initialization
    if cli.command.is_none() {
        Cli::command().print_help().expect("Failed to print help");
        process::exit(0);
    }

    // Only initialize heavy dependencies when actually needed
    use tracing::Level;

    // Configure logging based on verbosity flags and MCP mode detection
    use is_terminal::IsTerminal;
    let is_mcp_mode =
        matches!(cli.command, Some(Commands::Serve)) && !std::io::stdin().is_terminal();

    let log_level = if is_mcp_mode {
        Level::DEBUG // More verbose for MCP mode to help with debugging
    } else if cli.quiet {
        Level::ERROR
    } else if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    if is_mcp_mode {
        // In MCP mode, write logs to .swissarmyhammer/log for debugging
        use std::fs;
        use std::path::PathBuf;

        let log_dir = if let Some(home) = dirs::home_dir() {
            home.join(".swissarmyhammer")
        } else {
            PathBuf::from(".swissarmyhammer")
        };

        // Ensure the directory exists
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("Warning: Failed to create log directory: {}", e);
        }

        let log_file = log_dir.join("mcp.log");

        // Try to open the log file
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            Ok(file) => {
                tracing_subscriber::fmt()
                    .with_writer(file)
                    .with_max_level(log_level)
                    .with_ansi(false) // No color codes in file
                    .init();
            }
            Err(e) => {
                // Fallback to stderr if file logging fails
                eprintln!("Warning: Failed to open log file, using stderr: {}", e);
                tracing_subscriber::fmt()
                    .with_writer(std::io::stderr)
                    .with_max_level(log_level)
                    .init();
            }
        }
    } else {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_max_level(log_level)
            .init();
    }

    let exit_code = match cli.command {
        Some(Commands::Serve) => {
            tracing::info!("Starting MCP server");
            run_server().await
        }
        Some(Commands::Doctor) => {
            tracing::info!("Running diagnostics");
            run_doctor()
        }
        Some(Commands::List {
            format,
            verbose,
            source,
            category,
            search,
        }) => {
            tracing::info!("Listing prompts");
            run_list(format, verbose, source, category, search)
        }
        Some(Commands::Validate { quiet, format }) => {
            tracing::info!("Validating prompts");
            run_validate(quiet, format)
        }
        Some(Commands::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        }) => {
            tracing::info!("Testing prompt");
            run_test(&Commands::Test {
                prompt_name: prompt_name.clone(),
                file: file.clone(),
                arguments: arguments.clone(),
                raw,
                copy,
                save: save.clone(),
                debug,
            })
            .await
        }
        Some(Commands::Search {
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
        }) => {
            tracing::info!("Searching prompts");
            run_search(
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
        }
        Some(Commands::Completion { shell }) => {
            tracing::info!("Generating completion for {:?}", shell);
            run_completions(shell)
        }
        None => {
            // This case is handled early above for performance
            unreachable!()
        }
    };

    process::exit(exit_code);
}

async fn run_server() -> i32 {
    use rmcp::serve_server;
    use rmcp::transport::io::stdio;
    use swissarmyhammer::{mcp::McpServer, PromptLibrary};
    use tokio_util::sync::CancellationToken;

    // Create library and server
    let library = PromptLibrary::new();
    let server = McpServer::new(library);

    // Initialize prompts (this will load user and local prompts)
    if let Err(e) = server.initialize().await {
        tracing::error!("Failed to initialize MCP server: {}", e);
        return 1;
    }

    // Don't start file watching here - it will be started when MCP client connects
    // File watching is started in the ServerHandler::initialize method
    tracing::info!("MCP server initialized, file watching will start when client connects");

    // Set up cancellation token
    let ct = CancellationToken::new();
    let ct_clone = ct.clone();

    // Set up signal handlers
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");

        tracing::info!("Shutdown signal received");
        ct_clone.cancel();
    });

    // Start the rmcp SDK server with stdio transport
    match serve_server(server, stdio()).await {
        Ok(_running_service) => {
            tracing::info!("MCP server started successfully");

            // Wait for cancellation
            ct.cancelled().await;

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

fn run_validate(quiet: bool, format: ValidateFormat) -> i32 {
    use validate;

    match validate::run_validate_command(quiet, format) {
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

#[allow(clippy::too_many_arguments)]
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
        query,
        fields,
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
    ) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Search error: {}", e);
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
