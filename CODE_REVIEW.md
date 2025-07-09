# Code Review - doctor/mod.rs

## Issues to Address

### Critical Issues

- [ ] **Duplication**: The `eprintln!` on line 843 violates coding standard - DO NOT use print statements for warnings/logs in library code, only in CLI main
- [ ] **Incomplete Error Handling**: The `check_disk_space` function on non-Unix systems returns hardcoded values (1000, 10000) instead of implementing actual disk space checking
- [ ] **String Duplication**: Format strings like "Workflow directory permissions: {:?}" are duplicated multiple times - violates DRY principle
- [ ] **Primitive Type Usage**: Using raw `u64` for disk space instead of a proper type like `DiskSpace { mb: u64 }` violates data structure standards

### Code Quality Issues

- [ ] **Magic Numbers**: `LOW_DISK_SPACE_MB` constant is 100 but no justification for this value - should be configurable or better documented
- [ ] **Inconsistent Error Messages**: Some checks use "Failed to..." while others use "Cannot..." - standardize error message format
- [ ] **Missing Type Safety**: `get_workflow_directories()` returns tuples instead of a proper struct - should be `WorkflowDirectoryInfo { path: WorkflowDirectory, category: WorkflowCategory }`
- [ ] **Test File Cleanup**: Line 841 uses `eprintln!` for test file cleanup failure - should use proper logging or Result handling

### Pattern Violations

- [ ] **Function Length**: Several functions exceed 120 lines (e.g., `print_results`, `check_workflow_permissions`) - refactor into smaller functions
- [ ] **Hardcoded Paths**: Using `.swissarmyhammer` as a string literal in multiple places - should be a constant
- [ ] **Platform-Specific Code**: `#[cfg(unix)]` and `#[cfg(not(unix))]` blocks duplicate logic - extract common behavior

### Test Coverage

- [ ] **Missing Tests**: No tests for disk space checking functions
- [ ] **Mock Usage**: Tests don't verify actual behavior, just that functions don't crash
- [ ] **Error Path Testing**: No tests for error conditions in workflow checks

### Documentation

- [ ] **Missing Module Documentation**: No module-level documentation explaining the purpose of the doctor diagnostics
- [ ] **Incomplete Function Documentation**: Many public functions lack proper doc comments
- [ ] **No Examples**: Public API functions should include usage examples

### Performance

- [ ] **Redundant Directory Walks**: Multiple `WalkDir` iterations over the same directories - could be consolidated
- [ ] **Inefficient String Operations**: Using `format!` in loops when `write!` to a buffer would be more efficient

### Security

- [ ] **Path Traversal**: No validation that workflow directories don't contain path traversal sequences
- [ ] **Permission Check Race Condition**: Checking permissions then using the directory creates a TOCTOU vulnerability

### Refactoring Opportunities

- [ ] **Extract Check Creation**: Pattern of creating `Check` objects is repeated - extract to builder or factory method
- [ ] **Consolidate Directory Checks**: Similar logic for checking prompts and workflows - extract common behavior
- [ ] **Type-Safe Exit Codes**: Using raw integers (0, 1, 2) for exit codes - create an enum

## Summary

The refactoring improves code organization with constants and the `WorkflowDirectory` type, but introduces several issues that violate our coding standards. Primary concerns are code duplication, missing type safety, and incomplete error handling. The workflow diagnostics functionality is comprehensive but needs refinement to meet our quality standards.