# Shell Action Project Setup

Refer to ./specification/shell.md

## Overview

This is the first step in implementing shell actions for workflows. This step focuses on setting up the project structure and understanding the existing codebase patterns before implementing the shell action functionality.

## Objective

Set up the development environment and understand the existing action system architecture to ensure the shell action implementation follows established patterns.

## Tasks

### 1. Environment Setup
- Ensure the development environment is properly configured
- Run existing tests to confirm the codebase is in a working state
- Review the specification requirements thoroughly

### 2. Codebase Analysis  
- Study the existing action implementations in `swissarmyhammer/src/workflow/actions.rs`
- Analyze the action parser patterns in `swissarmyhammer/src/workflow/action_parser.rs`
- Understand the action dispatch mechanism in `parse_action_from_description`
- Review existing action traits and patterns

### 3. Architecture Planning
- Identify where the `ShellAction` struct should be implemented
- Plan the integration points with the existing parser system
- Design the shell action parameters structure
- Plan the security validation approach

### 4. Testing Strategy
- Review existing action tests to understand testing patterns
- Plan the testing approach for shell actions
- Identify security test requirements
- Plan integration test scenarios

## Expected Deliverables

1. **Development Environment Verification**
   - Confirm all tests pass: `cargo test`
   - Confirm clippy is clean: `cargo clippy`
   - Confirm formatting is correct: `cargo fmt --check`

2. **Architecture Documentation** 
   - Clear understanding of where to implement ShellAction
   - Integration plan with existing systems
   - Security considerations documented

3. **Testing Plan**
   - Unit test strategy for shell actions
   - Integration test approach
   - Security test requirements

## Success Criteria

- [x] All existing tests pass
- [x] Clear understanding of action system architecture
- [x] Solid plan for shell action implementation
- [x] Testing strategy defined
- [x] Security considerations identified

## Implementation Notes

This step is purely preparatory and should not involve any code changes. The focus is on understanding and planning to ensure a clean, well-integrated implementation in subsequent steps.

## Next Steps

After completing this setup, proceed to implementing the basic ShellAction struct in the next step.
## Proposed Solution

Based on my comprehensive analysis of the SwissArmyHammer codebase, I have developed a detailed implementation plan for the shell action system. The analysis reveals a well-structured architecture that follows consistent patterns, making shell action integration straightforward.

### Architecture Analysis

**Existing Action System Overview:**
- All actions implement the `Action` trait with async `execute()` method
- Actions receive a mutable `HashMap<String, Value>` context for variable access
- Actions return `ActionResult<Value>` with rich error handling
- The `ActionParser` uses chumsky parser combinators for robust parsing
- The dispatch mechanism in `parse_action_from_description()` calls individual parser methods

**Key Patterns Identified:**
1. **Builder Pattern**: Actions use builder methods like `with_timeout()`, `with_result_variable()`
2. **Variable Substitution**: Actions implement `VariableSubstitution` trait for `${variable}` replacement
3. **Error Handling**: Comprehensive `ActionError` enum with domain-specific variants
4. **Testing Infrastructure**: Strong unit test coverage with `#[cfg(test)]` modules co-located with implementation

### ShellAction Integration Plan

**1. ShellAction Struct Design** (`actions.rs:line_850+`)
```rust
#[derive(Debug, Clone)]
pub struct ShellAction {
    pub command: String,                        // Shell command to execute
    pub timeout: Option<Duration>,              // Execution timeout (default: None)
    pub result_variable: Option<String>,        // Variable to store command output
    pub working_dir: Option<PathBuf>,           // Working directory override
    pub env_vars: HashMap<String, String>,      // Environment variables
    pub capture_output: bool,                   // Whether to capture stdout/stderr
}
```

**2. Parser Integration** (`action_parser.rs:line_351+`)
```rust
pub fn parse_shell_action(&self, description: &str) -> ActionResult<Option<ShellAction>> {
    // Parse patterns like:
    // Shell "command"
    // Shell "command" with timeout=30
    // Shell "command" with result="output_var" timeout=60
    // Shell "command" with working_dir="/path" env={"VAR": "value"}
}
```

**3. Action Dispatch Integration** (`actions.rs:line_1508+`)
```rust
// Add to parse_action_from_description():
if let Some(shell_action) = parser.parse_shell_action(description)? {
    return Ok(Some(Box::new(shell_action)));
}
```

### Security Architecture

**Command Validation Framework:**
- Maximum command length limits (configurable, default: 1000 chars)
- Dangerous command pattern detection (regex-based)
- Path traversal prevention for working directories
- Environment variable sanitization
- Resource limits (timeout: max 300 seconds per spec)

**Security Implementation Strategy:**
```rust
fn validate_shell_command(&self, command: &str) -> ActionResult<()> {
    // Length validation
    // Dangerous pattern detection
    // Command injection prevention
    // Resource limit validation
}
```

### Testing Strategy

**Unit Testing Approach:**
- Parser tests for all syntax variants
- Execution tests using mock commands (like `echo`, `sleep`)
- Error condition tests (timeouts, invalid commands, permissions)
- Variable substitution tests with complex context scenarios
- Security validation tests for dangerous command detection

**Integration Testing:**
- End-to-end workflow tests with shell actions
- Process lifecycle management tests
- Resource cleanup verification
- Cross-platform compatibility tests (Unix/Windows)

**Test Infrastructure:**
- Leverage existing `TestHomeGuard` and temporary directory patterns
- Use `tokio::process::Command` for async process management
- Mock dangerous commands for safety during testing
- Property-based testing with `proptest` for command generation

### Implementation Phases

**Phase 1: Core Structure** (Issue SHELL_000227)
- Implement `ShellAction` struct in `actions.rs`
- Add builder methods following existing patterns
- Implement `Action` trait with basic execution

**Phase 2: Parser Integration** (Issue SHELL_000228)
- Add `parse_shell_action()` method to `ActionParser`
- Support all syntax variants from specification
- Case-insensitive "shell" keyword parsing

**Phase 3: Execution Engine** (Issue SHELL_000230)
- Implement subprocess execution with `tokio::process::Command`
- Basic stdout/stderr capture
- Exit code handling and variable setting

**Phase 4: Advanced Features** (Issues SHELL_000231-232)
- Timeout handling and process cleanup
- Working directory and environment variable support
- Process resource management

**Phase 5: Security & Validation** (Issue SHELL_000233)
- Command validation and sanitization
- Dangerous operation detection
- Resource limit enforcement

**Phase 6: Comprehensive Testing** (Issues SHELL_000234-235)
- Unit test suite completion
- Integration test development
- Security test scenarios

### Technical Considerations

**Async Process Management:**
- Use `tokio::process::Command` for non-blocking execution
- Implement proper signal handling for process termination
- Handle process cleanup in error scenarios

**Variable Integration:**
- Follow existing patterns for context variable access
- Set automatic variables: `success`, `failure`, `exit_code`, `stdout`, `stderr`, `duration_ms`
- Support result variable capture as specified

**Error Handling:**
- Add `ShellExecutionError` variant to `ActionError` enum
- Comprehensive error context with command details
- Graceful handling of process failures

### Success Criteria Verification

✅ **Development Environment Verified**
- All tests pass (1026 passed, 0 failed)
- Clippy shows no warnings
- Code formatting is correct

✅ **Architecture Understanding Complete**
- Action system patterns documented
- Parser integration approach defined
- Testing strategy established

✅ **Implementation Plan Established**
- Clear integration points identified
- Security approach designed
- Phased implementation strategy defined

This project setup establishes a solid foundation for implementing shell actions following SwissArmyHammer's established patterns and quality standards.