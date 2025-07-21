# Workflow Action Cost Tracking Integration

## Summary

Integrate cost tracking with the workflow action system to automatically start/stop cost tracking sessions during issue workflow execution and associate costs with specific workflow runs.

## Context

The SwissArmyHammer workflow system (`src/workflow/`) manages issue execution through various actions. This step integrates cost tracking into the workflow lifecycle, ensuring cost sessions are properly managed and associated with workflow runs.

## Requirements

### Workflow Integration Points

1. **Session Lifecycle Management**
   - Start cost tracking session when workflow begins
   - Associate sessions with `WorkflowRunId` and issue numbers
   - Complete sessions when workflow finishes (success/failure)
   - Handle session cleanup for interrupted workflows

2. **Action-Level Tracking**
   - Track costs for individual workflow actions
   - Associate API calls with specific action types
   - Measure action-level performance and costs
   - Provide granular cost breakdown

3. **Metrics System Integration**
   - Extend existing `WorkflowMetrics` with cost data
   - Include cost information in workflow completion events
   - Integrate with existing metrics collection patterns

### Implementation Strategy

1. **Workflow Executor Extension**
   - Modify `WorkflowExecutor` to manage cost sessions
   - Add cost tracking hooks to workflow lifecycle events
   - Ensure session cleanup in error conditions

2. **Action System Integration**
   - Extend action execution to include cost context
   - Associate MCP calls with active workflow sessions
   - Track action-specific cost attribution

3. **Metrics Integration**
   - Add cost fields to `RunMetrics` structure
   - Extend workflow summary metrics with cost data
   - Maintain cost trends in global metrics

## Implementation Details

### File Modifications
- Extend: `swissarmyhammer/src/workflow/executor/core.rs`
- Modify: `swissarmyhammer/src/workflow/metrics.rs`
- Update: `swissarmyhammer/src/workflow/run.rs`

### Integration Architecture

```rust
// Extend WorkflowExecutor with cost tracking
pub struct WorkflowExecutor {
    // existing fields...
    cost_tracker: Option<Arc<Mutex<CostTracker>>>,
}

// Extend RunMetrics with cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    // existing fields...
    pub cost_metrics: Option<CostMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    pub total_cost: Decimal,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub api_call_count: usize,
    pub cost_by_action: HashMap<String, ActionCostBreakdown>,
}
```

### Session Management
- Create cost session when workflow run starts
- Use `WorkflowRunId` as session identifier
- Associate with issue number for reporting
- Handle concurrent workflow executions

### Action Cost Attribution
- Track which action triggered each API call
- Maintain action-level cost breakdown
- Support nested action cost tracking
- Provide detailed cost attribution

### Error Handling
- Graceful session cleanup on workflow failure
- Continue workflow execution even if cost tracking fails
- Log cost tracking errors without failing workflows
- Maintain partial cost data when possible

## Testing Requirements

### Integration Testing
- Workflow execution with cost tracking enabled/disabled
- Multiple concurrent workflow cost tracking
- Session cleanup verification for failed workflows
- Cost data accuracy in metrics integration

### Cost Attribution Testing
- Action-level cost tracking accuracy
- Nested action cost attribution
- API call association with correct actions
- Cost breakdown validation

### Performance Testing
- Overhead of cost tracking on workflow execution
- Memory usage impact of cost session management
- Concurrent workflow cost tracking performance

## Integration

This step builds on:
- Step 000190: Uses `CostTracker` for session management
- Step 000194: Relies on MCP integration for API call capture
- Step 000195: Uses accurate token counting

Integrates with:
- Existing workflow metrics system
- Issue storage for cost reporting (future steps)

## Success Criteria

- [ ] Cost sessions properly managed throughout workflow lifecycle
- [ ] Accurate cost attribution to workflow runs and actions
- [ ] Seamless integration with existing workflow metrics
- [ ] Minimal performance impact on workflow execution
- [ ] Robust error handling preserving workflow functionality
- [ ] Comprehensive test coverage for all workflow scenarios
- [ ] Cost data available in workflow completion events

## Notes

- Follow existing workflow patterns and error handling
- Ensure cost tracking is optional and configurable
- Maintain backward compatibility with existing workflows
- Consider workflow action nesting and composition
- Handle long-running workflows with session timeouts
- Test with realistic workflow execution patterns
- Preserve existing workflow performance characteristics