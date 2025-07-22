# Implement Comprehensive Memoranda Testing

## Overview
Create a complete test suite for all memoranda functionality, ensuring reliability and correctness across storage, MCP, CLI, and search operations.

## Tasks

### 1. Unit Tests
- **Storage Tests** (`swissarmyhammer/src/memoranda/storage.rs`):
  - CRUD operations with various data sizes
  - Directory creation and permission handling
  - File corruption recovery
  - Error condition handling
  - Concurrent access testing
- **Data Structure Tests** (extend existing in `mod.rs`):
  - Serialization edge cases
  - Invalid data handling
  - ULID generation and validation

### 2. Integration Tests  
- **MCP Integration Tests** (`swissarmyhammer/tests/mcp_memoranda_tests.rs`):
  - End-to-end MCP tool testing
  - Error response formatting
  - Large memo handling
  - Concurrent MCP requests
- **CLI Integration Tests** (`swissarmyhammer-cli/tests/memo_cli_tests.rs`):
  - All CLI commands working correctly
  - Stdin/stdout handling
  - Error exit codes
  - Command completion

### 3. Performance Tests
- **Storage Performance** (`swissarmyhammer/benches/memo_benchmarks.rs`):
  - Benchmark memo creation/retrieval with large collections
  - Search performance with various query types
  - Memory usage patterns
- **MCP Performance Tests**:
  - Response time testing for all MCP operations
  - Large context retrieval performance

### 4. Edge Case Testing
- **File System Edge Cases**:
  - No write permissions
  - Disk full scenarios  
  - Corrupted memo files
  - Very large memo content (approaching 1MB limit)
- **Search Edge Cases**:
  - Empty search queries
  - Special characters in search terms
  - Very long search queries
  - Unicode content search

### 5. Mock and Test Utilities
- Create mock storage implementation for testing
- Test data generation utilities
- Helper functions for integration test setup
- Temporary directory management for tests

### 6. Property-Based Testing
- Use `proptest` crate for property-based testing:
  - Round-trip serialization properties
  - Search result consistency
  - ULID uniqueness properties
  - Storage operation invariants

## Test Coverage Goals
- **Unit tests**: >95% code coverage
- **Integration tests**: All user-facing operations
- **Performance tests**: Baseline performance metrics established
- **Edge cases**: All known failure modes tested

## Test Categories
```rust
// Example test structure
mod storage_tests {
    mod unit_tests { /* ... */ }
    mod integration_tests { /* ... */ }
    mod performance_tests { /* ... */ }
    mod edge_case_tests { /* ... */ }
}

mod mcp_tests { /* ... */ }
mod cli_tests { /* ... */ }  
mod search_tests { /* ... */ }
```

## Implementation Notes
- Follow existing test patterns from issues and other modules
- Use appropriate test fixtures and helper utilities
- Ensure tests are deterministic and can run in parallel
- Include both positive and negative test cases

## Acceptance Criteria
- [ ] All core functionality covered by unit tests
- [ ] MCP integration tests passing
- [ ] CLI integration tests passing  
- [ ] Performance benchmarks established
- [ ] Edge case tests covering known failure modes
- [ ] All tests passing in CI/CD pipeline
- [ ] Test coverage reports showing >90% coverage