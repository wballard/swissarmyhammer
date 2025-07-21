//! Workflow execution metrics collection
//!
//! This module provides comprehensive metrics tracking for workflow execution,
//! including timing, success/failure rates, and resource usage statistics.

pub mod cleanup;
pub mod cost;
pub mod trends;

#[cfg(test)]
pub mod cost_integration_tests;

#[cfg(test)]
pub mod performance_tests;

// Re-export from submodules
pub use cost::{ActionCostBreakdown, CostMetrics};
pub use trends::ResourceTrends;

use crate::cost::CostSessionId;
use crate::workflow::{StateId, WorkflowName, WorkflowRunId, WorkflowRunStatus};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::warn;

/// Configuration for workflow metrics collection and memory management
#[derive(Debug, Clone)]
pub struct WorkflowMetricsConfig {
    /// Maximum number of workflow metrics to keep in memory
    pub max_workflow_metrics: usize,
    /// Maximum number of state durations per run
    pub max_state_durations_per_run: usize,
    /// Maximum number of data points to keep in resource trends
    pub max_trend_data_points: usize,
    /// Maximum number of run metrics to keep in memory
    pub max_run_metrics: usize,
    /// Maximum age of completed runs before cleanup (in days)
    pub max_completed_run_age_days: i64,
    /// Maximum age of workflow summary metrics before cleanup (in days)
    pub max_workflow_summary_age_days: i64,
}

impl Default for WorkflowMetricsConfig {
    fn default() -> Self {
        Self {
            max_workflow_metrics: 100,
            max_state_durations_per_run: 50,
            max_trend_data_points: 100,
            max_run_metrics: 1000,
            max_completed_run_age_days: 7,
            max_workflow_summary_age_days: 30,
        }
    }
}

/// Metrics collector for workflow execution
#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
    /// Configuration for metrics collection
    pub config: WorkflowMetricsConfig,
    /// Metrics for individual workflow runs
    pub run_metrics: HashMap<WorkflowRunId, RunMetrics>,
    /// Aggregated metrics by workflow name
    pub workflow_metrics: HashMap<WorkflowName, WorkflowSummaryMetrics>,
    /// Global execution statistics
    pub global_metrics: GlobalMetrics,
}

/// Metrics for a single workflow run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMetrics {
    /// Unique run identifier
    pub run_id: WorkflowRunId,
    /// Name of the workflow
    pub workflow_name: WorkflowName,
    /// When the run started
    pub started_at: DateTime<Utc>,
    /// When the run completed (if it has)
    pub completed_at: Option<DateTime<Utc>>,
    /// Final status of the run
    pub status: WorkflowRunStatus,
    /// Number of state transitions executed
    pub transition_count: usize,
    /// Total execution duration
    pub duration: Option<Duration>,
    /// Total execution duration (alias for compatibility)
    pub total_duration: Option<Duration>,
    /// Error details if the run failed
    pub error_details: Option<String>,
    /// Time spent in each state with timestamps for LRU eviction
    pub state_durations: HashMap<StateId, (Duration, DateTime<Utc>)>,
    /// Memory usage metrics
    pub memory_metrics: Option<MemoryMetrics>,
    /// Cost tracking metrics
    pub cost_metrics: Option<CostMetrics>,
}

/// Memory usage tracking for a workflow run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Peak memory usage during execution
    pub peak_memory_bytes: u64,
    /// Initial memory usage at start
    pub initial_memory_bytes: u64,
    /// Final memory usage at completion
    pub final_memory_bytes: u64,
    /// Number of context variables
    pub context_variables_count: usize,
    /// Size of execution history
    pub history_size: usize,
}

/// Aggregated metrics for a workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummaryMetrics {
    /// Workflow name
    pub workflow_name: WorkflowName,
    /// Total number of runs
    pub total_runs: usize,
    /// Number of successful runs
    pub successful_runs: usize,
    /// Number of failed runs
    pub failed_runs: usize,
    /// Number of cancelled runs
    pub cancelled_runs: usize,
    /// Average execution duration
    pub average_duration: Option<Duration>,
    /// Minimum execution duration
    pub min_duration: Option<Duration>,
    /// Maximum execution duration
    pub max_duration: Option<Duration>,
    /// Average number of transitions
    pub average_transitions: f64,
    /// Most frequently executed states
    pub hot_states: Vec<StateExecutionCount>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// Cost summary for this workflow
    pub cost_summary: Option<WorkflowCostSummary>,
}

/// Cost summary for workflow runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCostSummary {
    /// Total cost across all runs
    pub total_cost: Decimal,
    /// Average cost per run
    pub average_cost: Decimal,
    /// Minimum cost per run
    pub min_cost: Decimal,
    /// Maximum cost per run
    pub max_cost: Decimal,
    /// Total number of tokens used
    pub total_tokens: u32,
    /// Average tokens per run
    pub average_tokens: u32,
    /// Cost trend over time
    pub cost_trend: Vec<(DateTime<Utc>, Decimal)>,
    /// Token efficiency trend over time
    pub efficiency_trend: Vec<(DateTime<Utc>, f64)>,
    /// Most expensive action across all runs
    pub most_expensive_action: Option<String>,
    /// Most token-intensive action across all runs
    pub most_token_intensive_action: Option<String>,
}

/// State execution count for hot state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateExecutionCount {
    /// State identifier
    pub state_id: StateId,
    /// Number of times this state was executed
    pub count: usize,
}

/// Global execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    /// Total number of workflow runs across all workflows
    pub total_runs: usize,
    /// Total number of successful runs
    pub total_successful_runs: usize,
    /// Total number of failed runs
    pub total_failed_runs: usize,
    /// Total number of cancelled runs
    pub total_cancelled_runs: usize,
    /// Average run duration across all workflows
    pub average_run_duration: Option<Duration>,
    /// System resource usage trends
    pub resource_trends: ResourceTrends,
    /// Global cost tracking metrics
    pub cost_metrics: Option<GlobalCostMetrics>,
}

/// Global cost tracking metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCostMetrics {
    /// Total system cost across all workflows
    pub total_system_cost: Decimal,
    /// Average cost per run across all workflows
    pub average_cost_per_run: Decimal,
    /// Total tokens used across all workflows
    pub total_tokens: u32,
    /// Average tokens per run across all workflows
    pub average_tokens_per_run: u32,
    /// System-wide cost efficiency (average output/input ratio)
    pub system_efficiency_ratio: f64,
    /// Total API calls across all workflows
    pub total_api_calls: usize,
    /// Average cost per API call across system
    pub average_cost_per_api_call: Decimal,
}

impl WorkflowMetrics {
    /// Create a new metrics collector with default configuration
    pub fn new() -> Self {
        Self::with_config(WorkflowMetricsConfig::default())
    }

    /// Create a new metrics collector with custom configuration
    pub fn with_config(config: WorkflowMetricsConfig) -> Self {
        Self {
            config,
            run_metrics: HashMap::new(),
            workflow_metrics: HashMap::new(),
            global_metrics: GlobalMetrics::new(),
        }
    }

    /// Start tracking a new workflow run
    pub fn start_run(&mut self, run_id: WorkflowRunId, workflow_name: WorkflowName) {
        // Validate inputs
        if !Self::is_valid_workflow_name(&workflow_name) {
            warn!(
                workflow_name = %workflow_name.as_str(),
                run_id = %run_id,
                "Attempted to start run with invalid workflow name, ignoring request"
            );
            return;
        }
        let run_metrics = RunMetrics {
            run_id,
            workflow_name,
            started_at: Utc::now(),
            completed_at: None,
            status: WorkflowRunStatus::Running,
            transition_count: 0,
            duration: None,
            total_duration: None,
            error_details: None,
            state_durations: HashMap::new(),
            memory_metrics: None,
            cost_metrics: None,
        };

        self.run_metrics.insert(run_id, run_metrics);

        // Enforce bounds checking - remove oldest run metrics if we exceed the limit
        if self.run_metrics.len() > self.config.max_run_metrics {
            self.cleanup_old_run_metrics();
        }

        self.update_global_metrics();
    }

    /// Record state execution time
    pub fn record_state_execution(
        &mut self,
        run_id: &WorkflowRunId,
        state_id: StateId,
        duration: Duration,
    ) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            // Keep only the most recent state durations to prevent unbounded growth
            if run_metrics.state_durations.len() >= self.config.max_state_durations_per_run {
                // Find and remove the least recently used (oldest timestamp) entry
                if let Some(lru_state) = run_metrics
                    .state_durations
                    .iter()
                    .min_by_key(|(_, (_, timestamp))| timestamp)
                    .map(|(state, _)| state.clone())
                {
                    run_metrics.state_durations.remove(&lru_state);
                }
            }

            let now = Utc::now();
            run_metrics
                .state_durations
                .insert(state_id, (duration, now));
            run_metrics.transition_count += 1;
            self.update_global_metrics();
        }
    }

    /// Record a state transition
    pub fn record_transition(&mut self, run_id: &WorkflowRunId) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.transition_count += 1;
        }
    }

    /// Complete a workflow run
    pub fn complete_run(
        &mut self,
        run_id: &WorkflowRunId,
        status: WorkflowRunStatus,
        total_duration: Duration,
    ) {
        let workflow_name = if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.completed_at = Some(Utc::now());
            run_metrics.status = status;
            run_metrics.duration = Some(total_duration);
            run_metrics.total_duration = Some(total_duration);
            run_metrics.workflow_name.clone()
        } else {
            return;
        };

        self.update_workflow_summary_metrics(&workflow_name);
        self.update_global_metrics();
    }

    /// Complete a workflow run with error details
    pub fn complete_run_with_error(
        &mut self,
        run_id: &WorkflowRunId,
        status: WorkflowRunStatus,
        total_duration: Duration,
        error_details: Option<String>,
    ) {
        let workflow_name = if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.completed_at = Some(Utc::now());
            run_metrics.status = status;
            run_metrics.duration = Some(total_duration);
            run_metrics.total_duration = Some(total_duration);
            run_metrics.error_details = error_details;
            run_metrics.workflow_name.clone()
        } else {
            return;
        };

        self.update_workflow_summary_metrics(&workflow_name);
        self.update_global_metrics();
    }

    /// Start cost tracking for a run
    pub fn start_cost_tracking(&mut self, run_id: &WorkflowRunId, session_id: CostSessionId) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.cost_metrics = Some(CostMetrics::new(session_id));
        }
    }

    /// Complete cost tracking for a run
    pub fn complete_cost_tracking(&mut self, run_id: &WorkflowRunId) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            if let Some(cost_metrics) = &mut run_metrics.cost_metrics {
                cost_metrics.complete();
            }
        }
    }

    /// Update cost metrics for a run
    pub fn update_cost_metrics(&mut self, run_id: &WorkflowRunId, cost_metrics: CostMetrics) {
        // Update cost trends in global resource trends before moving the cost_metrics
        self.update_cost_trends(&cost_metrics);

        // Now get the mutable reference to run_metrics and update it
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.cost_metrics = Some(cost_metrics);
        }
    }

    /// Update cost trends in global resource trends
    pub fn update_cost_trends(&mut self, cost_metrics: &CostMetrics) {
        // Add cost trend point
        self.global_metrics
            .resource_trends
            .add_cost_point(cost_metrics.total_cost, self.config.max_trend_data_points);

        // Add token efficiency trend point if available
        if let Some(efficiency) = cost_metrics.token_efficiency_ratio() {
            self.global_metrics
                .resource_trends
                .add_token_efficiency_point(efficiency, self.config.max_trend_data_points);
        }

        // Add average cost per call trend point if available
        if let Some(avg_cost) = cost_metrics.average_cost_per_call() {
            self.global_metrics
                .resource_trends
                .add_avg_cost_per_call_point(avg_cost, self.config.max_trend_data_points);
        }
    }

    /// Add action cost to a run's cost metrics
    pub fn add_action_cost(
        &mut self,
        run_id: &WorkflowRunId,
        action_name: String,
        breakdown: ActionCostBreakdown,
    ) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            if let Some(cost_metrics) = &mut run_metrics.cost_metrics {
                cost_metrics.add_action_cost(action_name, breakdown);
            }
        }
    }

    /// Update memory metrics for a specific run
    pub fn update_memory_metrics(&mut self, run_id: &WorkflowRunId, memory_metrics: MemoryMetrics) {
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.memory_metrics = Some(memory_metrics);
        }
    }

    /// Get metrics for a specific run
    pub fn get_run_metrics(&self, run_id: &WorkflowRunId) -> Option<&RunMetrics> {
        self.run_metrics.get(run_id)
    }

    /// Get aggregated metrics for a workflow
    pub fn get_workflow_metrics(
        &self,
        workflow_name: &WorkflowName,
    ) -> Option<&WorkflowSummaryMetrics> {
        self.workflow_metrics.get(workflow_name)
    }

    /// Get global metrics
    pub fn get_global_metrics(&self) -> &GlobalMetrics {
        &self.global_metrics
    }

    /// Update workflow summary metrics
    /// Calculate duration statistics for a set of runs
    fn calculate_duration_stats(
        &self,
        runs: &[&RunMetrics],
    ) -> (Option<Duration>, Option<Duration>, Option<Duration>, f64) {
        let durations: Vec<_> = runs.iter().filter_map(|run| run.duration).collect();
        let average_duration = if !durations.is_empty() {
            let total_nanos: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
            Some(Duration::from_nanos(total_nanos / durations.len() as u64))
        } else {
            None
        };

        let min_duration = durations.iter().min().copied();
        let max_duration = durations.iter().max().copied();

        let average_transitions = if !runs.is_empty() {
            runs.iter().map(|run| run.transition_count).sum::<usize>() as f64 / runs.len() as f64
        } else {
            0.0
        };

        (
            average_duration,
            min_duration,
            max_duration,
            average_transitions,
        )
    }

    /// Calculate hot states (most frequently executed states) from run data
    fn calculate_hot_states(&self, runs: &[&RunMetrics]) -> Vec<StateExecutionCount> {
        let mut state_counts: HashMap<StateId, usize> = HashMap::new();
        for run in runs {
            for state_id in run.state_durations.keys() {
                *state_counts.entry(state_id.clone()).or_insert(0) += 1;
            }
        }

        let mut hot_states: Vec<StateExecutionCount> = state_counts
            .into_iter()
            .map(|(state_id, count)| StateExecutionCount { state_id, count })
            .collect();
        hot_states.sort_by(|a, b| b.count.cmp(&a.count));
        hot_states.truncate(10); // Keep only top 10

        hot_states
    }

    /// Enforce workflow metrics limits by removing oldest entries
    fn enforce_workflow_metrics_limits(&mut self) {
        if self.workflow_metrics.len() > self.config.max_workflow_metrics {
            let oldest_workflow = self
                .workflow_metrics
                .iter()
                .min_by_key(|(_, metrics)| metrics.last_updated)
                .map(|(name, _)| name.clone());

            if let Some(oldest) = oldest_workflow {
                self.workflow_metrics.remove(&oldest);
            }
        }
    }

    fn update_workflow_summary_metrics(&mut self, workflow_name: &WorkflowName) {
        let runs: Vec<_> = self
            .run_metrics
            .values()
            .filter(|run| &run.workflow_name == workflow_name)
            .collect();

        if runs.is_empty() {
            return;
        }

        let total_runs = runs.len();
        let successful_runs = runs
            .iter()
            .filter(|run| run.status == WorkflowRunStatus::Completed)
            .count();
        let failed_runs = runs
            .iter()
            .filter(|run| run.status == WorkflowRunStatus::Failed)
            .count();
        let cancelled_runs = runs
            .iter()
            .filter(|run| run.status == WorkflowRunStatus::Cancelled)
            .count();

        let (average_duration, min_duration, max_duration, average_transitions) =
            self.calculate_duration_stats(&runs);

        let hot_states = self.calculate_hot_states(&runs);

        let cost_summary = self.calculate_workflow_cost_summary(&runs);

        let summary = WorkflowSummaryMetrics {
            workflow_name: workflow_name.clone(),
            total_runs,
            successful_runs,
            failed_runs,
            cancelled_runs,
            average_duration,
            min_duration,
            max_duration,
            average_transitions,
            hot_states,
            last_updated: Utc::now(),
            cost_summary,
        };

        self.workflow_metrics.insert(workflow_name.clone(), summary);

        self.enforce_workflow_metrics_limits();
    }

    /// Calculate basic cost statistics for workflow runs
    fn calculate_basic_cost_stats(
        &self,
        cost_runs: &[&CostMetrics],
    ) -> (Decimal, Decimal, Decimal, Decimal, u32, u32) {
        let total_cost = cost_runs.iter().map(|cm| cm.total_cost).sum::<Decimal>();
        let total_tokens = cost_runs.iter().map(|cm| cm.total_tokens()).sum::<u32>();

        let average_cost = if !cost_runs.is_empty() {
            total_cost / Decimal::from(cost_runs.len())
        } else {
            Decimal::ZERO
        };

        let average_tokens = if !cost_runs.is_empty() {
            total_tokens / cost_runs.len() as u32
        } else {
            0
        };

        let min_cost = cost_runs
            .iter()
            .map(|cm| cm.total_cost)
            .min()
            .unwrap_or(Decimal::ZERO);

        let max_cost = cost_runs
            .iter()
            .map(|cm| cm.total_cost)
            .max()
            .unwrap_or(Decimal::ZERO);

        (
            total_cost,
            average_cost,
            min_cost,
            max_cost,
            total_tokens,
            average_tokens,
        )
    }

    /// Build cost and efficiency trends from run data
    fn build_cost_trends(
        &self,
        runs: &[&RunMetrics],
    ) -> (Vec<(DateTime<Utc>, Decimal)>, Vec<(DateTime<Utc>, f64)>) {
        let cost_trend: Vec<_> = runs
            .iter()
            .filter_map(|run| {
                run.cost_metrics
                    .as_ref()
                    .and_then(|cm| run.completed_at.map(|completed| (completed, cm.total_cost)))
            })
            .collect();

        let efficiency_trend: Vec<_> = runs
            .iter()
            .filter_map(|run| {
                run.cost_metrics.as_ref().and_then(|cm| {
                    cm.token_efficiency_ratio()
                        .and_then(|ratio| run.completed_at.map(|completed| (completed, ratio)))
                })
            })
            .collect();

        (cost_trend, efficiency_trend)
    }

    /// Find the most expensive and most token-intensive actions
    fn find_most_expensive_actions(
        &self,
        cost_runs: &[&CostMetrics],
    ) -> (Option<String>, Option<String>) {
        let mut all_actions: HashMap<&str, (Decimal, u32)> = HashMap::new();
        for cost_metrics in cost_runs {
            for (action_name, breakdown) in &cost_metrics.cost_by_action {
                let entry = all_actions
                    .entry(action_name.as_str())
                    .or_insert((Decimal::ZERO, 0));
                entry.0 += breakdown.cost;
                entry.1 += breakdown.total_tokens();
            }
        }

        let most_expensive_action = all_actions
            .iter()
            .max_by(|a, b| a.1 .0.cmp(&b.1 .0))
            .map(|(name, _)| name.to_string());

        let most_token_intensive_action = all_actions
            .iter()
            .max_by_key(|&(_, (_, tokens))| tokens)
            .map(|(name, _)| name.to_string());

        (most_expensive_action, most_token_intensive_action)
    }

    /// Calculate cost summary for a set of workflow runs
    fn calculate_workflow_cost_summary(&self, runs: &[&RunMetrics]) -> Option<WorkflowCostSummary> {
        let cost_runs: Vec<_> = runs
            .iter()
            .filter_map(|run| run.cost_metrics.as_ref())
            .collect();

        if cost_runs.is_empty() {
            return None;
        }

        let (total_cost, average_cost, min_cost, max_cost, total_tokens, average_tokens) =
            self.calculate_basic_cost_stats(&cost_runs);

        let (cost_trend, efficiency_trend) = self.build_cost_trends(runs);

        let (most_expensive_action, most_token_intensive_action) =
            self.find_most_expensive_actions(&cost_runs);

        Some(WorkflowCostSummary {
            total_cost,
            average_cost,
            min_cost,
            max_cost,
            total_tokens,
            average_tokens,
            cost_trend,
            efficiency_trend,
            most_expensive_action,
            most_token_intensive_action,
        })
    }

    /// Update global metrics
    fn update_global_metrics(&mut self) {
        let total_runs = self.run_metrics.len();
        let total_successful_runs = self
            .run_metrics
            .values()
            .filter(|run| run.status == WorkflowRunStatus::Completed)
            .count();
        let total_failed_runs = self
            .run_metrics
            .values()
            .filter(|run| run.status == WorkflowRunStatus::Failed)
            .count();
        let total_cancelled_runs = self
            .run_metrics
            .values()
            .filter(|run| run.status == WorkflowRunStatus::Cancelled)
            .count();

        let completed_runs: Vec<_> = self
            .run_metrics
            .values()
            .filter_map(|run| run.duration)
            .collect();
        let average_run_duration = if !completed_runs.is_empty() {
            let total_nanos: u64 = completed_runs.iter().map(|d| d.as_nanos() as u64).sum();
            Some(Duration::from_nanos(
                total_nanos / completed_runs.len() as u64,
            ))
        } else {
            None
        };

        // Calculate global cost metrics
        let cost_metrics = self.calculate_global_cost_metrics();

        self.global_metrics = GlobalMetrics {
            total_runs,
            total_successful_runs,
            total_failed_runs,
            total_cancelled_runs,
            average_run_duration,
            resource_trends: self.global_metrics.resource_trends.clone(),
            cost_metrics,
        };
    }

    /// Calculate global cost metrics across all workflow runs
    fn calculate_global_cost_metrics(&self) -> Option<GlobalCostMetrics> {
        let cost_runs: Vec<_> = self
            .run_metrics
            .values()
            .filter_map(|run| run.cost_metrics.as_ref())
            .collect();

        if cost_runs.is_empty() {
            return None;
        }

        let total_system_cost = cost_runs.iter().map(|cm| cm.total_cost).sum::<Decimal>();
        let total_tokens = cost_runs.iter().map(|cm| cm.total_tokens()).sum::<u32>();
        let total_api_calls = cost_runs.iter().map(|cm| cm.api_call_count).sum::<usize>();

        let average_cost_per_run = if !cost_runs.is_empty() {
            total_system_cost / Decimal::from(cost_runs.len())
        } else {
            Decimal::ZERO
        };

        let average_tokens_per_run = if !cost_runs.is_empty() {
            total_tokens / cost_runs.len() as u32
        } else {
            0
        };

        let average_cost_per_api_call = if total_api_calls > 0 {
            total_system_cost / Decimal::from(total_api_calls)
        } else {
            Decimal::ZERO
        };

        // Calculate system-wide efficiency ratio
        let total_input_tokens = cost_runs
            .iter()
            .map(|cm| cm.total_input_tokens)
            .sum::<u32>();
        let total_output_tokens = cost_runs
            .iter()
            .map(|cm| cm.total_output_tokens)
            .sum::<u32>();

        let system_efficiency_ratio = if total_input_tokens > 0 {
            total_output_tokens as f64 / total_input_tokens as f64
        } else {
            0.0
        };

        Some(GlobalCostMetrics {
            total_system_cost,
            average_cost_per_run,
            total_tokens,
            average_tokens_per_run,
            system_efficiency_ratio,
            total_api_calls,
            average_cost_per_api_call,
        })
    }

    /// Validate workflow name
    fn is_valid_workflow_name(name: &WorkflowName) -> bool {
        !name.as_str().trim().is_empty()
    }
}

impl Default for WorkflowMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalMetrics {
    /// Create new global metrics
    pub fn new() -> Self {
        Self {
            total_runs: 0,
            total_successful_runs: 0,
            total_failed_runs: 0,
            total_cancelled_runs: 0,
            average_run_duration: None,
            resource_trends: ResourceTrends::new(),
            cost_metrics: None,
        }
    }
}

impl MemoryMetrics {
    /// Create new memory metrics
    pub fn new() -> Self {
        Self {
            peak_memory_bytes: 0,
            initial_memory_bytes: 0,
            final_memory_bytes: 0,
            context_variables_count: 0,
            history_size: 0,
        }
    }

    /// Update peak memory if current usage is higher
    pub fn update_peak_memory(&mut self, current_bytes: u64) {
        if current_bytes > self.peak_memory_bytes {
            self.peak_memory_bytes = current_bytes;
        }
    }

    /// Set final memory usage
    pub fn set_final_memory(&mut self, bytes: u64) {
        self.final_memory_bytes = bytes;
    }

    /// Calculate memory growth
    pub fn memory_growth(&self) -> i64 {
        self.final_memory_bytes as i64 - self.initial_memory_bytes as i64
    }

    /// Update memory metrics with current values
    pub fn update(&mut self, memory_bytes: u64, context_vars: usize, history_size: usize) {
        self.update_peak_memory(memory_bytes);
        self.context_variables_count = context_vars;
        self.history_size = history_size;
    }
}

impl Default for GlobalMetrics {
    fn default() -> Self {
        Self::new()
    }
}
