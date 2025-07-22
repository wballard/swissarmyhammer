# Cost Tracking for Issues - Product Requirements Document

## Overview

This PRD defines a comprehensive cost tracking system that monitors and reports the financial cost of completing issues in the SwissArmyHammer system. The feature will track API usage, token consumption, and associated costs for Claude Code interactions, providing detailed cost breakdowns within issue markdown files.

## Problem Statement

Currently, there is no visibility into the cost associated with completing individual issues. Users need to understand:
- How much each issue costs in terms of API calls and tokens
- Which APIs are being used and their frequency
- Aggregate cost statistics across issues
- Cost trends over time

## Goals

### Primary Goals
- Track all Claude Code API interactions during issue completion
- Log token usage and API call counts with timestamps
- Calculate costs based on Claude Code pricing model (paid vs. max plan)
- Display cost information at the bottom of completed issue markdown files
- Provide aggregate cost statistics and trends

### Secondary Goals
- Integration with existing workflow metrics system
- Minimal performance impact on issue processing
- Historical cost analysis and reporting
- Cost optimization insights

## User Stories

### As a developer
- I want to see how much each completed issue cost in API usage
- I want to understand which types of issues are most expensive
- I want to track my API usage against my Claude Code plan limits

### As a project manager
- I want to see aggregate costs across all issues
- I want to identify cost trends and optimization opportunities
- I want to budget for future development work based on historical costs

### As a team lead
- I want to compare cost efficiency across different types of issues
- I want to set cost budgets for issue completion
- I want visibility into API usage patterns

## Requirements

### Functional Requirements

#### FR1: Cost Data Collection
- **FR1.1**: Track all Claude Code API calls during issue workflow execution
- **FR1.2**: Record token counts for input and output
- **FR1.3**: Capture API call timestamps and duration
- **FR1.4**: Identify API endpoints used (e.g., /v1/messages, /v1/complete)
- **FR1.5**: Handle both Claude Code (paid) and Claude Code Max (unlimited) plans

#### FR2: Cost Calculation
- **FR2.1**: Calculate costs based on current Claude Code pricing model
- **FR2.2**: Support different pricing tiers and token costs
- **FR2.3**: Handle free tier usage and overage calculations
- **FR2.4**: For Claude Code Max, show token usage instead of monetary cost

#### FR3: Issue Integration
- **FR3.1**: Append cost section to completed issue markdown files
- **FR3.2**: Display costs in human-readable format
- **FR3.3**: Show breakdown by API type and timestamp
- **FR3.4**: Include aggregate statistics (total cost, total tokens, call count)

#### FR4: Data Persistence
- **FR4.1**: Store cost data in issue markdown files
- **FR4.2**: Maintain cost history across issue updates
- **FR4.3**: Support cost data migration for existing issues

#### FR5: Aggregation and Reporting
- **FR5.1**: Calculate total costs across all completed issues
- **FR5.2**: Provide cost breakdown by time period
- **FR5.3**: Generate cost trend reports
- **FR5.4**: Support filtering by issue type, date range, or cost threshold

### Non-Functional Requirements

#### NFR1: Performance
- Cost tracking must not add more than 50ms overhead per API call
- Batch cost calculations to minimize impact on issue completion time
- Async cost data persistence to avoid blocking workflows

#### NFR2: Reliability
- Cost tracking failures must not prevent issue completion
- Graceful degradation when cost APIs are unavailable
- Data consistency across concurrent issue processing

#### NFR3: Security
- No storage of API keys or sensitive authentication data
- Cost data access restricted to authorized users
- Audit trail for cost data modifications

#### NFR4: Maintainability
- Integration with existing metrics and workflow systems
- Clear separation of cost tracking from core issue functionality
- Comprehensive logging and error handling

## Technical Architecture

### System Components

#### Cost Tracker
```rust
pub struct CostTracker {
    /// Current cost session for tracking API calls
    current_session: Option<CostSession>,
    /// Cost calculation engine
    calculator: CostCalculator,
    /// Storage backend for persisting cost data
    storage: CostStorage,
}

pub struct CostSession {
    /// Issue identifier
    issue_id: String,
    /// Session start time
    started_at: DateTime<Utc>,
    /// Collected API calls
    api_calls: Vec<ApiCall>,
    /// Session metadata
    metadata: SessionMetadata,
}
```

#### API Call Tracking
```rust
pub struct ApiCall {
    /// Timestamp of the API call
    timestamp: DateTime<Utc>,
    /// API endpoint called
    endpoint: String,
    /// Request token count
    input_tokens: u32,
    /// Response token count
    output_tokens: u32,
    /// Call duration
    duration: Duration,
    /// Response status
    status: ApiCallStatus,
}
```

#### Cost Calculation
```rust
pub struct CostCalculator {
    /// Pricing model (paid vs max plan)
    pricing_model: PricingModel,
    /// Current pricing rates
    rates: PricingRates,
}

pub enum PricingModel {
    /// Claude Code paid plan with per-token pricing
    Paid(PaidPlanConfig),
    /// Claude Code Max with unlimited usage
    Max(MaxPlanConfig),
}
```

### Integration Points

#### Workflow Actions Integration
- Hook into existing `PromptAction` execution
- Intercept Claude Code CLI calls in workflow system
- Extend `WorkflowMetrics` to include cost tracking

#### MCP Protocol Integration
- Monitor MCP tool requests and responses
- Track token usage in MCP message exchanges
- Integrate with existing MCP error handling

#### Issue Storage Integration
- Extend issue markdown format with cost section
- Integrate with `FileSystemIssueStorage`
- Preserve existing issue metadata structure

### Data Flow

1. **Issue Workflow Start**: Initialize `CostSession` when issue workflow begins
2. **API Call Interception**: Capture all Claude Code API interactions
3. **Real-time Tracking**: Update session with token counts and timing
4. **Cost Calculation**: Calculate costs based on current pricing model
5. **Issue Completion**: Append cost data to issue markdown file
6. **Aggregation**: Update global cost statistics and trends

## Implementation Details

### Issue Markdown Format Extension

#### Cost Section Structure
```markdown
## Cost Analysis

**Total Cost**: $2.34 (or "Unlimited Plan - 15,420 tokens used" for Max plan)
**Total API Calls**: 12
**Total Input Tokens**: 8,450
**Total Output Tokens**: 6,970
**Session Duration**: 2m 34s
**Completed**: 2024-01-15 14:32:17 UTC

### API Call Breakdown

| Timestamp | Endpoint | Input Tokens | Output Tokens | Duration | Cost |
|-----------|----------|--------------|---------------|----------|------|
| 14:30:15 | /v1/messages | 1,200 | 850 | 1.2s | $0.18 |
| 14:31:22 | /v1/messages | 2,100 | 1,400 | 2.1s | $0.31 |
| ... | ... | ... | ... | ... | ... |

### Cost Summary
- **Average cost per call**: $0.19
- **Most expensive call**: $0.45 (2,500 input + 1,800 output tokens)
- **Token efficiency**: 0.82 (output/input ratio)
```

### Configuration Options

#### YAML Configuration Extension
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"  # or "max"
  rates:
    input_token_cost: 0.000015  # per token
    output_token_cost: 0.000075  # per token
  aggregation:
    enabled: true
    retention_days: 90
  reporting:
    include_in_issues: true
    detailed_breakdown: true
```

### Database Schema (Optional)

For advanced aggregation and reporting, an optional SQLite database can be used:

```sql
CREATE TABLE cost_sessions (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    total_cost DECIMAL(10,4),
    total_calls INTEGER,
    total_input_tokens INTEGER,
    total_output_tokens INTEGER,
    pricing_model TEXT NOT NULL
);

CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    endpoint TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    duration_ms INTEGER,
    cost DECIMAL(8,4),
    FOREIGN KEY (session_id) REFERENCES cost_sessions(id)
);
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Implement `CostTracker` and `CostSession` structs
- [ ] Create API call interception mechanism
- [ ] Basic cost calculation for paid plans
- [ ] Unit tests for core functionality

### Phase 2: Integration (Week 3)
- [ ] Integrate with workflow actions system
- [ ] Hook into MCP protocol handlers
- [ ] Extend issue markdown format
- [ ] Integration tests with existing systems

### Phase 3: Advanced Features (Week 4)
- [ ] Support for Claude Code Max plan tracking
- [ ] Aggregation and reporting functionality
- [ ] Configuration system integration
- [ ] Performance optimization

### Phase 4: Polish and Documentation (Week 5)
- [ ] Comprehensive error handling
- [ ] Performance benchmarking
- [ ] User documentation
- [ ] Migration guide for existing issues

## Testing Strategy

### Unit Tests
- Cost calculation accuracy across different pricing models
- API call interception and data collection
- Issue markdown format generation
- Error handling and edge cases

### Integration Tests
- End-to-end issue completion with cost tracking
- MCP protocol integration
- Workflow system integration
- Configuration system integration

### Performance Tests
- Overhead measurement for cost tracking
- Memory usage analysis
- Concurrent issue processing
- Large dataset handling

## Success Metrics

### Technical Metrics
- Cost tracking overhead < 50ms per API call
- 99.9% data accuracy for cost calculations
- Zero cost tracking failures causing issue completion failures
- < 5% memory overhead for cost tracking

### User Adoption Metrics
- 80% of users find cost information useful
- 60% of users use cost data for decision making
- Cost-based optimization in 40% of projects
- Positive feedback on cost visibility

## Risks and Mitigations

### Risk 1: Performance Impact
- **Mitigation**: Async processing, batching, and performance monitoring
- **Fallback**: Configurable cost tracking with granular enable/disable options

### Risk 2: Pricing Model Changes
- **Mitigation**: Configurable pricing rates with automatic updates
- **Fallback**: Manual rate configuration and migration tools

### Risk 3: API Interception Complexity
- **Mitigation**: Use existing workflow hooks and MCP integration points
- **Fallback**: Optional cost tracking with manual entry capabilities

### Risk 4: Data Storage Scaling
- **Mitigation**: Data retention policies and efficient storage formats
- **Fallback**: Configurable retention and archival systems

## Dependencies

### Internal Dependencies
- Existing workflow system (`workflow/` module)
- Issue storage system (`issues/` module)
- MCP protocol implementation (`mcp/` module)
- Configuration system (`config.rs`)
- Metrics system (`workflow/metrics.rs`)

### External Dependencies
- Claude Code API pricing information
- Token counting libraries
- Date/time handling (existing `chrono` dependency)
- Serialization (existing `serde` dependency)

## Future Enhancements

### v2.0 Features
- Cost budgeting and alerts
- Cost optimization recommendations
- Integration with project management tools
- Cost-based issue prioritization

### v3.0 Features
- Multi-user cost tracking and attribution
- Advanced analytics and machine learning insights
- API cost forecasting
- Integration with billing systems

## Acceptance Criteria

### Must Have
- [ ] All Claude Code API calls are tracked during issue completion
- [ ] Token counts are accurately recorded and calculated
- [ ] Cost information appears at bottom of completed issue markdown files
- [ ] Both paid and Max plan pricing models are supported
- [ ] Performance overhead is minimal (< 50ms per API call)
- [ ] Cost tracking failures do not break issue completion

### Should Have
- [ ] Aggregate cost statistics across issues
- [ ] Configurable cost tracking options
- [ ] Historical cost data analysis
- [ ] Cost trend reporting

### Nice to Have
- [ ] Cost optimization insights
- [ ] Budget alerts and notifications
- [ ] Integration with external reporting tools
- [ ] Advanced cost analytics dashboard

## Conclusion

This cost tracking feature will provide essential visibility into the financial impact of issue completion, enabling better resource planning and cost optimization. By building on the existing metrics and workflow infrastructure, we can deliver a robust, performant, and user-friendly cost tracking system that integrates seamlessly with current workflows.

The phased implementation approach ensures incremental delivery of value while maintaining system stability and performance. The comprehensive testing strategy and risk mitigation plans address the primary concerns around performance impact and data accuracy.