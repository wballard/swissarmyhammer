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