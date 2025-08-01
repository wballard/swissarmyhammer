# SwissArmyHammer Error Handling and Resilience Patterns

## Hierarchical Error Design

**Comprehensive Error Taxonomy**
```rust
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SwissArmyHammerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Git operation '{operation}' failed: {details}")]
    GitOperationFailed { operation: String, details: String },
    // ... specialized variants for each domain
}
```

**Domain-Specific Error Types**
- `WorkflowError`: State machine and execution errors
- `ActionError`: Workflow action execution failures
- `ParseError`: Syntax and validation errors
- `ValidationError`: Content and structure validation
- `StorageError`: Backend storage operations
- `McpError`: Model Context Protocol communication
- `ConfigError`: Configuration and environment issues

## Error Context and Chaining

**Context Extension Pattern**
```rust
pub trait ErrorContext<T> {
    fn context<S: Into<String>>(self, msg: S) -> Result<T>;
    fn with_context<F, S>(self, f: F) -> Result<T>;
}
```

**Error Chain Formatting**
- `ErrorChain` struct for detailed error reporting
- Recursive source error traversal
- Structured display with indentation levels

**Rich Error Information**
- Structured fields in error variants (operation, path, details)
- Helper functions for consistent error creation
- Standardized error message formats

## Special Error Handling: ABORT ERROR Pattern

**Critical Failure Pattern**
```rust
impl SwissArmyHammerError {
    pub const ABORT_ERROR_PREFIX: &'static str = "ABORT ERROR";
    
    pub fn is_abort_error(&self) -> bool {
        // Pattern detection for immediate termination
    }
    
    pub fn cannot_switch_issue_to_issue() -> Self {
        // Enforces branching workflow rules
    }
}
```

**Use Cases for ABORT ERROR**
- User policy violations that shouldn't be recovered from
- Operations leaving system in inconsistent state
- Workflow rules that must be strictly enforced

## Resilience and Recovery Patterns

**Graceful Degradation**
- Non-fatal errors logged but don't crash application
- Fallback mechanisms for missing resources
- Default values when configuration is incomplete

**Resource Cleanup**
- RAII pattern with custom `Drop` implementations
- `ProcessGuard` for automatic process termination
- `TestHomeGuard` for test environment isolation
- Automatic file handle cleanup

**Retry Mechanisms**
- Rate limiting with configurable backoff
- Exponential backoff for external service calls
- Circuit breaker pattern for MCP connections

## Error Propagation Strategy

**Consistent Result Types**
- `Result<T>` type alias throughout codebase
- `?` operator for clean error propagation
- Context addition at appropriate abstraction levels

**Logging Integration**
- Structured logging with `tracing` crate
- Error context preserved in log messages
- Different log levels based on error severity

**Exit Code Strategy**
```rust
const EXIT_SUCCESS: i32 = 0;   // Successful execution
const EXIT_WARNING: i32 = 1;   // General error or warnings
const EXIT_ERROR: i32 = 2;     // Validation errors or critical failures
```

## Testing Error Conditions

**Comprehensive Error Testing**
- Error propagation validation
- Error message content assertions
- Recovery mechanism verification
- Resource cleanup testing

**Mock Error Scenarios**
- Simulated I/O failures
- Network timeout simulation
- Invalid input validation
- State corruption scenarios

This error handling strategy provides robust failure handling while maintaining system reliability and providing clear diagnostic information for debugging and monitoring.