//! Workflow execution engine

use crate::workflow::{
    StateId, TransitionCondition, Workflow, WorkflowRun,
    WorkflowRunStatus,
};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during workflow execution
#[derive(Debug, Error)]
pub enum ExecutorError {
    /// State referenced in workflow does not exist
    #[error("State not found: {0}")]
    StateNotFound(String),
    /// Transition is invalid or not allowed
    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
    /// Workflow execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    /// Attempted to resume a completed workflow
    #[error("Workflow already completed")]
    WorkflowCompleted,
    /// Expression evaluation failed
    #[error("Expression evaluation failed: {0}")]
    ExpressionError(String),
}

/// Result type for executor operations
pub type ExecutorResult<T> = Result<T, ExecutorError>;

/// Workflow execution engine
pub struct WorkflowExecutor {
    /// Execution history for debugging
    execution_history: Vec<ExecutionEvent>,
}

/// Event recorded during workflow execution
#[derive(Debug, Clone)]
pub struct ExecutionEvent {
    /// When the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Type of execution event
    pub event_type: ExecutionEventType,
    /// Human-readable details about the event
    pub details: String,
}

/// Types of events that can occur during workflow execution
#[derive(Debug, Clone)]
pub enum ExecutionEventType {
    /// Workflow execution started
    Started,
    /// Transitioned to a new state
    StateTransition,
    /// Executed a state's action
    StateExecution,
    /// Evaluated a transition condition
    ConditionEvaluated,
    /// Workflow completed successfully
    Completed,
    /// Workflow execution failed
    Failed,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self {
            execution_history: Vec::new(),
        }
    }

    /// Start a new workflow run
    pub async fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ExecutionFailed(errors.join("; ")))?;

        let mut run = WorkflowRun::new(workflow);
        
        self.log_event(ExecutionEventType::Started, format!("Started workflow: {}", run.workflow.name));
        
        // Execute the initial state
        self.execute_state(&mut run)?;
        
        Ok(run)
    }

    /// Resume a workflow from saved state
    pub async fn resume_workflow(&mut self, mut run: WorkflowRun) -> ExecutorResult<WorkflowRun> {
        if run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed {
            return Err(ExecutorError::WorkflowCompleted);
        }

        self.log_event(
            ExecutionEventType::Started,
            format!("Resumed workflow: {} from state: {}", run.workflow.name, run.current_state),
        );

        // Continue execution from current state
        self.execute_state(&mut run)?;
        
        Ok(run)
    }

    /// Execute the current state and evaluate transitions
    pub fn execute_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let current_state_id = run.current_state.clone();
        
        // Get the current state
        let current_state = run
            .workflow
            .states
            .get(&current_state_id)
            .ok_or_else(|| ExecutorError::StateNotFound(current_state_id.to_string()))?
            .clone();

        self.log_event(
            ExecutionEventType::StateExecution,
            format!("Executing state: {} - {}", current_state.id, current_state.description),
        );

        // TODO: Execute state action when action system is implemented
        // For now, we just log the state execution

        // Check if this is a terminal state
        if current_state.is_terminal {
            run.complete();
            self.log_event(ExecutionEventType::Completed, "Workflow completed".to_string());
            return Ok(());
        }

        // Evaluate transitions
        if let Some(next_state) = self.evaluate_transitions(run)? {
            self.transition_to(run, next_state)?;
        }

        Ok(())
    }

    /// Evaluate all transitions from the current state
    pub fn evaluate_transitions(&mut self, run: &WorkflowRun) -> ExecutorResult<Option<StateId>> {
        let current_state = &run.current_state;
        
        // Find all transitions from current state
        let transitions: Vec<_> = run
            .workflow
            .transitions
            .iter()
            .filter(|t| &t.from_state == current_state)
            .collect();

        for transition in transitions {
            if self.evaluate_condition(&transition.condition, &run.context)? {
                self.log_event(
                    ExecutionEventType::ConditionEvaluated,
                    format!(
                        "Condition '{}' evaluated to true for transition: {} -> {}",
                        transition.condition.condition_type, transition.from_state, transition.to_state
                    ),
                );
                return Ok(Some(transition.to_state.clone()));
            }
        }

        Ok(None)
    }

    /// Transition to a new state
    pub fn transition_to(
        &mut self,
        run: &mut WorkflowRun,
        next_state: StateId,
    ) -> ExecutorResult<()> {
        // Verify the state exists
        if !run.workflow.states.contains_key(&next_state) {
            return Err(ExecutorError::StateNotFound(next_state.to_string()));
        }

        self.log_event(
            ExecutionEventType::StateTransition,
            format!("Transitioning from {} to {}", run.current_state, next_state),
        );

        // Update the run
        run.transition_to(next_state);

        // Execute the new state
        self.execute_state(run)
    }

    /// Evaluate a transition condition
    fn evaluate_condition(
        &self,
        condition: &TransitionCondition,
        _context: &HashMap<String, Value>,
    ) -> ExecutorResult<bool> {
        match condition.condition_type.as_str() {
            "always" => Ok(true),
            "never" => Ok(false),
            "on_success" => {
                // Check if the last action was successful
                // For now, we'll assume success
                Ok(true)
            }
            "on_failure" => {
                // Check if the last action failed
                // For now, we'll assume no failure
                Ok(false)
            }
            "custom" => {
                if let Some(expression) = &condition.expression {
                    // TODO: Implement expression evaluation
                    // For now, return an error
                    Err(ExecutorError::ExpressionError(format!(
                        "Custom expression evaluation not yet implemented: {}",
                        expression
                    )))
                } else {
                    Err(ExecutorError::ExpressionError(
                        "Custom condition requires an expression".to_string(),
                    ))
                }
            }
            _ => Err(ExecutorError::InvalidTransition(format!(
                "Unknown condition type: {}",
                condition.condition_type
            ))),
        }
    }

    /// Log an execution event
    fn log_event(&mut self, event_type: ExecutionEventType, details: String) {
        let event = ExecutionEvent {
            timestamp: chrono::Utc::now(),
            event_type,
            details,
        };
        // Debug logging would go here
        self.execution_history.push(event);
    }

    /// Get the execution history
    pub fn get_history(&self) -> &[ExecutionEvent] {
        &self.execution_history
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowName, State, Transition};

    fn create_test_workflow() -> Workflow {
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

        workflow.add_state(State {
            id: StateId::new("end"),
            description: "End state".to_string(),
            is_terminal: true,
            allows_parallel: false,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("processing"),
            condition: TransitionCondition {
                condition_type: "always".to_string(),
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("processing"),
            to_state: StateId::new("end"),
            condition: TransitionCondition {
                condition_type: "on_success".to_string(),
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow
    }

    #[tokio::test]
    async fn test_start_workflow() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        
        let run = executor.start_workflow(workflow).await.unwrap();
        
        assert_eq!(run.workflow.name.as_str(), "Test Workflow");
        // The workflow executes through to completion immediately
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "end");
        assert!(!executor.get_history().is_empty());
    }

    #[tokio::test]
    async fn test_workflow_execution_to_completion() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        
        let run = executor.start_workflow(workflow).await.unwrap();
        
        // The workflow should have executed through to completion
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "end");
        
        // Check execution history
        let history = executor.get_history();
        assert!(history.iter().any(|e| matches!(e.event_type, ExecutionEventType::Started)));
        assert!(history.iter().any(|e| matches!(e.event_type, ExecutionEventType::Completed)));
    }

    #[tokio::test]
    async fn test_evaluate_transitions_always_condition() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow);
        
        let next_state = executor.evaluate_transitions(&run).unwrap();
        assert_eq!(next_state, Some(StateId::new("processing")));
    }

    #[tokio::test]
    async fn test_resume_completed_workflow_fails() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let mut run = WorkflowRun::new(workflow);
        run.complete();
        
        let result = executor.resume_workflow(run).await;
        assert!(matches!(result, Err(ExecutorError::WorkflowCompleted)));
    }

    #[tokio::test]
    async fn test_transition_to_invalid_state() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let mut run = WorkflowRun::new(workflow);
        
        let result = executor
            .transition_to(&mut run, StateId::new("non_existent"));
        
        assert!(matches!(result, Err(ExecutorError::StateNotFound(_))));
    }
}