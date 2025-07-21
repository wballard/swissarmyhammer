# Foundation Integration Tests

## Summary

Implement comprehensive integration tests for the cost tracking foundation (steps 000190-000192). This ensures all core components work together correctly before proceeding to API interception.

## Context

The foundation components (`CostTracker`, `CostCalculator`, configuration system) need thorough integration testing to verify they work together correctly. This step validates the complete foundation before building API interception on top.

## Requirements

### Test Coverage Areas

1. **End-to-End Session Lifecycle**
   - Session creation with configuration
   - API call recording and cost calculation
   - Session completion and cleanup
   - Error handling throughout lifecycle

2. **Configuration Integration**
   - YAML configuration loading and parsing
   - Environment variable overrides
   - Configuration validation and error handling
   - Runtime configuration changes

3. **Cost Calculation Integration**
   - Different pricing models (paid vs max)
   - Various token counts and cost scenarios
   - Precision and accuracy validation
   - Edge cases (zero tokens, large numbers)

4. **Memory Management**
   - Session limits and cleanup
   - Memory usage under load
   - Concurrent session handling
   - Cleanup interval testing

### Integration Test Scenarios

1. **Happy Path Testing**
   - Complete workflow from session start to cost reporting
   - Multiple concurrent sessions
   - Configuration changes during runtime

2. **Error Condition Testing**
   - Invalid configuration handling
   - Session timeout scenarios
   - Memory limit exceeded conditions
   - Malformed API call data

3. **Performance Testing**
   - Large token counts
   - Many concurrent sessions
   - Memory usage validation
   - Cleanup performance

## Implementation Details

### File Location
- Create: `swissarmyhammer/src/cost/integration_tests.rs`
- Test helper utilities in `swissarmyhammer/src/cost/test_utils.rs`

### Test Infrastructure
- Mock API call data generators
- Configuration builders for testing
- Session lifecycle helpers
- Performance measurement utilities

### Test Data
Create realistic test scenarios:
- Typical issue workflow token counts
- Various API call patterns
- Different configuration combinations
- Error and edge case data

## Testing Requirements

### Unit Integration Tests
```rust
#[tokio::test]
async fn test_complete_cost_tracking_workflow() {
    // Test complete session lifecycle with realistic data
}

#[tokio::test]
async fn test_configuration_integration() {
    // Test configuration loading and usage
}

#[tokio::test]
async fn test_concurrent_sessions() {
    // Test multiple simultaneous cost tracking sessions
}
```

### Performance Tests
- Memory usage validation
- Session creation/cleanup timing
- Large dataset handling
- Concurrent access performance

### Error Handling Tests
- Configuration error propagation
- Session timeout handling
- Invalid data recovery
- Resource cleanup verification

## Integration

This step validates:
- Step 000190: Core data structures work correctly
- Step 000191: Cost calculation integrates properly
- Step 000192: Configuration system functions as expected

Prepares for:
- Step 000194: MCP protocol integration
- Step 000195: Token counting implementation

## Success Criteria

- [ ] Complete end-to-end integration tests for cost tracking foundation
- [ ] Configuration integration tests covering all scenarios
- [ ] Performance tests validating memory and timing requirements
- [ ] Error handling tests for all failure modes
- [ ] Concurrent session testing with realistic loads
- [ ] Test utilities and helpers for future development
- [ ] CI/CD integration with appropriate test timeouts

## Notes

- Use realistic test data based on actual Claude Code usage patterns
- Ensure tests are deterministic and don't depend on external state
- Include both positive and negative test cases
- Test configuration edge cases thoroughly
- Validate cleanup and resource management
- Consider future testing needs for subsequent steps