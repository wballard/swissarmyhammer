use std::process;
use swissarmyhammer::cli::{Cli, Commands, OutputFormat, PromptSource, ValidateFormat};
use swissarmyhammer::mcp::MCPServer;
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
        Some(Commands::Completion { shell }) => {
            tracing::info!("Generating completion for {:?}", shell);
            run_completion(shell)
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
    use swissarmyhammer::doctor::Doctor;
    
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
    use swissarmyhammer::list;
    
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
    use swissarmyhammer::validate;
    
    match validate::run_validate_command(path, all, quiet, format) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Validation error: {}", e);
            2
        }
    }
}

async fn run_test(command: &Commands) -> i32 {
    use swissarmyhammer::test::TestRunner;
    
    let mut runner = TestRunner::new();
    match runner.run(command).await {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("Test error: {}", e);
            1
        }
    }
}

fn run_completion(shell: clap_complete::Shell) -> i32 {
    use swissarmyhammer::completions;
    
    match completions::print_completion(shell) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Completion error: {}", e);
            1
        }
    }
}
