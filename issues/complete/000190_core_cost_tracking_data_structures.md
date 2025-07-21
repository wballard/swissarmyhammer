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

## Proposed Solution

Based on analysis of the existing `workflow/metrics.rs` patterns, I will implement the cost tracking data structures following these steps:

### 1. Module Structure
- Create `swissarmyhammer/src/cost/mod.rs` with module exports
- Create `swissarmyhammer/src/cost/tracker.rs` with core data structures
- Update `swissarmyhammer/src/lib.rs` to include cost module

### 2. Core Data Structures
- **`CostError`** enum using `thiserror::Error` for consistent error handling
- **`ApiCall`** struct with timestamp, endpoint, token counts, duration, status
- **`CostSession`** struct with ULID identifier, issue metadata, API call collection, session timing
- **`CostTracker`** struct for session lifecycle management with HashMap storage

### 3. Design Patterns (following workflow/metrics.rs)
- All structs implement `Debug`, `Clone`, `Serialize`, `Deserialize`
- Use `chrono::DateTime<Utc>` for timestamps
- Use `std::time::Duration` for durations
- Use `ulid::Ulid` for session identifiers (wrapped in newtype)
- Constants for maximum limits (similar to `MAX_RUN_METRICS`, etc.)
- Validation functions for input data
- Memory management with cleanup functions
- Comprehensive error handling

### 4. Memory Management
- Maximum session limits (similar to `MAX_RUN_METRICS = 1000`)
- Maximum API calls per session (similar to `MAX_STATE_DURATIONS_PER_RUN = 50`)
- Automatic cleanup of old completed sessions
- Configurable retention policies

### 5. Testing Strategy
- Unit tests for all data structures
- Serialization/deserialization roundtrip tests
- Memory limit enforcement tests
- Error condition validation tests
- Input validation tests (empty strings, negative values, etc.)

This approach ensures consistency with existing codebase patterns while providing the foundation for future cost tracking features.