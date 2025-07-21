# Create YAML Configuration Struct

## Overview
Create a YamlConfig struct that can deserialize YAML configuration from `swissarmyhammer.yaml` files.

## Context
Building on the base_branch field added in the previous step, we now need a separate struct to handle YAML deserialization. This follows the pattern used by other Rust CLI tools where the configuration file schema may differ from the internal config representation.

## Requirements
- Create YamlConfig struct with serde Deserialize derive
- Include base_branch field as the initial supported option
- Add conversion from YamlConfig to Config fields
- Use Option<T> for all fields to allow partial configuration
- Add comprehensive documentation

## Implementation Details

### YamlConfig Struct
Create a new struct specifically for YAML deserialization:
```rust
/// Configuration loaded from swissarmyhammer.yaml file
#[derive(Debug, Clone, Deserialize)]
pub struct YamlConfig {
    /// Base branch for pull requests
    pub base_branch: Option<String>,
}
```

### Default Implementation
```rust
impl Default for YamlConfig {
    fn default() -> Self {
        Self {
            base_branch: None,
        }
    }
}
```

### Integration Helper
Add method to apply YAML config to existing Config:
```rust
impl YamlConfig {
    /// Apply YAML configuration values to an existing Config
    /// YAML values take precedence over existing values
    pub fn apply_to_config(&self, config: &mut Config) {
        if let Some(ref base_branch) = self.base_branch {
            config.base_branch = base_branch.clone();
        }
    }
}
```

### Error Handling
Ensure proper error handling for YAML deserialization errors by using appropriate Result types.

## Acceptance Criteria
- [ ] YamlConfig struct exists with proper serde attributes
- [ ] base_branch field is Option<String> type
- [ ] apply_to_config method correctly updates Config instance
- [ ] Default implementation provides empty configuration
- [ ] Struct has comprehensive documentation
- [ ] Code compiles without warnings
- [ ] Unit tests verify struct behavior

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Dependencies
- serde (already in Cargo.toml)
- serde_yaml (already in Cargo.toml)

## Notes
This step establishes the foundation for YAML configuration without yet implementing file loading. The Option-based approach allows users to specify only the settings they want to override.