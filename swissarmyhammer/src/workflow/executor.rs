//! Workflow execution engine

use crate::workflow::{
    parse_action_from_description, ActionError, ConditionType, StateId, StateType,
    TransitionCondition, Workflow, WorkflowRun, WorkflowRunStatus,
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
        limit: usize,
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
    /// Action execution failed
    #[error("Action execution failed: {0}")]
    ActionError(#[from] ActionError),
}

/// Result type for executor operations
pub type ExecutorResult<T> = Result<T, ExecutorError>;

/// Maximum number of state transitions allowed in a single execution
const MAX_TRANSITIONS: usize = 1000;

/// Default maximum execution history size to prevent unbounded growth
const DEFAULT_MAX_HISTORY_SIZE: usize = 10000;

/// Context key for last action result
const LAST_ACTION_RESULT_KEY: &str = "last_action_result";

/// Represents a parallel execution branch
#[derive(Debug)]
struct ParallelBranch {
    /// The state this branch is currently in
    current_state: StateId,
    /// The execution context for this branch
    context: HashMap<String, Value>,
    /// History for this branch
    history: Vec<(StateId, chrono::DateTime<chrono::Utc>)>,
}

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
#[derive(Debug, Clone, Copy)]
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
    pub async fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ValidationFailed(errors.join("; ")))?;

        let mut run = WorkflowRun::new(workflow);

        self.log_event(
            ExecutionEventType::Started,
            format!("Started workflow: {}", run.workflow.name),
        );

        // Execute the initial state with transition limit
        self.execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await?;

        Ok(run)
    }

    /// Resume a workflow from saved state
    pub async fn resume_workflow(&mut self, mut run: WorkflowRun) -> ExecutorResult<WorkflowRun> {
        if run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed {
            return Err(ExecutorError::WorkflowCompleted);
        }

        self.log_event(
            ExecutionEventType::Started,
            format!(
                "Resumed workflow: {} from state: {}",
                run.workflow.name, run.current_state
            ),
        );

        // Continue execution from current state with transition limit
        self.execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await?;

        Ok(run)
    }

    /// Check if workflow execution should stop
    fn is_workflow_finished(&self, run: &WorkflowRun) -> bool {
        run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed
    }

    /// Execute a single execution cycle: state execution and potential transition
    async fn execute_single_cycle(&mut self, run: &mut WorkflowRun) -> ExecutorResult<bool> {
        self.execute_single_state(run).await?;

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
    async fn execute_state_with_limit(
        &mut self,
        run: &mut WorkflowRun,
        remaining_transitions: usize,
    ) -> ExecutorResult<()> {
        if remaining_transitions == 0 {
            return Err(ExecutorError::TransitionLimitExceeded {
                limit: MAX_TRANSITIONS,
            });
        }

        let mut current_remaining = remaining_transitions;

        loop {
            let transition_performed = self.execute_single_cycle(run).await?;

            if !transition_performed {
                // Either workflow finished or no transitions available
                break;
            }

            current_remaining -= 1;
            if current_remaining == 0 {
                return Err(ExecutorError::TransitionLimitExceeded {
                    limit: MAX_TRANSITIONS,
                });
            }
        }

        Ok(())
    }

    /// Execute the current state and evaluate transitions
    pub async fn execute_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        self.execute_state_with_limit(run, MAX_TRANSITIONS).await
    }

    /// Execute a single state without transitioning
    async fn execute_single_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let current_state_id = &run.current_state;

        // Check if this is a fork state
        if self.is_fork_state(run, current_state_id) {
            return self.execute_fork_state(run).await;
        }

        // Check if this is a join state
        if self.is_join_state(run, current_state_id) {
            return self.execute_join_state(run).await;
        }

        // Get the current state
        let current_state = run
            .workflow
            .states
            .get(current_state_id)
            .ok_or_else(|| ExecutorError::StateNotFound(current_state_id.clone()))?;

        // Extract values we need before the mutable borrow
        let state_description = current_state.description.clone();
        let is_terminal = current_state.is_terminal;

        self.log_event(
            ExecutionEventType::StateExecution,
            format!(
                "Executing state: {} - {}",
                current_state.id, current_state.description
            ),
        );

        // Execute state action if one can be parsed from the description
        self.execute_state_action(run, &state_description).await?;

        // Check if this is a terminal state
        if is_terminal {
            run.complete();
            self.log_event(
                ExecutionEventType::Completed,
                "Workflow completed".to_string(),
            );
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
                        transition.condition.condition_type.as_str(),
                        transition.from_state,
                        transition.to_state
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
    pub async fn transition_to(
        &mut self,
        run: &mut WorkflowRun,
        next_state: StateId,
    ) -> ExecutorResult<()> {
        self.perform_transition(run, next_state)?;
        self.execute_state(run).await
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
                Value::Bool(success) => {
                    if expect_success {
                        *success
                    } else {
                        !*success
                    }
                }
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
            ConditionType::OnSuccess => Ok(self.evaluate_action_condition(context, true, true)),
            ConditionType::OnFailure => Ok(self.evaluate_action_condition(context, false, false)),
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

    /// Execute action parsed from state description
    async fn execute_state_action(
        &mut self,
        run: &mut WorkflowRun,
        state_description: &str,
    ) -> ExecutorResult<()> {
        // Parse action from state description
        if let Some(action) = parse_action_from_description(state_description)? {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Executing action: {}", action.description()),
            );

            // Execute the action and update context
            match action.execute(&mut run.context).await {
                Ok(result) => {
                    self.log_event(
                        ExecutionEventType::StateExecution,
                        format!("Action completed successfully with result: {}", result),
                    );
                }
                Err(action_error) => {
                    // Mark action as failed in context
                    run.context
                        .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(false));

                    // Categorize the error for appropriate handling
                    match &action_error {
                        ActionError::Timeout { timeout } => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("Action timed out after {:?}", timeout),
                            );
                        }
                        ActionError::ClaudeError(msg) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("Claude command failed: {}", msg),
                            );
                        }
                        ActionError::VariableError(msg) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("Variable operation failed: {}", msg),
                            );
                        }
                        ActionError::IoError(io_err) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("IO operation failed: {}", io_err),
                            );
                        }
                        ActionError::JsonError(json_err) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("JSON parsing failed: {}", json_err),
                            );
                        }
                        ActionError::ParseError(msg) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("Action parsing failed: {}", msg),
                            );
                        }
                        ActionError::ExecutionError(msg) => {
                            self.log_event(
                                ExecutionEventType::Failed,
                                format!("Action execution failed: {}", msg),
                            );
                        }
                    }

                    // Propagate the error
                    return Err(ExecutorError::ActionError(action_error));
                }
            }
        }

        Ok(())
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

    /// Check if a state matches a specific state type
    fn is_state_type(&self, run: &WorkflowRun, state_id: &StateId, state_type: StateType) -> bool {
        run.workflow
            .states
            .get(state_id)
            .map(|state| state.state_type == state_type)
            .unwrap_or(false)
    }

    /// Check if a state is a fork state
    fn is_fork_state(&self, run: &WorkflowRun, state_id: &StateId) -> bool {
        self.is_state_type(run, state_id, StateType::Fork)
    }

    /// Check if a state is a join state
    fn is_join_state(&self, run: &WorkflowRun, state_id: &StateId) -> bool {
        self.is_state_type(run, state_id, StateType::Join)
    }

    /// Find all outgoing transitions from a fork state
    fn find_fork_transitions(&self, run: &WorkflowRun, fork_state: &StateId) -> Vec<StateId> {
        run.workflow
            .transitions
            .iter()
            .filter(|t| &t.from_state == fork_state)
            .map(|t| t.to_state.clone())
            .collect()
    }

    /// Find the join state for a set of parallel branches
    ///
    /// Locates the join state where all parallel branches converge.
    /// A valid join state must:
    /// 1. Be of type StateType::Join
    /// 2. Have incoming transitions from ALL branch states
    ///
    /// # Algorithm
    /// - Examines all transitions in the workflow
    /// - For each transition from a branch state to a join-type state
    /// - Verifies all other branches also transition to the same state
    /// - Returns the first valid join state found
    ///
    /// # Returns
    /// - `Some(StateId)` if a valid join state is found
    /// - `None` if no join state exists for all branches
    fn find_join_state(&self, run: &WorkflowRun, branch_states: &[StateId]) -> Option<StateId> {
        // Find a state that all branches transition to
        for transition in &run.workflow.transitions {
            if branch_states.contains(&transition.from_state) {
                // Check if this target state is a join state
                if self.is_join_state(run, &transition.to_state) {
                    // Verify all branches lead to this join state
                    let all_branches_lead_here = branch_states.iter().all(|branch| {
                        run.workflow
                            .transitions
                            .iter()
                            .any(|t| &t.from_state == branch && t.to_state == transition.to_state)
                    });

                    if all_branches_lead_here {
                        return Some(transition.to_state.clone());
                    }
                }
            }
        }
        None
    }

    /// Execute a fork state - spawn parallel branches
    ///
    /// Fork states enable parallel execution by spawning multiple execution branches.
    /// Each branch starts from a different state and executes independently until
    /// they all converge at a join state. The algorithm:
    ///
    /// 1. Validates the fork state has at least 2 outgoing transitions
    /// 2. Finds the join state where all branches converge
    /// 3. Creates isolated contexts for each branch (copy of parent context)
    /// 4. Executes each branch sequentially (future: parallel with tokio tasks)
    /// 5. Merges branch contexts using last-write-wins strategy
    /// 6. Transitions to the join state with merged context
    ///
    /// # Errors
    /// - Fork state has no outgoing transitions
    /// - Fork state has only one outgoing transition
    /// - No join state found for the fork branches
    /// - Branch execution fails or doesn't reach join state
    async fn execute_fork_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let fork_state = run.current_state.clone();

        self.log_event(
            ExecutionEventType::StateExecution,
            format!("Executing fork state: {}", fork_state),
        );

        // Find all outgoing transitions from the fork state
        let branch_states = self.find_fork_transitions(run, &fork_state);

        if branch_states.is_empty() {
            return Err(ExecutorError::ExecutionFailed(
                format!(
                    "Fork state '{}' has no outgoing transitions. Fork states must have at least two outgoing transitions to parallel branches",
                    fork_state
                ),
            ));
        }

        if branch_states.len() < 2 {
            return Err(ExecutorError::ExecutionFailed(
                format!(
                    "Fork state '{}' has only {} outgoing transition. Fork states must have at least two outgoing transitions for parallel execution",
                    fork_state,
                    branch_states.len()
                ),
            ));
        }

        // Find the join state where branches will converge
        let join_state = self.find_join_state(run, &branch_states).ok_or_else(|| {
            ExecutorError::ExecutionFailed(
                format!(
                    "No join state found for fork '{}' with branches: {:?}. All fork branches must eventually converge at a join state",
                    fork_state,
                    branch_states
                ),
            )
        })?;

        self.log_event(
            ExecutionEventType::StateExecution,
            format!(
                "Fork {} spawning {} branches to join at {}",
                fork_state,
                branch_states.len(),
                join_state
            ),
        );

        // For simplicity in this initial implementation, we'll execute branches sequentially
        // but track them as if they were parallel for the context merging logic
        let mut completed_branches = Vec::new();

        for branch_state in branch_states {
            // Create a branch with a copy of the current context
            let mut branch = ParallelBranch {
                current_state: branch_state.clone(),
                context: run.context.clone(),
                history: vec![(branch_state.clone(), chrono::Utc::now())],
            };

            // Execute this branch until it reaches the join state
            self.execute_branch_to_join(&run.workflow, &mut branch, &join_state)
                .await?;

            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Branch {} completed", branch_state),
            );

            completed_branches.push(branch);
        }

        // Merge contexts from all branches
        self.merge_branch_contexts(run, completed_branches)?;

        // Transition to the join state
        run.transition_to(join_state);

        Ok(())
    }

    /// Execute a join state - merge parallel contexts
    async fn execute_join_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let join_state = run.current_state.clone();

        self.log_event(
            ExecutionEventType::StateExecution,
            format!("Executing join state: {}", join_state),
        );

        // Join state execution is mostly handled in the fork state
        // Here we just log that we've reached the join point
        self.log_event(
            ExecutionEventType::StateExecution,
            format!("All branches joined at: {}", join_state),
        );

        Ok(())
    }

    /// Execute a single branch until it reaches the join state
    ///
    /// Executes a parallel branch in isolation with its own context copy.
    /// The branch executes state actions and follows transitions until it
    /// reaches the target join state. Branch execution is sequential but
    /// isolated from other branches.
    ///
    /// # Arguments
    /// - `workflow`: The workflow definition containing states and transitions
    /// - `branch`: Mutable branch state with context and execution history
    /// - `join_state`: Target join state where this branch should converge
    ///
    /// # Errors
    /// - State not found in workflow
    /// - Transition limit exceeded (prevents infinite loops)
    /// - Branch doesn't reach join state (stuck or missing transitions)
    async fn execute_branch_to_join(
        &mut self,
        workflow: &Workflow,
        branch: &mut ParallelBranch,
        join_state: &StateId,
    ) -> ExecutorResult<()> {
        let mut transitions = 0;
        const MAX_BRANCH_TRANSITIONS: usize = 100;

        while &branch.current_state != join_state && transitions < MAX_BRANCH_TRANSITIONS {
            // Get the current state
            let current_state = workflow
                .states
                .get(&branch.current_state)
                .ok_or_else(|| ExecutorError::StateNotFound(branch.current_state.clone()))?;

            // Execute state action if one can be parsed from the description
            if let Some(action) = parse_action_from_description(&current_state.description)? {
                self.log_event(
                    ExecutionEventType::StateExecution,
                    format!("Branch executing action: {}", action.description()),
                );

                match action.execute(&mut branch.context).await {
                    Ok(_result) => {
                        // Mark action as successful
                        branch
                            .context
                            .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
                    }
                    Err(_action_error) => {
                        // Mark action as failed
                        branch
                            .context
                            .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(false));
                    }
                }
            }

            // Find next transition based on conditions
            let next_state = workflow
                .transitions
                .iter()
                .filter(|t| t.from_state == branch.current_state)
                .find(|t| {
                    // Evaluate the condition (simplified version)
                    match &t.condition.condition_type {
                        ConditionType::Always => true,
                        ConditionType::OnSuccess => branch
                            .context
                            .get(LAST_ACTION_RESULT_KEY)
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        ConditionType::OnFailure => !branch
                            .context
                            .get(LAST_ACTION_RESULT_KEY)
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        _ => false, // Skip custom conditions for now
                    }
                })
                .map(|t| t.to_state.clone());

            if let Some(next) = next_state {
                branch.current_state = next.clone();
                branch.history.push((next, chrono::Utc::now()));
                transitions += 1;
            } else {
                break;
            }
        }

        if transitions >= MAX_BRANCH_TRANSITIONS {
            return Err(ExecutorError::TransitionLimitExceeded {
                limit: MAX_BRANCH_TRANSITIONS,
            });
        }

        // Check if the branch reached the join state
        if &branch.current_state != join_state {
            return Err(ExecutorError::ExecutionFailed(
                format!(
                    "Branch execution stopped at state '{}' without reaching join state '{}'. Branch may be stuck or missing required transitions",
                    branch.current_state,
                    join_state
                ),
            ));
        }

        Ok(())
    }

    /// Merge contexts from parallel branches
    ///
    /// Combines execution contexts from all parallel branches using a
    /// last-write-wins strategy. Variables from later branches override
    /// variables from earlier branches if there are conflicts.
    ///
    /// The merge strategy:
    /// 1. Iterates through branches in order
    /// 2. For each branch, copies all variables to main context
    /// 3. Skips execution-specific keys (last_action_result)
    /// 4. Merges branch execution history into main history
    ///
    /// # Future Improvements
    /// - Configurable merge strategies (first-wins, explicit conflict resolution)
    /// - Type-aware merging for complex data structures
    /// - Conflict detection and reporting
    fn merge_branch_contexts(
        &mut self,
        run: &mut WorkflowRun,
        branches: Vec<ParallelBranch>,
    ) -> ExecutorResult<()> {
        self.log_event(
            ExecutionEventType::StateExecution,
            format!("Merging contexts from {} branches", branches.len()),
        );

        // Simple merge strategy: combine all variables from all branches
        // In case of conflicts, later branches override earlier ones
        for branch in branches {
            for (key, value) in branch.context {
                // Skip the last_action_result key as it's execution-specific
                if key != LAST_ACTION_RESULT_KEY {
                    run.context.insert(key, value);
                }
            }

            // Merge history
            run.history.extend(branch.history);
        }

        Ok(())
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
    use crate::workflow::test_helpers::*;
    use crate::workflow::{Transition, WorkflowName};

    fn create_test_workflow() -> Workflow {
        let mut workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );

        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state("processing", "Processing state", false));
        workflow.add_state(create_state("end", "End state", true));

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
        assert!(history
            .iter()
            .any(|e| matches!(e.event_type, ExecutionEventType::Started)));
        assert!(history
            .iter()
            .any(|e| matches!(e.event_type, ExecutionEventType::Completed)));
    }

    #[test]
    fn test_evaluate_transitions_always_condition() {
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
            .transition_to(&mut run, StateId::new("non_existent"))
            .await;

        assert!(matches!(result, Err(ExecutorError::StateNotFound(_))));
    }

    #[tokio::test]
    async fn test_max_transition_limit() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with infinite loop
        let mut workflow = Workflow::new(
            WorkflowName::new("Infinite Loop"),
            "A workflow that loops forever".to_string(),
            StateId::new("loop_state"),
        );

        workflow.add_state(create_state(
            "loop_state",
            "State that loops to itself",
            false,
        ));

        // Add a terminal state to pass validation
        workflow.add_state(create_state("terminal", "Terminal state", true));

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

        let result = executor.start_workflow(workflow).await;
        assert!(
            matches!(result, Err(ExecutorError::TransitionLimitExceeded { limit }) if limit == MAX_TRANSITIONS)
        );
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

        let result = executor
            .evaluate_condition(&condition, &run.context)
            .unwrap();
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
        assert!(
            matches!(result, Err(ExecutorError::ExpressionError(msg)) if msg.contains("requires an expression"))
        );
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

    #[tokio::test]
    async fn test_fork_join_parallel_execution() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with fork and join
        let mut workflow = Workflow::new(
            WorkflowName::new("Fork Join Test"),
            "Test parallel execution".to_string(),
            StateId::new("start"),
        );

        // Add states
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state_with_type(
            "fork1",
            "Fork state",
            StateType::Fork,
            false,
        ));
        workflow.add_state(create_state("branch1", "Branch 1", false));
        workflow.add_state(create_state("branch2", "Branch 2", false));
        workflow.add_state(create_state_with_type(
            "join1",
            "Join state",
            StateType::Join,
            false,
        ));
        workflow.add_state(create_state("end", "End state", true));

        // Add transitions
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("fork1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("fork1"),
            to_state: StateId::new("branch1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("fork1"),
            to_state: StateId::new("branch2"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("branch1"),
            to_state: StateId::new("join1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("branch2"),
            to_state: StateId::new("join1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("join1"),
            to_state: StateId::new("end"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        let run = executor.start_workflow(workflow).await.unwrap();

        // After execution, workflow should be completed
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "end");

        // History should show parallel branch execution
        let history = executor.get_history();

        // Should have events for both branches
        assert!(history.iter().any(|e| e.details.contains("branch1")));
        assert!(history.iter().any(|e| e.details.contains("branch2")));
    }

    #[tokio::test]
    async fn test_fork_join_context_merging() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with fork and join that sets variables in parallel branches
        let mut workflow = Workflow::new(
            WorkflowName::new("Context Merge Test"),
            "Test context merging at join".to_string(),
            StateId::new("start"),
        );

        // Add states with actions that set variables
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state_with_type(
            "fork1",
            "Fork state",
            StateType::Fork,
            false,
        ));
        workflow.add_state(create_state(
            "branch1",
            "Set branch1_result=\"success\"",
            false,
        ));
        workflow.add_state(create_state(
            "branch2",
            "Set branch2_result=\"success\"",
            false,
        ));
        workflow.add_state(create_state_with_type(
            "join1",
            "Join state",
            StateType::Join,
            false,
        ));
        workflow.add_state(create_state("end", "End state", true));

        // Add transitions (same as previous test)
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("fork1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("fork1"),
            to_state: StateId::new("branch1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("fork1"),
            to_state: StateId::new("branch2"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("branch1"),
            to_state: StateId::new("join1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("branch2"),
            to_state: StateId::new("join1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("join1"),
            to_state: StateId::new("end"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        let run = executor.start_workflow(workflow).await.unwrap();

        // After execution, both branch variables should be in the final context
        assert!(run.context.contains_key("branch1_result"));
        assert!(run.context.contains_key("branch2_result"));
        assert_eq!(run.status, WorkflowRunStatus::Completed);
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
