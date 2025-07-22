# Command Line Interface

SwissArmyHammer provides a comprehensive command-line interface for managing prompts, running the MCP server, and integrating with your development workflow.

## Installation

```bash
# Install from Git repository (requires Rust)
cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli

# Ensure ~/.cargo/bin is in your PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

## Basic Usage

```bash
swissarmyhammer [COMMAND] [OPTIONS]
```

## Global Options

- `--help, -h` - Display help information
- `--version, -V` - Display version information

## Commands Overview

| Command | Description |
|---------|-------------|
| [`serve`](./cli-serve.md) | Run as MCP server for Claude Code integration |
| [`search`](./cli-search.md) | Search and discover prompts with powerful filtering |
| [`test`](./cli-test.md) | Interactively test prompts with arguments |
| [`config`](#configuration-commands) | Manage SwissArmyHammer configuration |
| [`doctor`](./cli-doctor.md) | Diagnose configuration and setup issues |
| [`completion`](./cli-completion.md) | Generate shell completion scripts |

## Quick Examples

### Start MCP Server
```bash
# Run as MCP server (for Claude Code)
swissarmyhammer serve
```

### Search for Prompts
```bash
# Search for code-related prompts
swissarmyhammer search code

# Search with regex in descriptions
swissarmyhammer search --regex "test.*unit" --in description
```

### Test a Prompt
```bash
# Interactively test a prompt
swissarmyhammer test code-review

# Test with predefined arguments
swissarmyhammer test code-review --arg code="fn main() { println!(\"Hello\"); }"
```

### Check Setup
```bash
# Diagnose any configuration issues
swissarmyhammer doctor
```

### Generate Shell Completions
```bash
# Generate Bash completions
swissarmyhammer completion bash > ~/.bash_completion.d/swissarmyhammer

# Generate Zsh completions
swissarmyhammer completion zsh > ~/.zfunc/_swissarmyhammer
```

### Manage Configuration
```bash
# View current configuration
swissarmyhammer config show

# Validate configuration
swissarmyhammer config validate

# Generate example configuration
swissarmyhammer config init

# Get configuration help
swissarmyhammer config help
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Command line usage error
- `3` - Configuration error
- `4` - Prompt not found
- `5` - Template rendering error

## Configuration

SwissArmyHammer looks for prompts in these directories (in order):

1. Built-in prompts (embedded in the binary)
2. User prompts: `~/.swissarmyhammer/prompts/`
3. Local prompts: `./.swissarmyhammer/prompts/` (current directory)

## Configuration Commands

### `swissarmyhammer config`

Manage SwissArmyHammer configuration.

For detailed configuration guide and examples, see [Configuration](configuration.md). For complete schema reference, see [Configuration Schema](configuration-schema.md).

#### Subcommands

##### `config show`
Display current configuration values and their sources.

```bash
swissarmyhammer config show
```

Example output:
```
📋 Current Configuration:
base_branch: main
issue_branch_prefix: issue/
issue_number_width: 6
```

##### `config validate`
Validate the current configuration for errors.

```bash
swissarmyhammer config validate
```

Returns exit code 0 if valid, 1 if invalid.

##### `config init`
Generate an example configuration file in the current directory.

```bash
swissarmyhammer config init
```

Creates `swissarmyhammer.yaml` with example configuration.

##### `config help`
Show detailed configuration help and documentation.

```bash
swissarmyhammer config help
```

For detailed command documentation, see the individual command pages linked in the table above.
