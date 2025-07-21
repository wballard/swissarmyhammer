//! Cleanup logic for workflow metrics

use super::WorkflowMetrics;
use chrono::Utc;

/// Maximum number of run metrics to keep in memory
pub const MAX_RUN_METRICS: usize = 1000;

/// Maximum age of completed runs before cleanup (in days)
pub const MAX_COMPLETED_RUN_AGE_DAYS: i64 = 7;

/// Maximum age of workflow summary metrics before cleanup (in days)
pub const MAX_WORKFLOW_SUMMARY_AGE_DAYS: i64 = 30;

impl WorkflowMetrics {
    /// Clean up old run metrics when limit is exceeded
    pub(super) fn cleanup_old_run_metrics(&mut self) {
        // Find the oldest completed runs and remove them
        let mut completed_runs: Vec<_> = self
            .run_metrics
            .iter()
            .filter(|(_, run)| run.completed_at.is_some())
            .map(|(id, run)| (*id, run.completed_at.unwrap()))
            .collect();

        // Sort by completion time (oldest first)
        completed_runs.sort_by_key(|(_, completed_at)| *completed_at);

        // Remove the oldest runs to get back under the limit
        let excess_count = self.run_metrics.len().saturating_sub(MAX_RUN_METRICS);
        completed_runs
            .into_iter()
            .take(excess_count)
            .for_each(|(run_id, _)| {
                self.run_metrics.remove(&run_id);
            });
    }

    /// Comprehensive cleanup of old metrics data
    pub fn cleanup_old_metrics(&mut self) {
        let now = Utc::now();
        let mut removed_runs = 0;
        let mut removed_workflows = 0;

        // Clean up old completed runs
        let cutoff_date = now - chrono::Duration::days(MAX_COMPLETED_RUN_AGE_DAYS);
        let runs_to_remove: Vec<_> = self
            .run_metrics
            .iter()
            .filter(|(_, run)| {
                if let Some(completed_at) = run.completed_at {
                    completed_at < cutoff_date
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect();

        for run_id in runs_to_remove {
            self.run_metrics.remove(&run_id);
            removed_runs += 1;
        }

        // Clean up old workflow summary metrics
        let workflow_cutoff_date = now - chrono::Duration::days(MAX_WORKFLOW_SUMMARY_AGE_DAYS);
        let workflows_to_remove: Vec<_> = self
            .workflow_metrics
            .iter()
            .filter(|(_, summary)| summary.last_updated < workflow_cutoff_date)
            .map(|(name, _)| name.clone())
            .collect();

        for workflow_name in workflows_to_remove {
            self.workflow_metrics.remove(&workflow_name);
            removed_workflows += 1;
        }

        // Update global metrics after cleanup
        self.update_global_metrics();

        if removed_runs > 0 || removed_workflows > 0 {
            tracing::info!(
                "Metrics cleanup completed: removed {} old runs and {} old workflow summaries",
                removed_runs,
                removed_workflows
            );
        }
    }
}
