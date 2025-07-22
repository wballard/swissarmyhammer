# Core Cost Tracking Data Structures

## Summary

Implement the fundamental data structures for cost tracking: `CostTracker`, `CostSession`, and `ApiCall`. These structures will form the foundation of the cost tracking system and integrate with the existing metrics infrastructure.

## Context

The SwissArmyHammer system needs comprehensive cost tracking for Claude Code API interactions during issue workflow execution. This step establishes the core data structures that will track API calls, token usage, and cost calculations throughout issue processing.

## Requirements

### Core Data Structures

1. **CostTracker** - Main coordinator for cost tracking sessions
   - Manages current active sessions
   - Integrates with cost calculator and storage
   - Provides session lifecycle management

2. **CostSession** - Tracks cost data for a single issue workflow
   - Issue identifier and session metadata
   - Collection of API calls during workflow
   - Session timing and status tracking

3. **ApiCall** - Individual API call record
   - Timestamp and endpoint information
   - Input/output token counts
   - Call duration and status

### Integration Points

- Use existing `chrono` dependency for timestamp handling
- Follow existing `serde` patterns for serialization
- Integrate with `WorkflowMetrics` architecture patterns
- Use `ULID` for session identifiers (following project standards)

## Implementation Details

### File Location
- Create new module: `swissarmyhammer/src/cost/mod.rs`
- Data structures in: `swissarmyhammer/src/cost/tracker.rs`

### Key Requirements
- All structures must implement `Debug`, `Clone`, `Serialize`, `Deserialize`
- Use proper error types with `thiserror::Error`
- Follow existing naming conventions and documentation patterns
- Include comprehensive rustdoc documentation with examples

### Error Handling
- Create `CostError` enum for cost tracking specific errors
- Handle session lifecycle errors gracefully
- Validate input data (empty strings, negative values, etc.)

### Memory Management
- Implement reasonable limits (similar to `workflow/metrics.rs`)
- Automatic cleanup of old sessions
- Configurable retention policies

## Testing Requirements

- Unit tests for all data structures
- Serialization/deserialization tests
- Memory limit enforcement tests
- Error condition handling tests

## Integration

This step provides the foundation for subsequent cost tracking features:
- Step 191 will add cost calculation logic
- Step 192 will integrate with configuration system
- Step 194 will use these structures for API call capture

## Success Criteria

- [ ] `CostTracker`, `CostSession`, and `ApiCall` structs implemented
- [ ] Comprehensive error handling with `CostError` enum
- [ ] Full test coverage for data structures
- [ ] Integration with existing serialization patterns
- [ ] Memory management and cleanup functionality
- [ ] Complete rustdoc documentation with examples

## Notes

- Follow the patterns established in `workflow/metrics.rs` for consistency
- Use existing project dependencies and avoid adding new ones
- Ensure thread-safety for concurrent workflow execution
- Keep performance overhead minimal