# Workflow System Test Plan

## Overview
This document outlines a comprehensive test plan for the Swiss Army Hammer workflow system. The workflow system allows users to define and execute complex automation sequences using Mermaid state diagrams and action descriptions.

## Test Scope

### 1. Workflow Parser Tests
Tests for parsing Mermaid state diagrams and converting them to internal workflow representations.

#### 1.1 Basic Parsing
- **Test Case**: Parse simple state diagram with linear flow
  - Input: Basic Mermaid diagram with states A → B → C
  - Expected: Workflow with 3 states and 2 transitions
  
- **Test Case**: Parse workflow with initial and terminal states
  - Input: Diagram with [*] → Start → End → [*]
  - Expected: Initial state set correctly, terminal state marked

#### 1.2 Complex State Diagrams
- **Test Case**: Parse fork/join states
  - Input: Diagram with <<fork>> and <<join>> annotations
  - Expected: Fork and Join state types identified, parallel execution enabled

- **Test Case**: Parse nested workflows
  - Input: Diagram with substates and concurrent regions
  - Expected: Proper nesting and parallel execution flags

#### 1.3 Action Parsing
- **Test Case**: Extract actions from markdown
  - Input: Markdown with ## Actions section
  - Expected: Actions mapped to states correctly

- **Test Case**: Parse various action formats
  - Input: Different action syntaxes (Execute prompt, Log, Set, Wait)
  - Expected: Actions parsed into correct types

#### 1.4 Error Handling
- **Test Case**: Invalid diagram syntax
  - Input: Malformed Mermaid syntax
  - Expected: ParseError with descriptive message

- **Test Case**: Missing initial state
  - Input: Diagram without [*] → State transition
  - Expected: NoInitialState error

- **Test Case**: Unreachable states
  - Input: States not connected to main flow
  - Expected: InvalidStructure error

### 2. Action Parser Tests
Tests for parsing individual action descriptions.

#### 2.1 Prompt Actions
- **Test Case**: Basic prompt execution
  - Input: `Execute prompt "hello-world"`
  - Expected: PromptAction with correct name

- **Test Case**: Prompt with arguments
  - Input: `Execute prompt "analyze" with file="test.rs" verbose="true"`
  - Expected: Arguments parsed correctly

- **Test Case**: Prompt with result variable
  - Input: `Execute prompt "get-data" with result="data"`
  - Expected: Result variable set

#### 2.2 Wait Actions
- **Test Case**: Duration wait
  - Input: `Wait 30 seconds`, `Wait 5 minutes`, `Wait 1 hour`
  - Expected: Correct Duration values

- **Test Case**: User input wait
  - Input: `Wait for user input`
  - Expected: WaitAction with no duration

#### 2.3 Log Actions
- **Test Case**: Log levels
  - Input: `Log "message"`, `Log warning "message"`, `Log error "message"`
  - Expected: Correct LogLevel variants

#### 2.4 Variable Actions
- **Test Case**: Set variable
  - Input: `Set var_name="value"`
  - Expected: SetVariableAction with name and value

- **Test Case**: Variable substitution
  - Input: `Set output="${result}"`
  - Expected: Variable reference preserved

#### 2.5 Sub-workflow Actions
- **Test Case**: Run workflow
  - Input: `Run workflow "sub-flow"`
  - Expected: SubWorkflowAction created

- **Test Case**: Delegate with inputs
  - Input: `Delegate to "process" with data="${input}"`
  - Expected: Input variables mapped

### 3. Workflow Executor Tests
Tests for workflow execution engine.

#### 3.1 Basic Execution
- **Test Case**: Linear workflow execution
  - Setup: Simple A → B → C workflow
  - Expected: States executed in order, workflow completes

- **Test Case**: Terminal state handling
  - Setup: Workflow with terminal states
  - Expected: Execution stops at terminal state

#### 3.2 Transitions
- **Test Case**: Conditional transitions
  - Setup: Transitions with OnSuccess/OnFailure conditions
  - Expected: Correct branch taken based on action result

- **Test Case**: Custom condition evaluation
  - Setup: CEL expressions in transitions
  - Expected: Expressions evaluated against context

#### 3.3 Error Handling
- **Test Case**: Action failure handling
  - Setup: Action that throws error
  - Expected: OnFailure transition taken or workflow fails

- **Test Case**: Retry behavior
  - Setup: Action with retry configuration
  - Expected: Retries with exponential backoff

- **Test Case**: Rate limit handling
  - Setup: Action that returns rate limit error
  - Expected: Automatic retry after wait period

#### 3.4 Context Management
- **Test Case**: Variable persistence
  - Setup: Set variables in one state, use in another
  - Expected: Variables available throughout execution

- **Test Case**: Result variable storage
  - Setup: Prompt with result variable
  - Expected: Result stored in context

### 4. Integration Tests
End-to-end tests with real workflows.

#### 4.1 CLI Integration
- **Test Case**: Run workflow command
  - Command: `swissarmyhammer flow run hello-world`
  - Expected: Workflow executes successfully

- **Test Case**: Workflow with variables
  - Command: `swissarmyhammer flow run workflow --var key=value`
  - Expected: Variables available in workflow

- **Test Case**: Dry run mode
  - Command: `swissarmyhammer flow run workflow --dry-run`
  - Expected: Execution plan shown, no actual execution

- **Test Case**: Test mode
  - Command: `swissarmyhammer flow run workflow --test`
  - Expected: Mocked execution with coverage report

#### 4.2 Workflow Lifecycle
- **Test Case**: Pause and resume
  - Setup: Long-running workflow
  - Expected: Can pause and resume from saved state

- **Test Case**: Timeout handling
  - Setup: Workflow with timeout
  - Expected: Execution stops after timeout

### 5. Performance Tests
Tests for workflow system performance.

#### 5.1 Large Workflows
- **Test Case**: Many states
  - Setup: Workflow with 100+ states
  - Expected: Reasonable parse and execution time

- **Test Case**: Deep nesting
  - Setup: Deeply nested sub-workflows
  - Expected: No stack overflow, proper execution

#### 5.2 Concurrent Execution
- **Test Case**: Parallel states
  - Setup: Fork/join with parallel branches
  - Expected: Branches execute concurrently

### 6. Storage Tests
Tests for workflow persistence.

#### 6.1 File System Storage
- **Test Case**: Load builtin workflows
  - Setup: Default installation
  - Expected: All builtin workflows loadable

- **Test Case**: Custom workflow directory
  - Setup: User-defined workflow directory
  - Expected: Workflows loaded from custom path

#### 6.2 Workflow Run Storage
- **Test Case**: Save run state
  - Setup: Workflow execution
  - Expected: Run state persisted correctly

- **Test Case**: Resume from saved state
  - Setup: Saved workflow run
  - Expected: Execution continues from saved point

## Test Implementation Strategy

### Unit Tests
- Location: `swissarmyhammer/src/workflow/*/tests.rs`
- Framework: Rust's built-in test framework
- Coverage target: 80%+

### Integration Tests
- Location: `swissarmyhammer-cli/tests/`
- Framework: Rust integration tests with Command
- Focus: CLI commands and end-to-end scenarios

### Test Workflows
Create test-specific workflows in `workflows/test/`:
- `test-linear.md`: Simple linear flow
- `test-branching.md`: Conditional branching
- `test-parallel.md`: Fork/join parallelism
- `test-error-handling.md`: Error scenarios
- `test-sub-workflows.md`: Nested workflows

### Mocking Strategy
- Mock Claude responses for prompt actions
- Mock external system calls
- Provide test-specific action implementations

## Success Criteria
- All unit tests pass
- Integration tests cover major use cases
- Performance benchmarks meet targets
- Error messages are clear and actionable
- Documentation examples work as expected