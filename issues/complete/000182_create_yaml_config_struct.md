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

## Proposed Solution

Based on examination of the existing codebase, I will implement the YamlConfig struct as specified in the requirements:

1. **Add necessary imports**: Import serde's Deserialize trait at the top of config.rs
2. **Create YamlConfig struct**: Implement the struct with proper serde attributes and documentation
3. **Implement Default trait**: Provide a sensible default implementation that returns None for all optional fields
4. **Add apply_to_config method**: Create a method that applies YAML configuration values to an existing Config instance
5. **Write comprehensive tests**: Create unit tests to verify:
   - Default implementation returns expected values
   - apply_to_config correctly updates Config fields when values are Some(_)
   - apply_to_config preserves existing Config values when YAML fields are None
   - YAML deserialization works correctly with serde_yaml

The implementation will follow the existing patterns in the codebase, maintaining consistency with the current Config struct design and testing approach.