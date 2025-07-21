//! Workflow execution metrics collection
//!
//! This module provides comprehensive metrics tracking for workflow execution,
//! including timing, success/failure rates, and resource usage statistics.

pub mod cleanup;
pub mod cost;
pub mod trends;

// Re-export from submodules
pub use cleanup::MAX_RUN_METRICS;
pub use cost::{ActionCostBreakdown, CostMetrics};
pub use trends::ResourceTrends;

use crate::cost::CostSessionId;
use crate::workflow::{StateId, WorkflowName, WorkflowRunId, WorkflowRunStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Maximum number of workflow metrics to keep in memory
pub const MAX_WORKFLOW_METRICS: usize = 100;

/// Maximum number of state durations per run
pub const MAX_STATE_DURATIONS_PER_RUN: usize = 50;

/// Metrics collector for workflow execution
#[derive(Debug, Clone)]
pub struct WorkflowMetrics {
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
    /// Time spent in each state
    pub state_durations: HashMap<StateId, Duration>,
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
}

impl WorkflowMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            run_metrics: HashMap::new(),
            workflow_metrics: HashMap::new(),
            global_metrics: GlobalMetrics::new(),
        }
    }

    /// Start tracking a new workflow run
    pub fn start_run(&mut self, run_id: WorkflowRunId, workflow_name: WorkflowName) {
        // Validate inputs
        if !Self::is_valid_workflow_name(&workflow_name) {
            return;
        }
        let run_metrics = RunMetrics {
            run_id,
            workflow_name: workflow_name.clone(),
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
        if self.run_metrics.len() > MAX_RUN_METRICS {
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
            if run_metrics.state_durations.len() >= MAX_STATE_DURATIONS_PER_RUN {
                // Remove a random old entry to make space
                if let Some(oldest_state) = run_metrics.state_durations.keys().next().cloned() {
                    run_metrics.state_durations.remove(&oldest_state);
                }
            }

            run_metrics.state_durations.insert(state_id, duration);
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
        if let Some(run_metrics) = self.run_metrics.get_mut(run_id) {
            run_metrics.cost_metrics = Some(cost_metrics);
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

        let durations: Vec<_> = runs.iter().filter_map(|run| run.duration).collect();
        let average_duration = if !durations.is_empty() {
            let total_nanos: u64 = durations.iter().map(|d| d.as_nanos() as u64).sum();
            Some(Duration::from_nanos(total_nanos / durations.len() as u64))
        } else {
            None
        };

        let min_duration = durations.iter().min().copied();
        let max_duration = durations.iter().max().copied();

        let average_transitions = if total_runs > 0 {
            runs.iter().map(|run| run.transition_count).sum::<usize>() as f64 / total_runs as f64
        } else {
            0.0
        };

        // Calculate hot states
        let mut state_counts: HashMap<StateId, usize> = HashMap::new();
        for run in &runs {
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
        };

        self.workflow_metrics.insert(workflow_name.clone(), summary);

        // Enforce workflow metrics limit
        if self.workflow_metrics.len() > MAX_WORKFLOW_METRICS {
            // Remove oldest workflow metrics
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

        self.global_metrics = GlobalMetrics {
            total_runs,
            total_successful_runs,
            total_failed_runs,
            total_cancelled_runs,
            average_run_duration,
            resource_trends: self.global_metrics.resource_trends.clone(),
        };
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
