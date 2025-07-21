# Configuration Schema Reference

This page provides a complete reference for the `swissarmyhammer.yaml` configuration file schema.

For configuration basics and examples, see the [Configuration Guide](configuration.md). For CLI configuration commands, see the [CLI Reference](cli-reference.md#configuration-commands).

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
  issue_branch_prefix:
    type: string
    description: Prefix for issue branches
    default: "issue/"
    minLength: 1
    maxLength: 50
  issue_number_width:
    type: integer
    description: Width for issue numbers in display
    default: 6
    minimum: 1
    maximum: 10
```

## Validation Rules

### `base_branch`
- Must be a valid git branch name
- Cannot be empty
- Cannot contain spaces or special characters: `~ ^ : ? * [ \`
- Maximum length: 200 characters
- Examples of valid values: `main`, `develop`, `feature/v2`, `release-1.0`
- Examples of invalid values: `""`, `branch with spaces`, `branch~invalid`

### `issue_branch_prefix`
- Must be a non-empty string
- Cannot exceed 50 characters
- Used as the prefix for all issue branches
- Examples of valid values: `issue/`, `feature/`, `bug/`, `task-`
- Examples of invalid values: `""` (empty string), `very-long-prefix-that-exceeds-the-maximum-length`

### `issue_number_width`
- Must be a positive integer
- Minimum value: 1
- Maximum value: 10
- Determines the padding width for issue numbers in display
- Examples of valid values: `6`, `4`, `8`
- Examples of invalid values: `0`, `11`, `-1`

## Configuration File Discovery

SwissArmyHammer looks for configuration files in this order:

1. `./swissarmyhammer.yaml` in the current working directory

If no configuration file is found, SwissArmyHammer will use environment variables and built-in defaults.

## Environment Variable Mapping

| Configuration Key | Environment Variable | Default Value |
|------------------|---------------------|---------------|
| `base_branch` | `SWISSARMYHAMMER_BASE_BRANCH` | `"main"` |
| `issue_branch_prefix` | `SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX` | `"issue/"` |
| `issue_number_width` | `SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH` | `6` |

## Error Handling

Configuration errors are handled gracefully:

- **File not found**: Uses environment variables and defaults
- **Invalid YAML syntax**: Shows detailed parsing error with line numbers
- **Invalid values**: Shows validation error with helpful hints
- **Permission errors**: Falls back to environment variables and defaults

All configuration errors include helpful error messages and suggestions for resolution.

For more configuration troubleshooting help, see the [Troubleshooting Guide](troubleshooting.md#configuration-issues).