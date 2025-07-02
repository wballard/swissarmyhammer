use clap::{Parser, Subcommand, ValueEnum};
use is_terminal::IsTerminal;
use std::io;

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum PromptSource {
    Builtin,
    User,
    Local,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ValidateFormat {
    Text,
    Json,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    TarGz,
    Zip,
    Directory,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ImportStrategy {
    Skip,
    Overwrite,
    Rename,
    Merge,
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
Validates prompt files for syntax errors and best practices.
Checks YAML front matter, template variables, and suggests improvements.

Usage modes:
  swissarmyhammer validate file.md      # Validate single file
  swissarmyhammer validate dir/         # Validate directory
  swissarmyhammer validate --all        # Validate all prompt directories

Validation checks:
- YAML front matter syntax
- Required fields (title, description)
- Template variables match arguments
- File encoding and line endings
- Best practice recommendations

Examples:
  swissarmyhammer validate prompts/my-prompt.md    # Validate one file
  swissarmyhammer validate --all                   # Validate all prompts
  swissarmyhammer validate --quiet --all           # CI/CD mode (exit code only)
  swissarmyhammer validate --format json --all     # JSON output for tooling
")]
    Validate {
        /// Path to file or directory to validate
        path: Option<String>,

        /// Validate all prompt directories (builtin, user, local)
        #[arg(long)]
        all: bool,

        /// Only show errors, no warnings or info
        #[arg(short, long)]
        quiet: bool,

        /// Output format
        #[arg(long, value_enum, default_value = "text")]
        format: ValidateFormat,
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
    /// Export prompts for sharing
    #[command(long_about = "
Export prompts to various formats for sharing with teams or the community.
Supports exporting single prompts, categories, or entire collections.

Export formats:
  tar.gz    - Compressed tar archive (default)
  zip       - ZIP archive for Windows compatibility
  directory - Uncompressed directory structure

Examples:
  swissarmyhammer export my-prompt                    # Export single prompt
  swissarmyhammer export --category debug             # Export category
  swissarmyhammer export --all                        # Export everything
  swissarmyhammer export --all --format zip           # Export as ZIP
  swissarmyhammer export --source user --format dir   # Export user prompts to directory
")]
    Export {
        /// Prompt name to export (alternative to --all or --category)
        prompt_name: Option<String>,

        /// Export all prompts
        #[arg(long)]
        all: bool,

        /// Export prompts from specific category
        #[arg(long)]
        category: Option<String>,

        /// Filter by source
        #[arg(long, value_enum)]
        source: Option<PromptSource>,

        /// Output format
        #[arg(long, value_enum, default_value = "tar-gz")]
        format: ExportFormat,

        /// Output file/directory path
        #[arg(short, long)]
        output: Option<String>,

        /// Include author and licensing metadata
        #[arg(long)]
        metadata: bool,

        /// Exclude patterns (gitignore-style)
        #[arg(long)]
        exclude: Vec<String>,
    },
    /// Import prompts from archives, URLs, or Git repositories
    #[command(long_about = "
Import prompts from various sources including archives, URLs, and Git repositories.
Provides safety features like validation, conflict resolution, and rollback.

Import sources:
  file      - Local archive (.tar.gz, .zip)
  url       - Remote archive URL
  git       - Git repository

Conflict resolution strategies:
  skip      - Skip conflicting prompts
  overwrite - Replace existing prompts
  rename    - Add suffix to conflicting prompts
  merge     - Combine metadata intelligently

Examples:
  swissarmyhammer import prompts.tar.gz               # Import local archive
  swissarmyhammer import https://example.com/prompts.tar.gz  # Import from URL
  swissarmyhammer import git@github.com:user/prompts.git     # Import from Git
  swissarmyhammer import --dry-run prompts.zip       # Preview import
  swissarmyhammer import --strategy rename archive.tar.gz   # Handle conflicts
")]
    Import {
        /// Source to import from (file path, URL, or Git repository)
        source: String,

        /// Preview import without making changes
        #[arg(long)]
        dry_run: bool,

        /// Conflict resolution strategy
        #[arg(long, value_enum, default_value = "skip")]
        strategy: ImportStrategy,

        /// Target directory for import
        #[arg(long)]
        target: Option<String>,

        /// Skip validation of prompts before importing
        #[arg(long = "no-validate")]
        no_validate: bool,

        /// Skip creating backup before overwriting  
        #[arg(long = "no-backup")]
        no_backup: bool,

        /// Show detailed progress information
        #[arg(short, long)]
        verbose: bool,
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

    #[test]
    fn test_cli_export_subcommand_basic() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "export", "my-prompt"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Export {
            prompt_name,
            all,
            category,
            source,
            format,
            output,
            metadata,
            exclude,
        }) = cli.command
        {
            assert_eq!(prompt_name, Some("my-prompt".to_string()));
            assert!(!all);
            assert_eq!(category, None);
            assert_eq!(source, None);
            assert!(matches!(format, ExportFormat::TarGz));
            assert_eq!(output, None);
            assert!(!metadata);
            assert!(exclude.is_empty());
        } else {
            panic!("Expected Export command");
        }
    }

    #[test]
    fn test_cli_export_subcommand_all_with_options() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "export",
            "--all",
            "--format",
            "zip",
            "--output",
            "prompts.zip",
            "--metadata",
            "--exclude",
            "*.tmp",
            "--exclude",
            "draft-*",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Export {
            prompt_name,
            all,
            category,
            source,
            format,
            output,
            metadata,
            exclude,
        }) = cli.command
        {
            assert_eq!(prompt_name, None);
            assert!(all);
            assert_eq!(category, None);
            assert_eq!(source, None);
            assert!(matches!(format, ExportFormat::Zip));
            assert_eq!(output, Some("prompts.zip".to_string()));
            assert!(metadata);
            assert_eq!(exclude, vec!["*.tmp", "draft-*"]);
        } else {
            panic!("Expected Export command");
        }
    }

    #[test]
    fn test_cli_export_subcommand_category() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "export",
            "--category",
            "debug",
            "--source",
            "user",
            "--format",
            "directory",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Export {
            prompt_name,
            all,
            category,
            source,
            format,
            output,
            metadata,
            exclude,
        }) = cli.command
        {
            assert_eq!(prompt_name, None);
            assert!(!all);
            assert_eq!(category, Some("debug".to_string()));
            assert!(matches!(source, Some(PromptSource::User)));
            assert!(matches!(format, ExportFormat::Directory));
            assert_eq!(output, None);
            assert!(!metadata);
            assert!(exclude.is_empty());
        } else {
            panic!("Expected Export command");
        }
    }

    #[test]
    fn test_cli_import_subcommand_basic() {
        let result = Cli::try_parse_from_args(["swissarmyhammer", "import", "prompts.tar.gz"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Import {
            source,
            dry_run,
            strategy,
            target,
            no_validate,
            no_backup,
            verbose,
        }) = cli.command
        {
            assert_eq!(source, "prompts.tar.gz");
            assert!(!dry_run);
            assert!(matches!(strategy, ImportStrategy::Skip));
            assert_eq!(target, None);
            assert!(!no_validate);
            assert!(!no_backup);
            assert!(!verbose);
        } else {
            panic!("Expected Import command");
        }
    }

    #[test]
    fn test_cli_import_subcommand_with_options() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "import",
            "https://example.com/prompts.tar.gz",
            "--dry-run",
            "--strategy",
            "overwrite",
            "--target",
            "/tmp/prompts",
            "--no-validate",
            "--no-backup",
            "--verbose",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Import {
            source,
            dry_run,
            strategy,
            target,
            no_validate,
            no_backup,
            verbose,
        }) = cli.command
        {
            assert_eq!(source, "https://example.com/prompts.tar.gz");
            assert!(dry_run);
            assert!(matches!(strategy, ImportStrategy::Overwrite));
            assert_eq!(target, Some("/tmp/prompts".to_string()));
            assert!(no_validate);
            assert!(no_backup);
            assert!(verbose);
        } else {
            panic!("Expected Import command");
        }
    }

    #[test]
    fn test_cli_import_subcommand_git_repository() {
        let result = Cli::try_parse_from_args([
            "swissarmyhammer",
            "import",
            "git@github.com:user/prompts.git",
            "--strategy",
            "rename",
        ]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        if let Some(Commands::Import {
            source,
            dry_run,
            strategy,
            target,
            no_validate,
            no_backup,
            verbose,
        }) = cli.command
        {
            assert_eq!(source, "git@github.com:user/prompts.git");
            assert!(!dry_run);
            assert!(matches!(strategy, ImportStrategy::Rename));
            assert_eq!(target, None);
            assert!(!no_validate);
            assert!(!no_backup);
            assert!(!verbose);
        } else {
            panic!("Expected Import command");
        }
    }
}
