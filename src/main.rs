use std::process;
use swissarmyhammer::cli::{Cli, Commands};
use tracing::Level;

fn main() {
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
            run_server()
        }
        Some(Commands::Doctor) => {
            tracing::info!("Running diagnostics");
            run_doctor()
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
                run_server()
            }
        }
    };

    process::exit(exit_code);
}

fn run_server() -> i32 {
    // TODO: Implement MCP server
    println!("MCP server would start here");
    0
}

fn run_doctor() -> i32 {
    // TODO: Implement diagnostics
    println!("Diagnostics would run here");
    0
}
