//! Cost tracking integration for workflow execution

use super::{ExecutionEventType, WorkflowExecutor};
use crate::cost::{CostSessionId, CostTracker, IssueId};
use crate::workflow::WorkflowRun;
use std::sync::{Arc, Mutex};

impl WorkflowExecutor {
    /// Set cost tracker for workflow execution cost tracking
    pub fn set_cost_tracker(&mut self, cost_tracker: Arc<Mutex<CostTracker>>) {
        self.cost_tracker = Some(cost_tracker);
    }

    /// Get cost tracker reference if available
    pub fn get_cost_tracker(&self) -> Option<&Arc<Mutex<CostTracker>>> {
        self.cost_tracker.as_ref()
    }

    /// Start cost tracking session for workflow run
    pub(super) fn start_cost_tracking_session(
        &mut self,
        run: &WorkflowRun,
        issue_id: Option<String>,
    ) -> Option<CostSessionId> {
        let cost_tracker = self.cost_tracker.clone();
        if let Some(cost_tracker) = cost_tracker {
            match cost_tracker.lock() {
                Ok(mut tracker) => {
                    // Create issue ID from workflow name or provided issue_id
                    let issue_id = issue_id
                        .unwrap_or_else(|| format!("workflow_{}", run.workflow.name.as_str()));

                    match IssueId::new(issue_id) {
                        Ok(issue_id) => {
                            match tracker.start_session(issue_id) {
                                Ok(session_id) => {
                                    // Release the lock before accessing self
                                    drop(tracker);

                                    // Start cost tracking in metrics
                                    self.metrics.start_cost_tracking(&run.id, session_id);

                                    self.log_event(
                                        ExecutionEventType::Started,
                                        format!("Started cost tracking session: {}", session_id),
                                    );

                                    Some(session_id)
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to start cost tracking session for run {}: {}",
                                        run.id,
                                        e
                                    );
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to create issue ID for cost tracking: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to acquire cost tracker lock for run {}: {}",
                        run.id,
                        e
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    /// Complete cost tracking session for workflow run
    pub(super) fn complete_cost_tracking_session(
        &mut self,
        run: &WorkflowRun,
        session_id: CostSessionId,
        success: bool,
    ) {
        let cost_tracker = self.cost_tracker.clone();
        if let Some(cost_tracker) = cost_tracker {
            match cost_tracker.lock() {
                Ok(mut tracker) => {
                    let status = if success {
                        crate::cost::CostSessionStatus::Completed
                    } else {
                        crate::cost::CostSessionStatus::Failed
                    };

                    if let Err(e) = tracker.complete_session(&session_id, status) {
                        tracing::warn!(
                            "Failed to complete cost tracking session {} for run {}: {}",
                            session_id,
                            run.id,
                            e
                        );
                    } else {
                        // Release the lock before accessing self
                        drop(tracker);

                        // Complete cost tracking in metrics
                        self.metrics.complete_cost_tracking(&run.id);

                        self.log_event(
                            ExecutionEventType::Completed,
                            format!("Completed cost tracking session: {}", session_id),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to acquire cost tracker lock to complete session {} for run {}: {}",
                        session_id,
                        run.id,
                        e
                    );
                }
            }
        }
    }

    /// Complete workflow cost tracking by getting session ID from metadata
    pub(super) fn complete_workflow_cost_tracking(&mut self, run: &WorkflowRun, success: bool) {
        if let Some(session_id_str) = run.metadata.get("cost_session_id") {
            if let Ok(ulid) = ulid::Ulid::from_string(session_id_str) {
                let session_id = CostSessionId::from_ulid(ulid);
                self.complete_cost_tracking_session(run, session_id, success);
            }
        }
    }
}
