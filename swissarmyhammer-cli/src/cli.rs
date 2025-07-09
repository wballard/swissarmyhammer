use clap::{Parser, Subcommand, ValueEnum};
use is_terminal::IsTerminal;
use std::io;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, serde::Serialize)]
pub enum PromptSource {
    Builtin,
    User,
    Local,
    Dynamic,
}

impl std::fmt::Display for PromptSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptSource::Builtin => write!(f, "builtin"),
            PromptSource::User => write!(f, "user"),
            PromptSource::Local => write!(f, "local"),
            PromptSource::Dynamic => write!(f, "dynamic"),
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
    /// List all available prompts
    #[command(long_about = "
Lists all available prompts from all sources (built-in, user, local).
Shows prompt names, titles, descriptions, and source information.

Output formats:
  table  - Formatted table (default)
  json   - JSON output for scripting
  yaml   - YAML output for scripting

Examples:
  swissarmyhammer list                        # Show all prompts in table format
  swissarmyhammer list --format json         # Output as JSON
  swissarmyhammer list --verbose             # Show full details including arguments
  swissarmyhammer list --source builtin      # Show only built-in prompts
  swissarmyhammer list --search debug        # Search for prompts containing 'debug'
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
        source: Option<PromptSource>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Search prompts by name or description
        #[arg(long)]
        search: Option<String>,
    },
    /// Validate prompt files for syntax and best practices
    #[command(long_about = "
Validates all prompt files for syntax errors and best practices.
Checks YAML front matter, template variables, and suggests improvements.

Validation checks:
- YAML front matter syntax (skipped for .liquid files with {% partial %} marker)
- Required fields (title, description)
- Template variables match arguments
- Liquid template syntax
- Best practice recommendations

Examples:
  swissarmyhammer validate                 # Validate all prompts
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

        /// Specific workflow directories to validate (can be specified multiple times)
        #[arg(long = "workflow-dir", value_name = "DIR")]
        workflow_dirs: Vec<String>,
    },
    /// Test prompts interactively with sample arguments
    #[command(long_about = "
Test prompts interactively to see how they render with different arguments.
Helps debug template errors and refine prompt content before using in Claude Code.

Usage modes:
  swissarmyhammer test prompt-name                    # Test by name (interactive)
  swissarmyhammer test -f path/to/prompt.md          # Test from file
  swissarmyhammer test prompt-name --arg key=value   # Non-interactive mode

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
  swissarmyhammer test code-review                           # Interactive test
  swissarmyhammer test -f my-prompt.md                       # Test file
  swissarmyhammer test help --arg topic=git                  # Non-interactive
  swissarmyhammer test plan --debug --save output.md         # Debug + save
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
  swissarmyhammer search \"code review\"        # Basic search
  swissarmyhammer search \"debug.*error\" -r   # Regex search
  swissarmyhammer search help --fuzzy          # Fuzzy matching

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
  swissarmyhammer search \"python code\"        # Find Python-related prompts
  swissarmyhammer search \"review\" --full       # Detailed results for review prompts
  swissarmyhammer search \".*test.*\" --regex     # Regex pattern matching
  swissarmyhammer search help --fuzzy --limit 5  # Fuzzy search, max 5 results
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
        source: Option<PromptSource>,

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
        source: Option<PromptSource>,
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
        let result = Cli::try_parse_from_args(["swissarmyhammer", "test", "help"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        }) = cli.command
        {
            assert_eq!(prompt_name, Some("help".to_string()));
            assert_eq!(file, None);
            assert!(arguments.is_empty());
            assert!(!raw);
            assert!(!copy);
            assert_eq!(save, None);
            assert!(!debug);
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_file() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "test", "-f", "test.md"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        }) = cli.command
        {
            assert_eq!(prompt_name, None);
            assert_eq!(file, Some("test.md".to_string()));
            assert!(arguments.is_empty());
            assert!(!raw);
            assert!(!copy);
            assert_eq!(save, None);
            assert!(!debug);
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_arguments() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "test",
            "help",
            "--arg",
            "topic=git",
            "--arg",
            "format=markdown",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        }) = cli.command
        {
            assert_eq!(prompt_name, Some("help".to_string()));
            assert_eq!(file, None);
            assert_eq!(arguments, vec!["topic=git", "format=markdown"]);
            assert!(!raw);
            assert!(!copy);
            assert_eq!(save, None);
            assert!(!debug);
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_cli_test_subcommand_with_all_flags() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
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
        if let Some(Commands::Test {
            prompt_name,
            file,
            arguments,
            raw,
            copy,
            save,
            debug,
        }) = cli.command
        {
            assert_eq!(prompt_name, Some("help".to_string()));
            assert_eq!(file, None);
            assert!(arguments.is_empty());
            assert!(raw);
            assert!(copy);
            assert_eq!(save, Some("output.md".to_string()));
            assert!(debug);
        } else {
            panic!("Expected Test command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_basic() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "search", "code review"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Search {
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
        }) = cli.command
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
            panic!("Expected Search command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_with_flags() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
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
        if let Some(Commands::Search {
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
        }) = cli.command
        {
            assert_eq!(query, "debug.*error");
            assert_eq!(r#in, None);
            assert!(regex);
            assert!(fuzzy);
            assert!(case_sensitive);
            assert!(matches!(source, Some(PromptSource::Builtin)));
            assert_eq!(has_arg, Some("language".to_string()));
            assert!(!no_args);
            assert!(full);
            assert!(matches!(format, OutputFormat::Json));
            assert!(highlight);
            assert_eq!(limit, Some(5));
        } else {
            panic!("Expected Search command");
        }
    }

    #[test]
    fn test_cli_search_subcommand_with_fields() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "search",
            "python",
            "--in",
            "name,description,content",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Search { query, r#in, .. }) = cli.command {
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
            panic!("Expected Search command");
        }
    }
}
