//! Tests for cost metrics integration with workflow metrics

use super::super::metrics::*;
use super::cost::*;
use crate::cost::CostSessionId;
use crate::workflow::{WorkflowName, WorkflowRunId, WorkflowRunStatus};
use rust_decimal::Decimal;
use std::time::Duration;

#[cfg(test)]
mod tests {
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

        // Create a test action breakdown
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
    fn test_cost_metrics_efficiency_calculations() {
        let session_id = CostSessionId::new();
        let cost_metrics = create_test_cost_metrics(
            session_id,
            Decimal::new(150, 4), // $0.0150
            1000,                 // input tokens
            500,                  // output tokens
            3,                    // api calls
        );

        // Test cost per token calculation
        let cost_per_token = cost_metrics.cost_per_token().unwrap();
        // $0.0150 / 1500 tokens = $0.00001 per token
        assert_eq!(cost_per_token, Decimal::new(1, 5)); // $0.00001 per token

        // Test token efficiency ratio
        let efficiency_ratio = cost_metrics.token_efficiency_ratio().unwrap();
        assert_eq!(efficiency_ratio, 0.5); // 500/1000

        // Test average cost per call
        let avg_cost_per_call = cost_metrics.average_cost_per_call().unwrap();
        assert_eq!(avg_cost_per_call, Decimal::new(50, 4)); // $0.0050 per call
    }

    #[test]
    fn test_cost_attribution() {
        let session_id = CostSessionId::new();
        let mut cost_metrics = CostMetrics::new(session_id);

        // Add multiple actions with different costs
        let mut action1 = ActionCostBreakdown::new("action1".to_string());
        action1.cost = Decimal::new(300, 4); // $0.03
        action1.input_tokens = 200;
        action1.output_tokens = 100;

        let mut action2 = ActionCostBreakdown::new("action2".to_string());
        action2.cost = Decimal::new(700, 4); // $0.07
        action2.input_tokens = 500;
        action2.output_tokens = 300;

        cost_metrics.add_action_cost("action1".to_string(), action1);
        cost_metrics.add_action_cost("action2".to_string(), action2);
        cost_metrics.complete();

        // Test cost attribution percentages
        let attribution = cost_metrics.cost_attribution();
        assert_eq!(attribution.get("action1"), Some(&30.0)); // 30% of total cost
        assert_eq!(attribution.get("action2"), Some(&70.0)); // 70% of total cost

        // Test most expensive action
        let (most_expensive, _) = cost_metrics.most_expensive_action().unwrap();
        assert_eq!(most_expensive, "action2");

        // Test most token-intensive action
        let (most_token_intensive, _) = cost_metrics.most_token_intensive_action().unwrap();
        assert_eq!(most_token_intensive, "action2");
    }

    #[test]
    fn test_workflow_metrics_cost_integration() {
        let mut metrics = WorkflowMetrics::new();
        let run_id = WorkflowRunId::new();
        let workflow_name = WorkflowName::new("test_workflow");
        let session_id = CostSessionId::new();

        // Start a workflow run
        metrics.start_run(run_id, workflow_name.clone());

        // Start cost tracking
        metrics.start_cost_tracking(&run_id, session_id);

        // Add some cost data
        let cost_metrics = create_test_cost_metrics(
            session_id,
            Decimal::new(250, 4), // $0.025
            800,                  // input tokens
            400,                  // output tokens
            2,                    // api calls
        );

        metrics.update_cost_metrics(&run_id, cost_metrics);

        // Verify cost metrics are stored in run metrics
        let run_metrics = metrics.get_run_metrics(&run_id).unwrap();
        assert!(run_metrics.cost_metrics.is_some());

        let stored_cost_metrics = run_metrics.cost_metrics.as_ref().unwrap();
        assert_eq!(stored_cost_metrics.total_cost, Decimal::new(250, 4));
        assert_eq!(stored_cost_metrics.total_tokens(), 1200);
    }

    #[test]
    fn test_workflow_summary_cost_aggregation() {
        let mut metrics = WorkflowMetrics::new();
        let workflow_name = WorkflowName::new("test_workflow");

        // Create multiple runs with different costs
        let test_data = vec![
            (Decimal::new(100, 4), 500, 250, 1), // $0.01, 750 tokens, 1 api call
            (Decimal::new(200, 4), 800, 400, 2), // $0.02, 1200 tokens, 2 api calls
            (Decimal::new(150, 4), 600, 300, 1), // $0.015, 900 tokens, 1 api call
        ];

        for (cost, input_tokens, output_tokens, api_calls) in test_data {
            let run_id = WorkflowRunId::new();
            let session_id = CostSessionId::new();

            metrics.start_run(run_id, workflow_name.clone());
            metrics.start_cost_tracking(&run_id, session_id);

            let cost_metrics =
                create_test_cost_metrics(session_id, cost, input_tokens, output_tokens, api_calls);

            metrics.update_cost_metrics(&run_id, cost_metrics);
            metrics.complete_run(
                &run_id,
                WorkflowRunStatus::Completed,
                Duration::from_secs(10),
            );
        }

        // Check workflow summary cost aggregation
        let workflow_summary = metrics.get_workflow_metrics(&workflow_name).unwrap();
        assert!(workflow_summary.cost_summary.is_some());

        let cost_summary = workflow_summary.cost_summary.as_ref().unwrap();
        assert_eq!(cost_summary.total_cost, Decimal::new(450, 4)); // Sum of all costs
        assert_eq!(cost_summary.average_cost, Decimal::new(150, 4)); // Average cost
        assert_eq!(cost_summary.min_cost, Decimal::new(100, 4)); // Min cost
        assert_eq!(cost_summary.max_cost, Decimal::new(200, 4)); // Max cost
        assert_eq!(cost_summary.total_tokens, 2850); // Sum of all tokens
        assert_eq!(cost_summary.average_tokens, 950); // Average tokens
    }

    #[test]
    fn test_global_cost_metrics() {
        let mut metrics = WorkflowMetrics::new();

        // Create runs across multiple workflows
        let workflows = vec![
            ("workflow1", Decimal::new(300, 4), 1000, 500),
            ("workflow2", Decimal::new(200, 4), 800, 400),
            ("workflow1", Decimal::new(250, 4), 900, 450),
        ];

        for (workflow_name, cost, input_tokens, output_tokens) in workflows {
            let run_id = WorkflowRunId::new();
            let session_id = CostSessionId::new();
            let wf_name = WorkflowName::new(workflow_name);

            metrics.start_run(run_id, wf_name);
            metrics.start_cost_tracking(&run_id, session_id);

            let cost_metrics =
                create_test_cost_metrics(session_id, cost, input_tokens, output_tokens, 2);

            metrics.update_cost_metrics(&run_id, cost_metrics);
            metrics.complete_run(
                &run_id,
                WorkflowRunStatus::Completed,
                Duration::from_secs(5),
            );
        }

        // Check global cost metrics
        let global_metrics = metrics.get_global_metrics();
        assert!(global_metrics.cost_metrics.is_some());

        let global_cost = global_metrics.cost_metrics.as_ref().unwrap();
        assert_eq!(global_cost.total_system_cost, Decimal::new(750, 4)); // Sum of all costs
        assert_eq!(global_cost.average_cost_per_run, Decimal::new(250, 4)); // Average per run
        assert_eq!(global_cost.total_tokens, 4050); // Sum of all tokens
        assert_eq!(global_cost.average_tokens_per_run, 1350); // Average tokens per run
        assert_eq!(global_cost.total_api_calls, 6); // 3 runs * 2 api calls each
    }

    #[test]
    fn test_cost_trends_integration() {
        let mut metrics = WorkflowMetrics::new();
        let run_id = WorkflowRunId::new();
        let workflow_name = WorkflowName::new("test_workflow");
        let session_id = CostSessionId::new();

        metrics.start_run(run_id, workflow_name);
        metrics.start_cost_tracking(&run_id, session_id);

        let cost_metrics = create_test_cost_metrics(
            session_id,
            Decimal::new(175, 4), // $0.0175
            700,                  // input tokens
            350,                  // output tokens
            2,                    // api calls
        );

        // Update cost metrics should trigger trend updates
        metrics.update_cost_metrics(&run_id, cost_metrics.clone());

        // Check that trends were updated
        let global_metrics = metrics.get_global_metrics();
        assert_eq!(global_metrics.resource_trends.cost_trend.len(), 1);
        assert_eq!(
            global_metrics.resource_trends.token_efficiency_trend.len(),
            1
        );
        assert_eq!(
            global_metrics.resource_trends.avg_cost_per_call_trend.len(),
            1
        );

        // Verify trend values
        let cost_trend_value = global_metrics.resource_trends.cost_trend[0].1;
        assert_eq!(cost_trend_value, Decimal::new(175, 4));

        let efficiency_trend_value = global_metrics.resource_trends.token_efficiency_trend[0].1;
        assert_eq!(efficiency_trend_value, 0.5); // 350/700

        let avg_cost_trend_value = global_metrics.resource_trends.avg_cost_per_call_trend[0].1;
        assert_eq!(avg_cost_trend_value, Decimal::new(875, 5)); // 0.0175/2
    }

    #[test]
    fn test_cost_metrics_with_zero_values() {
        let mut metrics = WorkflowMetrics::new();
        let run_id = WorkflowRunId::new();
        let workflow_name = WorkflowName::new("test_workflow");
        let session_id = CostSessionId::new();

        metrics.start_run(run_id, workflow_name);
        metrics.start_cost_tracking(&run_id, session_id);

        // Create cost metrics with zero values to test edge cases
        let cost_metrics = create_test_cost_metrics(
            session_id,
            Decimal::ZERO, // $0.00
            0,             // no tokens
            0,             // no output tokens
            0,             // no api calls
        );

        metrics.update_cost_metrics(&run_id, cost_metrics);

        let run_metrics = metrics.get_run_metrics(&run_id).unwrap();
        let stored_cost_metrics = run_metrics.cost_metrics.as_ref().unwrap();

        // Test that efficiency calculations handle zero gracefully
        assert!(stored_cost_metrics.cost_per_token().is_none());
        assert!(stored_cost_metrics.token_efficiency_ratio().is_none());
        assert!(stored_cost_metrics.average_cost_per_call().is_none());

        let attribution = stored_cost_metrics.cost_attribution();
        assert!(attribution.is_empty());
    }

    #[test]
    fn test_cost_summary_without_cost_data() {
        let mut metrics = WorkflowMetrics::new();
        let run_id = WorkflowRunId::new();
        let workflow_name = WorkflowName::new("test_workflow");

        // Start and complete a run without any cost tracking
        metrics.start_run(run_id, workflow_name.clone());
        metrics.complete_run(
            &run_id,
            WorkflowRunStatus::Completed,
            Duration::from_secs(5),
        );

        // Check that workflow summary handles missing cost data gracefully
        let workflow_summary = metrics.get_workflow_metrics(&workflow_name).unwrap();
        assert!(workflow_summary.cost_summary.is_none());

        // Check global metrics
        let global_metrics = metrics.get_global_metrics();
        assert!(global_metrics.cost_metrics.is_none());
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that existing metrics still work without cost data
        let mut metrics = WorkflowMetrics::new();
        let run_id = WorkflowRunId::new();
        let workflow_name = WorkflowName::new("test_workflow");

        metrics.start_run(run_id, workflow_name.clone());
        metrics.record_transition(&run_id);
        metrics.complete_run(
            &run_id,
            WorkflowRunStatus::Completed,
            Duration::from_secs(3),
        );

        let run_metrics = metrics.get_run_metrics(&run_id).unwrap();
        assert_eq!(run_metrics.transition_count, 1);
        assert_eq!(run_metrics.status, WorkflowRunStatus::Completed);
        assert!(run_metrics.cost_metrics.is_none()); // Should be None without cost tracking

        let workflow_summary = metrics.get_workflow_metrics(&workflow_name).unwrap();
        assert_eq!(workflow_summary.total_runs, 1);
        assert_eq!(workflow_summary.successful_runs, 1);
        assert!(workflow_summary.cost_summary.is_none()); // Should be None without cost data
    }
}
