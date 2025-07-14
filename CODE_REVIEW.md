# Code Review TODO List

## Business Logic Migration from CLI to Library (Issue #000143)

### 1. [X] Move Advanced Search Logic from CLI to Library
**Location**: `swissarmyhammer-cli/src/search.rs:49-189`
**Issue**: Significant search logic remains in CLI including regex search, case-sensitive search, argument filtering, score calculation, and excerpt generation
**Action**: 
- Move regex search implementation to library's `SearchEngine`
- Move case-sensitive search option to library
- Move argument filtering logic (has_arg, no_args) to library
- Move score calculation logic to library
- Move excerpt generation with highlighting to library
- Add methods: `regex_search()`, `search_with_options()`, `generate_excerpt()`

### 2. [ ] Create Validation Module in Library
**Location**: `swissarmyhammer-cli/src/validate.rs:113-1107`
**Issue**: Entire validation framework exists in CLI but should be in library
**Action**:
- Create `swissarmyhammer/src/validation/` module
- Move `ContentValidator` trait to library
- Move all validator implementations (EncodingValidator, LineEndingValidator, YamlTypoValidator, etc.)
- Move Liquid syntax validation logic
- Move variable usage validation
- Move workflow validation logic
- Create proper module structure with `mod.rs`, `validators.rs`, `prompt_validator.rs`, `workflow_validator.rs`

### 3. [ ] Move Template Rendering with Environment Variables to Library
**Location**: `swissarmyhammer-cli/src/test.rs`
**Issue**: Environment variable support for template rendering is in CLI
**Action**:
- Add `render_with_env()` method to library's `Template` or `TemplateEngine`
- Move interactive argument collection logic as a utility
- Support environment variable interpolation in templates

### 4. [ ] Move Workflow Execution Utilities to Library
**Location**: `swissarmyhammer-cli/src/flow.rs`
**Issue**: Common workflow execution patterns are implemented in CLI
**Action**:
- Move variable parsing utilities to library
- Move interactive variable collection to library
- Move execution timeout handling to library
- Move test mode execution support to library
- Add methods: `parse_variables()`, `collect_variables_interactive()`

## Code Quality Issues

### 5. [ ] Fix Code Duplication Between CLI and Library
**Issue**: `PromptSource` enum and conversion logic is duplicated
**Action**:
- Remove duplicate `PromptSource` enum from CLI
- Use library's `PromptSource` directly in CLI
- Simplify conversion logic between CLI and library types

### 6. [ ] Add Integration Tests
**Issue**: Missing integration tests between CLI and library
**Action**:
- Create integration tests that verify CLI correctly uses library functionality
- Test end-to-end scenarios for filtering, searching, and validation

### 7. [ ] Enhance Documentation
**Issue**: Documentation could be more comprehensive
**Action**:
- Add module-level documentation for new library modules
- Add usage examples in documentation
- Document the separation of concerns between CLI and library

### 8. [ ] Fix Remaining Test Failures
**Issue**: Some tests may be failing due to refactoring
**Action**:
- Run `cargo test` and fix any failing tests
- Update tests that depend on moved functionality
- Ensure all tests pass in both CLI and library

## Architecture Improvements

### 9. [ ] Consider Creating a Facade Pattern for CLI
**Issue**: CLI modules directly use multiple library modules
**Action**:
- Consider creating a high-level API in the library that simplifies CLI usage
- Reduce coupling between CLI and library internals

### 10. [ ] Review Error Handling Consistency
**Issue**: Error handling patterns may differ between CLI and library
**Action**:
- Ensure consistent error types and handling
- Propagate library errors properly to CLI
- Provide helpful error messages to users

## Linting Issues (2025-07-14)

### 11. [X] Fix Rust Formatting Issues
**Location**: Multiple files need formatting
**Issue**: cargo fmt check found formatting issues
**Action**:
- `swissarmyhammer/src/prompt_filter.rs`: Fix line formatting (lines 58, 131, 138, 142, 162, 178, 246, 270)
- `swissarmyhammer-cli/src/list.rs`: Remove unnecessary blank lines (lines 41, 51, 58)
**Status**: FIXED - Ran `cargo fmt` to automatically format all files

### 12. [X] Fix Duplicate Binary Target Warning
**Location**: `/Users/wballard/github/swissarmyhammer/swissarmyhammer-cli/Cargo.toml`
**Issue**: File `src/main.rs` is present in multiple build targets (`sah` and `swissarmyhammer`)
**Action**:
- Review Cargo.toml binary target configuration
- Decide if both binaries are needed or consolidate to one
**Status**: NO ACTION NEEDED - This is intentional. `sah` is an alias for `swissarmyhammer` (added in commit 01cb414). The warning is harmless.