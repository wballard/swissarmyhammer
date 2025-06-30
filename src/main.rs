use std::process;
use swissarmyhammer::cli::{Cli, Commands};
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
