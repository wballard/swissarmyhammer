//! Cross-issue cost aggregation and analytics
//!
//! This module provides comprehensive cost aggregation capabilities for analyzing
//! costs across multiple issues, identifying trends, and generating insights for
//! project-wide cost optimization.

pub mod analyzer;
pub mod reports;
pub mod trends;

#[cfg(test)]
pub mod tests;

pub use analyzer::{CostAggregator, AggregationError, AggregationResult};
pub use reports::{ReportGenerator, ExportFormat, AggregatedReport};
pub use trends::{TrendAnalyzer, TrendAnalysis};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project-wide cost summary with comprehensive analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCostSummary {
    /// Total cost across all completed issues
    pub total_cost: Decimal,
    /// Total number of issues included in the analysis
    pub total_issues: usize,
    /// Average cost per issue
    pub average_cost_per_issue: Decimal,
    /// Median cost per issue
    pub median_cost_per_issue: Decimal,
    /// Cost trend analysis
    pub cost_trend: CostTrend,
    /// Efficiency metrics
    pub efficiency_metrics: EfficiencyMetrics,
    /// Analysis period
    pub period: DateRange,
    /// Cost breakdown by issue type or category
    pub cost_breakdown: HashMap<String, Decimal>,
    /// Outlier issues (high cost or unusual patterns)
    pub outliers: Vec<IssueOutlier>,
    /// Time when analysis was generated
    pub generated_at: DateTime<Utc>,
}

/// Cost trend analysis with statistical measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    /// Daily cost data points
    pub daily_costs: Vec<(DateTime<Utc>, Decimal)>,
    /// Weekly cost aggregations
    pub weekly_costs: Vec<(DateTime<Utc>, Decimal)>,
    /// Monthly cost aggregations
    pub monthly_costs: Vec<(DateTime<Utc>, Decimal)>,
    /// Overall trend direction
    pub trend_direction: TrendDirection,
    /// Growth rate (positive for increasing, negative for decreasing)
    pub growth_rate: f64,
    /// Statistical confidence in trend analysis (0.0 to 1.0)
    pub confidence: f64,
    /// Moving average costs (7-day window)
    pub moving_average: Vec<(DateTime<Utc>, Decimal)>,
    /// Seasonal patterns detected
    pub seasonal_patterns: Vec<SeasonalPattern>,
}

/// Trend direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Costs are increasing over time
    Increasing,
    /// Costs are decreasing over time
    Decreasing,
    /// Costs remain relatively stable
    Stable,
    /// Costs show high volatility without clear direction
    Volatile,
}

/// Efficiency metrics for development cost analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Cost per API call across all issues
    pub cost_per_api_call: Decimal,
    /// Cost per token (input and output combined)
    pub cost_per_token: Decimal,
    /// Average session duration in minutes
    pub avg_session_duration_minutes: f64,
    /// Cost per session
    pub cost_per_session: Decimal,
    /// Token efficiency (output tokens per input token)
    pub token_efficiency: f64,
    /// Most expensive operations identified
    pub expensive_operations: Vec<ExpensiveOperation>,
    /// Cost efficiency score (0.0 to 1.0, higher is better)
    pub efficiency_score: f64,
}

/// Date range for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// Start date (inclusive)
    pub start: DateTime<Utc>,
    /// End date (inclusive)
    pub end: DateTime<Utc>,
}

impl DateRange {
    /// Create a new date range
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self { start, end }
    }

    /// Get the duration in days
    pub fn duration_days(&self) -> i64 {
        (self.end - self.start).num_days()
    }
}

/// Issue outlier identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueOutlier {
    /// Issue identifier
    pub issue_id: String,
    /// Total cost for this issue
    pub cost: Decimal,
    /// Outlier type (high cost, unusual pattern, etc.)
    pub outlier_type: OutlierType,
    /// Standard deviations from mean
    pub standard_deviations: f64,
    /// Reason for flagging as outlier
    pub reason: String,
}

/// Types of cost outliers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutlierType {
    /// Issue with exceptionally high cost
    HighCost,
    /// Issue with unusual token usage patterns
    UnusualTokenUsage,
    /// Issue with abnormal session count
    AbnormalSessions,
    /// Issue with unexpected API call patterns
    UnusualApiPatterns,
}

/// Seasonal pattern detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    /// Pattern type (daily, weekly, monthly)
    pub pattern_type: PatternType,
    /// Strength of the pattern (0.0 to 1.0)
    pub strength: f64,
    /// Description of the pattern
    pub description: String,
}

/// Types of seasonal patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    /// Daily patterns (time of day effects)
    Daily,
    /// Weekly patterns (day of week effects)
    Weekly,
    /// Monthly patterns (month effects)
    Monthly,
}

/// Expensive operation identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpensiveOperation {
    /// Operation description
    pub operation: String,
    /// Total cost for this operation type
    pub total_cost: Decimal,
    /// Number of occurrences
    pub occurrences: u64,
    /// Average cost per occurrence
    pub avg_cost: Decimal,
    /// Percentage of total project cost
    pub cost_percentage: f64,
}