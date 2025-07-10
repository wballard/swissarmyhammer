# CLI Command Migration Guide

This guide helps users migrate from the old CLI command structure to the new prompt subcommand structure in SwissArmyHammer.

## Overview

The CLI has been restructured to organize prompt-related commands under a unified `prompt` subcommand. This improves command organization and makes the CLI more intuitive.

## Command Changes

### List Prompts

**Old Command:**
```bash
swissarmyhammer list
```

**New Command:**
```bash
swissarmyhammer prompt list
```

Options remain the same:
- `--format <FORMAT>`: Output format (json, yaml, table)
- `--verbose`: Show additional details
- `--source <SOURCE>`: Filter by source
- `--category <CATEGORY>`: Filter by category
- `--search <SEARCH>`: Search in names and descriptions

### Search Prompts

**Old Command:**
```bash
swissarmyhammer search <QUERY>
```

**New Command:**
```bash
swissarmyhammer prompt search <QUERY>
```

All search options remain unchanged.

### Validate Prompts

**Old Command:**
```bash
swissarmyhammer validate
```

**New Command:**
```bash
swissarmyhammer prompt validate
```

Options remain the same:
- `--quiet`: Suppress output
- `--format <FORMAT>`: Output format
- `--workflow-dirs <DIRS>`: Additional directories to validate

### Test Prompts

**Old Command:**
```bash
swissarmyhammer test <PROMPT_NAME>
```

**New Command:**
```bash
swissarmyhammer prompt test <PROMPT_NAME>
```

All test options remain unchanged.

## Other Commands

The following commands remain at the top level and are unchanged:
- `swissarmyhammer serve` - Start MCP server
- `swissarmyhammer doctor` - Run diagnostics
- `swissarmyhammer completion <SHELL>` - Generate shell completions
- `swissarmyhammer flow` - Flow-related commands

## Benefits of the New Structure

1. **Better Organization**: All prompt-related operations are grouped together
2. **Clearer Intent**: The `prompt` prefix makes it clear what resource you're working with
3. **Room for Growth**: Easy to add new prompt operations without cluttering the top-level namespace
4. **Consistent with Modern CLIs**: Follows patterns used by tools like `git`, `docker`, and `kubectl`

## Quick Reference

| Old Command | New Command |
|------------|-------------|
| `swissarmyhammer list` | `swissarmyhammer prompt list` |
| `swissarmyhammer search <query>` | `swissarmyhammer prompt search <query>` |
| `swissarmyhammer validate` | `swissarmyhammer prompt validate` |
| `swissarmyhammer test <name>` | `swissarmyhammer prompt test <name>` |

## Shell Completion

If you use shell completion, regenerate your completion scripts:

```bash
# Bash
swissarmyhammer completion bash > ~/.bash_completion.d/swissarmyhammer

# Zsh
swissarmyhammer completion zsh > ~/.zsh/completions/_swissarmyhammer

# Fish
swissarmyhammer completion fish > ~/.config/fish/completions/swissarmyhammer.fish
```

## Getting Help

For help with any command, use the `--help` flag:

```bash
swissarmyhammer --help
swissarmyhammer prompt --help
swissarmyhammer prompt list --help
```