# Implement YAML Configuration File Loading

## Overview
Implement the logic to load and parse YAML configuration files using the discovery mechanism from the previous step.

## Context
Building on the YamlConfig struct and file discovery logic, this step implements the actual file reading and YAML parsing. This follows best practices from the rust ecosystem for configuration file handling.

## Requirements
- Load YAML content from discovered configuration files
- Parse YAML into YamlConfig struct using serde_yaml
- Handle YAML parsing errors gracefully with clear error messages
- Return appropriate errors for IO failures vs YAML parsing failures
- Add comprehensive logging for configuration loading process

## Implementation Details

### YAML Loading Function
Add a method to load YAML configuration:
```rust
impl YamlConfig {
    /// Load YAML configuration from a file path
    /// Returns the parsed configuration or an error with context
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        use std::fs;
        
        let path = path.as_ref();
        tracing::info!("Loading YAML configuration from: {:?}", path);
        
        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::FileRead {
                path: path.to_path_buf(),
                source: e,
            })?;
            
        // Parse YAML content
        let config: YamlConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::YamlParse {
                path: path.to_path_buf(),
                source: e,
            })?;
            
        tracing::info!("Successfully loaded YAML configuration: {:?}", config);
        Ok(config)
    }
    
    /// Try to load YAML configuration, returning default if file not found
    pub fn load_or_default() -> Result<Self, ConfigError> {
        match Config::find_yaml_config_file() {
            Some(path) => Self::load_from_file(path),
            None => {
                tracing::debug!("No configuration file found, using default YAML config");
                Ok(Self::default())
            }
        }
    }
}
```

### Error Types
Define proper error types for configuration loading:
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file {path}: {source}")]
    FileRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to parse YAML configuration from {path}: {source}")]
    YamlParse {
        path: std::path::PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
}
```

### Integration with Existing Code
Ensure the error types work well with the existing error handling patterns in the codebase.

## Acceptance Criteria
- [ ] YamlConfig::load_from_file() successfully loads valid YAML files
- [ ] YamlConfig::load_or_default() handles missing files gracefully
- [ ] Clear, helpful error messages for malformed YAML
- [ ] Clear, helpful error messages for IO errors
- [ ] Proper logging at appropriate levels
- [ ] Unit tests cover all error scenarios
- [ ] Unit tests cover successful loading scenarios
- [ ] Integration with find_yaml_config_file() works correctly
- [ ] Code compiles without warnings

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Dependencies
- serde_yaml (already available)
- thiserror (already available)
- std::fs

## Test Cases
- Valid YAML file loads correctly
- Invalid YAML syntax produces clear error
- Non-existent file handled gracefully
- IO permission errors handled gracefully
- Empty YAML file loads as default
- YAML with only some fields populates correctly

## Notes
This step establishes robust YAML loading but doesn't yet integrate with the main Config::new() method. The error handling is designed to be helpful for users debugging their configuration files.