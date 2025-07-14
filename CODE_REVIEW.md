# Code Review TODO List

## Errors and Warnings to Fix

### 1. [x] Cargo warning: Multiple binaries using same main.rs
**Warning**: `/Users/wballard/github/swissarmyhammer/swissarmyhammer-cli/Cargo.toml: file /Users/wballard/github/swissarmyhammer/swissarmyhammer-cli/src/main.rs` found to be present in multiple build targets:
- `bin` target `sah`
- `bin` target `swissarmyhammer`

**Issue**: Both binaries are pointing to the same main.rs file, which causes Cargo to emit a warning.
**Resolution**: This is intentional behavior implemented in issue #000141 to create a short alias `sah` for the `swissarmyhammer` CLI. The warning is informational only and doesn't affect functionality. Both binaries share the same code intentionally.

### 2. [x] Failing test: test_validate_command_loads_same_workflows_as_flow_list
**Error**: Test panicked at `swissarmyhammer-cli/src/validate.rs:1790:49`
```
called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Issue**: The test is trying to access a file or directory that doesn't exist.
**Fix**: Investigate the test to understand what file/directory it expects and ensure proper test setup.