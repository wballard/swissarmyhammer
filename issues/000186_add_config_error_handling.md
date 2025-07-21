# Add Configuration Error Handling and Validation

## Overview
Enhance the configuration system with comprehensive error handling, validation, and user-friendly error messages for common configuration issues.

## Context
Building on the integrated YAML configuration system, this step focuses on providing excellent user experience when configuration issues occur. Following the specification's requirement for "clear error messages for invalid YAML", we need robust error handling and validation.

## Requirements
- Add configuration validation for all settings
- Provide helpful error messages with suggestions for fixes
- Add validation for base_branch format (no invalid characters, not empty)
- Handle edge cases gracefully (permissions, file locks, etc.)
- Add configuration file schema validation
- Implement configuration testing/validation CLI command support

## Implementation Details

### Enhanced Error Types
Expand the ConfigError enum with more specific error variants:
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file {path}: {source}")]
    FileRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Invalid YAML syntax in {path}:\n{source}\n\nHint: Check for proper indentation and YAML formatting")]
    YamlParse {
        path: std::path::PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    
    #[error("Invalid configuration value for '{field}': {value}\n{hint}")]
    InvalidValue {
        field: String,
        value: String,
        hint: String,
    },
    
    #[error("Configuration validation failed: {message}")]
    Validation { message: String },
}
```

### Configuration Validation
Add validation methods to Config:
```rust
impl Config {
    /// Validate the current configuration settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate base_branch
        self.validate_base_branch()?;
        
        // Validate numeric ranges
        self.validate_numeric_ranges()?;
        
        // Validate string lengths
        self.validate_string_lengths()?;
        
        Ok(())
    }
    
    fn validate_base_branch(&self) -> Result<(), ConfigError> {
        if self.base_branch.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: "base_branch".to_string(),
                value: self.base_branch.clone(),
                hint: "base_branch cannot be empty. Use 'main' or 'develop' or another valid git branch name".to_string(),
            });
        }
        
        // Check for invalid git branch characters
        let invalid_chars = ['~', '^', ':', '?', '*', '[', '\\', ' '];
        for ch in invalid_chars.iter() {
            if self.base_branch.contains(*ch) {
                return Err(ConfigError::InvalidValue {
                    field: "base_branch".to_string(),
                    value: self.base_branch.clone(),
                    hint: format!("base_branch contains invalid character '{}'. Git branch names cannot contain: ~ ^ : ? * [ \\ <space>", ch),
                });
            }
        }
        
        Ok(())
    }
    
    fn validate_numeric_ranges(&self) -> Result<(), ConfigError> {
        if self.issue_number_width == 0 {
            return Err(ConfigError::InvalidValue {
                field: "issue_number_width".to_string(),
                value: self.issue_number_width.to_string(),
                hint: "issue_number_width must be greater than 0".to_string(),
            });
        }
        
        if self.min_issue_number >= self.max_issue_number {
            return Err(ConfigError::Validation {
                message: format!(
                    "min_issue_number ({}) must be less than max_issue_number ({})",
                    self.min_issue_number, self.max_issue_number
                ),
            });
        }
        
        Ok(())
    }
    
    fn validate_string_lengths(&self) -> Result<(), ConfigError> {
        if self.issue_branch_prefix.len() > 50 {
            return Err(ConfigError::InvalidValue {
                field: "issue_branch_prefix".to_string(),
                value: self.issue_branch_prefix.clone(),
                hint: "issue_branch_prefix cannot exceed 50 characters".to_string(),
            });
        }
        
        Ok(())
    }
}
```

### Enhanced YAML Loading with Validation
Update YAML loading to include validation:
```rust
impl YamlConfig {
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        // ... existing loading code ...
        
        // Validate the loaded configuration
        let config = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::YamlParse {
                path: path.to_path_buf(),
                source: e,
            })?;
            
        // Apply basic validation to YAML values before returning
        config.validate_yaml_values()?;
        
        tracing::info!("Successfully loaded and validated YAML configuration: {:?}", config);
        Ok(config)
    }
}

impl YamlConfig {
    fn validate_yaml_values(&self) -> Result<(), ConfigError> {
        if let Some(ref base_branch) = self.base_branch {
            if base_branch.is_empty() {
                return Err(ConfigError::InvalidValue {
                    field: "base_branch".to_string(),
                    value: base_branch.clone(),
                    hint: "base_branch cannot be empty in YAML configuration".to_string(),
                });
            }
        }
        Ok(())
    }
}
```

### Configuration Help and Examples
Add helper methods for configuration guidance:
```rust
impl Config {
    /// Generate an example YAML configuration file content
    pub fn example_yaml_config() -> &'static str {
        r#"# swissarmyhammer.yaml
# Configuration file for Swiss Army Hammer

# Base branch that pull requests will merge into
base_branch: "main"
"#
    }
    
    /// Get configuration validation help message
    pub fn validation_help() -> &'static str {
        r#"Configuration Validation Help:

- base_branch: Must be a valid git branch name (no spaces, special characters)
- All numeric values must be positive and within reasonable ranges
- String values must not exceed maximum lengths

For more help, see: https://github.com/wballard/swissarmyhammer/docs/configuration
"#
    }
}
```

## Acceptance Criteria
- [ ] Comprehensive validation for all configuration fields
- [ ] Clear, helpful error messages with suggestions
- [ ] base_branch validation prevents invalid git branch names
- [ ] Numeric range validation prevents invalid values
- [ ] YAML parsing errors include helpful formatting hints
- [ ] Configuration validation can be run independently
- [ ] Example configuration generation works
- [ ] All error scenarios have unit tests
- [ ] Integration with existing error handling
- [ ] Performance impact is minimal

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Test Cases
- Invalid base_branch names (empty, with spaces, with special chars)
- Numeric values out of range (0, negative, excessive)
- String values too long
- Valid configurations pass validation
- YAML syntax errors provide helpful messages
- File permission errors handled gracefully

## Dependencies
- thiserror (already available)
- Existing error handling patterns

## Notes
This step significantly improves user experience by providing clear guidance when configuration issues occur. The validation prevents runtime errors and guides users toward correct configuration.