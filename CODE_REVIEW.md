# Code Review TODO List

## Issue #000147 - Abort Error Mechanism for Workflows

### 1. [ ] Fix Formatting Issues
**Location**: Multiple files
**Issue**: Code doesn't pass `cargo fmt --check`
**Action**:
- Run `cargo fmt` to fix formatting in:
  - `swissarmyhammer/src/workflow/actions.rs:1228` - Remove trailing whitespace
  - `swissarmyhammer/src/workflow/actions_tests/prompt_action_tests.rs:25,94,104,114` - Fix line breaks
  - `swissarmyhammer/src/workflow/executor/core.rs:668` - Remove blank lines
  - `swissarmyhammer-cli/src/validate.rs` - Multiple formatting issues with imports and function signatures

### 2. [ ] Fix Dead Code Warning
**Location**: `swissarmyhammer-cli/src/validate.rs:1270`
**Issue**: Function `run_validate_command` is never used
**Action**:
- Either remove the unused function or add `#[allow(dead_code)]` if it's intended for future use
- Investigate why this function exists but isn't called

### 3. [ ] Add Integration Tests for Abort Error
**Location**: New test file needed
**Issue**: While unit tests exist, there's no integration test demonstrating the full abort error flow
**Action**:
- Create integration test that shows abort error propagating from prompt to workflow exit
- Test abort error in single workflow scenario
- Test abort error in nested workflow scenario (sub-workflow aborts, parent should exit)
- Test multiple levels of nesting (3+ levels deep)

### 4. [ ] Improve SubWorkflow Abort Error Detection
**Location**: `swissarmyhammer/src/workflow/actions.rs:1216-1229`
**Issue**: Abort error detection in sub-workflows relies on string matching in context
**Action**:
- Consider a more robust mechanism for propagating abort errors from sub-workflows
- Perhaps use a dedicated field or error type in WorkflowRun
- Current implementation checks context["result"] for "ABORT ERROR:" prefix which is fragile

### 5. [ ] Add Edge Case Tests for Abort Error Pattern
**Location**: `swissarmyhammer/src/workflow/actions_tests/prompt_action_tests.rs`
**Issue**: Tests don't cover all edge cases
**Action**:
- Add test for empty message after "ABORT ERROR:"
- Add test for very long error messages
- Add test for special characters in error message
- Add test for Unicode characters in error message

### 6. [ ] Enhance Documentation with More Examples
**Location**: `doc/src/workflows.md`
**Issue**: Documentation could be more comprehensive
**Action**:
- Add example showing nested workflow abort propagation
- Add example of abort error in a loop or retry scenario
- Document what happens to workflow state when abort occurs
- Document interaction with compensation actions

### 7. [ ] Add Logging Context for Abort Errors
**Location**: `swissarmyhammer/src/workflow/executor/core.rs:668`
**Issue**: Abort errors could use more context in logs
**Action**:
- Log which action triggered the abort
- Log the current state when abort occurred
- Consider adding workflow path (parent -> child -> grandchild) in nested scenarios

### 8. [ ] Test Concurrent Sub-Workflows with Abort
**Location**: New test needed
**Issue**: No tests for parallel sub-workflows where one aborts
**Action**:
- Create test with multiple parallel sub-workflows
- Test scenario where one sub-workflow aborts while others are running
- Verify all sub-workflows are properly terminated

### 9. [ ] Add Performance Benchmark
**Location**: New benchmark needed
**Issue**: No performance assessment of abort error checking
**Action**:
- Add benchmark comparing prompt execution with and without abort check
- Ensure the string prefix check doesn't significantly impact performance
- Consider regex pre-compilation if pattern becomes more complex

### 10. [ ] Consider Abort Error Configuration
**Location**: Design consideration
**Issue**: "ABORT ERROR:" prefix is hardcoded
**Action**:
- Consider making the abort pattern configurable
- Could allow different abort patterns for different use cases
- Maintain backward compatibility with default pattern

### 11. [ ] Validate Error Message Extraction
**Location**: `swissarmyhammer/src/workflow/actions.rs:745-749`
**Issue**: Error message extraction could be more robust
**Action**:
- Handle case where "ABORT ERROR:" appears multiple times
- Consider trimming more whitespace variants (tabs, multiple spaces)
- Add validation that extracted message is not empty

### 12. [ ] Document Abort Error Best Practices
**Location**: `doc/src/workflows.md`
**Issue**: Missing guidance on when to use abort errors
**Action**:
- Add section on abort error best practices
- Document anti-patterns (e.g., using abort for normal flow control)
- Provide guidelines on error message formatting
- Explain difference between abort errors and regular failures