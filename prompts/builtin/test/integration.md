---
name: test-integration
title: Generate Integration Tests
description: Create integration tests to verify component interactions
arguments:
  - name: system_description
    description: Description of the system/components to test
    required: true
  - name: test_scenarios
    description: Specific scenarios to test (comma-separated)
    required: false
    default: "basic flow"
  - name: framework
    description: Testing framework to use
    required: false
    default: "auto-detect"
  - name: environment
    description: Test environment setup requirements
    required: false
    default: "local"
---

# Integration Test Generation

## System Overview
{{system_description}}

## Test Scenarios
{{test_scenarios}}

## Test Environment
{{environment}}

## Integration Test Strategy

### 1. Test Scope
Define boundaries and interactions:
- Component boundaries
- External dependencies
- Data flow between components
- Integration points

### 2. Test Scenarios

#### End-to-End Flows
- Complete user journeys
- Multi-step processes
- Cross-component transactions
- Data consistency verification

#### Component Integration
- API contract testing
- Message passing
- Shared state management
- Event propagation

#### External Systems
- Database interactions
- Third-party APIs
- Message queues
- File systems

### 3. Test Environment Setup

#### Infrastructure
- Docker containers
- Test databases
- Mock services
- Network configuration

#### Test Data
- Seed data scripts
- Test user accounts
- Sample datasets
- Reset procedures

#### Configuration
- Environment variables
- Service endpoints
- Authentication tokens
- Feature flags

### 4. Test Implementation

#### Test Structure
{% if framework == "jest" or framework == "auto-detect" %}
```javascript
describe('Integration: {{system_description}}', () => {
  beforeAll(async () => {
    // Setup test environment
    // Initialize services
    // Seed test data
  });

  afterAll(async () => {
    // Cleanup
    // Reset state
  });

  test('{{test_scenarios}}', async () => {
    // Arrange
    // Act
    // Assert
  });
});
```
{% elsif framework == "mocha" %}
```javascript
describe('Integration: {{system_description}}', function() {
  before(async function() {
    // Setup test environment
  });

  after(async function() {
    // Cleanup
  });

  it('should handle {{test_scenarios}}', async function() {
    // Arrange
    // Act
    // Assert
  });
});
```
{% elsif framework == "pytest" %}
```python
import pytest

class TestIntegration:
    @pytest.fixture(scope="class")
    def setup_environment(self):
        # Setup test environment
        yield
        # Cleanup

    def test_{{test_scenarios | replace: " ", "_" | replace: ",", "_" | downcase}}(self, setup_environment):
        # Arrange
        # Act
        # Assert
        pass
```
{% else %}
```
// {{framework}} test structure
describe('Integration: {{system_description}}', () => {
  beforeAll(async () => {
    // Setup test environment
  });

  test('{{test_scenarios}}', async () => {
    // Arrange
    // Act  
    // Assert
  });
});
```
{% endif %}

#### Verification Points
- Response validation
- Data persistence
- Side effects
- Error handling
- Performance metrics

### 5. Best Practices

#### Reliability
- Idempotent tests
- Proper cleanup
- Retry mechanisms
- Timeout handling

#### Debugging
- Detailed logging
- Request/response capture
- State snapshots
- Error screenshots

#### Performance
- Parallel execution where possible
- Shared setup optimization
- Resource pooling
- Efficient assertions

### 6. Common Patterns
- API testing patterns
- Database testing strategies
- Async operation handling
- Error simulation
- Transaction rollback