# Cross-Issue Cost Aggregation and Statistics

## Summary

Implement cross-issue cost aggregation and statistical analysis to provide project-wide cost insights, trends, and optimization recommendations across all completed issues.

## Context

Individual issue cost tracking provides valuable data, but project-wide aggregation enables broader insights like total project costs, cost trends over time, cost efficiency analysis, and optimization opportunities. This step implements comprehensive aggregation across all issues.

## Requirements

### Aggregation Features

1. **Project-Wide Statistics**
   - Total cost across all completed issues
   - Average cost per issue type
   - Cost trends over time periods
   - Cost distribution analysis

2. **Cost Trend Analysis**
   - Daily/weekly/monthly cost trends
   - Seasonal patterns and variations
   - Cost efficiency improvements over time
   - Performance correlation analysis

3. **Optimization Insights**
   - High-cost issue identification
   - Cost efficiency recommendations
   - Token usage optimization suggestions
   - API usage pattern analysis

4. **Reporting and Analytics**
   - Cost summary reports
   - Trend visualization data
   - Comparative analysis between issues
   - Export capabilities for external analysis

### Implementation Strategy

1. **Aggregation Engine**
   - Scan all completed issues for cost data
   - Parse cost sections from markdown files
   - Aggregate data from metrics system
   - Query database when available

2. **Statistical Analysis**
   - Calculate statistical measures (mean, median, std dev)
   - Identify outliers and anomalies
   - Compute trend coefficients
   - Generate efficiency metrics

3. **Reporting System**
   - Generate aggregate reports
   - Provide data export capabilities
   - Support different time ranges
   - Enable filtering and grouping

## Implementation Details

### File Structure
- Create: `swissarmyhammer/src/cost/aggregation/`
- Add: `mod.rs`, `analyzer.rs`, `reports.rs`, `trends.rs`

### Core Components

```rust
pub struct CostAggregator {
    issue_storage: Arc<dyn IssueStorage>,
    metrics: Arc<WorkflowMetrics>,
    database: Option<Arc<CostDatabase>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCostSummary {
    pub total_cost: Decimal,
    pub total_issues: usize,
    pub average_cost_per_issue: Decimal,
    pub cost_trend: CostTrend,
    pub efficiency_metrics: EfficiencyMetrics,
    pub period: DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    pub daily_costs: Vec<(DateTime<Utc>, Decimal)>,
    pub trend_direction: TrendDirection,
    pub growth_rate: f64,
    pub confidence: f64,
}

pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}
```

### Aggregation Functions
- Scan completed issues directory
- Extract cost data from markdown sections
- Combine with metrics system data
- Validate data consistency

### Analysis Algorithms
- Statistical analysis (mean, median, percentiles)
- Trend calculation using regression analysis
- Outlier detection and flagging
- Efficiency metric computation

### Reporting Features
- Generate summary reports in multiple formats
- Provide data for visualization
- Support custom date ranges and filters
- Enable export to CSV/JSON formats

## Configuration Integration

Add to existing config:
```yaml
cost_tracking:
  aggregation:
    enabled: true
    scan_interval_hours: 24  # How often to update aggregations
    retention_days: 365      # How long to keep aggregated data
    trend_analysis_days: 30  # Period for trend calculation
    outlier_threshold: 2.0   # Standard deviations for outlier detection
```

## Testing Requirements

### Aggregation Testing
- Cost data aggregation accuracy
- Statistical calculation validation
- Trend analysis correctness
- Performance with large datasets

### Integration Testing
- Multi-source data aggregation
- Consistency across storage backends
- Real-time vs batch aggregation
- Configuration integration

### Performance Testing
- Large project cost aggregation
- Memory usage optimization
- Query performance validation
- Concurrent aggregation handling

## Integration

This step integrates with:
- Step 000198: Reads cost data from issue markdown
- Step 000199: Uses metrics system aggregation
- Step 000200: Leverages database queries when available
- All previous cost tracking components

## Use Cases

### Project Management
- Track total project costs
- Budget vs actual analysis
- Cost forecasting
- Resource allocation optimization

### Development Optimization
- Identify expensive development patterns
- Track cost efficiency improvements
- Compare different approaches
- Guide development practices

### Business Analysis
- ROI analysis for development work
- Cost per feature analysis
- Trend reporting for stakeholders
- Budget planning support

## Success Criteria

- [ ] Accurate cross-issue cost aggregation
- [ ] Comprehensive statistical analysis
- [ ] Trend analysis and prediction capabilities
- [ ] Performance optimization for large datasets
- [ ] Multiple data source integration
- [ ] Flexible reporting and export options
- [ ] Real-time and batch aggregation support

## Notes

- Design for scalability with large numbers of issues
- Consider incremental aggregation for performance
- Handle missing or partial cost data gracefully
- Support different aggregation time periods
- Provide meaningful insights and recommendations
- Consider future integration with project management tools
- Test with realistic project sizes and data volumes
- Support both real-time and batch aggregation modes