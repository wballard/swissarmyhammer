use clap::{Parser, Subcommand};
use colored::*;
use is_terminal::IsTerminal;
use std::io;

#[derive(Parser, Debug)]
#[command(name = "swissarmyhammer")]
#[command(version)]
#[command(about = "An MCP server for managing prompts as markdown files")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run as MCP server (default when invoked via stdio)
    Serve,
    /// Diagnose configuration and setup issues
    Doctor,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    pub fn try_parse_from_args<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(args)
    }

    pub fn is_tty() -> bool {
        io::stdout().is_terminal()
    }

    pub fn should_use_color() -> bool {
        Self::is_tty() && std::env::var("NO_COLOR").is_err()
    }

    pub fn show_setup_instructions() {
        let use_color = Self::should_use_color();

        if use_color {
            println!("{}", "🔨 swissarmyhammer".bold().blue());
            println!(
                "{}",
                "An MCP server for managing prompts as markdown files".italic()
            );
        } else {
            println!("🔨 swissarmyhammer");
            println!("An MCP server for managing prompts as markdown files");
        }

        println!();

        if use_color {
            println!("{}", "Getting Started:".bold().yellow());
        } else {
            println!("Getting Started:");
        }

        println!("Add this server to your Claude Code MCP configuration:");
        println!();

        if use_color {
            println!("{}", "Configuration for Claude Code:".bold());
        } else {
            println!("Configuration for Claude Code:");
        }

        let config = r#"{
  "mcpServers": {
    "swissarmyhammer": {
      "command": "swissarmyhammer",
      "args": ["serve"]
    }
  }
}"#;

        if use_color {
            println!("{}", config.dimmed());
        } else {
            println!("{}", config);
        }

        println!();

        if use_color {
            println!("{}", "Commands:".bold().green());
            println!("  {} - Run as MCP server", "serve".cyan());
            println!("  {} - Diagnose setup issues", "doctor".cyan());
            println!("  {} - Show this help", "--help".cyan());
        } else {
            println!("Commands:");
            println!("  serve - Run as MCP server");
            println!("  doctor - Diagnose setup issues");
            println!("  --help - Show this help");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help_works() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "--help"]);
        assert!(result.is_err()); // Help exits with error code but that's expected

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_cli_version_works() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "--version"]);
        assert!(result.is_err()); // Version exits with error code but that's expected

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_cli_no_subcommand() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_cli_serve_subcommand() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "serve"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(matches!(cli.command, Some(Commands::Serve)));
    }

    #[test]
    fn test_cli_doctor_subcommand() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "doctor"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(matches!(cli.command, Some(Commands::Doctor)));
    }

    #[test]
    fn test_cli_verbose_flag() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "--verbose"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_cli_quiet_flag() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "--quiet"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.quiet);
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_serve_with_verbose() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "--verbose", "serve"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.verbose);
        assert!(matches!(cli.command, Some(Commands::Serve)));
    }

    #[test]
    fn test_cli_invalid_subcommand() {
        let result = Cli::try_parse_from_args(&["swissarmyhammer", "invalid"]);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }
}
