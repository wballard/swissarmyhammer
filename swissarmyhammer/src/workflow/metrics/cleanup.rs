//! Cleanup logic for workflow metrics

use super::WorkflowMetrics;
use chrono::Utc;

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
        let excess_count = self.run_metrics.len().saturating_sub(self.config.max_run_metrics);
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
        let cutoff_date = now - chrono::Duration::days(self.config.max_completed_run_age_days);
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
        let workflow_cutoff_date = now - chrono::Duration::days(self.config.max_workflow_summary_age_days);
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
