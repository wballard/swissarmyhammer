# MCP Protocol Integration for Cost Tracking

## Summary

Integrate cost tracking with the MCP (Model Control Protocol) system to capture Claude Code API calls. This step hooks into the existing MCP infrastructure to automatically track API interactions during workflow execution.

## Context

The SwissArmyHammer system uses MCP protocol for Claude Code interactions (see `src/mcp/` module). This step extends the MCP handlers to capture API calls, token counts, and timing data for cost tracking purposes.

## Requirements

### MCP Integration Points

1. **API Call Interception**
   - Hook into MCP request/response cycle
   - Capture all Claude Code API interactions
   - Extract token usage from API responses
   - Record timing and status information

2. **Token Extraction**
   - Parse Claude API response headers for usage data
   - Handle different API endpoint token reporting
   - Support both streaming and non-streaming responses
   - Validate token count accuracy

3. **Error Handling**
   - Track failed API calls for cost analysis
   - Handle rate limiting and timeout scenarios
   - Preserve cost data even when API calls fail
   - Graceful degradation when cost tracking fails

### Implementation Strategy

1. **MCP Handler Extension**
   - Extend existing MCP tool handlers
   - Add cost tracking middleware to request pipeline
   - Integrate with `CostTracker` from step 000190

2. **API Response Processing**
   - Parse Claude API response format for token data
   - Handle different response structures (streaming vs batch)
   - Extract usage statistics from response metadata

3. **Session Association**
   - Link API calls to active cost tracking sessions
   - Handle concurrent workflow executions
   - Maintain session context through MCP calls

## Implementation Details

### File Modifications
- Extend: `swissarmyhammer/src/mcp/tool_handlers.rs`
- Add: `swissarmyhammer/src/mcp/cost_tracking.rs`
- Modify: `swissarmyhammer/src/mcp/responses.rs`

### Integration Architecture
```rust
// Example integration pattern
pub struct CostTrackingMcpHandler<T: McpHandler> {
    inner_handler: T,
    cost_tracker: Arc<Mutex<CostTracker>>,
}

impl<T: McpHandler> McpHandler for CostTrackingMcpHandler<T> {
    async fn handle_request(&self, request: McpRequest) -> McpResponse {
        let start_time = Instant::now();
        let response = self.inner_handler.handle_request(request).await;
        
        // Extract cost data and update tracker
        if let Ok(usage) = extract_token_usage(&response) {
            self.cost_tracker.lock().await.record_api_call(/* ... */);
        }
        
        response
    }
}
```

### Token Usage Extraction
- Parse `usage` field from Claude API responses
- Handle different API versions and response formats
- Validate token counts for consistency
- Fall back to estimation when exact counts unavailable

### Session Context Management
- Use thread-local storage or request context for session tracking
- Handle session lifecycle across multiple API calls
- Support concurrent workflows with separate sessions

## Testing Requirements

- Mock MCP interactions with cost tracking
- Token extraction accuracy tests
- Session association verification
- Error condition handling tests
- Performance impact measurement

## Integration

This step builds on:
- Step 000190: Uses `CostTracker` and related structures
- Step 000191: Integrates with cost calculation
- Step 000192: Uses configuration for tracking preferences

Prepares for:
- Step 000195: Token counting validation
- Step 000196: Workflow action integration

## Success Criteria

- [ ] MCP handlers extended with cost tracking capabilities
- [ ] Accurate token usage extraction from API responses
- [ ] Proper session association for concurrent workflows
- [ ] Comprehensive error handling for API failures
- [ ] Minimal performance impact on MCP operations
- [ ] Integration tests with real MCP scenarios
- [ ] Graceful degradation when cost tracking fails

## Proposed Solution

I will implement MCP protocol integration for cost tracking through the following approach:

### 1. Architecture Design
- Create a `CostTrackingMcpHandler` wrapper that decorates existing MCP handlers
- Use the decorator pattern to add cost tracking without modifying core MCP logic
- Integrate with the existing `CostTracker` from step 000190
- Maintain session context through thread-local storage or request context

### 2. Implementation Steps
1. **Analyze existing MCP structure** - Review current `src/mcp/` module to understand handler patterns
2. **Create cost tracking middleware** - Implement wrapper handler in `src/mcp/cost_tracking.rs`
3. **Token extraction logic** - Parse Claude API response headers and body for usage data
4. **Session management** - Associate API calls with active cost tracking sessions
5. **Error handling** - Ensure cost tracking gracefully handles API failures
6. **Integration testing** - Verify token extraction accuracy and session association

### 3. Key Components
- `CostTrackingMcpHandler<T>` - Generic wrapper for any MCP handler
- `TokenUsageExtractor` - Parses API responses for token data
- `SessionContextManager` - Links API calls to cost tracking sessions
- Integration with existing `CostTracker::record_api_call()` method

### 4. Testing Strategy
- Unit tests for token extraction accuracy
- Integration tests with mock MCP scenarios  
- Performance benchmarks to ensure minimal overhead
- Error condition handling tests
- Concurrent session validation

This approach ensures minimal disruption to existing MCP functionality while providing comprehensive cost tracking capabilities.

## Notes

- Follow existing MCP patterns and error handling
- Ensure cost tracking doesn't interfere with core MCP functionality
- Consider different Claude API versions and response formats
- Handle both successful and failed API calls
- Maintain thread safety for concurrent API calls
- Test with realistic MCP usage patterns from existing workflows