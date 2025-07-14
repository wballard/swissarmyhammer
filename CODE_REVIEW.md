# Code Review TODO List

## Business Logic Migration from CLI to Library (Issue #000143)

### 1. [ ] Create Validation Module in Library
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

### 2. [ ] Move Template Rendering with Environment Variables to Library
**Location**: `swissarmyhammer-cli/src/test.rs`
**Issue**: Environment variable support for template rendering is in CLI
**Action**:
- Add `render_with_env()` method to library's `Template` or `TemplateEngine`
- Move interactive argument collection logic as a utility
- Support environment variable interpolation in templates

### 3. [ ] Move Workflow Execution Utilities to Library
**Location**: `swissarmyhammer-cli/src/flow.rs`
**Issue**: Common workflow execution patterns are implemented in CLI
**Action**:
- Move variable parsing utilities to library
- Move interactive variable collection to library
- Move execution timeout handling to library
- Move test mode execution support to library
- Add methods: `parse_variables()`, `collect_variables_interactive()`

## Code Quality Issues

### 4. [ ] Fix Code Duplication Between CLI and Library
**Issue**: `PromptSource` enum and conversion logic is duplicated
**Action**:
- Remove duplicate `PromptSource` enum from CLI
- Use library's `PromptSource` directly in CLI
- Simplify conversion logic between CLI and library types

### 5. [ ] Add Integration Tests
**Issue**: Missing integration tests between CLI and library
**Action**:
- Create integration tests that verify CLI correctly uses library functionality
- Test end-to-end scenarios for filtering, searching, and validation

### 6. [ ] Enhance Documentation
**Issue**: Documentation could be more comprehensive
**Action**:
- Add module-level documentation for new library modules
- Add usage examples in documentation
- Document the separation of concerns between CLI and library

### 7. [ ] Fix Remaining Test Failures
**Issue**: Some tests may be failing due to refactoring
**Action**:
- Run `cargo test` and fix any failing tests
- Update tests that depend on moved functionality
- Ensure all tests pass in both CLI and library

## Architecture Improvements

### 8. [ ] Consider Creating a Facade Pattern for CLI
**Issue**: CLI modules directly use multiple library modules
**Action**:
- Consider creating a high-level API in the library that simplifies CLI usage
- Reduce coupling between CLI and library internals

### 9. [ ] Review Error Handling Consistency
**Issue**: Error handling patterns may differ between CLI and library
**Action**:
- Ensure consistent error types and handling
- Propagate library errors properly to CLI
- Provide helpful error messages to users

## New Code Quality Issues (2025-07-14)

### 10. [ ] Extract Common Argument Validation Logic
**Location**: `test.rs:342-350`, `prompts.rs:298-376`
**Issue**: Argument validation logic is duplicated across multiple render methods
**Action**:
- Create a shared trait or module for argument validation
- Consolidate validation logic from `render_prompt`, `render_prompt_with_env`, and Prompt methods
- Reduce code duplication in argument processing

### 11. [ ] Fix Performance Issues with Regular Expressions
**Location**: `template.rs:217-224`
**Issue**: Regular expressions are compiled on every call to `extract_template_variables`
**Action**:
- Use `lazy_static` or `once_cell` for compiled regular expressions
- Cache regex patterns that are used repeatedly
- Profile template parsing performance after optimization

### 12. [ ] Split Large Validation Module
**Location**: `validate.rs` (1945 lines)
**Issue**: The validate.rs file is too large and handles multiple concerns
**Action**:
- Split into separate modules: prompt_validator.rs, workflow_validator.rs, content_validator.rs
- Create clear module boundaries and interfaces
- Improve code organization and maintainability

### 13. [ ] Add Missing Tests for New Environment Variable Features
**Location**: `test.rs`, `prompts.rs`, `template.rs`
**Issue**: New `render_with_env` methods lack test coverage
**Action**:
- Add tests for `render_prompt_with_env` in test.rs
- Add tests for `render_with_partials_and_env` in prompts.rs
- Add tests for environment variable edge cases and error handling
- Test environment variable precedence and override behavior

### 14. [ ] Replace Hard-coded Values with Configuration
**Location**: Multiple files
**Issue**: Magic numbers and strings are hard-coded throughout
**Action**:
- Extract hard-coded emojis (test.rs:131) to constants
- Make validation thresholds configurable (validate.rs:321)
- Create named constants for line limits and complexity thresholds
- Move source location strings to enums or constants

### 15. [ ] Improve Error Context and Messages
**Location**: Throughout codebase
**Issue**: Error messages lack context about operations and files
**Action**:
- Add file paths to validation error messages
- Include operation context in error types
- Make validation suggestions more actionable
- Provide better error recovery hints

### 16. [ ] Add Input Validation for File Paths
**Location**: `test.rs:298`
**Issue**: User-provided file paths are used without validation
**Action**:
- Validate file paths before use
- Check for path traversal attempts
- Ensure paths are within expected directories
- Add proper error handling for invalid paths

### 17. [ ] Create Abstraction for Partial Resolution
**Location**: `template.rs:99-181`
**Issue**: Partial resolution logic could be more modular
**Action**:
- Extract partial resolution into a `PartialResolver` trait
- Allow custom partial resolution strategies
- Improve testability of partial resolution logic

### 18. [ ] Optimize Directory Scanning Performance
**Location**: `prompts.rs:1004`
**Issue**: WalkDir usage without filtering could be slow
**Action**:
- Add file extension filtering to WalkDir
- Implement parallel directory scanning for large trees
- Add progress reporting for long operations
- Consider caching directory scan results

### 19. [ ] Consolidate Metadata Parsing Logic
**Location**: `prompts.rs:1025-1219`
**Issue**: Duplicate metadata parsing in `load_file_with_base` and `load_from_string`
**Action**:
- Extract metadata parsing into a shared function
- Reduce code duplication in YAML parsing
- Improve error handling for malformed metadata

### 20. [ ] Implement Builder Pattern for Validation Configuration
**Location**: `validation/mod.rs`
**Issue**: Complex validation pipelines are hard to configure
**Action**:
- Create a ValidationConfigBuilder
- Allow fluent configuration of validators
- Simplify validation setup for common use cases
- Add preset configurations for typical scenarios