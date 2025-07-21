# Add YAML Configuration File Discovery

## Overview
Implement logic to discover `swissarmyhammer.yaml` configuration files in the repository root directory.

## Context
Following patterns from successful Rust CLI tools, we need a robust file discovery mechanism that can locate the configuration file in the appropriate location. This step focuses purely on file discovery without yet loading the content.

## Requirements
- Find `swissarmyhammer.yaml` in the current working directory (repository root)
- Handle cases where the file doesn't exist gracefully
- Use proper path handling that works cross-platform
- Add logging for configuration file discovery
- Return Option<PathBuf> for found configuration files

## Implementation Details

### Configuration File Discovery
Add a utility function to find the configuration file:
```rust
impl Config {
    /// Find the swissarmyhammer.yaml configuration file in the current directory
    /// Returns Some(path) if found, None if not found
    fn find_yaml_config_file() -> Option<std::path::PathBuf> {
        use std::path::Path;
        
        let config_path = Path::new("swissarmyhammer.yaml");
        
        if config_path.exists() && config_path.is_file() {
            tracing::debug!("Found configuration file: {:?}", config_path);
            Some(config_path.to_path_buf())
        } else {
            tracing::debug!("No swissarmyhammer.yaml configuration file found in current directory");
            None
        }
    }
}
```

### Error Handling
Handle potential IO errors when checking file existence:
- Use proper error handling for file system operations
- Log discovery status at appropriate levels
- Ensure method is robust to permission errors

### Testing Strategy
Add unit tests to verify:
- File found when present
- None returned when absent
- Proper handling of edge cases (directories named swissarmyhammer.yaml, etc.)

## Acceptance Criteria
- [ ] find_yaml_config_file() method exists and works correctly
- [ ] Method returns Option<PathBuf> as specified
- [ ] Proper logging of discovery status
- [ ] Cross-platform path handling
- [ ] Unit tests verify all scenarios
- [ ] No file loading is performed (just discovery)
- [ ] Code compiles without warnings

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Dependencies
- std::path
- tracing (already in use)

## Test Cases
- File exists and is readable → Some(path)
- File doesn't exist → None
- Path exists but is directory → None
- Permission denied on file → None (graceful handling)

## Notes
This step keeps file discovery separate from file loading for better testability and separation of concerns. The next step will handle actually reading and parsing the YAML content.