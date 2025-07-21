# Integrate YAML Configuration with Existing Config System

## Overview
Integrate YAML configuration loading into the existing Config::new() method, implementing the precedence hierarchy: YAML file > environment variables > defaults.

## Context
With YAML loading capabilities in place, this step integrates the new system with the existing environment variable configuration. Following best practices from other CLI tools, YAML configuration should take precedence over environment variables, which take precedence over defaults.

## Requirements
- Modify Config::new() to load YAML configuration first
- Apply YAML values with highest precedence
- Maintain environment variable support as fallback
- Keep default values as final fallback
- Handle YAML loading errors gracefully without breaking the application
- Add configuration source tracking for debugging

## Implementation Details

### Updated Config::new() Method
Modify the existing Config::new() implementation:
```rust
impl Config {
    /// Create a new configuration instance with values loaded from:
    /// 1. YAML file (highest precedence)
    /// 2. Environment variables
    /// 3. Defaults (lowest precedence)
    pub fn new() -> Self {
        // Start with defaults
        let mut config = Self::default();
        
        // Apply environment variables (override defaults)
        config.apply_env_vars();
        
        // Apply YAML configuration (override env vars and defaults)
        match YamlConfig::load_or_default() {
            Ok(yaml_config) => {
                yaml_config.apply_to_config(&mut config);
                tracing::info!("Configuration loaded successfully with YAML support");
            }
            Err(e) => {
                tracing::warn!("Failed to load YAML configuration, falling back to env vars and defaults: {}", e);
                // Continue with environment variables and defaults
            }
        }
        
        config
    }
    
    /// Apply environment variable configuration to this config
    fn apply_env_vars(&mut self) {
        let loader = EnvLoader::new("SWISSARMYHAMMER");
        
        self.issue_branch_prefix = loader.load_string("ISSUE_BRANCH_PREFIX", &self.issue_branch_prefix);
        self.issue_number_width = loader.load_parsed("ISSUE_NUMBER_WIDTH", self.issue_number_width);
        self.max_pending_issues_in_summary = loader.load_parsed("MAX_PENDING_ISSUES_IN_SUMMARY", self.max_pending_issues_in_summary);
        // ... apply all existing environment variables
        self.base_branch = loader.load_string("BASE_BRANCH", &self.base_branch);
    }
}
```

### Configuration Source Tracking
Add optional configuration source tracking for debugging:
```rust
#[derive(Debug, Clone)]
pub enum ConfigSource {
    Default,
    EnvironmentVariable,
    YamlFile(std::path::PathBuf),
}

// Add to Config struct for debugging (optional)
#[cfg(debug_assertions)]
pub struct Config {
    // ... existing fields
    pub _source_info: std::collections::HashMap<String, ConfigSource>,
}
```

### Error Recovery Strategy
Implement graceful degradation:
- Log YAML parsing errors but continue with env vars and defaults
- Only fail completely for critical system errors
- Provide clear guidance on fixing YAML syntax errors

## Acceptance Criteria
- [ ] Config::new() attempts to load YAML configuration first
- [ ] YAML values override environment variables when present
- [ ] Environment variables override defaults when YAML not present
- [ ] YAML parsing errors don't crash the application
- [ ] Clear logging shows configuration source precedence
- [ ] All existing functionality continues to work
- [ ] Integration tests verify precedence hierarchy
- [ ] Performance impact is minimal
- [ ] Code compiles without warnings

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Test Cases
- YAML + env vars + defaults: YAML takes precedence
- No YAML + env vars + defaults: env vars take precedence  
- No YAML + no env vars + defaults: defaults used
- Invalid YAML + env vars + defaults: falls back to env vars
- Missing YAML file: falls back to env vars and defaults
- Empty YAML file: env vars and defaults used appropriately

## Dependencies
- Existing EnvLoader functionality
- YamlConfig from previous steps
- Error handling from previous steps

## Notes
This step completes the core integration, making the new YAML configuration system fully functional while maintaining backward compatibility. The graceful error handling ensures that configuration issues don't prevent the application from starting.