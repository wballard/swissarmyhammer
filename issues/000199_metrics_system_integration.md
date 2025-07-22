# Metrics System Integration with Cost Data

## Summary

Integrate cost tracking data with the existing workflow metrics system to provide comprehensive cost analytics, trends, and aggregation across all workflow executions.

## Context

The SwissArmyHammer system has a robust metrics collection system (`src/workflow/metrics.rs`). This step extends it to include cost data, enabling cost trend analysis, aggregation statistics, and integration with the existing metrics infrastructure.

## Requirements

### Metrics Extension Areas

1. **RunMetrics Enhancement**
   - Add cost data to individual workflow run metrics
   - Include token usage statistics
   - Track cost efficiency metrics
   - Maintain cost attribution data

2. **WorkflowSummaryMetrics Extension**
   - Add cost aggregation for workflow types
   - Calculate average, min, max costs per workflow
   - Track cost trends over time
   - Include cost efficiency statistics

3. **GlobalMetrics Enhancement**
   - Add system-wide cost tracking
   - Calculate total costs across all workflows
   - Track cost trends and patterns
   - Include cost optimization metrics

4. **New Cost-Specific Metrics**
   - Cost trend analysis
   - Token efficiency tracking
   - API usage patterns
   - Cost optimization insights

### Implementation Strategy

1. **Extend Existing Structures**
   - Add cost fields to `RunMetrics`
   - Extend `WorkflowSummaryMetrics` with cost data
   - Update `GlobalMetrics` for cost aggregation
   - Maintain backward compatibility

2. **Cost Aggregation Logic**
   - Calculate cost statistics (avg, min, max, median)
   - Track cost trends over time
   - Compute efficiency metrics (cost per token, etc.)
   - Generate cost optimization insights

3. **Integration with Storage**
   - Persist cost metrics with existing storage
   - Support cost data querying and analysis
   - Enable cost metric cleanup and retention
   - Maintain cost history across restarts

## Implementation Details

### File Modifications
- Extend: `swissarmyhammer/src/workflow/metrics.rs`
- Add: Cost-specific metric types and calculations

### Data Structure Extensions

```rust
// Extend RunMetrics with cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    // existing fields...
    pub cost_data: Option<RunCostMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCostMetrics {
    pub total_cost: Decimal,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub api_call_count: usize,
    pub average_cost_per_call: Decimal,
    pub token_efficiency: f64, // output/input ratio
    pub cost_per_token: Decimal,
}

// Extend WorkflowSummaryMetrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummaryMetrics {
    // existing fields...
    pub cost_summary: Option<WorkflowCostSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCostSummary {
    pub total_cost: Decimal,
    pub average_cost: Decimal,
    pub min_cost: Decimal,
    pub max_cost: Decimal,
    pub cost_trend: Vec<(DateTime<Utc>, Decimal)>,
    pub efficiency_trend: Vec<(DateTime<Utc>, f64)>,
}
```

### Cost Aggregation Features
- Calculate cost statistics across runs
- Track cost trends over time windows
- Identify cost anomalies and spikes
- Compute efficiency metrics and improvements

### Trend Analysis
- Cost over time tracking
- Token efficiency trends
- API usage pattern analysis
- Cost optimization opportunities

### Performance Considerations
- Efficient cost metric calculations
- Memory usage for cost trend data
- Cleanup of old cost metrics
- Scalable aggregation algorithms

## Testing Requirements

### Integration Testing
- Cost data integration with existing metrics
- Aggregation accuracy validation
- Trend calculation correctness
- Performance impact measurement

### Cost Calculation Testing
- Statistical calculation accuracy
- Edge case handling (zero costs, single runs)
- Trend analysis validation
- Efficiency metric correctness

### Compatibility Testing
- Backward compatibility with existing metrics
- Migration of historical data
- Serialization/deserialization integrity
- Configuration integration

## Integration

This step integrates with:
- Step 000196: Gets cost data from workflow integration
- Step 000198: Provides data for issue cost sections
- Existing workflow metrics system

Builds toward:
- Step 000200: Database storage for advanced analytics
- Step 000202: Cross-issue aggregation

## Success Criteria

- [ ] Seamless integration with existing metrics system
- [ ] Cost data included in all metric collection points
- [ ] Accurate cost aggregation and trend analysis
- [ ] Performance impact within acceptable limits
- [ ] Backward compatibility maintained
- [ ] Comprehensive test coverage for cost metric scenarios
- [ ] Cost optimization insights and recommendations

## Notes

- Follow existing metrics patterns and conventions
- Ensure cost metrics don't impact existing metrics performance
- Consider memory usage with large cost datasets
- Support both paid and max plan metrics
- Handle missing cost data gracefully
- Provide meaningful cost optimization insights
- Maintain consistency with existing metric cleanup policies
- Test with realistic cost data scenarios and volumes