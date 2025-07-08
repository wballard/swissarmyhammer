//! Core workflow execution logic

use crate::workflow::{
    parse_action_from_description, ActionError, StateId, Workflow, WorkflowRun, WorkflowRunStatus,
    metrics::{WorkflowMetrics, MemoryMetrics},
};
use serde_json::Value;
use super::{ExecutorError, ExecutorResult, ExecutionEvent, ExecutionEventType, MAX_TRANSITIONS, DEFAULT_MAX_HISTORY_SIZE, LAST_ACTION_RESULT_KEY};
use std::time::Instant;
use std::collections::HashMap;
use cel_interpreter::Program;

/// Workflow execution engine
pub struct WorkflowExecutor {
    /// Execution history for debugging
    execution_history: Vec<ExecutionEvent>,
    /// Maximum size of execution history to prevent unbounded growth
    max_history_size: usize,
    /// Metrics collector for workflow execution
    metrics: WorkflowMetrics,
    /// Cache for compiled CEL programs
    cel_program_cache: HashMap<String, Program>,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self {
            execution_history: Vec::new(),
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            metrics: WorkflowMetrics::new(),
            cel_program_cache: HashMap::new(),
        }
    }

    /// Start a new workflow run
    pub async fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ValidationFailed(errors.join("; ")))?;

        let mut run = WorkflowRun::new(workflow);

        // Start metrics tracking for this run
        self.metrics.start_run(run.id, run.workflow.name.clone());

        self.log_event(
            ExecutionEventType::Started,
            format!("Started workflow: {}", run.workflow.name),
        );

        // Execute the initial state with transition limit
        let result = self.execute_state_with_limit(&mut run, MAX_TRANSITIONS).await;
        
        // Complete metrics tracking
        match &result {
            Ok(_) => {
                self.metrics.complete_run(&run.id, run.status, None);
            }
            Err(e) => {
                self.metrics.complete_run(&run.id, WorkflowRunStatus::Failed, Some(e.to_string()));
            }
        }

        result.map(|_| run)
    }

    /// Resume a workflow from saved state
    pub async fn resume_workflow(&mut self, mut run: WorkflowRun) -> ExecutorResult<WorkflowRun> {
        if run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed {
            return Err(ExecutorError::WorkflowCompleted);
        }

        // Start metrics tracking for resumed run
        self.metrics.start_run(run.id, run.workflow.name.clone());

        self.log_event(
            ExecutionEventType::Started,
            format!(
                "Resumed workflow: {} from state: {}",
                run.workflow.name, run.current_state
            ),
        );

        // Continue execution from current state with transition limit
        let result = self.execute_state_with_limit(&mut run, MAX_TRANSITIONS).await;
        
        // Complete metrics tracking
        match &result {
            Ok(_) => {
                self.metrics.complete_run(&run.id, run.status, None);
            }
            Err(e) => {
                self.metrics.complete_run(&run.id, WorkflowRunStatus::Failed, Some(e.to_string()));
            }
        }

        result.map(|_| run)
    }

    /// Check if workflow execution should stop
    pub fn is_workflow_finished(&self, run: &WorkflowRun) -> bool {
        run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed
    }

    /// Execute a single execution cycle: state execution and potential transition
    pub async fn execute_single_cycle(&mut self, run: &mut WorkflowRun) -> ExecutorResult<bool> {
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
    pub async fn execute_state_with_limit(
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
    pub async fn execute_single_state(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        let current_state_id = run.current_state.clone();

        // Check if this is a fork state
        if self.is_fork_state(run, &current_state_id) {
            return self.execute_fork_state(run).await;
        }

        // Check if this is a join state
        if self.is_join_state(run, &current_state_id) {
            return self.execute_join_state(run).await;
        }

        // Check if this is a choice state
        if self.is_choice_state(run, &current_state_id) {
            return self.execute_choice_state(run).await;
        }

        // Get the current state
        let current_state = run
            .workflow
            .states
            .get(&current_state_id)
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

        // Record state execution timing
        let state_start_time = Instant::now();
        
        // Execute state action if one can be parsed from the description
        self.execute_state_action(run, &state_description).await?;
        
        // Record state execution duration
        let state_duration = state_start_time.elapsed();
        self.metrics.record_state_execution(&run.id, current_state_id.clone(), state_duration);

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

    /// Perform a state transition without executing the new state
    pub fn perform_transition(
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

        // Record transition in metrics
        self.metrics.record_transition(&run.id);

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

    /// Execute action parsed from state description
    pub async fn execute_state_action(
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
    pub fn log_event(&mut self, event_type: ExecutionEventType, details: String) {
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

    /// Set the maximum history size
    pub fn set_max_history_size(&mut self, max_size: usize) {
        self.max_history_size = max_size;
    }

    /// Get workflow metrics
    pub fn get_metrics(&self) -> &WorkflowMetrics {
        &self.metrics
    }

    /// Get mutable access to workflow metrics
    pub fn get_metrics_mut(&mut self) -> &mut WorkflowMetrics {
        &mut self.metrics
    }

    /// Update memory metrics for a specific run
    pub fn update_memory_metrics(&mut self, run_id: &crate::workflow::WorkflowRunId, context_vars: usize, history_size: usize) {
        // Simple memory estimation - in production this would use actual memory profiling
        let estimated_memory = (context_vars * 1024) + (history_size * 256);
        let mut memory_metrics = MemoryMetrics::new();
        memory_metrics.update(estimated_memory as u64, context_vars, history_size);
        self.metrics.update_memory_metrics(run_id, memory_metrics);
    }

    /// Get or compile a CEL program from cache
    pub fn get_compiled_cel_program(&mut self, expression: &str) -> Result<&Program, Box<dyn std::error::Error>> {
        if !self.cel_program_cache.contains_key(expression) {
            let program = Program::compile(expression)?;
            self.cel_program_cache.insert(expression.to_string(), program);
        }
        Ok(self.cel_program_cache.get(expression).unwrap())
    }
    
    /// Check if a CEL program is cached
    pub fn is_cel_program_cached(&self, expression: &str) -> bool {
        self.cel_program_cache.contains_key(expression)
    }
    
    /// Get CEL program cache statistics
    pub fn get_cel_cache_stats(&self) -> (usize, usize) {
        (self.cel_program_cache.len(), self.cel_program_cache.capacity())
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}