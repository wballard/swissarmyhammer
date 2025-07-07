//! Workflow execution engine

use crate::workflow::{
    ConditionType, StateId, TransitionCondition, Workflow, WorkflowRun,
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
    StateNotFound(StateId),
    /// Transition is invalid or not allowed
    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
    /// Workflow validation failed before execution
    #[error("Workflow validation failed: {0}")]
    ValidationFailed(String),
    /// Maximum transition limit exceeded to prevent infinite loops
    #[error("Maximum transition limit of {limit} exceeded")]
    TransitionLimitExceeded { 
        /// The maximum number of transitions that was exceeded
        limit: usize 
    },
    /// Generic workflow execution failure
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

/// Maximum number of state transitions allowed in a single execution
const MAX_TRANSITIONS: usize = 1000;

/// Default maximum execution history size to prevent unbounded growth
const DEFAULT_MAX_HISTORY_SIZE: usize = 10000;

/// Context key for last action result
const LAST_ACTION_RESULT_KEY: &str = "last_action_result";

/// Workflow execution engine
pub struct WorkflowExecutor {
    /// Execution history for debugging
    execution_history: Vec<ExecutionEvent>,
    /// Maximum size of execution history to prevent unbounded growth
    max_history_size: usize,
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
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
        }
    }

    /// Start a new workflow run
    pub fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ValidationFailed(errors.join("; ")))?;

        let mut run = WorkflowRun::new(workflow);
        
        self.log_event(ExecutionEventType::Started, format!("Started workflow: {}", run.workflow.name));
        
        // Execute the initial state with transition limit
        self.execute_state_with_limit(&mut run, MAX_TRANSITIONS)?;
        
        Ok(run)
    }

    /// Resume a workflow from saved state
    pub fn resume_workflow(&mut self, mut run: WorkflowRun) -> ExecutorResult<WorkflowRun> {
        if run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed {
            return Err(ExecutorError::WorkflowCompleted);
        }

        self.log_event(
            ExecutionEventType::Started,
            format!("Resumed workflow: {} from state: {}", run.workflow.name, run.current_state),
        );

        // Continue execution from current state with transition limit
        self.execute_state_with_limit(&mut run, MAX_TRANSITIONS)?;
        
        Ok(run)
    }

    /// Check if workflow execution should stop
    fn is_workflow_finished(&self, run: &WorkflowRun) -> bool {
        run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed
    }

    /// Execute a single execution cycle: state execution and potential transition
    fn execute_single_cycle(&mut self, run: &mut WorkflowRun) -> ExecutorResult<bool> {
        self.execute_single_state(run)?;
        
        // Check if workflow is complete after state execution
        if self.is_workflow_finished(run) {
            return Ok(false); // No transition needed, workflow finished
        }
        
        // Evaluate and perform transition
        if let Some(next_state) = self.evaluate_transitions(run)? {
            self.perform_transition(run, next_state)?;
            Ok(true) // Transition performed
        } else {
            // No valid transitions found, workflow is stuck
            Ok(false)
        }
    }

    /// Execute states with a maximum transition limit to prevent infinite loops
    fn execute_state_with_limit(&mut self, run: &mut WorkflowRun, remaining_transitions: usize) -> ExecutorResult<()> {
        if remaining_transitions == 0 {
            return Err(ExecutorError::TransitionLimitExceeded { 
                limit: MAX_TRANSITIONS 
            });
        }
        
        let mut current_remaining = remaining_transitions;
        
        loop {
            let transition_performed = self.execute_single_cycle(run)?;
            
            if !transition_performed {
                // Either workflow finished or no transitions available
                break;
            }
            
            current_remaining -= 1;
            if current_remaining == 0 {
                return Err(ExecutorError::TransitionLimitExceeded { 
                    limit: MAX_TRANSITIONS 
                });
            }
        }
        
        Ok(())
    }
    
    /// Execute the current state and evaluate transitions
    pub fn execute_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        self.execute_state_with_limit(run, MAX_TRANSITIONS)
    }
    
    /// Execute a single state without transitioning
    fn execute_single_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let current_state_id = &run.current_state;
        
        // Get the current state
        let current_state = run
            .workflow
            .states
            .get(current_state_id)
            .ok_or_else(|| ExecutorError::StateNotFound(current_state_id.clone()))?;

        self.log_event(
            ExecutionEventType::StateExecution,
            format!("Executing state: {} - {}", current_state.id, current_state.description),
        );

        // Execute state action (placeholder for future action system implementation)

        // Check if this is a terminal state
        if current_state.is_terminal {
            run.complete();
            self.log_event(ExecutionEventType::Completed, "Workflow completed".to_string());
            return Ok(());
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
                        transition.condition.condition_type.as_str(), transition.from_state, transition.to_state
                    ),
                );
                return Ok(Some(transition.to_state.clone()));
            }
        }

        Ok(None)
    }

    /// Perform a state transition without executing the new state
    fn perform_transition(
        &mut self,
        run: &mut WorkflowRun,
        next_state: StateId,
    ) -> ExecutorResult<()> {
        // Verify the state exists
        if !run.workflow.states.contains_key(&next_state) {
            return Err(ExecutorError::StateNotFound(next_state.clone()));
        }

        self.log_event(
            ExecutionEventType::StateTransition,
            format!("Transitioning from {} to {}", run.current_state, next_state),
        );

        // Update the run
        run.transition_to(next_state);
        
        Ok(())
    }
    
    /// Transition to a new state (public API that includes execution)
    pub fn transition_to(
        &mut self,
        run: &mut WorkflowRun,
        next_state: StateId,
    ) -> ExecutorResult<()> {
        self.perform_transition(run, next_state)?;
        self.execute_state(run)
    }

    /// Helper function to evaluate action-based conditions (success/failure)
    fn evaluate_action_condition(
        &self,
        context: &HashMap<String, Value>,
        expect_success: bool,
        default_value: bool,
    ) -> bool {
        if let Some(last_action_result) = context.get(LAST_ACTION_RESULT_KEY) {
            match last_action_result {
                Value::Bool(success) => if expect_success { *success } else { !*success },
                _ => default_value, // Default value if not a boolean
            }
        } else {
            default_value // Default value if no result in context
        }
    }

    /// Evaluate a transition condition
    fn evaluate_condition(
        &self,
        condition: &TransitionCondition,
        context: &HashMap<String, Value>,
    ) -> ExecutorResult<bool> {
        match &condition.condition_type {
            ConditionType::Always => Ok(true),
            ConditionType::Never => Ok(false),
            ConditionType::OnSuccess => {
                Ok(self.evaluate_action_condition(context, true, true))
            }
            ConditionType::OnFailure => {
                Ok(self.evaluate_action_condition(context, false, false))
            }
            ConditionType::Custom => {
                if let Some(expression) = &condition.expression {
                    // Expression evaluation not yet implemented
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
        }
    }

    /// Log an execution event
    fn log_event(&mut self, event_type: ExecutionEventType, details: String) {
        let event = ExecutionEvent {
            timestamp: chrono::Utc::now(),
            event_type,
            details,
        };
        // Could add logging here when log crate is available
        self.execution_history.push(event);
        
        // Trim history if it exceeds max size
        if self.execution_history.len() > self.max_history_size {
            let trim_count = self.execution_history.len() - self.max_history_size;
            self.execution_history.drain(0..trim_count);
        }
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
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("processing"),
            to_state: StateId::new("end"),
            condition: TransitionCondition {
                condition_type: ConditionType::OnSuccess,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow
    }

    #[test]
    fn test_start_workflow() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        
        let run = executor.start_workflow(workflow).unwrap();
        
        assert_eq!(run.workflow.name.as_str(), "Test Workflow");
        // The workflow executes through to completion immediately
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "end");
        assert!(!executor.get_history().is_empty());
    }

    #[test]
    fn test_workflow_execution_to_completion() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        
        let run = executor.start_workflow(workflow).unwrap();
        
        // The workflow should have executed through to completion
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "end");
        
        // Check execution history
        let history = executor.get_history();
        assert!(history.iter().any(|e| matches!(e.event_type, ExecutionEventType::Started)));
        assert!(history.iter().any(|e| matches!(e.event_type, ExecutionEventType::Completed)));
    }

    #[test]
    fn test_evaluate_transitions_always_condition() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow);
        
        let next_state = executor.evaluate_transitions(&run).unwrap();
        assert_eq!(next_state, Some(StateId::new("processing")));
    }

    #[test]
    fn test_resume_completed_workflow_fails() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let mut run = WorkflowRun::new(workflow);
        run.complete();
        
        let result = executor.resume_workflow(run);
        assert!(matches!(result, Err(ExecutorError::WorkflowCompleted)));
    }

    #[test]
    fn test_transition_to_invalid_state() {
        let mut executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let mut run = WorkflowRun::new(workflow);
        
        let result = executor
            .transition_to(&mut run, StateId::new("non_existent"));
        
        assert!(matches!(result, Err(ExecutorError::StateNotFound(_))));
    }

    #[test]
    fn test_max_transition_limit() {
        let mut executor = WorkflowExecutor::new();
        
        // Create a workflow with infinite loop
        let mut workflow = Workflow::new(
            WorkflowName::new("Infinite Loop"),
            "A workflow that loops forever".to_string(),
            StateId::new("loop_state"),
        );
        
        workflow.add_state(State {
            id: StateId::new("loop_state"),
            description: "State that loops to itself".to_string(),
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        // Add a terminal state to pass validation
        workflow.add_state(State {
            id: StateId::new("terminal"),
            description: "Terminal state".to_string(),
            is_terminal: true,
            allows_parallel: false,
            metadata: HashMap::new(),
        });
        
        workflow.add_transition(Transition {
            from_state: StateId::new("loop_state"),
            to_state: StateId::new("loop_state"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });
        
        let result = executor.start_workflow(workflow);
        assert!(matches!(result, Err(ExecutorError::TransitionLimitExceeded { limit }) if limit == MAX_TRANSITIONS));
    }

    #[test]
    fn test_never_condition() {
        let executor = WorkflowExecutor::new();
        let workflow = create_test_workflow();
        let run = WorkflowRun::new(workflow);
        
        let condition = TransitionCondition {
            condition_type: ConditionType::Never,
            expression: None,
        };
        
        let result = executor.evaluate_condition(&condition, &run.context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_custom_condition_without_expression() {
        let executor = WorkflowExecutor::new();
        let run = WorkflowRun::new(create_test_workflow());
        
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: None,
        };
        
        let result = executor.evaluate_condition(&condition, &run.context);
        assert!(matches!(result, Err(ExecutorError::ExpressionError(msg)) if msg.contains("requires an expression")));
    }


    #[test]
    fn test_execution_history_limit() {
        let mut executor = WorkflowExecutor::new();
        executor.max_history_size = 10; // Set small limit for testing
        
        // Add many events to trigger trimming
        for i in 0..20 {
            executor.log_event(ExecutionEventType::Started, format!("Event {}", i));
        }
        
        // History should be trimmed to stay under limit
        assert!(executor.get_history().len() <= executor.max_history_size);
    }

    #[test]
    fn test_on_success_condition_with_context() {
        let executor = WorkflowExecutor::new();
        let mut context = HashMap::new();
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
        
        let condition = TransitionCondition {
            condition_type: ConditionType::OnSuccess,
            expression: None,
        };
        
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);
        
        // Test with false result
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(false));
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_on_failure_condition_with_context() {
        let executor = WorkflowExecutor::new();
        let mut context = HashMap::new();
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(false));
        
        let condition = TransitionCondition {
            condition_type: ConditionType::OnFailure,
            expression: None,
        };
        
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);
        
        // Test with true result
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(!result);
    }
}