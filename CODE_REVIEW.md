# Code Review TODO List

## Issue #000146 - State Name Pollution in Nested Workflows

### 1. [ ] Fix Formatting Issues
**Location**: Multiple files
**Issue**: Code doesn't pass `cargo fmt --check`
**Action**:
- Run `cargo fmt` to fix formatting in:
  - `swissarmyhammer/src/workflow/actions.rs:1186`
  - `swissarmyhammer/src/workflow/actions_tests/common.rs:34`
  - `swissarmyhammer/src/workflow/actions_tests/simple_state_pollution_test.rs:1`
  - `swissarmyhammer/src/workflow/actions_tests/sub_workflow_action_tests.rs:42`
  - `swissarmyhammer/src/workflow/actions_tests/sub_workflow_state_pollution_tests.rs:62-182`
  - `swissarmyhammer-cli/src/validate.rs:1971-1988`
  - `swissarmyhammer-cli/src/flow.rs:1`
  - `swissarmyhammer-cli/src/main.rs:174`
  - `swissarmyhammer-cli/tests/mcp_server_shutdown_test.rs:11-96`

### 2. [ ] Fix Clippy Warnings
**Location**: `swissarmyhammer/src/workflow/actions_tests/`
**Issue**: Unused imports and dead code warnings
**Action**:
- Remove unused import `crate::workflow::actions_tests::common::*` from `sub_workflow_action_tests.rs:4`
- Remove unused import `WorkflowStorage` from `sub_workflow_state_pollution_tests.rs:6`
- Either use or mark as `#[allow(dead_code)]` the functions in `common.rs`:
  - `create_test_context()` at line 7
  - `create_context_with_special_chars()` at line 27

### 3. [ ] Improve Test Documentation
**Location**: `swissarmyhammer/src/workflow/actions_tests/sub_workflow_state_pollution_tests.rs`
**Issue**: Tests have debugging eprintln! statements that should be removed or converted to proper logging
**Action**:
- Remove or convert `eprintln!` statements to proper tracing/logging
- Add more descriptive test documentation explaining what each test validates

### 4. [ ] Add Error Handling for Missing Workflows
**Location**: `swissarmyhammer/src/workflow/actions_tests/sub_workflow_state_pollution_tests.rs`
**Issue**: Tests create workflows in temp directories but don't handle potential file I/O errors gracefully
**Action**:
- Add proper error handling for file operations
- Consider using `Result` types in test setup functions
- Add cleanup in case of test failures

### 5. [ ] Consolidate Test Utilities
**Location**: `swissarmyhammer/src/workflow/actions_tests/mod.rs` and `common.rs`
**Issue**: Test utilities are duplicated between mod.rs and common.rs
**Action**:
- Consolidate `create_test_context()` functions
- Remove duplication between files
- Create a clear separation of concerns

### 6. [ ] Add Integration Test for Actual State Pollution Issue
**Location**: New test file needed
**Issue**: While unit tests exist, there's no integration test that demonstrates the actual issue from the bug report
**Action**:
- Create an integration test that reproduces the exact scenario from issue #000146
- Test should verify that actions are executed on the correct workflow
- Test should verify that state names don't collide between parent and child workflows

### 7. [ ] Improve Logging Context
**Location**: `swissarmyhammer/src/workflow/executor/core.rs`
**Issue**: While workflow name was added to logs, consider adding more context
**Action**:
- Consider adding workflow run ID to logs for better traceability
- Consider adding parent workflow context when executing sub-workflows
- Ensure consistent logging format across all workflow execution points

### 8. [ ] Add More Comprehensive Sub-workflow Tests
**Location**: `swissarmyhammer/src/workflow/actions_tests/sub_workflow_state_pollution_tests.rs`
**Issue**: Tests could be more comprehensive
**Action**:
- Add test for deeply nested workflows (3+ levels)
- Add test for parallel sub-workflow execution
- Add test for sub-workflows that fail and ensure parent state is preserved
- Add test for circular dependency detection

### 9. [ ] Document State Isolation Implementation
**Location**: `swissarmyhammer/src/workflow/actions.rs`
**Issue**: The state isolation implementation details are not well documented
**Action**:
- Add documentation explaining how state isolation works
- Document the context separation between parent and child workflows
- Add inline comments explaining the workflow stack mechanism

### 10. [ ] Consider Adding State Namespace Support
**Location**: `swissarmyhammer/src/workflow/`
**Issue**: Current solution relies on execution context separation
**Action**:
- Consider implementing explicit namespace support for states
- This would allow states to be prefixed with workflow names internally
- Would provide stronger guarantees against state name collisions

### 11. [ ] Fix Test Brittleness
**Location**: `swissarmyhammer/src/workflow/actions_tests/sub_workflow_state_pollution_tests.rs`
**Issue**: Tests change working directory which could affect other tests if run in parallel
**Action**:
- Refactor tests to avoid changing global state (current directory)
- Use absolute paths instead of changing directories
- Ensure tests are properly isolated

### 12. [ ] Add Performance Tests
**Location**: New test file needed
**Issue**: No performance tests for nested workflow execution
**Action**:
- Add benchmarks for nested workflow execution
- Test performance impact of the state isolation mechanism
- Ensure no significant performance regression

### 13. [ ] Update Changelog and Documentation
**Location**: Project documentation
**Issue**: The fix for state pollution should be documented
**Action**:
- Update CHANGELOG.md with the fix details
- Update any workflow documentation to explain state isolation
- Add examples showing nested workflows with same state names