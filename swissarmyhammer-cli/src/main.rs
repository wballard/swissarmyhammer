use std::process;
mod cli;
mod completions;
mod doctor;
mod error;
mod exit_codes;
mod flow;
mod issue;
mod list;
mod logging;
// prompt_loader module removed - using SDK's PromptResolver directly
mod prompt;
mod search;
mod signal_handler;
mod test;
mod validate;

use clap::CommandFactory;
use cli::{Cli, Commands, ConfigAction};
use exit_codes::{EXIT_ERROR, EXIT_SUCCESS, EXIT_WARNING};
use logging::FileWriterGuard;

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    // Fast path for help - avoid expensive initialization
    if cli.command.is_none() {
        Cli::command().print_help().expect("Failed to print help");
        process::exit(EXIT_SUCCESS);
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
    } else if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::TRACE
    } else {
        Level::INFO
    };

    if is_mcp_mode {
        // In MCP mode, write logs to .swissarmyhammer/log for debugging
        use std::fs;
        use std::path::PathBuf;

        let log_dir = PathBuf::from(".swissarmyhammer");

        // Ensure the directory exists
        if let Err(e) = fs::create_dir_all(&log_dir) {
            tracing::warn!("Failed to create log directory: {}", e);
        }

        let log_filename =
            std::env::var("SWISSARMYHAMMER_LOG_FILE").unwrap_or_else(|_| "mcp.log".to_string());
        let log_file = log_dir.join(log_filename);

        // Try to open the log file - use unbuffered writing for immediate flushing
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            Ok(file) => {
                // Use Arc<Mutex<File>> for thread-safe, unbuffered writing
                use std::sync::{Arc, Mutex};
                let shared_file = Arc::new(Mutex::new(file));

                tracing_subscriber::fmt()
                    .with_writer(move || {
                        let file = shared_file.clone();
                        Box::new(FileWriterGuard::new(file)) as Box<dyn std::io::Write>
                    })
                    .with_max_level(log_level)
                    .with_ansi(false) // No color codes in file
                    .init();
            }
            Err(e) => {
                // Fallback to stderr if file logging fails
                tracing::warn!("Failed to open log file, using stderr: {}", e);
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
        Some(Commands::Prompt { subcommand }) => {
            tracing::info!("Running prompt command");
            run_prompt(subcommand).await
        }
        Some(Commands::Completion { shell }) => {
            tracing::info!("Generating completion for {:?}", shell);
            run_completions(shell)
        }
        Some(Commands::Flow { subcommand }) => {
            tracing::info!("Running flow command");
            run_flow(subcommand).await
        }
        Some(Commands::Validate {
            quiet,
            format,
            workflow_dirs,
        }) => {
            tracing::info!("Running validate command");
            run_validate(quiet, format, workflow_dirs)
        }
        Some(Commands::Issue { subcommand }) => {
            tracing::info!("Running issue command");
            run_issue(subcommand).await
        }
        Some(Commands::Config { action }) => {
            tracing::info!("Running config command");
            run_config(action)
        }
        None => {
            // This case is handled early above for performance
            unreachable!()
        }
    };

    // Ensure all logs are flushed before process exit
    if is_mcp_mode {
        // Give tracing sufficient time to flush any pending logs
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    process::exit(exit_code);
}

async fn run_server() -> i32 {
    use rmcp::serve_server;
    use rmcp::transport::io::stdio;
    use swissarmyhammer::{mcp::McpServer, PromptLibrary};

    // Create library and server
    let library = PromptLibrary::new();
    let server = match McpServer::new(library) {
        Ok(server) => server,
        Err(e) => {
            tracing::error!("Failed to create MCP server: {}", e);
            return EXIT_WARNING;
        }
    };

    // Initialize prompts (this will load user and local prompts)
    if let Err(e) = server.initialize().await {
        tracing::error!("Failed to initialize MCP server: {}", e);
        return EXIT_WARNING;
    }

    // Don't start file watching here - it will be started when MCP client connects
    // File watching is started in the ServerHandler::initialize method
    tracing::info!("MCP server initialized, file watching will start when client connects");

    // Start the rmcp SDK server with stdio transport
    let running_service = match serve_server(server, stdio()).await {
        Ok(service) => {
            tracing::info!("MCP server started successfully");
            service
        }
        Err(e) => {
            tracing::error!("MCP server error: {}", e);
            return EXIT_WARNING;
        }
    };

    // Wait for the service to complete - this will return when:
    // - The client disconnects (transport closed)
    // - The server is cancelled
    // - A serious error occurs
    match running_service.waiting().await {
        Ok(quit_reason) => {
            // The QuitReason enum is not exported by rmcp, so we'll just log it
            tracing::info!("MCP server stopped: {:?}", quit_reason);
        }
        Err(e) => {
            tracing::error!("MCP server task error: {}", e);
            return EXIT_WARNING;
        }
    }

    tracing::info!("MCP server shutting down gracefully");
    EXIT_SUCCESS
}

fn run_doctor() -> i32 {
    use doctor::Doctor;

    let mut doctor = Doctor::new();
    match doctor.run_diagnostics() {
        Ok(exit_code) => exit_code,
        Err(e) => {
            tracing::error!("Doctor error: {}", e);
            EXIT_ERROR
        }
    }
}

async fn run_prompt(subcommand: cli::PromptSubcommand) -> i32 {
    use error::handle_cli_result;
    use prompt;

    handle_cli_result(prompt::run_prompt_command(subcommand).await)
}

fn run_completions(shell: clap_complete::Shell) -> i32 {
    use completions;

    match completions::print_completion(shell) {
        Ok(_) => EXIT_SUCCESS,
        Err(e) => {
            tracing::error!("Completion error: {}", e);
            EXIT_WARNING
        }
    }
}

async fn run_flow(subcommand: cli::FlowSubcommand) -> i32 {
    use flow;

    match flow::run_flow_command(subcommand).await {
        Ok(_) => EXIT_SUCCESS,
        Err(e) => {
            tracing::error!("Flow error: {}", e);
            EXIT_WARNING
        }
    }
}

async fn run_issue(subcommand: cli::IssueCommands) -> i32 {
    use issue;

    match issue::handle_issue_command(subcommand).await {
        Ok(_) => EXIT_SUCCESS,
        Err(e) => {
            tracing::error!("Issue error: {}", e);
            EXIT_WARNING
        }
    }
}

/// Runs the validate command to check prompt files and workflows for syntax and best practices.
///
/// This function validates:
/// - All prompt files from builtin, user, and local directories
/// - YAML front matter syntax (skipped for .liquid files with {% partial %} marker)
/// - Required fields (title, description)
/// - Template variables match arguments
/// - Liquid template syntax
/// - Workflow structure and connectivity in .mermaid files
///
/// # Arguments
///
/// * `quiet` - Only show errors, no warnings or info
/// * `format` - Output format (text or json)
/// * `workflow_dirs` - [DEPRECATED] This parameter is ignored
///
/// # Returns
///
/// Exit code:
/// - 0: Success (no errors or warnings)
/// - 1: Warnings found
/// - 2: Errors found
fn run_validate(quiet: bool, format: cli::ValidateFormat, workflow_dirs: Vec<String>) -> i32 {
    use validate;

    match validate::run_validate_command_with_dirs(quiet, format, workflow_dirs) {
        Ok(exit_code) => exit_code,
        Err(e) => {
            tracing::error!("Validate error: {}", e);
            EXIT_ERROR
        }
    }
}

/// Runs the config command to manage configuration settings.
///
/// Provides comprehensive configuration management including:
/// - Showing current configuration values and sources
/// - Validating configuration for correctness
/// - Generating example configuration files
/// - Displaying configuration help and documentation
///
/// # Arguments
///
/// * `action` - The configuration action to perform (Show, Validate, Init, Help)
///
/// # Returns
///
/// Exit code:
/// - 0: Success
/// - 1: Configuration validation failed
/// - 2: Error occurred
fn run_config(action: ConfigAction) -> i32 {
    use swissarmyhammer::config::Config;

    // Initialize configuration (Config::new() handles errors internally and falls back to defaults)
    let config = Config::new();

    // For non-validate commands, validate config and warn if there are issues
    if !matches!(action, ConfigAction::Validate) {
        if let Err(e) = config.validate() {
            tracing::warn!("Configuration validation failed: {}", e);
            tracing::warn!("Using fallback values for invalid settings");
        }
    }

    match handle_config_command(action, &config) {
        Ok(_) => EXIT_SUCCESS,
        Err(e) => {
            tracing::error!("Config command error: {}", e);
            EXIT_ERROR
        }
    }
}

/// Handle the specific configuration command action
fn handle_config_command(action: ConfigAction, config: &swissarmyhammer::config::Config) -> Result<(), Box<dyn std::error::Error>> {
    use swissarmyhammer::config::Config;

    match action {
        ConfigAction::Show => {
            println!("📋 Current Configuration:");
            println!("base_branch: {}", config.base_branch);
            println!("issue_branch_prefix: {}", config.issue_branch_prefix);
            println!("issue_number_width: {}", config.issue_number_width);
            println!("max_pending_issues_in_summary: {}", config.max_pending_issues_in_summary);
            println!("min_issue_number: {}", config.min_issue_number);
            println!("max_issue_number: {}", config.max_issue_number);
            println!("issue_number_digits: {}", config.issue_number_digits);
            println!("max_content_length: {}", config.max_content_length);
            println!("max_line_length: {}", config.max_line_length);
            println!("max_issue_name_length: {}", config.max_issue_name_length);
            println!("cache_ttl_seconds: {}", config.cache_ttl_seconds);
            println!("cache_max_size: {}", config.cache_max_size);
            println!("virtual_issue_number_base: {}", config.virtual_issue_number_base);
            println!("virtual_issue_number_range: {}", config.virtual_issue_number_range);
            
            // Show configuration file location if found
            if let Some(config_file) = Config::find_yaml_config_file() {
                println!("\n📍 Configuration File: {:?}", config_file);
            } else {
                println!("\n📍 Configuration File: None found (using environment variables and defaults)");
            }
        }
        
        ConfigAction::Validate => {
            match config.validate() {
                Ok(()) => {
                    println!("✅ Configuration is valid");
                }
                Err(e) => {
                    println!("❌ Configuration validation failed: {}", e);
                    println!("\n{}", Config::validation_help());
                    return Err(Box::new(e));
                }
            }
        }
        
        ConfigAction::Init => {
            let config_path = "swissarmyhammer.yaml";
            
            if std::path::Path::new(config_path).exists() {
                eprintln!("❌ Configuration file already exists: {}", config_path);
                eprintln!("Remove it first or use a different location.");
                return Err("Configuration file already exists".into());
            }
            
            std::fs::write(config_path, Config::example_yaml_config())?;
            println!("✅ Created example configuration file: {}", config_path);
            println!("Edit this file to customize your configuration.");
        }
        
        ConfigAction::Guide => {
            println!("📖 Configuration Help\n");
            println!("SwissArmyHammer supports configuration via:");
            println!("  1. YAML file (swissarmyhammer.yaml) - highest precedence");
            println!("  2. Environment variables (SWISSARMYHAMMER_*) - medium precedence");
            println!("  3. Built-in defaults - lowest precedence");
            println!("\nConfiguration file locations searched:");
            println!("  1. Current directory: ./swissarmyhammer.yaml");
            println!("  2. User config directory: ~/.config/swissarmyhammer/swissarmyhammer.yaml");
            println!("  3. User home directory: ~/swissarmyhammer.yaml");
            println!("\nExample configuration file:");
            println!("{}", Config::example_yaml_config());
            println!("{}", Config::validation_help());
        }
    }
    
    Ok(())
}
