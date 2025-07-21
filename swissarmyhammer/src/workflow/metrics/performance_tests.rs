//! Performance tests for cost metrics integration

use super::super::metrics::*;
use super::cost::*;
use crate::cost::CostSessionId;
use crate::workflow::{WorkflowName, WorkflowRunId, WorkflowRunStatus};
use rust_decimal::Decimal;
use std::time::{Duration, Instant};

#[cfg(test)]
mod performance_tests {
    use super::*;

    /// Helper function to create test cost metrics
    fn create_test_cost_metrics(
        session_id: CostSessionId,
        total_cost: Decimal,
        input_tokens: u32,
        output_tokens: u32,
        api_calls: usize,
    ) -> CostMetrics {
        let mut cost_metrics = CostMetrics::new(session_id);

        let mut breakdown = ActionCostBreakdown::new("test_action".to_string());
        breakdown.cost = total_cost;
        breakdown.input_tokens = input_tokens;
        breakdown.output_tokens = output_tokens;
        breakdown.api_call_count = api_calls;

        cost_metrics.add_action_cost("test_action".to_string(), breakdown);
        cost_metrics.complete();

        cost_metrics
    }

    #[test]
    fn test_cost_metrics_performance() {
        const NUM_RUNS: usize = 1000;
        let mut metrics = WorkflowMetrics::new();
        let workflow_name = WorkflowName::new("performance_test_workflow");

        // Measure time for operations with cost tracking
        let start = Instant::now();

        for i in 0..NUM_RUNS {
            let run_id = WorkflowRunId::new();
            let session_id = CostSessionId::new();

            // Start workflow run
            metrics.start_run(run_id, workflow_name.clone());

            // Start cost tracking
            metrics.start_cost_tracking(&run_id, session_id);

            // Add cost metrics
            let cost_metrics = create_test_cost_metrics(
                session_id,
                Decimal::new((i + 1) as i64 * 100, 4), // Variable cost
                (i as u32 + 1) * 10,                   // Variable input tokens
                (i as u32 + 1) * 5,                    // Variable output tokens
                2,                                     // API calls
            );

            metrics.update_cost_metrics(&run_id, cost_metrics);

            // Complete the run
            metrics.complete_run(
                &run_id,
                WorkflowRunStatus::Completed,
                Duration::from_millis(i as u64 + 1),
            );
        }

        let elapsed_with_cost = start.elapsed();
        println!(
            "Time for {} runs with cost tracking: {:?}",
            NUM_RUNS, elapsed_with_cost
        );

        // Verify all metrics were processed
        assert_eq!(
            metrics.run_metrics.len(),
            NUM_RUNS.min(metrics.config.max_run_metrics)
        );

        let global_metrics = metrics.get_global_metrics();
        assert!(global_metrics.cost_metrics.is_some());

        let workflow_summary = metrics.get_workflow_metrics(&workflow_name).unwrap();
        assert!(workflow_summary.cost_summary.is_some());

        // Ensure performance is reasonable (should complete within 1 second for 1000 runs)
        assert!(
            elapsed_with_cost < Duration::from_secs(1),
            "Performance test took too long: {:?}",
            elapsed_with_cost
        );
    }

    #[test]
    fn test_baseline_performance_without_cost() {
        const NUM_RUNS: usize = 1000;
        let mut metrics = WorkflowMetrics::new();
        let workflow_name = WorkflowName::new("baseline_performance_test");

        // Measure time for operations without cost tracking
        let start = Instant::now();

        for i in 0..NUM_RUNS {
            let run_id = WorkflowRunId::new();

            // Start workflow run
            metrics.start_run(run_id, workflow_name.clone());

            // Complete the run without cost tracking
            metrics.complete_run(
                &run_id,
                WorkflowRunStatus::Completed,
                Duration::from_millis(i as u64 + 1),
            );
        }

        let elapsed_without_cost = start.elapsed();
        println!(
            "Time for {} runs without cost tracking: {:?}",
            NUM_RUNS, elapsed_without_cost
        );

        // Verify metrics were processed correctly
        let workflow_summary = metrics.get_workflow_metrics(&workflow_name).unwrap();
        assert!(workflow_summary.cost_summary.is_none()); // No cost data

        let global_metrics = metrics.get_global_metrics();
        assert!(global_metrics.cost_metrics.is_none()); // No cost data

        // Ensure performance is reasonable
        assert!(
            elapsed_without_cost < Duration::from_secs(1),
            "Baseline performance test took too long: {:?}",
            elapsed_without_cost
        );
    }

    #[test]
    fn test_cost_aggregation_performance() {
        const NUM_WORKFLOWS: usize = 50;
        const RUNS_PER_WORKFLOW: usize = 20;
        let mut metrics = WorkflowMetrics::new();

        let start = Instant::now();

        // Create multiple workflows with multiple runs each
        for workflow_idx in 0..NUM_WORKFLOWS {
            let workflow_name = WorkflowName::new(format!("workflow_{}", workflow_idx));

            for run_idx in 0..RUNS_PER_WORKFLOW {
                let run_id = WorkflowRunId::new();
                let session_id = CostSessionId::new();

                metrics.start_run(run_id, workflow_name.clone());
                metrics.start_cost_tracking(&run_id, session_id);

                let cost_metrics = create_test_cost_metrics(
                    session_id,
                    Decimal::new((run_idx + 1) as i64 * 50, 4),
                    (run_idx as u32 + 1) * 100,
                    (run_idx as u32 + 1) * 50,
                    3,
                );

                metrics.update_cost_metrics(&run_id, cost_metrics);
                metrics.complete_run(
                    &run_id,
                    WorkflowRunStatus::Completed,
                    Duration::from_millis(run_idx as u64 + 10),
                );
            }
        }

        let elapsed = start.elapsed();
        println!(
            "Time for {} workflows with {} runs each: {:?}",
            NUM_WORKFLOWS, RUNS_PER_WORKFLOW, elapsed
        );

        // Verify aggregation worked correctly
        assert!(!metrics.workflow_metrics.is_empty());
        let global_metrics = metrics.get_global_metrics();
        assert!(global_metrics.cost_metrics.is_some());

        // Check that cost trends were updated
        assert!(!global_metrics.resource_trends.cost_trend.is_empty());
        assert!(!global_metrics
            .resource_trends
            .token_efficiency_trend
            .is_empty());

        // Ensure aggregation performance is reasonable (should complete within 2 seconds)
        assert!(
            elapsed < Duration::from_secs(2),
            "Cost aggregation performance test took too long: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_cost_trend_memory_usage() {
        let mut metrics = WorkflowMetrics::new();
        let workflow_name = WorkflowName::new("trend_memory_test");

        // Add enough data points to test memory bounds
        for i in 0..super::super::trends::MAX_TREND_DATA_POINTS + 50 {
            let run_id = WorkflowRunId::new();
            let session_id = CostSessionId::new();

            metrics.start_run(run_id, workflow_name.clone());
            metrics.start_cost_tracking(&run_id, session_id);

            let cost_metrics =
                create_test_cost_metrics(session_id, Decimal::new(i as i64 * 10, 3), 100, 50, 1);

            metrics.update_cost_trends(&cost_metrics);
        }

        let global_metrics = metrics.get_global_metrics();

        // Verify that trend data is bounded to prevent memory issues
        assert!(
            global_metrics.resource_trends.cost_trend.len()
                <= super::super::trends::MAX_TREND_DATA_POINTS
        );
        assert!(
            global_metrics.resource_trends.token_efficiency_trend.len()
                <= super::super::trends::MAX_TREND_DATA_POINTS
        );
        assert!(
            global_metrics.resource_trends.avg_cost_per_call_trend.len()
                <= super::super::trends::MAX_TREND_DATA_POINTS
        );
    }
}
