# Add base_branch Configuration Field

## Overview
Add the `base_branch` field to the existing Config struct to support configuring the target branch for pull requests.

## Context
The current Config struct in `swissarmyhammer/src/config.rs` contains various configuration options loaded from environment variables. We need to add support for a `base_branch` setting as specified in `specification/configuration.md`.

## Requirements
- Add `base_branch: String` field to the Config struct
- Set default value to "main" 
- Add environment variable support as `SWISSARMYHAMMER_BASE_BRANCH`
- Update Config::new() to load the new field
- Update Default implementation
- Add field to tests

## Implementation Details

### Config Struct Changes
Add the field to the Config struct:
```rust
pub struct Config {
    // existing fields...
    /// Base branch for pull requests (default: "main")
    pub base_branch: String,
}
```

### Default Implementation
Update the default to include base_branch:
```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            // existing fields...
            base_branch: "main".to_string(),
        }
    }
}
```

### Environment Variable Support
Add to Config::new():
```rust
impl Config {
    pub fn new() -> Self {
        let loader = EnvLoader::new("SWISSARMYHAMMER");
        
        Self {
            // existing fields...
            base_branch: loader.load_string("BASE_BRANCH", "main"),
        }
    }
}
```

### Test Updates
Update existing tests to include the new field and add specific tests for base_branch configuration.

## Acceptance Criteria
- [ ] Config struct contains base_branch field with String type
- [ ] Default value is "main"
- [ ] Environment variable SWISSARMYHAMMER_BASE_BRANCH is supported
- [ ] All existing tests pass
- [ ] New tests verify base_branch configuration works correctly
- [ ] Code compiles without warnings

## Files to Modify
- `swissarmyhammer/src/config.rs`

## Notes
This is a foundational change that will be built upon in subsequent steps to add YAML file configuration support.

## Proposed Solution

I will implement this by following the existing patterns in the Config struct:

1. **Add field to Config struct** - Add `base_branch: String` field with documentation comment following existing patterns
2. **Update Default implementation** - Add `base_branch: "main".to_string()` to match the existing pattern
3. **Update Config::new()** - Add `base_branch: loader.load_string("BASE_BRANCH", "main")` to load from env var with fallback
4. **Update all tests** - Ensure all existing tests include the new field and add specific test coverage for base_branch configuration

The implementation follows the existing patterns exactly - using `load_string` for string fields with sensible defaults, maintaining consistency with the codebase style.