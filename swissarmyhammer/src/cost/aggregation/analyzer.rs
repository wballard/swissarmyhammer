//! Cost aggregation analysis engine
//!
//! This module provides the core aggregation engine that scans completed issues,
//! extracts cost data from multiple sources, and performs comprehensive analysis.

use super::{
    CostTrend, DateRange, EfficiencyMetrics, IssueOutlier, OutlierType,
    ProjectCostSummary, TrendDirection,
};
use crate::config::AggregationConfig;
#[cfg(feature = "database")]
use crate::cost::CostDatabase;
use crate::issues::IssueStorage;
use crate::workflow::metrics::WorkflowMetrics;
use rust_decimal::prelude::ToPrimitive;
use chrono::Utc;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Maximum number of issues to analyze in a single aggregation run
const MAX_ISSUES_PER_AGGREGATION: usize = 10000;

/// Minimum number of data points needed for statistical analysis
const MIN_DATA_POINTS_FOR_STATISTICS: usize = 3;

/// Outlier threshold in standard deviations
const DEFAULT_OUTLIER_THRESHOLD: f64 = 2.0;

/// Cost aggregation errors
#[derive(Error, Debug)]
pub enum AggregationError {
    /// Error accessing issue storage
    #[error("Issue storage error: {0}")]
    IssueStorage(String),

    /// Error accessing cost database
    #[error("Database error: {0}")]
    Database(String),

    /// Error parsing cost data from issues
    #[error("Cost parsing error: {issue_id}: {message}")]
    CostParsing {
        /// Issue ID where parsing failed
        issue_id: String,
        /// Error message
        message: String,
    },

    /// Insufficient data for analysis
    #[error("Insufficient data: {message}")]
    InsufficientData {
        /// Description of what data is missing
        message: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// I/O error during file operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for aggregation operations
pub type AggregationResult<T> = Result<T, AggregationError>;


/// Main cost aggregation engine
pub struct CostAggregator {
    /// Issue storage backend
    issue_storage: Arc<dyn IssueStorage + Send + Sync>,
    /// Workflow metrics collector
    metrics: Arc<WorkflowMetrics>,
    /// Optional cost database for enhanced analysis
    #[cfg(feature = "database")]
    database: Option<Arc<CostDatabase>>,
    /// Aggregation configuration
    config: AggregationConfig,
}

impl CostAggregator {
    /// Create a new cost aggregator
    pub fn new(
        issue_storage: Arc<dyn IssueStorage + Send + Sync>,
        metrics: Arc<WorkflowMetrics>,
        #[cfg(feature = "database")]
        database: Option<Arc<CostDatabase>>,
        config: AggregationConfig,
    ) -> Self {
        Self {
            issue_storage,
            metrics,
            #[cfg(feature = "database")]
            database,
            config,
        }
    }

    /// Generate comprehensive project cost summary
    pub async fn generate_project_summary(
        &self,
        date_range: Option<DateRange>,
    ) -> AggregationResult<ProjectCostSummary> {
        if !self.config.enabled {
            return Err(AggregationError::Configuration(
                "Cost aggregation is disabled".to_string(),
            ));
        }

        let effective_range = date_range.unwrap_or_else(|| {
            let end = Utc::now();
            let start = end - chrono::Duration::days(self.config.trend_analysis_days as i64);
            DateRange::new(start, end)
        });

        // Collect cost data from multiple sources
        let issue_costs = self.collect_issue_costs(&effective_range).await?;

        if issue_costs.len() < self.config.min_issues_for_analysis {
            return Err(AggregationError::InsufficientData {
                message: format!(
                    "Need at least {} issues for analysis, found {}",
                    self.config.min_issues_for_analysis,
                    issue_costs.len()
                ),
            });
        }

        // Perform statistical analysis
        let total_cost = issue_costs.values().sum();
        let total_issues = issue_costs.len();
        let costs: Vec<Decimal> = issue_costs.values().cloned().collect();
        let average_cost_per_issue = total_cost / Decimal::from(total_issues);
        let median_cost_per_issue = self.calculate_median(&costs);

        // Generate trend analysis
        let cost_trend = self.analyze_cost_trends(&issue_costs, &effective_range).await?;

        // Calculate efficiency metrics
        let efficiency_metrics = self.calculate_efficiency_metrics(&issue_costs).await?;

        // Identify outliers
        let outliers = self.identify_outliers(&issue_costs)?;

        // Generate cost breakdown by category
        let cost_breakdown = self.generate_cost_breakdown(&issue_costs).await?;

        Ok(ProjectCostSummary {
            total_cost,
            total_issues,
            average_cost_per_issue,
            median_cost_per_issue,
            cost_trend,
            efficiency_metrics,
            period: effective_range,
            cost_breakdown,
            outliers,
            generated_at: Utc::now(),
        })
    }

    /// Collect cost data from all available sources
    async fn collect_issue_costs(
        &self,
        date_range: &DateRange,
    ) -> AggregationResult<HashMap<String, Decimal>> {
        let mut issue_costs = HashMap::new();

        // First, try to get data from database if available
        #[cfg(feature = "database")]
        if let Some(ref database) = self.database {
            match self.collect_database_costs(database, date_range).await {
                Ok(db_costs) => {
                    issue_costs.extend(db_costs);
                }
                Err(e) => {
                    tracing::warn!("Failed to collect costs from database: {}", e);
                }
            }
        }

        // Then, collect from issue markdown files and metrics
        let markdown_costs = self.collect_markdown_costs(date_range).await?;
        let metrics_costs = self.collect_metrics_costs(date_range).await?;

        // Merge data, preferring database data when available
        for (issue_id, cost) in markdown_costs {
            issue_costs.entry(issue_id).or_insert(cost);
        }

        for (issue_id, cost) in metrics_costs {
            issue_costs.entry(issue_id).or_insert(cost);
        }

        // Limit the number of issues to prevent memory issues
        if issue_costs.len() > MAX_ISSUES_PER_AGGREGATION {
            tracing::warn!(
                "Too many issues for aggregation ({} > {}), truncating",
                issue_costs.len(),
                MAX_ISSUES_PER_AGGREGATION
            );

            let mut sorted_issues: Vec<_> = issue_costs.into_iter().collect();
            sorted_issues.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by cost descending
            sorted_issues.truncate(MAX_ISSUES_PER_AGGREGATION);
            issue_costs = sorted_issues.into_iter().collect();
        }

        Ok(issue_costs)
    }

    /// Collect cost data from the database
    #[cfg(feature = "database")]
    async fn collect_database_costs(
        &self,
        database: &CostDatabase,
        date_range: &DateRange,
    ) -> AggregationResult<HashMap<String, Decimal>> {
        let mut costs = HashMap::new();

        // Query database for issue cost summaries in the date range
        let summaries = database
            .get_issue_summaries_in_range(date_range.start, date_range.end)
            .await
            .map_err(|e| AggregationError::Database(e.to_string()))?;

        for summary in summaries {
            costs.insert(
                summary.issue_id,
                Decimal::try_from(summary.total_cost).unwrap_or(Decimal::ZERO),
            );
        }

        Ok(costs)
    }

    /// Collect cost data from issue markdown files
    async fn collect_markdown_costs(
        &self,
        _date_range: &DateRange,
    ) -> AggregationResult<HashMap<String, Decimal>> {
        let mut costs = HashMap::new();

        // Get all issues and filter for completed ones
        let issues = self
            .issue_storage
            .list_issues()
            .await
            .map_err(|e| AggregationError::IssueStorage(format!("Failed to list issues: {}", e)))?;

        for issue in issues {
            // Only process completed issues
            if issue.completed {
                if let Some(cost) = self.extract_cost_from_issue_content(&issue.content).await? {
                    let issue_id = format!("{:06}_{}", issue.number, issue.name);
                    costs.insert(issue_id, cost);
                }
            }
        }

        Ok(costs)
    }

    /// Collect cost data from workflow metrics
    async fn collect_metrics_costs(
        &self,
        _date_range: &DateRange,
    ) -> AggregationResult<HashMap<String, Decimal>> {
        let mut costs = HashMap::new();

        // Extract cost data from workflow metrics
        // For now, we'll extract from run metrics and global metrics
        for (run_id, run_metrics) in &self.metrics.run_metrics {
            if let Some(ref cost_metrics) = run_metrics.cost_metrics {
                // Use workflow name as issue identifier (simplified approach)
                costs.insert(
                    format!("{}_{}", run_metrics.workflow_name.as_str(), run_id),
                    cost_metrics.total_cost,
                );
            }
        }

        Ok(costs)
    }

    /// Extract cost information from issue markdown content
    async fn extract_cost_from_issue_content(
        &self,
        content: &str,
    ) -> AggregationResult<Option<Decimal>> {
        // Look for cost section in markdown
        if let Some(cost_section) = self.find_cost_section(content) {
            self.parse_cost_from_section(&cost_section)
                .map(Some)
                .ok_or_else(|| AggregationError::CostParsing {
                    issue_id: "unknown".to_string(),
                    message: "Failed to parse cost from section".to_string(),
                })
        } else {
            Ok(None)
        }
    }

    /// Find the cost section in issue markdown content
    fn find_cost_section<'a>(&self, content: &'a str) -> Option<&'a str> {
        // Look for patterns like "## Cost Analysis", "## Costs", etc.
        let cost_markers = ["## Cost Analysis", "## Costs", "## Cost Breakdown"];

        for marker in &cost_markers {
            if let Some(start) = content.find(marker) {
                let section_start = start;
                // Find the end of the section (next ## header or end of file)
                let remaining = &content[section_start..];
                if let Some(end) = remaining[marker.len()..].find("\n## ") {
                    return Some(&content[section_start..section_start + marker.len() + end]);
                } else {
                    return Some(&content[section_start..]);
                }
            }
        }

        None
    }

    /// Parse cost amount from a cost section
    fn parse_cost_from_section(&self, section: &str) -> Option<Decimal> {
        // Look for patterns like "$0.0123", "Total: $0.0456", "Cost: 0.0789"
        use regex::Regex;

        let patterns = [
            r"\$(\d+\.?\d*)",
            r"Total:?\s*\$?(\d+\.?\d*)",
            r"Cost:?\s*\$?(\d+\.?\d*)",
            r"USD:?\s*(\d+\.?\d*)",
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(section) {
                    if let Some(amount) = captures.get(1) {
                        if let Ok(decimal) = amount.as_str().parse::<Decimal>() {
                            return Some(decimal);
                        }
                    }
                }
            }
        }

        None
    }


    /// Calculate median of a list of decimals
    fn calculate_median(&self, values: &[Decimal]) -> Decimal {
        if values.is_empty() {
            return Decimal::ZERO;
        }

        let mut sorted = values.to_vec();
        sorted.sort();

        let len = sorted.len();
        if len % 2 == 0 {
            (sorted[len / 2 - 1] + sorted[len / 2]) / Decimal::from(2)
        } else {
            sorted[len / 2]
        }
    }

    /// Analyze cost trends over time
    async fn analyze_cost_trends(
        &self,
        _issue_costs: &HashMap<String, Decimal>,
        _date_range: &DateRange,
    ) -> AggregationResult<CostTrend> {
        // For now, return a basic trend analysis
        // In a real implementation, this would analyze temporal patterns
        Ok(CostTrend {
            daily_costs: Vec::new(),
            weekly_costs: Vec::new(),
            monthly_costs: Vec::new(),
            trend_direction: TrendDirection::Stable,
            growth_rate: 0.0,
            confidence: 0.5,
            moving_average: Vec::new(),
            seasonal_patterns: Vec::new(),
        })
    }

    /// Calculate efficiency metrics
    async fn calculate_efficiency_metrics(
        &self,
        _issue_costs: &HashMap<String, Decimal>,
    ) -> AggregationResult<EfficiencyMetrics> {
        Ok(EfficiencyMetrics {
            cost_per_api_call: Decimal::ZERO,
            cost_per_token: Decimal::ZERO,
            avg_session_duration_minutes: 0.0,
            cost_per_session: Decimal::ZERO,
            token_efficiency: 0.0,
            expensive_operations: Vec::new(),
            efficiency_score: 0.5,
        })
    }

    /// Identify cost outliers
    pub fn identify_outliers(
        &self,
        issue_costs: &HashMap<String, Decimal>,
    ) -> AggregationResult<Vec<IssueOutlier>> {
        let mut outliers = Vec::new();

        if issue_costs.len() < MIN_DATA_POINTS_FOR_STATISTICS {
            return Ok(outliers);
        }

        let costs: Vec<Decimal> = issue_costs.values().cloned().collect();
        let mean = costs.iter().sum::<Decimal>() / Decimal::from(costs.len());

        // Calculate standard deviation
        let variance = costs
            .iter()
            .map(|&cost| {
                let diff = cost - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(costs.len());

        let std_dev = if let Some(variance_f64) = variance.to_f64() {
            if let Ok(sqrt_result) = Decimal::try_from(variance_f64.sqrt()) {
                sqrt_result
            } else {
                Decimal::ZERO
            }
        } else {
            Decimal::ZERO
        };

        if std_dev > Decimal::ZERO {
            for (issue_id, &cost) in issue_costs {
                let z_score = (cost - mean) / std_dev;
                let z_score_f64 = z_score.to_f64().unwrap_or(0.0);

                if z_score_f64.abs() > self.config.outlier_threshold {
                    outliers.push(IssueOutlier {
                        issue_id: issue_id.clone(),
                        cost,
                        outlier_type: if z_score_f64 > 0.0 {
                            OutlierType::HighCost
                        } else {
                            OutlierType::UnusualTokenUsage
                        },
                        standard_deviations: z_score_f64,
                        reason: format!(
                            "Cost deviates {:.2} standard deviations from mean",
                            z_score_f64
                        ),
                    });
                }
            }
        }

        Ok(outliers)
    }

    /// Generate cost breakdown by category
    pub async fn generate_cost_breakdown(
        &self,
        issue_costs: &HashMap<String, Decimal>,
    ) -> AggregationResult<HashMap<String, Decimal>> {
        let mut breakdown = HashMap::new();

        // For now, just categorize by cost ranges
        let mut low_cost = Decimal::ZERO;
        let mut medium_cost = Decimal::ZERO;
        let mut high_cost = Decimal::ZERO;

        for &cost in issue_costs.values() {
            if cost < Decimal::new(1, 2) {
                // < $0.01
                low_cost += cost;
            } else if cost < Decimal::new(10, 2) {
                // < $0.10
                medium_cost += cost;
            } else {
                high_cost += cost;
            }
        }

        breakdown.insert("low_cost".to_string(), low_cost);
        breakdown.insert("medium_cost".to_string(), medium_cost);
        breakdown.insert("high_cost".to_string(), high_cost);

        Ok(breakdown)
    }
}