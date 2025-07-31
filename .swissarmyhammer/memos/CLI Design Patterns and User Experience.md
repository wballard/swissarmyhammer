# SwissArmyHammer CLI Design Patterns and User Experience

## Command Architecture with Clap

**Hierarchical Command Structure**
```rust
// Root CLI with global flags and subcommands
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    #[arg(long, short, help = "Enable verbose output")]
    pub verbose: bool,
    
    #[arg(long, short, help = "Quiet mode - suppress output")]
    pub quiet: bool,
}
```

**Structured Subcommand Pattern**
```rust
Commands::Prompt { subcommand: PromptSubcommand }
  ├─ List: Display available prompts
  ├─ Test: Validate prompt syntax and arguments
  ├─ Search: Full-text search across prompts
  └─ Render: Process templates with arguments
```

**Rich Help and Documentation**
- Detailed `long_about` descriptions with examples
- `value_name` for clear parameter documentation
- `help` attributes for concise parameter descriptions
- Built-in `--help` and command-specific help

## Argument Parsing Patterns

**Type-Safe Option Handling**
```rust
#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Table,  // Default human-readable format
    Json,   // Machine-readable JSON
    Yaml,   // Structured YAML output
}
```

**Key-Value Argument Pattern**
```rust
#[arg(long = "var", value_name = "KEY=VALUE", 
      help = "Set template variable")]
pub variables: Vec<String>,
```

**Flexible Input Sources**
- File paths for batch operations
- Stdin support using `-` convention
- Environment variable integration
- Configuration file loading

## Output Formatting Strategy

**Multi-Format Support**
```rust
match format {
    OutputFormat::Table => print_table(&data),
    OutputFormat::Json => print_json(&data),
    OutputFormat::Yaml => print_yaml(&data),
}
```

**Terminal-Aware Formatting**
- Color detection via `is_terminal::IsTerminal`
- `NO_COLOR` environment variable support
- Rich formatting with emojis and styling
- Progress indicators for long operations

**Consistent Display Patterns**
- Two-line compact format for lists
- Status indicators (✅ ❌ ⚠️)
- Aligned column output for tables
- JSON structured output for automation

## Error Handling and User Experience

**Structured CLI Error Handling**
```rust
pub struct CliError {
    pub message: String,
    pub exit_code: i32,
    pub source: Option<Box<dyn std::error::Error>>,
}
```

**Exit Code Strategy**
```rust
const EXIT_SUCCESS: i32 = 0;   // Operation completed successfully
const EXIT_WARNING: i32 = 1;   // Completed with warnings
const EXIT_ERROR: i32 = 2;     // Failed or validation errors
```

**User-Friendly Error Messages**
- Clear error descriptions without technical jargon
- Suggested fixes and next steps
- Error context preservation
- Graceful degradation for non-critical failures

## Logging and Debugging

**Adaptive Logging Configuration**
```rust
let log_level = match (cli.quiet, cli.debug, cli.verbose) {
    (true, _, _) => Level::ERROR,
    (_, true, _) => Level::DEBUG,
    (_, _, true) => Level::TRACE,
    _ => Level::INFO,
};
```

**MCP Mode Special Handling**
- File-based logging to `.swissarmyhammer/mcp.log`
- Thread-safe file writing with immediate flush
- Configurable log files via environment variables
- Enhanced debugging for protocol communication

## MCP Tool Integration Pattern

**CliToolContext Initialization**
```rust
// Common pattern across all MCP-integrated commands
let context = CliToolContext::new().await?;
```

**Structured Argument Creation**
```rust
// Pattern for tool arguments with type safety
let args = context.create_arguments(vec![
    ("name", json!(name)),
    ("content", json!(content)),
    ("append", json!(append)),
]);
```

**Consistent Tool Execution**
```rust
// Standard execution and response handling
let result = context.execute_tool("issue_create", args).await?;
println!("{}", response_formatting::format_success_response(&result));
```

**Key MCP Integration Patterns:**
- Single `CliToolContext` per command execution
- Type-safe argument construction with `serde_json::json!` macro
- Consistent error propagation with `?` operator
- Uniform response formatting for all tool outputs
- Empty argument vectors for parameterless tools

## Command Delegation Pattern

**Clean Separation of Concerns**
```rust
// Main dispatcher with MCP tool delegation
match cli.command {
    Some(Commands::Prompt { subcommand }) => run_prompt(subcommand).await,
    Some(Commands::Issue { subcommand }) => run_issue(subcommand).await,
    Some(Commands::Memo { subcommand }) => run_memo(subcommand).await,
    // ... each command delegates to MCP tool handlers
}
```

**MCP-Aware Command Handlers**
```rust
pub async fn handle_issue_command(
    command: IssueCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = CliToolContext::new().await?;
    
    match command {
        IssueCommands::Create { name, content, file } => {
            create_issue(&context, name, content, file).await?;
        }
        // ... pattern continues for all subcommands
    }
    Ok(())
}
```

**Async Command Handling**
- Tokio runtime for async MCP tool operations
- Context initialization before tool execution
- Graceful error handling through tool context
- Resource cleanup managed by context lifetime

## Shell Integration Features

**Comprehensive Completion Support**
```rust
Commands::Completion { shell } => {
    generate_completion(shell, &mut std::io::stdout());
}
```

**Supported Shells**
- Bash: Complete command and parameter completion
- Zsh: Advanced completion with descriptions
- Fish: Interactive completion with context
- PowerShell: Windows-native completion

## Performance Optimizations

**Fast Path for Common Operations**
```rust
// Avoid expensive initialization for help
if cli.command.is_none() {
    Cli::command().print_help().expect("Failed to print help");
    process::exit(EXIT_SUCCESS);
}
```

**Lazy Loading Strategy**
- Heavy dependencies loaded only when needed
- MCP server initialization deferred
- File system operations minimized
- Cache-friendly data structures

## Usability Enhancements

**Developer Experience**
- Rich error messages with context
- Extensive validation with helpful suggestions
- Autocomplete support for parameters
- Integration with development workflows

**Operational Features**
- Doctor command for system diagnostics
- Validation commands for content checking
- Debug modes for troubleshooting
- Machine-readable output formats

This CLI design demonstrates modern command-line interface patterns with emphasis on usability, performance, and integration with development workflows while maintaining backwards compatibility and extensibility.