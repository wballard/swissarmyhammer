use clap::{Parser, Subcommand, ValueEnum};
use is_terminal::IsTerminal;
use std::io;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

// Re-export PromptSource from the library
pub use swissarmyhammer::PromptSource;

// Create a wrapper for CLI argument parsing since the library's PromptSource doesn't derive ValueEnum
#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum PromptSourceArg {
    Builtin,
    User,
    Local,
    Dynamic,
}

impl From<PromptSourceArg> for PromptSource {
    fn from(arg: PromptSourceArg) -> Self {
        match arg {
            PromptSourceArg::Builtin => PromptSource::Builtin,
            PromptSourceArg::User => PromptSource::User,
            PromptSourceArg::Local => PromptSource::Local,
            PromptSourceArg::Dynamic => PromptSource::Dynamic,
        }
    }
}

impl From<PromptSource> for PromptSourceArg {
    fn from(source: PromptSource) -> Self {
        match source {
            PromptSource::Builtin => PromptSourceArg::Builtin,
            PromptSource::User => PromptSourceArg::User,
            PromptSource::Local => PromptSourceArg::Local,
            PromptSource::Dynamic => PromptSourceArg::Dynamic,
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ValidateFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum VisualizationFormat {
    Mermaid,
    Html,
    Json,
    Dot,
}

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

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

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
    /// Manage and test prompts
    #[command(long_about = "
Manage prompts with support for listing, validating, testing, and searching.
Prompts are markdown files with YAML front matter that define reusable templates.

Basic usage:
  swissarmyhammer prompt list                    # List all prompts
  swissarmyhammer prompt validate                # Validate prompt files
  swissarmyhammer prompt test <name>             # Test a prompt
  swissarmyhammer prompt search <query>          # Search prompts

Examples:
  swissarmyhammer prompt list --source builtin
  swissarmyhammer prompt validate --quiet
  swissarmyhammer prompt test code-review --arg file=main.rs
  swissarmyhammer prompt search \"python code\"
")]
    Prompt {
        #[command(subcommand)]
        subcommand: PromptSubcommand,
    },
    /// Execute and manage workflows
    #[command(long_about = "
Execute and manage workflows with support for starting new runs and resuming existing ones.
Workflows are defined as state machines that can execute actions and tools including Claude commands.

Basic usage:
  swissarmyhammer flow run my-workflow           # Start new workflow
  swissarmyhammer flow resume <run_id>           # Resume paused workflow
  swissarmyhammer flow list                      # List available workflows
  swissarmyhammer flow status <run_id>           # Check run status
  swissarmyhammer flow logs <run_id>             # View execution logs

Workflow execution:
  --vars key=value                               # Pass initial variables
  --interactive                                  # Step-by-step execution
  --dry-run                                      # Show execution plan
  --timeout 60s                                  # Set execution timeout

Examples:
  swissarmyhammer flow run code-review --vars file=main.rs
  swissarmyhammer flow run deploy --dry-run
  swissarmyhammer flow resume a1b2c3d4 --interactive
  swissarmyhammer flow list --format json
  swissarmyhammer flow status a1b2c3d4 --watch
")]
    Flow {
        #[command(subcommand)]
        subcommand: FlowSubcommand,
    },
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
    /// Validate prompt files and workflows for syntax and best practices
    #[command(long_about = "
Validates BOTH prompt files AND workflows for syntax errors and best practices.

This command comprehensively validates:
- All prompt files from builtin, user, and local directories
- All workflow files from standard locations (builtin, user, local)

NOTE: The --workflow-dir parameter is deprecated and will be ignored.
Workflows are now only loaded from standard locations.

Validation checks:
- YAML front matter syntax (skipped for .liquid files with {% partial %} marker)
- Required fields (title, description)
- Template variables match arguments
- Liquid template syntax
- Workflow structure and connectivity
- Best practice recommendations

Examples:
  swissarmyhammer validate                 # Validate all prompts and workflows
  swissarmyhammer validate --quiet         # CI/CD mode (exit code only)
  swissarmyhammer validate --format json   # JSON output for tooling
")]
    Validate {
        /// Only show errors, no warnings or info
        #[arg(short, long)]
        quiet: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: ValidateFormat,

        /// [DEPRECATED] This parameter is ignored. Workflows are now only loaded from standard locations.
        #[arg(long = "workflow-dir", value_name = "DIR", hide = true)]
        workflow_dirs: Vec<String>,
    },
    /// Issue management commands
    #[command(long_about = "
Manage issues with comprehensive CLI commands for creating, updating, and tracking work items.
Issues are stored as markdown files in the ./issues directory with automatic numbering.

Basic usage:
  swissarmyhammer issue create <name>           # Create new issue
  swissarmyhammer issue list                    # List all issues
  swissarmyhammer issue show <number>           # Show issue details
  swissarmyhammer issue update <number>         # Update issue content
  swissarmyhammer issue complete <number>       # Mark issue as complete
  swissarmyhammer issue work <number>           # Start working on issue (creates git branch)
  swissarmyhammer issue merge <number>          # Merge completed issue to main
  swissarmyhammer issue current                 # Show current issue
  swissarmyhammer issue status                  # Show project status

Examples:
  swissarmyhammer issue create \"Bug fix\" --content \"Fix login issue\"
  swissarmyhammer issue list --format json --active
  swissarmyhammer issue show 123 --raw
  swissarmyhammer issue update 123 --content \"Updated description\" --append
  swissarmyhammer issue work 123
  swissarmyhammer issue merge 123 --keep-branch
")]
    Issue {
        #[command(subcommand)]
        subcommand: IssueCommands,
    },
    /// Configuration management commands
    #[command(long_about = "
Manage SwissArmyHammer configuration with comprehensive CLI commands for viewing,
validating, and initializing configuration settings. Configuration is loaded from
YAML files, environment variables, and defaults with proper precedence.

Configuration precedence (highest to lowest):
  1. YAML file (swissarmyhammer.yaml) - highest precedence
  2. Environment variables (SWISSARMYHAMMER_*) - medium precedence  
  3. Built-in defaults - lowest precedence

Basic usage:
  swissarmyhammer config show                   # Show current configuration
  swissarmyhammer config validate               # Validate configuration
  swissarmyhammer config init                   # Create example config file
  swissarmyhammer config help                   # Show configuration help

Examples:
  swissarmyhammer config show                   # Display all config values
  swissarmyhammer config validate               # Check for config errors
  swissarmyhammer config init                   # Generate swissarmyhammer.yaml
  swissarmyhammer config help                   # Get configuration guidance
")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PromptSubcommand {
    /// List all available prompts
    #[command(long_about = "
Lists all available prompts from all sources (built-in, user, local).
Shows prompt names, titles, descriptions, and source information.

Output formats:
  table  - Formatted table (default)
  json   - JSON output for scripting
  yaml   - YAML output for scripting

Examples:
  swissarmyhammer prompt list                        # Show all prompts in table format
  swissarmyhammer prompt list --format json         # Output as JSON
  swissarmyhammer prompt list --verbose             # Show full details including arguments
  swissarmyhammer prompt list --source builtin      # Show only built-in prompts
  swissarmyhammer prompt list --search debug        # Search for prompts containing 'debug'
")]
    List {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Show verbose output including arguments
        #[arg(short, long)]
        verbose: bool,

        /// Filter by source
        #[arg(long, value_enum)]
        source: Option<PromptSourceArg>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Search prompts by name or description
        #[arg(long)]
        search: Option<String>,
    },
    /// Test prompts interactively with sample arguments
    #[command(long_about = "
Test prompts interactively to see how they render with different arguments.
Helps debug template errors and refine prompt content before using in Claude Code.

Usage modes:
  swissarmyhammer prompt test prompt-name                    # Test by name (interactive)
  swissarmyhammer prompt test -f path/to/prompt.md          # Test from file
  swissarmyhammer prompt test prompt-name --arg key=value   # Non-interactive mode

Interactive features:
- Prompts for each argument with descriptions
- Shows default values (press Enter to accept)
- Validates required arguments
- Supports multi-line input

Output options:
  --raw     Show rendered prompt without formatting
  --copy    Copy rendered prompt to clipboard
  --save    Save rendered prompt to file
  --debug   Show template processing details

Examples:
  swissarmyhammer prompt test code-review                           # Interactive test
  swissarmyhammer prompt test -f my-prompt.md                       # Test file
  swissarmyhammer prompt test help --arg topic=git                  # Non-interactive
  swissarmyhammer prompt test plan --debug --save output.md         # Debug + save
  swissarmyhammer prompt test code-review --set author=John --set version=1.0  # With template variables
")]
    Test {
        /// Prompt name to test (alternative to --file)
        prompt_name: Option<String>,

        /// Path to prompt file to test
        #[arg(short, long)]
        file: Option<String>,

        /// Non-interactive mode: specify arguments as key=value pairs
        #[arg(long = "arg", value_name = "KEY=VALUE")]
        arguments: Vec<String>,

        /// Set template variables for liquid rendering as key=value pairs
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,

        /// Show raw output without formatting
        #[arg(long)]
        raw: bool,

        /// Copy rendered prompt to clipboard
        #[arg(long)]
        copy: bool,

        /// Save rendered prompt to file
        #[arg(long, value_name = "FILE")]
        save: Option<String>,

        /// Show debug information (template, args, processing steps)
        #[arg(long)]
        debug: bool,
    },
    /// Search for prompts with advanced filtering and ranking
    #[command(long_about = "
Search for prompts using powerful full-text search with fuzzy matching.
Searches prompt names, titles, descriptions, content, and arguments.

Basic usage:
  swissarmyhammer prompt search \"code review\"        # Basic search
  swissarmyhammer prompt search \"debug.*error\" -r   # Regex search
  swissarmyhammer prompt search help --fuzzy          # Fuzzy matching

Search scope:
  --in name,description,content               # Search specific fields
  --source builtin                           # Search only builtin prompts
  --has-arg language                         # Find prompts with 'language' argument

Output options:
  --full                                     # Show complete prompt details
  --json                                     # JSON output for tooling
  --limit 10                                 # Limit number of results
  --highlight                                # Highlight matching terms

Examples:
  swissarmyhammer prompt search \"python code\"        # Find Python-related prompts
  swissarmyhammer prompt search \"review\" --full       # Detailed results for review prompts
  swissarmyhammer prompt search \".*test.*\" --regex     # Regex pattern matching
  swissarmyhammer prompt search help --fuzzy --limit 5  # Fuzzy search, max 5 results
")]
    Search {
        /// Search query
        query: String,

        /// Search in specific fields (name, title, description, content, arguments)
        #[arg(long, value_delimiter = ',')]
        r#in: Option<Vec<String>>,

        /// Use regular expressions
        #[arg(short, long)]
        regex: bool,

        /// Enable fuzzy matching for typo tolerance
        #[arg(short, long)]
        fuzzy: bool,

        /// Case-sensitive search
        #[arg(long)]
        case_sensitive: bool,

        /// Filter by source
        #[arg(long, value_enum)]
        source: Option<PromptSourceArg>,

        /// Find prompts with specific argument name
        #[arg(long)]
        has_arg: Option<String>,

        /// Find prompts without any arguments
        #[arg(long)]
        no_args: bool,

        /// Show complete prompt details
        #[arg(long)]
        full: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Highlight matching terms in output
        #[arg(long)]
        highlight: bool,

        /// Maximum number of results to show
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

#[derive(Subcommand, Debug)]
pub enum FlowSubcommand {
    /// Run a workflow
    Run {
        /// Workflow name to run
        workflow: String,

        /// Initial variables as key=value pairs
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,

        /// Set template variables for liquid rendering in action strings as key=value pairs
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,

        /// Interactive mode - prompt at each state
        #[arg(short, long)]
        interactive: bool,

        /// Dry run - show execution plan without running
        #[arg(long)]
        dry_run: bool,

        /// Test mode - execute with mocked actions and generate coverage report
        #[arg(long)]
        test: bool,

        /// Execution timeout (e.g., 30s, 5m, 1h)
        #[arg(long)]
        timeout: Option<String>,

        /// Quiet mode - only show errors
        #[arg(short, long)]
        quiet: bool,
    },
    /// Resume a paused workflow run
    Resume {
        /// Run ID to resume
        run_id: String,

        /// Interactive mode - prompt at each state
        #[arg(short, long)]
        interactive: bool,

        /// Execution timeout (e.g., 30s, 5m, 1h)
        #[arg(long)]
        timeout: Option<String>,

        /// Quiet mode - only show errors
        #[arg(short, long)]
        quiet: bool,
    },
    /// List available workflows
    List {
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Show verbose output including workflow details
        #[arg(short, long)]
        verbose: bool,

        /// Filter by source
        #[arg(long, value_enum)]
        source: Option<PromptSourceArg>,
    },
    /// Check status of a workflow run
    Status {
        /// Run ID to check
        run_id: String,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Watch for status changes
        #[arg(short, long)]
        watch: bool,
    },
    /// View logs for a workflow run
    Logs {
        /// Run ID to view logs for
        run_id: String,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Number of log lines to show (from end)
        #[arg(short = 'n', long)]
        tail: Option<usize>,

        /// Filter logs by level (info, warn, error)
        #[arg(long)]
        level: Option<String>,
    },
    /// View metrics for workflow runs
    Metrics {
        /// Run ID to view metrics for (optional - shows all if not specified)
        run_id: Option<String>,

        /// Workflow name to filter by
        #[arg(long)]
        workflow: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,

        /// Show global metrics summary
        #[arg(short, long)]
        global: bool,
    },
    /// Generate execution visualization
    Visualize {
        /// Run ID to visualize
        run_id: String,

        /// Output format
        #[arg(long, value_enum, default_value = "mermaid")]
        format: VisualizationFormat,

        /// Output file path (optional - prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Include timing information
        #[arg(long)]
        timing: bool,

        /// Include execution counts
        #[arg(long)]
        counts: bool,

        /// Show only executed path
        #[arg(long)]
        path_only: bool,
    },
    /// Test a workflow without executing actions (simulates dry run)
    #[command(long_about = "
Test workflows in simulation mode without actually executing actions.
This command provides a safe way to validate workflow logic and see what
actions would be executed without actually running them.

Features:
- Simulates all actions instead of executing them
- Claude prompts are echoed instead of sent to the API
- Generates coverage reports showing visited states and transitions
- Useful for testing workflow logic and debugging

Usage:
  swissarmyhammer flow test my-workflow
  swissarmyhammer flow test my-workflow --var key=value
  swissarmyhammer flow test my-workflow --set template_var=value

Examples:
  swissarmyhammer flow test hello-world                               # Test basic workflow
  swissarmyhammer flow test greeting --set name=John --set language=Spanish  # With template variables
  swissarmyhammer flow test code-review --var file=main.rs --timeout 60s     # With vars and timeout
  swissarmyhammer flow test deploy --interactive                      # Step-by-step execution

This is equivalent to 'flow run --test' but provided as a separate command
for better discoverability and clearer intent.
")]
    Test {
        /// Workflow name to test
        workflow: String,

        /// Initial variables as key=value pairs
        #[arg(long = "var", value_name = "KEY=VALUE")]
        vars: Vec<String>,

        /// Set template variables for liquid rendering in action strings as key=value pairs
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,

        /// Interactive mode - prompt at each state
        #[arg(short, long)]
        interactive: bool,

        /// Execution timeout (e.g., 30s, 5m, 1h)
        #[arg(long)]
        timeout: Option<String>,

        /// Quiet mode - only show errors
        #[arg(short, long)]
        quiet: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum IssueCommands {
    /// Create a new issue
    Create {
        /// Issue name
        name: String,
        /// Issue content (use - for stdin)
        #[arg(short, long)]
        content: Option<String>,
        /// Read content from file
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
    },
    /// List all issues
    List {
        /// Show completed issues
        #[arg(short, long)]
        completed: bool,
        /// Show active issues only
        #[arg(short, long)]
        active: bool,
        /// Output format (table, json, markdown)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Show issue details
    Show {
        /// Issue number
        number: u32,
        /// Show raw content
        #[arg(short, long)]
        raw: bool,
    },
    /// Update an issue
    Update {
        /// Issue number
        number: u32,
        /// New content (use - for stdin)
        #[arg(short, long)]
        content: Option<String>,
        /// Read content from file
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
        /// Append to existing content
        #[arg(short, long)]
        append: bool,
    },
    /// Mark issue as complete
    Complete {
        /// Issue number
        number: u32,
    },
    /// Start working on an issue
    Work {
        /// Issue number
        number: u32,
    },
    /// Merge completed issue
    Merge {
        /// Issue number
        number: u32,
        /// Keep branch after merge
        #[arg(short, long)]
        keep_branch: bool,
    },
    /// Show current issue
    Current,
    /// Show project status
    Status,
}

/// Configuration management commands
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration values and sources
    #[command(long_about = "
Display all current configuration values and their sources. Shows the effective
configuration after applying precedence rules (YAML > Environment > Defaults).

This command helps you understand which configuration values are active and
where they come from, making it easier to debug configuration issues.

Examples:
  swissarmyhammer config show                   # Show all configuration values
")]
    Show,
    /// Validate current configuration
    #[command(long_about = "
Validate the current configuration for correctness and consistency. Checks for:
- Valid git branch names (no invalid characters, proper format)
- Numeric values within acceptable ranges
- String values within length limits
- Configuration consistency and logical constraints

Exit codes:
  0 - Configuration is valid
  1 - Configuration validation failed

Examples:
  swissarmyhammer config validate               # Validate current config
")]
    Validate,
    /// Generate example configuration file
    #[command(long_about = "
Create an example configuration file (swissarmyhammer.yaml) in the current
directory with documented settings and sensible defaults.

This command helps you get started with configuration customization by providing
a template file with all available options and their descriptions.

The generated file includes:
- All configurable settings with their default values
- Comments explaining each setting
- Examples of valid values

Examples:
  swissarmyhammer config init                   # Create swissarmyhammer.yaml
")]
    Init,
    /// Show configuration guide and documentation
    #[command(long_about = "
Display comprehensive guide about SwissArmyHammer configuration including:
- Configuration precedence rules
- Available configuration options
- Environment variable names
- YAML file format examples
- Validation rules and constraints

This is a detailed guide to understanding and working with SwissArmyHammer
configuration in all its forms.

Examples:
  swissarmyhammer config guide                  # Show detailed config guide
")]
    Guide,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }

    #[allow(dead_code)]
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

    #[test]
    fn test_cli_test_subcommand_with_prompt_name() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "prompt", "test", "help"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Test {
                prompt_name,
                file,
                arguments,
                set,
                raw,
                copy,
                save,
                debug,
            } = subcommand
            {
                assert_eq!(prompt_name, Some("help".to_string()));
                assert_eq!(file, None);
                assert!(arguments.is_empty());
                assert!(set.is_empty());
                assert!(!raw);
                assert!(!copy);
                assert_eq!(save, None);
                assert!(!debug);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_file() {
        let result =
            Cli::try_parse_from_args(["swissarmyhammer", "prompt", "test", "-f", "test.md"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Test {
                prompt_name,
                file,
                arguments,
                set,
                raw,
                copy,
                save,
                debug,
            } = subcommand
            {
                assert_eq!(prompt_name, None);
                assert_eq!(file, Some("test.md".to_string()));
                assert!(arguments.is_empty());
                assert!(set.is_empty());
                assert!(!raw);
                assert!(!copy);
                assert_eq!(save, None);
                assert!(!debug);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_arguments() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "prompt",
            "test",
            "help",
            "--arg",
            "topic=git",
            "--arg",
            "format=markdown",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Test {
                prompt_name,
                file,
                arguments,
                set,
                raw,
                copy,
                save,
                debug,
            } = subcommand
            {
                assert_eq!(prompt_name, Some("help".to_string()));
                assert_eq!(file, None);
                assert_eq!(arguments, vec!["topic=git", "format=markdown"]);
                assert!(set.is_empty());
                assert!(!raw);
                assert!(!copy);
                assert_eq!(save, None);
                assert!(!debug);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_all_flags() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "prompt",
            "test",
            "help",
            "--raw",
            "--copy",
            "--debug",
            "--save",
            "output.md",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Test {
                prompt_name,
                file,
                arguments,
                set,
                raw,
                copy,
                save,
                debug,
            } = subcommand
            {
                assert_eq!(prompt_name, Some("help".to_string()));
                assert_eq!(file, None);
                assert!(arguments.is_empty());
                assert!(set.is_empty());
                assert!(raw);
                assert!(copy);
                assert_eq!(save, Some("output.md".to_string()));
                assert!(debug);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_set_variables() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "prompt",
            "test",
            "help",
            "--arg",
            "topic=git",
            "--set",
            "author=John",
            "--set",
            "version=1.0",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Test {
                prompt_name,
                file,
                arguments,
                set,
                raw,
                copy,
                save,
                debug,
            } = subcommand
            {
                assert_eq!(prompt_name, Some("help".to_string()));
                assert_eq!(file, None);
                assert_eq!(arguments, vec!["topic=git"]);
                assert_eq!(set, vec!["author=John", "version=1.0"]);
                assert!(!raw);
                assert!(!copy);
                assert_eq!(save, None);
                assert!(!debug);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_basic() {
        let result =
            Cli::try_parse_from_args(["swissarmyhammer", "prompt", "search", "code review"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Search {
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
            } = subcommand
            {
                assert_eq!(query, "code review");
                assert_eq!(r#in, None);
                assert!(!regex);
                assert!(!fuzzy);
                assert!(!case_sensitive);
                assert_eq!(source, None);
                assert_eq!(has_arg, None);
                assert!(!no_args);
                assert!(!full);
                assert!(matches!(format, OutputFormat::Table));
                assert!(!highlight);
                assert_eq!(limit, None);
            } else {
                panic!("Expected Search subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_with_flags() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "prompt",
            "search",
            "debug.*error",
            "--regex",
            "--fuzzy",
            "--case-sensitive",
            "--source",
            "builtin",
            "--has-arg",
            "language",
            "--full",
            "--format",
            "json",
            "--highlight",
            "--limit",
            "5",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Search {
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
            } = subcommand
            {
                assert_eq!(query, "debug.*error");
                assert_eq!(r#in, None);
                assert!(regex);
                assert!(fuzzy);
                assert!(case_sensitive);
                assert!(matches!(source, Some(PromptSourceArg::Builtin)));
                assert_eq!(has_arg, Some("language".to_string()));
                assert!(!no_args);
                assert!(full);
                assert!(matches!(format, OutputFormat::Json));
                assert!(highlight);
                assert_eq!(limit, Some(5));
            } else {
                panic!("Expected Search subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_with_fields() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "prompt",
            "search",
            "python",
            "--in",
            "name,description,content",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::Search { query, r#in, .. } = subcommand {
                assert_eq!(query, "python");
                assert_eq!(
                    r#in,
                    Some(vec![
                        "name".to_string(),
                        "description".to_string(),
                        "content".to_string()
                    ])
                );
            } else {
                panic!("Expected Search subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_prompt_list_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "prompt", "list"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Prompt { subcommand }) = cli.command {
            if let PromptSubcommand::List {
                format,
                verbose,
                source,
                category,
                search,
            } = subcommand
            {
                assert!(matches!(format, OutputFormat::Table));
                assert!(!verbose);
                assert_eq!(source, None);
                assert_eq!(category, None);
                assert_eq!(search, None);
            } else {
                panic!("Expected List subcommand");
            }
        } else {
            panic!("Expected Prompt command");
        }
    }

    #[test]
    fn test_cli_validate_command() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "validate"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Validate {
            quiet,
            format,
            workflow_dirs,
        }) = cli.command
        {
            assert!(!quiet);
            assert!(matches!(format, ValidateFormat::Text));
            assert!(workflow_dirs.is_empty());
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn test_cli_validate_command_with_options() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "validate",
            "--quiet",
            "--format",
            "json",
            "--workflow-dir",
            "./workflows",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Validate {
            quiet,
            format,
            workflow_dirs,
        }) = cli.command
        {
            assert!(quiet);
            assert!(matches!(format, ValidateFormat::Json));
            assert_eq!(workflow_dirs, vec!["./workflows"]);
        } else {
            panic!("Expected Validate command");
        }
    }

    #[test]
    fn test_cli_flow_test_subcommand() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "flow", "test", "my-workflow"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Flow { subcommand }) = cli.command {
            if let FlowSubcommand::Test {
                workflow,
                vars,
                set,
                interactive,
                timeout,
                quiet,
            } = subcommand
            {
                assert_eq!(workflow, "my-workflow");
                assert!(vars.is_empty());
                assert!(set.is_empty());
                assert!(!interactive);
                assert_eq!(timeout, None);
                assert!(!quiet);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Flow command");
        }
    }

    #[test]
    fn test_cli_flow_test_subcommand_with_options() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "flow",
            "test",
            "my-workflow",
            "--var",
            "input=test",
            "--set",
            "author=Jane",
            "--set",
            "version=2.0",
            "--interactive",
            "--timeout",
            "30s",
            "--quiet",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Flow { subcommand }) = cli.command {
            if let FlowSubcommand::Test {
                workflow,
                vars,
                set,
                interactive,
                timeout,
                quiet,
            } = subcommand
            {
                assert_eq!(workflow, "my-workflow");
                assert_eq!(vars, vec!["input=test"]);
                assert_eq!(set, vec!["author=Jane", "version=2.0"]);
                assert!(interactive);
                assert_eq!(timeout, Some("30s".to_string()));
                assert!(quiet);
            } else {
                panic!("Expected Test subcommand");
            }
        } else {
            panic!("Expected Flow command");
        }
    }

    #[test]
    fn test_parse_args_panics_on_error() {
        // This test verifies that parse_args would panic on invalid input
        // We can't easily test the panic itself in unit tests, but we can verify
        // that the underlying try_parse_from_args returns an error
        let result = Cli::try_parse_from_args(["swissarmyhammer", "invalid-command"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_source_arg_conversions() {
        // Test From<PromptSourceArg> for PromptSource
        assert!(matches!(
            PromptSource::from(PromptSourceArg::Builtin),
            PromptSource::Builtin
        ));
        assert!(matches!(
            PromptSource::from(PromptSourceArg::User),
            PromptSource::User
        ));
        assert!(matches!(
            PromptSource::from(PromptSourceArg::Local),
            PromptSource::Local
        ));
        assert!(matches!(
            PromptSource::from(PromptSourceArg::Dynamic),
            PromptSource::Dynamic
        ));

        // Test From<PromptSource> for PromptSourceArg
        assert!(matches!(
            PromptSourceArg::from(PromptSource::Builtin),
            PromptSourceArg::Builtin
        ));
        assert!(matches!(
            PromptSourceArg::from(PromptSource::User),
            PromptSourceArg::User
        ));
        assert!(matches!(
            PromptSourceArg::from(PromptSource::Local),
            PromptSourceArg::Local
        ));
        assert!(matches!(
            PromptSourceArg::from(PromptSource::Dynamic),
            PromptSourceArg::Dynamic
        ));
    }

    #[test]
    fn test_prompt_source_arg_equality() {
        assert_eq!(PromptSourceArg::Builtin, PromptSourceArg::Builtin);
        assert_ne!(PromptSourceArg::Builtin, PromptSourceArg::User);
        assert_ne!(PromptSourceArg::User, PromptSourceArg::Local);
        assert_ne!(PromptSourceArg::Local, PromptSourceArg::Dynamic);
    }

    #[test]
    fn test_debug_flag() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--debug"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.debug);
        assert!(!cli.verbose);
        assert!(!cli.quiet);
    }

    #[test]
    fn test_combined_flags() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "--debug", "--verbose"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        assert!(cli.debug);
        assert!(cli.verbose);
        assert!(!cli.quiet);
    }
}
