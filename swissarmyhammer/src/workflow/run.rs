//! Workflow runtime execution types

use crate::workflow::{StateId, Workflow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

/// Unique identifier for workflow runs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowRunId(Ulid);

impl WorkflowRunId {
    /// Create a new random workflow run ID
    pub fn new() -> Self {
        Self(Ulid::new())
    }
}

impl Default for WorkflowRunId {
    fn default() -> Self {
        Self::new()
    }
}

/// Status of a workflow run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRunStatus {
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with an error
    Failed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow is paused
    Paused,
}

/// Runtime execution context for a workflow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRun {
    /// Unique identifier for this run
    pub id: WorkflowRunId,
    /// The workflow being executed
    pub workflow: Workflow,
    /// Current state ID
    pub current_state: StateId,
    /// Execution history (state_id, timestamp)
    pub history: Vec<(StateId, chrono::DateTime<chrono::Utc>)>,
    /// Variables/context for this run
    pub context: HashMap<String, serde_json::Value>,
    /// Run status
    pub status: WorkflowRunStatus,
    /// When the run started
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// When the run completed (if applicable)
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Metadata for debugging and monitoring
    pub metadata: HashMap<String, String>,
}

impl WorkflowRun {
    /// Create a new workflow run
    pub fn new(workflow: Workflow) -> Self {
        let now = chrono::Utc::now();
        let initial_state = workflow.initial_state.clone();
        Self {
            id: WorkflowRunId::new(),
            workflow,
            current_state: initial_state.clone(),
            history: vec![(initial_state, now)],
            context: HashMap::new(),
            status: WorkflowRunStatus::Running,
            started_at: now,
            completed_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Record a state transition
    pub fn transition_to(&mut self, state_id: StateId) {
        let now = chrono::Utc::now();
        self.history.push((state_id.clone(), now));
        self.current_state = state_id;
    }

    /// Mark the run as completed
    pub fn complete(&mut self) {
        self.status = WorkflowRunStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark the run as failed
    pub fn fail(&mut self) {
        self.status = WorkflowRunStatus::Failed;
        self.completed_at = Some(chrono::Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{State, WorkflowName};

    #[test]
    fn test_workflow_run_id_creation() {
        let id1 = WorkflowRunId::new();
        let id2 = WorkflowRunId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_workflow_run_creation() {
        let mut workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );
        
        workflow.add_state(State {
            id: StateId::new("start"),
            description: "Start state".to_string(),
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        let run = WorkflowRun::new(workflow);
        
        assert_eq!(run.workflow.name.as_str(), "Test Workflow");
        assert_eq!(run.current_state.as_str(), "start");
        assert_eq!(run.status, WorkflowRunStatus::Running);
        assert_eq!(run.history.len(), 1);
        assert_eq!(run.history[0].0.as_str(), "start");
    }

    #[test]
    fn test_workflow_run_transition() {
        let mut workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );
        
        workflow.add_state(State {
            id: StateId::new("start"),
            description: "Start state".to_string(),
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        workflow.add_state(State {
            id: StateId::new("processing"),
            description: "Processing state".to_string(),
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        let mut run = WorkflowRun::new(workflow);
        
        run.transition_to(StateId::new("processing"));
        
        assert_eq!(run.current_state.as_str(), "processing");
        assert_eq!(run.history.len(), 2);
        assert_eq!(run.history[1].0.as_str(), "processing");
    }

    #[test]
    fn test_workflow_run_completion() {
        let mut workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );
        
        workflow.add_state(State {
            id: StateId::new("start"),
            description: "Start state".to_string(),
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        let mut run = WorkflowRun::new(workflow);
        
        run.complete();
        
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert!(run.completed_at.is_some());
    }
}