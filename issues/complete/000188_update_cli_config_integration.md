# Update CLI Configuration Integration

## Overview
Update the CLI layer to properly integrate with the new YAML configuration system, ensuring all CLI commands can access and use the updated configuration.

## Context
With the YAML configuration system implemented in the library layer, this step ensures the CLI layer (`swissarmyhammer-cli`) properly integrates with and uses the new configuration capabilities. The CLI needs to handle configuration errors gracefully and provide helpful feedback to users.

## Requirements
- Update CLI initialization to use the new Config::new() method
- Add CLI commands for configuration management
- Integrate configuration validation with CLI error handling
- Add CLI support for generating example configuration files
- Ensure all CLI commands can access base_branch configuration
- Add configuration debugging/info commands

## Implementation Details

### CLI Configuration Integration
Update the main CLI initialization to use the enhanced configuration system:
```rust
// In swissarmyhammer-cli/src/main.rs or relevant CLI initialization
use swissarmyhammer::config::{Config, ConfigError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize configuration with enhanced error handling
    let config = match Config::new() {
        Ok(config) => {
            // Validate configuration
            if let Err(e) = config.validate() {
                eprintln!("❌ Configuration validation failed: {}", e);
                eprintln!("\n{}", Config::validation_help());
                std::process::exit(1);
            }
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            eprintln!("Falling back to default configuration...");
            Config::default()
        }
    };

    // Make config available to CLI commands
    run_cli_with_config(config)
}
```

### Configuration Management CLI Commands
Add new CLI subcommands for configuration management:
```rust
// In CLI args structure
#[derive(Parser, Debug)]
pub enum Commands {
    // ... existing commands ...
    
    /// Configuration management commands
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration values and sources
    Show,
    /// Validate current configuration
    Validate,
    /// Generate example configuration file
    Init,
    /// Show configuration help and documentation
    Help,
}
```

### Configuration Command Implementation
```rust
// In swissarmyhammer-cli/src/cli.rs or new config.rs module
pub fn handle_config_command(action: ConfigAction, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigAction::Show => {
            println!("📋 Current Configuration:");
            println!("base_branch: {}", config.base_branch);
            println!("issue_branch_prefix: {}", config.issue_branch_prefix);
            println!("issue_number_width: {}", config.issue_number_width);
            // ... show all configuration values
            
            // Show configuration sources if in debug mode
            #[cfg(debug_assertions)]
            if let Some(source_info) = config._source_info.as_ref() {
                println!("\n📍 Configuration Sources:");
                for (key, source) in source_info {
                    println!("  {}: {:?}", key, source);
                }
            }
        }
        
        ConfigAction::Validate => {
            match config.validate() {
                Ok(()) => {
                    println!("✅ Configuration is valid");
                }
                Err(e) => {
                    println!("❌ Configuration validation failed: {}", e);
                    println!("\n{}", Config::validation_help());
                    std::process::exit(1);
                }
            }
        }
        
        ConfigAction::Init => {
            let config_path = "swissarmyhammer.yaml";
            
            if std::path::Path::new(config_path).exists() {
                eprintln!("❌ Configuration file already exists: {}", config_path);
                eprintln!("Remove it first or use a different location.");
                std::process::exit(1);
            }
            
            std::fs::write(config_path, Config::example_yaml_config())?;
            println!("✅ Created example configuration file: {}", config_path);
            println!("Edit this file to customize your configuration.");
        }
        
        ConfigAction::Help => {
            println!("📖 Configuration Help\n");
            println!("SwissArmyHammer supports configuration via:");
            println!("  1. YAML file (swissarmyhammer.yaml) - highest precedence");
            println!("  2. Environment variables (SWISSARMYHAMMER_*) - medium precedence");
            println!("  3. Built-in defaults - lowest precedence");
            println!("\nExample configuration file:");
            println!("{}", Config::example_yaml_config());
            println!("{}", Config::validation_help());
        }
    }
    
    Ok(())
}
```

### Update Existing CLI Commands
Ensure existing CLI commands can access the base_branch configuration:
```rust
// In commands that need base_branch access
pub fn create_pull_request(config: &Config, /* other args */) -> Result<(), Box<dyn std::error::Error>> {
    let base_branch = &config.base_branch;
    println!("Creating PR targeting base branch: {}", base_branch);
    // ... rest of implementation
    Ok(())
}
```

### Enhanced Error Handling for Configuration
Add configuration-specific error handling throughout the CLI:
```rust
// Error handling patterns for CLI commands
pub fn handle_config_error(error: &ConfigError) {
    match error {
        ConfigError::YamlParse { path, source } => {
            eprintln!("❌ Invalid YAML in configuration file: {}", path.display());
            eprintln!("   YAML Error: {}", source);
            eprintln!("   Hint: Check file formatting and indentation");
            eprintln!("   Run 'swissarmyhammer config validate' for more details");
        }
        ConfigError::InvalidValue { field, value, hint } => {
            eprintln!("❌ Invalid configuration value for '{}': {}", field, value);
            eprintln!("   Hint: {}", hint);
        }
        ConfigError::FileRead { path, source } => {
            eprintln!("❌ Cannot read configuration file: {}", path.display());
            eprintln!("   Error: {}", source);
        }
        ConfigError::Validation { message } => {
            eprintln!("❌ Configuration validation error: {}", message);
        }
    }
}
```

### Configuration in Doctor Command
Update the doctor command to include configuration checks:
```rust
// In swissarmyhammer-cli/src/doctor/checks.rs
pub fn check_configuration(config: &Config) -> CheckResult {
    // Check if configuration is valid
    match config.validate() {
        Ok(()) => CheckResult::success("Configuration validation passed"),
        Err(e) => CheckResult::error(&format!("Configuration validation failed: {}", e)),
    }
}

pub fn check_configuration_file() -> CheckResult {
    match Config::find_yaml_config_file() {
        Some(path) => {
            match YamlConfig::load_from_file(&path) {
                Ok(_) => CheckResult::success(&format!("Configuration file loaded successfully: {}", path.display())),
                Err(e) => CheckResult::error(&format!("Configuration file error: {}", e)),
            }
        }
        None => CheckResult::info("No configuration file found (using defaults and environment variables)"),
    }
}
```

## Acceptance Criteria
- [ ] CLI properly initializes with new configuration system
- [ ] Configuration validation errors are handled gracefully
- [ ] New 'config' CLI command provides all management functions
- [ ] All existing CLI commands can access base_branch setting
- [ ] Configuration errors provide helpful guidance to users
- [ ] Doctor command includes configuration health checks
- [ ] CLI startup performance is not significantly impacted
- [ ] Error messages are user-friendly and actionable
- [ ] Integration tests verify CLI configuration behavior
- [ ] CLI help documentation updated for new config features

## Files to Modify
- `swissarmyhammer-cli/src/main.rs`
- `swissarmyhammer-cli/src/cli.rs`
- `swissarmyhammer-cli/src/doctor/checks.rs`
- New file: `swissarmyhammer-cli/src/config_commands.rs` (optional)

## Dependencies
- Updated swissarmyhammer library with new config system
- clap (for CLI argument parsing)
- Existing error handling patterns

## Test Cases
- CLI starts successfully with valid configuration
- CLI starts with fallback when configuration invalid
- 'config show' displays current configuration
- 'config validate' reports validation status
- 'config init' creates example configuration file
- 'config help' provides comprehensive guidance
- Doctor command reports configuration status
- All existing CLI functionality works with new config

## Notes
This step completes the integration by making the YAML configuration system fully accessible through the CLI interface. Users can now manage their configuration entirely through CLI commands while maintaining full backward compatibility.

## Proposed Solution

After analyzing the codebase, I can see that:

1. **Configuration System Ready**: The `swissarmyhammer/src/config.rs` module already provides a comprehensive configuration system with:
   - `Config::new()` method for loading configuration from YAML, env vars, and defaults
   - `Config::validate()` method for validation
   - `Config::example_yaml_config()` and `Config::validation_help()` helper methods
   - Full error handling with `ConfigError` enum
   - YAML file discovery and loading through `YamlConfig`

2. **Current CLI State**: The CLI in `swissarmyhammer-cli/src/cli.rs` has these commands:
   - Serve, Doctor, Prompt, Flow, Completion, Validate, Issue
   - **Missing**: Config command (this is what we need to add)

3. **Main.rs Integration**: Current `main.rs` doesn't use `Config::new()` - it handles logging/server initialization directly

### Implementation Plan:

1. **Update CLI Structure**: Add `Config` command to `Commands` enum in `cli.rs` with subcommands: Show, Validate, Init, Help

2. **Update Main.rs**: 
   - Initialize configuration with `Config::new()` early in main function
   - Pass config to command handlers that need it
   - Handle configuration errors gracefully with fallback to defaults

3. **Add Config Command Handler**: Create new config command handler that calls existing config methods

4. **Update Doctor**: Add configuration health checks to doctor using existing config validation methods

5. **Pass Config to Commands**: Update command handlers to accept and use the config (especially for base_branch access)

This leverages the existing robust configuration system and integrates it cleanly with the CLI without duplicating functionality.