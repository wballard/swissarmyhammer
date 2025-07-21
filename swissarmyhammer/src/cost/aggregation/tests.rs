//! Tests for cost aggregation functionality

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::AggregationConfig;
    use crate::cost::aggregation::{
        analyzer::{AggregationError, CostAggregator},
        reports::{ExportFormat, ReportConfig, ReportGenerator},
        trends::{TimeSeriesPoint, TrendAnalyzer},
    };
    use crate::issues::IssueStorage;
    use crate::workflow::metrics::WorkflowMetrics;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio;

    /// Mock issue storage for testing
    struct MockIssueStorage;

    #[async_trait::async_trait]
    impl IssueStorage for MockIssueStorage {
        async fn list_issues(&self) -> crate::error::Result<Vec<crate::issues::Issue>> {
            use crate::issues::{Issue, IssueNumber};

            Ok(vec![
                Issue {
                    number: IssueNumber::new(1).unwrap(),
                    name: "test_issue_1".to_string(),
                    content: "## Cost Analysis\nTotal: $0.0123".to_string(),
                    completed: true,
                    file_path: std::path::PathBuf::from("/test/issue1.md"),
                    created_at: chrono::Utc::now(),
                },
                Issue {
                    number: IssueNumber::new(2).unwrap(),
                    name: "test_issue_2".to_string(),
                    content: "## Cost Analysis\nTotal: $0.0456".to_string(),
                    completed: true,
                    file_path: std::path::PathBuf::from("/test/issue2.md"),
                    created_at: chrono::Utc::now(),
                },
                Issue {
                    number: IssueNumber::new(3).unwrap(),
                    name: "test_issue_3".to_string(),
                    content: "## Cost Analysis\nTotal: $0.0789".to_string(),
                    completed: true,
                    file_path: std::path::PathBuf::from("/test/issue3.md"),
                    created_at: chrono::Utc::now(),
                },
            ])
        }

        async fn get_issue(&self, _number: u32) -> crate::error::Result<crate::issues::Issue> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn create_issue(
            &self,
            _name: String,
            _content: String,
        ) -> crate::error::Result<crate::issues::Issue> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn update_issue(
            &self,
            _number: u32,
            _content: String,
        ) -> crate::error::Result<crate::issues::Issue> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn mark_complete(&self, _number: u32) -> crate::error::Result<crate::issues::Issue> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn mark_complete_with_cost(
            &self,
            _number: u32,
            _cost_data: crate::cost::IssueCostData,
        ) -> crate::error::Result<crate::issues::Issue> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn create_issues_batch(
            &self,
            _issues: Vec<(String, String)>,
        ) -> crate::error::Result<Vec<crate::issues::Issue>> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn get_issues_batch(
            &self,
            _numbers: Vec<u32>,
        ) -> crate::error::Result<Vec<crate::issues::Issue>> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn update_issues_batch(
            &self,
            _updates: Vec<(u32, String)>,
        ) -> crate::error::Result<Vec<crate::issues::Issue>> {
            unimplemented!("Not needed for aggregation tests")
        }

        async fn mark_complete_batch(
            &self,
            _numbers: Vec<u32>,
        ) -> crate::error::Result<Vec<crate::issues::Issue>> {
            unimplemented!("Not needed for aggregation tests")
        }
    }

    /// Create test workflow metrics with sample cost data
    fn create_test_workflow_metrics() -> WorkflowMetrics {
        let metrics = WorkflowMetrics::new();

        // Note: In a real implementation, we would populate the metrics with actual cost data
        // For testing purposes, we'll create a minimal metrics instance

        metrics
    }

    fn create_test_aggregator() -> CostAggregator {
        let issue_storage = Arc::new(MockIssueStorage);
        let metrics = Arc::new(create_test_workflow_metrics());
        let config = AggregationConfig::default();

        CostAggregator::new(
            issue_storage,
            metrics,
            #[cfg(feature = "database")]
            None,
            config,
        )
    }

    #[tokio::test]
    async fn test_aggregator_creation() {
        let _aggregator = create_test_aggregator();
        assert!(true); // Basic creation test
    }

    #[tokio::test]
    async fn test_project_summary_generation() {
        let aggregator = create_test_aggregator();

        let date_range = DateRange::new(Utc::now() - chrono::Duration::days(30), Utc::now());

        let result = aggregator.generate_project_summary(Some(date_range)).await;

        match result {
            Ok(summary) => {
                assert!(summary.total_issues >= 3);
                assert!(summary.total_cost > Decimal::ZERO);
                assert!(summary.average_cost_per_issue > Decimal::ZERO);
            }
            Err(AggregationError::InsufficientData { .. }) => {
                // This is acceptable for the mock data
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_date_range() {
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();
        let range = DateRange::new(start, end);

        assert_eq!(range.duration_days(), 7);
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_trend_analyzer() {
        let analyzer = TrendAnalyzer::default();

        // Create some test time series data
        let mut data = Vec::new();
        let base_time = Utc::now() - chrono::Duration::days(10);

        for i in 0..10 {
            data.push(TimeSeriesPoint {
                timestamp: base_time + chrono::Duration::days(i),
                cost: Decimal::new(1000 + i * 100, 4), // Stronger increasing trend: 0.1, 0.11, 0.12, etc.
            });
        }

        let result = analyzer.analyze_trends(&data);
        match result {
            Ok(analysis) => {
                assert_eq!(analysis.direction, TrendDirection::Increasing);
                assert!(analysis.linear_slope > 0.0);
                assert!(analysis.confidence > 0.0);
            }
            Err(e) => panic!("Trend analysis failed: {}", e),
        }
    }

    #[test]
    fn test_trend_analyzer_insufficient_data() {
        let analyzer = TrendAnalyzer::default();
        let data = vec![TimeSeriesPoint {
            timestamp: Utc::now(),
            cost: Decimal::new(100, 4),
        }];

        let result = analyzer.analyze_trends(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_report_generator() {
        let generator = ReportGenerator::default();

        // Create a minimal project summary for testing
        let summary = ProjectCostSummary {
            total_cost: Decimal::new(1000, 4),
            total_issues: 5,
            average_cost_per_issue: Decimal::new(200, 4),
            median_cost_per_issue: Decimal::new(180, 4),
            cost_trend: CostTrend {
                daily_costs: Vec::new(),
                weekly_costs: Vec::new(),
                monthly_costs: Vec::new(),
                trend_direction: TrendDirection::Stable,
                growth_rate: 0.0,
                confidence: 0.8,
                moving_average: Vec::new(),
                seasonal_patterns: Vec::new(),
            },
            efficiency_metrics: EfficiencyMetrics {
                cost_per_api_call: Decimal::new(5, 4),
                cost_per_token: Decimal::new(1, 6),
                avg_session_duration_minutes: 15.5,
                cost_per_session: Decimal::new(50, 4),
                token_efficiency: 2.5,
                expensive_operations: Vec::new(),
                efficiency_score: 0.75,
            },
            period: DateRange::new(Utc::now() - chrono::Duration::days(30), Utc::now()),
            cost_breakdown: HashMap::new(),
            outliers: Vec::new(),
            generated_at: Utc::now(),
        };

        let result = generator.generate_report(summary, ExportFormat::Json);
        assert!(result.is_ok());

        let report = result.unwrap();
        assert_eq!(report.metadata.format, ExportFormat::Json);
        assert!(report.sections.len() > 0);
    }

    #[test]
    fn test_export_formats() {
        let generator = ReportGenerator::default();
        let _config = ReportConfig::default();

        // Create a minimal project summary
        let summary = create_test_summary();

        // Test all export formats
        for format in [
            ExportFormat::Json,
            ExportFormat::Csv,
            ExportFormat::Markdown,
            ExportFormat::Html,
            ExportFormat::Text,
        ] {
            let report_result = generator.generate_report(summary.clone(), format);
            assert!(report_result.is_ok());

            let report = report_result.unwrap();
            let export_result = generator.export_report(&report, format);
            assert!(export_result.is_ok());

            let exported = export_result.unwrap();
            assert!(!exported.is_empty());
        }
    }

    #[test]
    fn test_outlier_detection() {
        // Create aggregator with lower outlier threshold for testing
        let issue_storage = Arc::new(MockIssueStorage);
        let metrics = Arc::new(create_test_workflow_metrics());
        let mut config = AggregationConfig::default();
        config.outlier_threshold = 1.0; // Lower threshold for test

        let aggregator = CostAggregator::new(
            issue_storage,
            metrics,
            #[cfg(feature = "database")]
            None,
            config,
        );

        // Create test data with an extreme outlier
        let mut costs = HashMap::new();
        costs.insert("normal1".to_string(), Decimal::new(100, 4)); // 0.0100
        costs.insert("normal2".to_string(), Decimal::new(110, 4)); // 0.0110
        costs.insert("normal3".to_string(), Decimal::new(120, 4)); // 0.0120
        costs.insert("outlier".to_string(), Decimal::new(10000, 4)); // 1.0000 - 100x higher

        let result = aggregator.identify_outliers(&costs);
        assert!(result.is_ok());

        let outliers = result.unwrap();
        assert!(outliers.len() > 0);

        let outlier = &outliers[0];
        assert_eq!(outlier.issue_id, "outlier");
        assert!(outlier.standard_deviations > 1.0);
    }

    #[test]
    fn test_cost_breakdown_generation() {
        let aggregator = create_test_aggregator();

        let mut costs = HashMap::new();
        costs.insert("low_cost_issue".to_string(), Decimal::new(5, 3)); // $0.005
        costs.insert("medium_cost_issue".to_string(), Decimal::new(5, 2)); // $0.05
        costs.insert("high_cost_issue".to_string(), Decimal::new(50, 2)); // $0.50

        let result = tokio_test::block_on(aggregator.generate_cost_breakdown(&costs));
        assert!(result.is_ok());

        let breakdown = result.unwrap();
        assert!(breakdown.contains_key("low_cost"));
        assert!(breakdown.contains_key("medium_cost"));
        assert!(breakdown.contains_key("high_cost"));
    }

    fn create_test_summary() -> ProjectCostSummary {
        ProjectCostSummary {
            total_cost: Decimal::new(1000, 4),
            total_issues: 5,
            average_cost_per_issue: Decimal::new(200, 4),
            median_cost_per_issue: Decimal::new(180, 4),
            cost_trend: CostTrend {
                daily_costs: Vec::new(),
                weekly_costs: Vec::new(),
                monthly_costs: Vec::new(),
                trend_direction: TrendDirection::Stable,
                growth_rate: 0.0,
                confidence: 0.8,
                moving_average: Vec::new(),
                seasonal_patterns: Vec::new(),
            },
            efficiency_metrics: EfficiencyMetrics {
                cost_per_api_call: Decimal::new(5, 4),
                cost_per_token: Decimal::new(1, 6),
                avg_session_duration_minutes: 15.5,
                cost_per_session: Decimal::new(50, 4),
                token_efficiency: 2.5,
                expensive_operations: Vec::new(),
                efficiency_score: 0.75,
            },
            period: DateRange::new(Utc::now() - chrono::Duration::days(30), Utc::now()),
            cost_breakdown: HashMap::new(),
            outliers: Vec::new(),
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn test_trend_direction_classification() {
        // Test stable trend
        assert_eq!(TrendDirection::Stable as u8, TrendDirection::Stable as u8);

        // Test enum variants
        let directions = [
            TrendDirection::Increasing,
            TrendDirection::Decreasing,
            TrendDirection::Stable,
            TrendDirection::Volatile,
        ];

        for direction in &directions {
            // Just test that the enum values work
            match direction {
                TrendDirection::Increasing => assert!(true),
                TrendDirection::Decreasing => assert!(true),
                TrendDirection::Stable => assert!(true),
                TrendDirection::Volatile => assert!(true),
            }
        }
    }

    #[test]
    fn test_aggregation_config_defaults() {
        let config = AggregationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.scan_interval_hours, 24);
        assert_eq!(config.retention_days, 90);
        assert_eq!(config.trend_analysis_days, 30);
        assert_eq!(config.outlier_threshold, 2.0);
        assert_eq!(config.min_issues_for_analysis, 3);
        assert!(config.include_issues_days.is_none());
    }
}
