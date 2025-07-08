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

    /// Parse a WorkflowRunId from a string representation
    pub fn parse(s: &str) -> Result<Self, String> {
        Ulid::from_string(s)
            .map(Self)
            .map_err(|e| format!("Invalid workflow run ID '{}': {}", s, e))
    }

    /// Convert WorkflowRunId to string representation
    pub fn to_string(&self) -> String {
        self.0.to_string()
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
            context: Default::default(),
            status: WorkflowRunStatus::Running,
            started_at: now,
            completed_at: None,
            metadata: Default::default(),
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
    use crate::workflow::test_helpers::*;

    #[test]
    fn test_workflow_run_id_creation() {
        let id1 = WorkflowRunId::new();
        let id2 = WorkflowRunId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_workflow_run_id_parse_and_to_string() {
        let id = WorkflowRunId::new();
        let id_str = id.to_string();
        
        // Test round-trip conversion
        let parsed_id = WorkflowRunId::parse(&id_str).unwrap();
        assert_eq!(id, parsed_id);
        assert_eq!(id_str, parsed_id.to_string());
    }

    #[test]
    fn test_workflow_run_id_parse_invalid() {
        let invalid_id = "invalid-ulid";
        let result = WorkflowRunId::parse(invalid_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid workflow run ID"));
    }

    #[test]
    fn test_workflow_run_id_parse_valid_ulid() {
        // Generate a valid ULID string
        let ulid = Ulid::new();
        let ulid_str = ulid.to_string();
        
        let parsed_id = WorkflowRunId::parse(&ulid_str).unwrap();
        assert_eq!(parsed_id.to_string(), ulid_str);
    }

    #[test]
    fn test_workflow_run_creation() {
        let mut workflow = create_workflow("Test Workflow", "A test workflow", "start");
        workflow.add_state(create_state("start", "Start state", false));

        let run = WorkflowRun::new(workflow);

        assert_eq!(run.workflow.name.as_str(), "Test Workflow");
        assert_eq!(run.current_state.as_str(), "start");
        assert_eq!(run.status, WorkflowRunStatus::Running);
        assert_eq!(run.history.len(), 1);
        assert_eq!(run.history[0].0.as_str(), "start");
    }

    #[test]
    fn test_workflow_run_transition() {
        let mut workflow = create_workflow("Test Workflow", "A test workflow", "start");
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state("processing", "Processing state", false));

        let mut run = WorkflowRun::new(workflow);

        run.transition_to(StateId::new("processing"));

        assert_eq!(run.current_state.as_str(), "processing");
        assert_eq!(run.history.len(), 2);
        assert_eq!(run.history[1].0.as_str(), "processing");
    }

    #[test]
    fn test_workflow_run_completion() {
        let mut workflow = create_workflow("Test Workflow", "A test workflow", "start");
        workflow.add_state(create_state("start", "Start state", false));

        let mut run = WorkflowRun::new(workflow);

        run.complete();

        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert!(run.completed_at.is_some());
    }
}
