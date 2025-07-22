# Configuration System

## Overview
Implement a configuration system using a `swissarmyhammer.yaml` file that can be placed in repositories where Swiss Army Hammer is running.

## Requirements

### Configuration File Location
- Look for `swissarmyhammer.yaml` in the root of the repository
- If not found, use sensible defaults

### Configuration Options

#### Base Branch Configuration
- **Key**: `base_branch`
- **Type**: string
- **Default**: `"main"`
- **Description**: The base branch that pull requests will merge into
- **Example**:
  ```yaml
  base_branch: "develop"
  ```

### Configuration File Schema
```yaml
# swissarmyhammer.yaml
base_branch: "main"  # The target branch for PRs
```

## Implementation Details

### File Format
- Use YAML format for human readability
- Support comments in the configuration file
- Validate configuration on load

### Error Handling
- Gracefully handle missing configuration file
- Provide clear error messages for invalid YAML
- Fall back to defaults for missing or invalid values
- Log configuration loading status

### Configuration Loading
- Load configuration at startup
- Cache configuration to avoid repeated file reads
- Support configuration reloading if needed

## Acceptance Criteria
- [ ] Configuration file can be placed in repository root
- [ ] `base_branch` can be configured and overrides default "main"
- [ ] Invalid YAML shows helpful error message
- [ ] Missing file uses default values
- [ ] Configuration is loaded and applied correctly
- [ ] Tests cover configuration loading and validation