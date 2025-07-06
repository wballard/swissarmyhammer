---
name: test-unit
title: Generate Unit Tests
description: Create comprehensive unit tests for code with good coverage
arguments:
  - name: framework
    description: Testing framework to use (e.g., jest, pytest, junit, cargo test)
    required: false
    default: "auto-detect"
  - name: style
    description: Testing style (BDD, TDD, classical)
    required: false
    default: "TDD"
  - name: coverage_target
    description: Target test coverage percentage
    required: false
    default: "85"
---

# Unit Test Generation

{% render code %}

## Test Configuration
- **Framework**: {{framework}}
- **Style**: {{style}}
- **Coverage Target**: {{coverage_target}}%

## Test Strategy

### 1. Test Structure
Based on {{style}} style:
- Descriptive test names
- Clear arrange-act-assert pattern
- Proper test isolation
- Meaningful assertions

### 2. Test Coverage Areas

#### Happy Path Tests
- Normal expected behavior
- Valid inputs
- Successful operations
- Expected return values

#### Edge Cases
- Boundary values
- Empty inputs
- Maximum/minimum values
- Special characters

#### Error Scenarios
- Invalid inputs
- Null/undefined handling
- Exception cases
- Error propagation

#### State Testing
- Initial state
- State transitions
- State persistence
- Concurrent access (if applicable)

### 3. Test Implementation

#### Setup and Teardown
- Test fixtures
- Mock dependencies
- Database transactions
- Clean state between tests

#### Assertions
- Value equality
- Type checking
- Error validation
- Side effect verification

#### Mocking Strategy
- External dependencies
- Time-based operations
- Random values
- Network calls

### 4. Best Practices

#### Test Quality
- One assertion per test
- Descriptive failure messages
- Fast execution
- Deterministic results

#### Maintainability
- DRY principle in test code
- Helper functions
- Clear test data
- Documentation

#### Coverage
- Line coverage
- Branch coverage
- Path coverage
- Edge case coverage

### 5. Example Test Structure
Provide complete test file with:
- Imports and setup
- Test suites/classes
- Individual test cases
- Helper functions
- Cleanup code