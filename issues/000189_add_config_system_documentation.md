# Add Configuration System Documentation

## Overview
Create comprehensive documentation for the new YAML configuration system, including user guides, examples, and integration documentation.

## Context
With the complete YAML configuration system implemented and integrated, this final step ensures users have excellent documentation to understand and use the new configuration capabilities. Following the specification's requirements, this includes clear explanations of the configuration file format, options, and usage patterns.

## Requirements
- Update user-facing documentation with configuration file information
- Add configuration examples and common use cases
- Document configuration precedence hierarchy
- Add troubleshooting guide for configuration issues
- Update CLI reference documentation
- Add configuration to the mdBook documentation
- Create configuration schema documentation

## Implementation Details

### Update Main Documentation
Add comprehensive configuration section to the documentation:

**File: `doc/src/configuration.md`**
```markdown
# Configuration

SwissArmyHammer supports flexible configuration through multiple sources, with a clear precedence hierarchy:

1. **YAML Configuration File** (highest precedence)
2. **Environment Variables** (medium precedence)  
3. **Built-in Defaults** (lowest precedence)

## Configuration File

Place a `swissarmyhammer.yaml` file in your repository root to configure SwissArmyHammer:

```yaml
# swissarmyhammer.yaml
# Configuration for Swiss Army Hammer

# Base branch that pull requests will merge into
base_branch: "main"
```

### Supported Options

#### `base_branch`
- **Type**: String
- **Default**: `"main"`
- **Description**: The base branch that pull requests will merge into
- **Example**: `base_branch: "develop"`

## Environment Variables

All configuration options can be set via environment variables with the `SWISSARMYHAMMER_` prefix:

```bash
export SWISSARMYHAMMER_BASE_BRANCH="develop"
```

## Configuration Precedence

Configuration values are loaded in the following order:

1. **YAML file**: Values from `swissarmyhammer.yaml` (if present)
2. **Environment variables**: `SWISSARMYHAMMER_*` environment variables
3. **Defaults**: Built-in default values

Later sources override earlier ones, so YAML configuration takes precedence over environment variables, which take precedence over defaults.

## CLI Configuration Management

### View Current Configuration
```bash
swissarmyhammer config show
```

### Validate Configuration  
```bash
swissarmyhammer config validate
```

### Generate Example Configuration
```bash
swissarmyhammer config init
```

### Get Configuration Help
```bash
swissarmyhammer config help
```

## Common Configuration Examples

### Development Team Setup
```yaml
# Development team targeting develop branch
base_branch: "develop"
```

### Open Source Project
```yaml  
# Open source project with main branch
base_branch: "main"
```

### Enterprise Setup
```yaml
# Enterprise with release branches
base_branch: "release/current"
```

## Troubleshooting

### Configuration Not Loading
1. Check that `swissarmyhammer.yaml` is in the repository root
2. Validate YAML syntax: `swissarmyhammer config validate`
3. Check file permissions and accessibility

### Invalid Configuration Values
- Run `swissarmyhammer config validate` for detailed error messages
- Check that branch names are valid git branch names
- Ensure all values match expected types and formats

### Configuration Conflicts
- Use `swissarmyhammer config show` to see current active configuration
- Remember: YAML file > Environment variables > Defaults
- Check for conflicting environment variables

For more help, run: `swissarmyhammer config help`
```

### Add Configuration Examples
**File: `doc/examples/configs/swissarmyhammer.yaml`**
```yaml
# Example SwissArmyHammer configuration file
# Place this file as 'swissarmyhammer.yaml' in your repository root

# Base branch that pull requests will merge into
# This is the target branch for all PRs created by SwissArmyHammer
base_branch: "main"

# You can also use "develop", "master", or any other branch name
# Examples:
# base_branch: "develop"      # For GitFlow development workflow
# base_branch: "master"       # For legacy repositories
# base_branch: "release/v2"   # For release-branch workflows
```

### Update CLI Reference Documentation
**File: `doc/src/cli-reference.md`**
```markdown
# CLI Reference

## Configuration Commands

### `swissarmyhammer config`

Manage SwissArmyHammer configuration.

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
```

### Update SUMMARY.md for mdBook
**File: `doc/src/SUMMARY.md`**
```markdown
# Summary

[Introduction](./introduction.md)
[Quick Start](./quick-start.md)
[Installation](./installation.md)

# User Guide
- [Configuration](./configuration.md)
- [Creating Prompts](./creating-prompts.md)
# ... rest of existing structure
```

### Add Configuration Schema Documentation
**File: `doc/src/configuration-schema.md`**
```markdown
# Configuration Schema Reference

This page provides a complete reference for the `swissarmyhammer.yaml` configuration file schema.

## Schema Definition

```yaml
# SwissArmyHammer configuration schema
type: object
properties:
  base_branch:
    type: string
    description: Base branch that pull requests will merge into
    default: "main"
    pattern: "^[a-zA-Z0-9._/-]+$"
    minLength: 1
    maxLength: 200
```

## Validation Rules

### `base_branch`
- Must be a valid git branch name
- Cannot be empty
- Cannot contain spaces or special characters: `~ ^ : ? * [ \`
- Maximum length: 200 characters
- Examples of valid values: `main`, `develop`, `feature/v2`, `release-1.0`
- Examples of invalid values: `""`, `branch with spaces`, `branch~invalid`

## Configuration File Discovery

SwissArmyHammer looks for configuration files in this order:

1. `./swissarmyhammer.yaml` in the current working directory

If no configuration file is found, SwissArmyHammer will use environment variables and built-in defaults.

## Environment Variable Mapping

| Configuration Key | Environment Variable | Default Value |
|------------------|---------------------|---------------|
| `base_branch` | `SWISSARMYHAMMER_BASE_BRANCH` | `"main"` |

## Error Handling

Configuration errors are handled gracefully:

- **File not found**: Uses environment variables and defaults
- **Invalid YAML syntax**: Shows detailed parsing error with line numbers
- **Invalid values**: Shows validation error with helpful hints
- **Permission errors**: Falls back to environment variables and defaults

All configuration errors include helpful error messages and suggestions for resolution.
```

### Add Troubleshooting Section
**File: `doc/src/troubleshooting.md`** (update existing or create)
```markdown
# Troubleshooting

## Configuration Issues

### Configuration File Not Loading

**Problem**: SwissArmyHammer isn't using my configuration file.

**Solutions**:
1. Ensure the file is named exactly `swissarmyhammer.yaml`
2. Place the file in your repository root (same directory where you run SwissArmyHammer)
3. Check file permissions: `ls -la swissarmyhammer.yaml`
4. Validate file syntax: `swissarmyhammer config validate`

### Invalid YAML Syntax

**Problem**: Getting YAML parsing errors.

**Solutions**:
1. Check indentation (use spaces, not tabs)
2. Ensure proper YAML syntax with online validators
3. Quote string values that contain special characters
4. Check for trailing spaces or special characters

Example of common YAML mistakes:
```yaml
# ❌ Wrong - uses tabs for indentation
base_branch:	"main"

# ❌ Wrong - inconsistent quotes  
base_branch: 'main"

# ✅ Correct
base_branch: "main"
```

### Branch Name Validation Errors

**Problem**: Getting "Invalid configuration value for base_branch".

**Solutions**:
1. Remove spaces from branch names: `feature branch` → `feature-branch`
2. Avoid special characters: `~ ^ : ? * [ \`
3. Use standard git branch naming conventions
4. Check if branch actually exists in your repository

### Environment Variables Not Working

**Problem**: Environment variables aren't being used.

**Solutions**:
1. Use the correct prefix: `SWISSARMYHAMMER_BASE_BRANCH`
2. Export variables in your shell: `export SWISSARMYHAMMER_BASE_BRANCH=develop`
3. Check variable is set: `echo $SWISSARMYHAMMER_BASE_BRANCH`
4. Remember YAML config overrides environment variables

## Getting Help

For configuration issues:
```bash
swissarmyhammer config help      # Configuration documentation
swissarmyhammer config validate  # Check current config  
swissarmyhammer config show      # Show active configuration
swissarmyhammer doctor          # Overall system health check
```
```

## Acceptance Criteria
- [ ] Comprehensive configuration documentation in mdBook
- [ ] Configuration examples with common use cases
- [ ] Complete CLI reference for config commands
- [ ] Configuration schema reference documentation
- [ ] Troubleshooting guide for configuration issues
- [ ] All documentation is clear and user-friendly
- [ ] Examples are tested and working
- [ ] Documentation builds successfully in mdBook
- [ ] Configuration precedence clearly explained
- [ ] Integration examples provided

## Files to Create/Modify
- `doc/src/configuration.md` (new)
- `doc/src/configuration-schema.md` (new)
- `doc/examples/configs/swissarmyhammer.yaml` (new)
- `doc/src/SUMMARY.md` (update)
- `doc/src/cli-reference.md` (update)
- `doc/src/troubleshooting.md` (update)

## Dependencies
- mdBook for documentation building
- Existing documentation structure

## Validation
- Documentation builds without errors
- All examples work as described
- Links are valid and functional
- Code examples are syntax-highlighted correctly
- Documentation is accessible and well-organized

## Notes
This step completes the configuration system implementation by ensuring users have excellent documentation and guidance. The documentation follows best practices for technical writing and provides both reference material and practical examples.