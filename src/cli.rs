use clap::{Parser, Subcommand};
use colored::*;
use is_terminal::IsTerminal;
use std::io;

#[derive(Parser, Debug)]
#[command(name = "swissarmyhammer")]
#[command(version)]
#[command(about = "An MCP server for managing prompts as markdown files")]
#[command(long_about = "
swissarmyhammer is an MCP (Model Context Protocol) server that manages
prompts as markdown files. It supports file watching, template substitution,
and seamless integration with Claude Code.

Example usage:
  swissarmyhammer serve     # Run as MCP server
  swissarmyhammer doctor    # Check configuration and setup
  swissarmyhammer completion bash > ~/.bashrc.d/swissarmyhammer  # Generate bash completions
")]
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
    #[command(long_about = "
Runs swissarmyhammer as an MCP server. This is the default mode when
invoked via stdio (e.g., by Claude Code). The server will:

- Load all prompts from builtin, user, and local directories
- Watch for file changes and reload prompts automatically
- Expose prompts via the MCP protocol
- Support template substitution with {{variables}}

Example:
  swissarmyhammer serve
  # Or configure in Claude Code's MCP settings
")]
    Serve,
    /// Diagnose configuration and setup issues
    #[command(long_about = "
Runs comprehensive diagnostics to help troubleshoot setup issues.
The doctor command will check:

- If swissarmyhammer is in your PATH
- Claude Code MCP configuration
- Prompt directories and permissions
- YAML syntax in prompt files
- File watching capabilities

Exit codes:
  0 - All checks passed
  1 - Warnings found
  2 - Errors found

Example:
  swissarmyhammer doctor
  swissarmyhammer doctor --verbose  # Show detailed diagnostics
")]
    Doctor,
    /// Generate shell completion scripts
    #[command(long_about = "
Generates shell completion scripts for various shells. Supports:
- bash
- zsh
- fish
- powershell

Examples:
  # Bash (add to ~/.bashrc or ~/.bash_profile)
  swissarmyhammer completion bash > ~/.local/share/bash-completion/completions/swissarmyhammer
  
  # Zsh (add to ~/.zshrc or a file in fpath)
  swissarmyhammer completion zsh > ~/.zfunc/_swissarmyhammer
  
  # Fish
  swissarmyhammer completion fish > ~/.config/fish/completions/swissarmyhammer.fish
  
  # PowerShell
  swissarmyhammer completion powershell >> $PROFILE
")]
    Completion {
        /// Shell to generate completion for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
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
            println!("  {} - Generate shell completions", "completion".cyan());
            println!("  {} - Show detailed help", "--help".cyan());
            println!();
            println!("{}", "Quick Start:".bold().yellow());
            println!("  1. Run {} to check your setup", "swissarmyhammer doctor".cyan());
            println!("  2. Add the configuration above to Claude Code");
            println!("  3. Create prompts in ~/.swissarmyhammer/prompts/");
            println!();
            println!("{}", "Example Prompt:".bold());
            println!("  Create a file {} with:", "~/.swissarmyhammer/prompts/myhelper.md".dimmed());
            println!("  ---");
            println!("  title: My Helper");
            println!("  description: A helpful prompt");
            println!("  arguments:");
            println!("    - name: topic");
            println!("      description: What to help with");
            println!("      required: true");
            println!("  ---");
            println!("  ");
            println!("  Help me understand {{{{topic}}}}");
        } else {
            println!("Commands:");
            println!("  serve - Run as MCP server");
            println!("  doctor - Diagnose setup issues");
            println!("  completion - Generate shell completions");
            println!("  --help - Show detailed help");
            println!();
            println!("Quick Start:");
            println!("  1. Run 'swissarmyhammer doctor' to check your setup");
            println!("  2. Add the configuration above to Claude Code");
            println!("  3. Create prompts in ~/.swissarmyhammer/prompts/");
            println!();
            println!("Example Prompt:");
            println!("  Create a file ~/.swissarmyhammer/prompts/myhelper.md with:");
            println!("  ---");
            println!("  title: My Helper");
            println!("  description: A helpful prompt");
            println!("  arguments:");
            println!("    - name: topic");
            println!("      description: What to help with");
            println!("      required: true");
            println!("  ---");
            println!("  ");
            println!("  Help me understand {{topic}}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_help_works() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--help"]);
        assert!(result.is_err()); // Help exits with error code but that's expected

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_cli_version_works() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--version"]);
        assert!(result.is_err()); // Version exits with error code but that's expected

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn test_cli_no_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.command.is_none());
        assert!(!cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_cli_serve_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "serve"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(matches!(cli.command, Some(Commands::Serve)));
    }

    #[test]
    fn test_cli_doctor_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "doctor"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(matches!(cli.command, Some(Commands::Doctor)));
    }

    #[test]
    fn test_cli_verbose_flag() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--verbose"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_cli_quiet_flag() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--quiet"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.quiet);
        assert!(!cli.verbose);
    }

    #[test]
    fn test_cli_serve_with_verbose() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--verbose", "serve"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.verbose);
        assert!(matches!(cli.command, Some(Commands::Serve)));
    }

    #[test]
    fn test_cli_invalid_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "invalid"]);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    }
}
