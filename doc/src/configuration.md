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

# Prefix for issue branches
issue_branch_prefix: "issue/"

# Width for issue numbers in display  
issue_number_width: 6
```

### Supported Options

For detailed validation rules and complete schema reference, see [Configuration Schema](configuration-schema.md).

#### `base_branch`
- **Type**: String
- **Default**: `"main"`
- **Description**: The base branch that pull requests will merge into
- **Example**: `base_branch: "develop"`

#### `issue_branch_prefix`
- **Type**: String
- **Default**: `"issue/"`
- **Description**: Prefix for issue branches
- **Example**: `issue_branch_prefix: "feature/"`

#### `issue_number_width`
- **Type**: Integer
- **Default**: `6`
- **Description**: Width for issue numbers in display
- **Example**: `issue_number_width: 8`

## Environment Variables

All configuration options can be set via environment variables with the `SWISSARMYHAMMER_` prefix:

```bash
export SWISSARMYHAMMER_BASE_BRANCH="develop"
export SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX="feature/"
export SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH="8"
```

## Configuration Precedence

Configuration values are loaded in the following order:

1. **YAML file**: Values from `swissarmyhammer.yaml` (if present)
2. **Environment variables**: `SWISSARMYHAMMER_*` environment variables
3. **Defaults**: Built-in default values

Later sources override earlier ones, so YAML configuration takes precedence over environment variables, which take precedence over defaults.

## CLI Configuration Management

For complete CLI command reference, see [CLI Reference](cli-reference.md#configuration-commands).

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

For additional configuration troubleshooting, see the [Troubleshooting Guide](troubleshooting.md#configuration-issues).