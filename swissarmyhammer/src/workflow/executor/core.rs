//! Core workflow execution logic

use super::{
    ExecutionEvent, ExecutionEventType, ExecutorError, ExecutorResult, DEFAULT_MAX_HISTORY_SIZE,
    LAST_ACTION_RESULT_KEY, MAX_TRANSITIONS,
};
#[cfg(test)]
use crate::cost::CostSessionId;
use crate::cost::CostTracker;
use crate::workflow::{
    metrics::{MemoryMetrics, WorkflowMetrics},
    parse_action_from_description_with_context, ActionError, CompensationKey, ErrorContext,
    StateId, Workflow, WorkflowCacheManager, WorkflowRun, WorkflowRunStatus,
};
use cel_interpreter::Program;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Workflow execution engine
pub struct WorkflowExecutor {
    /// Execution history for debugging
    pub(super) execution_history: Vec<ExecutionEvent>,
    /// Maximum size of execution history to prevent unbounded growth
    pub(super) max_history_size: usize,
    /// Metrics collector for workflow execution
    pub(super) metrics: WorkflowMetrics,
    /// Cache manager for performance optimizations
    pub(super) cache_manager: WorkflowCacheManager,
    /// Optional workflow storage for test mode
    pub(super) test_storage: Option<Arc<crate::workflow::storage::WorkflowStorage>>,
    /// Optional cost tracker for workflow execution cost tracking
    pub(super) cost_tracker: Option<Arc<Mutex<CostTracker>>>,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self {
            execution_history: Vec::new(),
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            metrics: WorkflowMetrics::new(),
            cache_manager: WorkflowCacheManager::new(),
            test_storage: None,
            cost_tracker: None,
        }
    }

    /// Create a new workflow executor with test storage
    pub fn with_test_storage(storage: Arc<crate::workflow::storage::WorkflowStorage>) -> Self {
        Self {
            execution_history: Vec::new(),
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            metrics: WorkflowMetrics::new(),
            cache_manager: WorkflowCacheManager::new(),
            test_storage: Some(storage),
            cost_tracker: None,
        }
    }

    /// Get the workflow storage (test storage if available, otherwise create file system storage)
    pub fn get_storage(&self) -> crate::Result<Arc<crate::workflow::storage::WorkflowStorage>> {
        if let Some(storage) = &self.test_storage {
            Ok(storage.clone())
        } else {
            Ok(Arc::new(
                crate::workflow::storage::WorkflowStorage::file_system()?,
            ))
        }
    }

    /// Start a new workflow run (initializes but doesn't execute)
    pub fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        self.start_workflow_with_issue_id(workflow, None)
    }

    /// Start a new workflow run with optional issue ID for cost tracking
    pub fn start_workflow_with_issue_id(
        &mut self,
        workflow: Workflow,
        issue_id: Option<String>,
    ) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ValidationFailed(errors.join("; ")))?;

        let mut run = WorkflowRun::new(workflow);

        // Start metrics tracking for this run
        self.metrics.start_run(run.id, run.workflow.name.clone());

        // Start cost tracking session if cost tracker is available
        if let Some(session_id) = self.start_cost_tracking_session(&run, issue_id) {
            // Store cost session ID in workflow run metadata
            run.metadata
                .insert("cost_session_id".to_string(), session_id.to_string());
        }

        self.log_event(
            ExecutionEventType::Started,
            format!("Started workflow: {}", run.workflow.name),
        );

        Ok(run)
    }

    /// Start and execute a new workflow run
    pub async fn start_and_execute_workflow(
        &mut self,
        workflow: Workflow,
    ) -> ExecutorResult<WorkflowRun> {
        self.start_and_execute_workflow_with_issue_id(workflow, None)
            .await
    }

    /// Start and execute a new workflow run with optional issue ID for cost tracking
    pub async fn start_and_execute_workflow_with_issue_id(
        &mut self,
        workflow: Workflow,
        issue_id: Option<String>,
    ) -> ExecutorResult<WorkflowRun> {
        let start_time = std::time::Instant::now();
        let mut run = self.start_workflow_with_issue_id(workflow, issue_id)?;

        // Execute the initial state with transition limit
        let result = self
            .execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await;

        // Complete cost tracking session if one was started
        self.complete_workflow_cost_tracking(&run, result.is_ok());

        // Complete metrics tracking
        let total_duration = start_time.elapsed();
        match &result {
            Ok(_) => {
                self.metrics
                    .complete_run(&run.id, run.status, total_duration);
            }
            Err(e) => {
                self.metrics.complete_run_with_error(
                    &run.id,
                    WorkflowRunStatus::Failed,
                    total_duration,
                    Some(e.to_string()),
                );
            }
        }

        result.map(|_| run)
    }

    /// Resume a workflow from saved state
    pub async fn resume_workflow(&mut self, mut run: WorkflowRun) -> ExecutorResult<WorkflowRun> {
        let start_time = std::time::Instant::now();
        if run.status == WorkflowRunStatus::Completed || run.status == WorkflowRunStatus::Failed {
            return Err(ExecutorError::WorkflowCompleted);
        }

        // Start metrics tracking for resumed run
        self.metrics.start_run(run.id, run.workflow.name.clone());

        // Resume cost tracking if session ID is in metadata
        if run.metadata.contains_key("cost_session_id") {
            // Session was already started, just log resume
            self.log_event(
                ExecutionEventType::Started,
                "Resumed cost tracking session".to_string(),
            );
        }

        self.log_event(
            ExecutionEventType::Started,
            format!(
                "Resumed workflow: {} from state: {}",
                run.workflow.name, run.current_state
            ),
        );

        // Continue execution from current state with transition limit
        let result = self
            .execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await;

        // Complete cost tracking session if one was started
        self.complete_workflow_cost_tracking(&run, result.is_ok());

        // Complete metrics tracking
        let total_duration = start_time.elapsed();
        match &result {
            Ok(_) => {
                self.metrics
                    .complete_run(&run.id, run.status, total_duration);
            }
            Err(e) => {
                self.metrics.complete_run_with_error(
                    &run.id,
                    WorkflowRunStatus::Failed,
                    total_duration,
                    Some(e.to_string()),
                );
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
        tracing::debug!("Execute single cycle for state: {}", run.current_state);

        // Execute the state and capture any errors
        let state_error = self.execute_state_and_capture_errors(run).await?;

        // Check if workflow is complete after state execution
        if self.is_workflow_finished(run) {
            return Ok(false); // No transition needed, workflow finished
        }

        // Evaluate and perform transition
        self.evaluate_and_perform_transition(run, state_error).await
    }

    /// Execute state and capture errors for later processing
    async fn execute_state_and_capture_errors(
        &mut self,
        run: &mut WorkflowRun,
    ) -> ExecutorResult<Option<ExecutorError>> {
        // Execute the state, but don't propagate action errors immediately
        // We need to check for OnFailure transitions first
        let state_result = self.execute_single_state(run).await;

        // If it's an action error, we'll handle it after checking transitions
        match state_result {
            Err(ExecutorError::ActionError(e)) => Ok(Some(ExecutorError::ActionError(e))),
            Err(ExecutorError::ManualInterventionRequired(msg)) => {
                // Manual intervention required, workflow is paused
                Ok(Some(ExecutorError::ManualInterventionRequired(msg)))
            }
            Err(other) => Err(other), // Propagate non-action errors
            Ok(()) => Ok(None),       // No error
        }
    }

    /// Evaluate transitions and perform them if available
    async fn evaluate_and_perform_transition(
        &mut self,
        run: &mut WorkflowRun,
        state_error: Option<ExecutorError>,
    ) -> ExecutorResult<bool> {
        // Handle manual intervention case
        if let Some(ExecutorError::ManualInterventionRequired(_)) = state_error {
            return Ok(false);
        }

        // Evaluate and perform transition
        if let Some(next_state) = self.evaluate_transitions(run)? {
            self.perform_transition(run, next_state)?;
            Ok(true) // Transition performed
        } else if let Some(error) = state_error {
            // No valid transitions found and we had an error
            Err(error)
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
            tracing::debug!(
                "Workflow execution loop - current state: {}",
                run.current_state
            );
            let transition_performed = self.execute_single_cycle(run).await?;

            if !transition_performed {
                // Either workflow finished or no transitions available
                tracing::debug!("No transition performed, exiting loop");
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

        // Skip execution for terminal states (they have no actions)
        if current_state_id.as_str() == "[*]" {
            tracing::debug!("Reached terminal state [*]");
            run.complete();
            // Don't log completion here - it's already been logged by the terminal state
            return Ok(());
        }

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

        tracing::trace!(
            "Executing state: {} - {} for workflow {}",
            current_state.id,
            current_state.description,
            run.workflow.name
        );
        self.log_event(
            ExecutionEventType::StateExecution,
            format!(
                "Executing state: {} - {} for workflow {}",
                current_state.id, current_state.description, run.workflow.name
            ),
        );

        // Record state execution timing
        let state_start_time = Instant::now();

        // Execute state action if one can be parsed from the description
        tracing::debug!(
            "About to execute action for state {} with description: {}",
            current_state_id,
            state_description
        );
        let action_executed = self.execute_state_action(run, &state_description).await?;

        // Record state execution duration
        let state_duration = state_start_time.elapsed();
        self.metrics
            .record_state_execution(&run.id, current_state_id.clone(), state_duration);

        // Check if this state requires manual intervention
        if self.requires_manual_intervention(run) {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("State {current_state_id} requires manual intervention"),
            );

            // Check if manual approval has been provided
            if !run
                .context
                .get("manual_approval")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                // Pause execution here - workflow will need to be resumed
                // Mark workflow as paused by returning the proper error type
                return Err(ExecutorError::ManualInterventionRequired(format!(
                    "State {current_state_id} requires manual approval"
                )));
            }
        }

        // Check if this is a terminal state
        if is_terminal {
            run.complete();
            tracing::debug!("Terminal state reached: {}", current_state_id);
            // Only log generic completion if no action was executed
            if !action_executed {
                self.log_event(
                    ExecutionEventType::Completed,
                    "Workflow completed".to_string(),
                );
            }
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
            return Err(ExecutorError::StateNotFound(next_state));
        }

        // Track compensation states from transition metadata
        if let Some(transition) = run
            .workflow
            .transitions
            .iter()
            .find(|t| t.from_state == run.current_state && t.to_state == next_state)
        {
            if let Some(comp_state) = transition.metadata.get("compensation_state") {
                // Store compensation state in context for this transition
                let comp_key = CompensationKey::for_state(&run.current_state);
                run.context
                    .insert(comp_key.into(), Value::String(comp_state.clone()));
            }
        }

        tracing::info!(
            "Transitioning from {} to {} for workflow {}",
            run.current_state,
            next_state,
            run.workflow.name
        );
        self.log_event(
            ExecutionEventType::StateTransition,
            format!(
                "Transitioning from {} to {} for workflow {}",
                run.current_state, next_state, run.workflow.name
            ),
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

    /// Find transitions TO the given state
    fn find_transitions_to_state<'a>(
        &self,
        run: &'a WorkflowRun,
        state_id: &StateId,
    ) -> Vec<&'a crate::workflow::Transition> {
        run.workflow
            .transitions
            .iter()
            .filter(|t| &t.to_state == state_id)
            .collect()
    }

    /// Get metadata value from transitions TO the current state
    fn get_transition_metadata(&self, run: &WorkflowRun, key: &str) -> Option<String> {
        let transitions = self.find_transitions_to_state(run, &run.current_state);
        for transition in transitions {
            if let Some(value) = transition.metadata.get(key) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Execute action parsed from state description
    pub async fn execute_state_action(
        &mut self,
        run: &mut WorkflowRun,
        state_description: &str,
    ) -> ExecutorResult<bool> {
        // Parse action from state description with liquid template rendering
        if let Some(action) =
            parse_action_from_description_with_context(state_description, &run.context)?
        {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Executing action: {}", action.description()),
            );

            // Execute the action and handle result
            let result = self.execute_action_direct(run, action).await;
            self.handle_action_result(run, result).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Execute action directly without retry logic
    async fn execute_action_direct(
        &mut self,
        run: &mut WorkflowRun,
        action: Box<dyn crate::workflow::Action>,
    ) -> Result<Value, ActionError> {
        action.execute(&mut run.context).await
    }

    /// Handle the result of action execution
    async fn handle_action_result(
        &mut self,
        run: &mut WorkflowRun,
        result: Result<Value, ActionError>,
    ) -> ExecutorResult<()> {
        match result {
            Ok(result_value) => {
                // Set standard variables that are available after every action
                run.context.insert("success".to_string(), Value::Bool(true));
                run.context
                    .insert("failure".to_string(), Value::Bool(false));

                // Only set is_error to false if it's not already true (preserve error state)
                if !run
                    .context
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    run.context
                        .insert("is_error".to_string(), Value::Bool(false));
                }

                run.context
                    .insert("result".to_string(), result_value.clone());

                // Also set the legacy last_action_result for backward compatibility
                run.context
                    .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));

                self.log_event(
                    ExecutionEventType::StateExecution,
                    format!("Action completed successfully with result: {result_value}"),
                );
                Ok(())
            }
            Err(action_error) => self.handle_action_error(run, action_error).await,
        }
    }

    /// Handle action execution error
    async fn handle_action_error(
        &mut self,
        run: &mut WorkflowRun,
        action_error: ActionError,
    ) -> ExecutorResult<()> {
        // Check if this is an abort error - if so, propagate immediately
        if matches!(action_error, ActionError::AbortError(_)) {
            // Log the abort error
            let error_details = self.format_action_error(&action_error);
            self.log_event(ExecutionEventType::Failed, error_details);

            // Mark workflow as failed
            run.status = WorkflowRunStatus::Failed;

            // Propagate the error immediately
            return Err(ExecutorError::ActionError(action_error));
        }

        // Set standard variables that are available after every action
        run.context
            .insert("success".to_string(), Value::Bool(false));
        run.context.insert("failure".to_string(), Value::Bool(true));
        run.context
            .insert("is_error".to_string(), Value::Bool(true));
        run.context.insert(
            "result".to_string(),
            Value::String(action_error.to_string()),
        );

        // Also set the legacy last_action_result for backward compatibility
        run.context
            .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(false));

        // Capture error context
        self.capture_error_context(run, &action_error);

        // Log the error with appropriate details
        let error_details = self.format_action_error(&action_error);
        self.log_event(ExecutionEventType::Failed, error_details);

        // Check for dead letter state configuration
        if let Some(dead_letter_state) = self.get_dead_letter_state(run) {
            return self
                .handle_dead_letter_transition(run, dead_letter_state, &action_error)
                .await;
        }

        // Execute compensation if needed
        if let Err(comp_error) = self.execute_compensation(run).await {
            self.log_event(
                ExecutionEventType::Failed,
                format!("Compensation failed: {comp_error}"),
            );
        }

        // Check if this state should be skipped on failure
        if self.should_skip_on_failure(run) {
            self.log_event(
                ExecutionEventType::StateExecution,
                "Skipped failed state due to skip_on_failure configuration".to_string(),
            );
            run.context
                .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
            return Ok(());
        }

        // Propagate the error
        Err(ExecutorError::ActionError(action_error))
    }

    /// Capture error context for the action error
    fn capture_error_context(&mut self, run: &mut WorkflowRun, action_error: &ActionError) {
        let error_context = ErrorContext::new(action_error.to_string(), run.current_state.clone());
        let error_context_json = serde_json::to_value(&error_context).unwrap_or(Value::Null);
        run.context
            .insert(ErrorContext::CONTEXT_KEY.to_string(), error_context_json);
    }

    /// Format action error for logging
    fn format_action_error(&self, action_error: &ActionError) -> String {
        match action_error {
            ActionError::Timeout { timeout } => {
                format!("Action timed out after {timeout:?}")
            }
            ActionError::ClaudeError(msg) => format!("Claude command failed: {msg}"),
            ActionError::VariableError(msg) => {
                format!("Variable operation failed: {msg}")
            }
            ActionError::IoError(io_err) => format!("IO operation failed: {io_err}"),
            ActionError::JsonError(json_err) => {
                format!("JSON parsing failed: {json_err}")
            }
            ActionError::ParseError(msg) => format!("Action parsing failed: {msg}"),
            ActionError::ExecutionError(msg) => {
                format!("Action execution failed: {msg}")
            }
            ActionError::RateLimit { message, wait_time } => {
                format!("Rate limit reached: {message}. Please wait {wait_time:?} before retrying.")
            }
            ActionError::AbortError(msg) => format!("ABORT ERROR: {msg}"),
        }
    }

    /// Handle transition to dead letter state
    async fn handle_dead_letter_transition(
        &mut self,
        run: &mut WorkflowRun,
        dead_letter_state: StateId,
        action_error: &ActionError,
    ) -> ExecutorResult<()> {
        // Add dead letter reason to context
        run.context.insert(
            "dead_letter_reason".to_string(),
            Value::String(format!("Max retries exhausted: {action_error}")),
        );

        // Transition to dead letter state
        self.log_event(
            ExecutionEventType::StateTransition,
            format!("Transitioning to dead letter state: {dead_letter_state}"),
        );
        self.perform_transition(run, dead_letter_state)?;

        // Mark action as successful to allow workflow to continue
        run.context
            .insert(LAST_ACTION_RESULT_KEY.to_string(), Value::Bool(true));
        Ok(())
    }

    /// Get dead letter state from transition metadata
    fn get_dead_letter_state(&self, run: &WorkflowRun) -> Option<StateId> {
        self.get_transition_metadata(run, "dead_letter_state")
            .map(|state| StateId::new(&state))
    }

    /// Check if state should be skipped on failure
    fn should_skip_on_failure(&self, run: &WorkflowRun) -> bool {
        self.get_transition_metadata(run, "skip_on_failure")
            .map(|v| v == "true")
            .unwrap_or(false)
    }

    /// Check if current state requires manual intervention
    pub fn requires_manual_intervention(&self, run: &WorkflowRun) -> bool {
        if let Some(state) = run.workflow.states.get(&run.current_state) {
            if let Some(intervention) = state.metadata.get("requires_manual_intervention") {
                return intervention == "true";
            }
        }
        false
    }

    /// Log an execution event
    pub fn log_event(&mut self, event_type: ExecutionEventType, details: String) {
        tracing::info!("{}: {}", event_type, &details);
        let event = ExecutionEvent {
            timestamp: chrono::Utc::now(),
            event_type,
            details,
        };
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
    pub fn update_memory_metrics(
        &mut self,
        run_id: &crate::workflow::WorkflowRunId,
        context_vars: usize,
        history_size: usize,
    ) {
        // Simple memory estimation - in production this would use actual memory profiling
        let estimated_memory = (context_vars * 1024) + (history_size * 256);
        let mut memory_metrics = MemoryMetrics::new();
        memory_metrics.update(estimated_memory as u64, context_vars, history_size);
        self.metrics.update_memory_metrics(run_id, memory_metrics);
    }

    /// Get or compile a CEL program from cache
    pub fn get_compiled_cel_program(
        &mut self,
        expression: &str,
    ) -> Result<std::sync::Arc<Program>, Box<dyn std::error::Error>> {
        self.cache_manager.cel_cache.get_or_compile(expression)
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod cost_integration_tests {
    use super::*;
    use crate::workflow::{test_helpers::*, ConditionType};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_workflow_executor_with_cost_tracker() {
        let mut executor = WorkflowExecutor::new();
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));

        // Test setting cost tracker
        executor.set_cost_tracker(cost_tracker.clone());
        assert!(executor.get_cost_tracker().is_some());

        // Test getting cost tracker reference
        let tracker_ref = executor.get_cost_tracker().unwrap();
        assert!(Arc::ptr_eq(tracker_ref, &cost_tracker));
    }

    #[test]
    fn test_start_workflow_with_cost_tracking() {
        let mut executor = WorkflowExecutor::new();
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
        executor.set_cost_tracker(cost_tracker.clone());

        // Create test workflow
        let mut workflow = create_workflow("Test Workflow", "Test workflow", "start");
        workflow.add_state(create_state("start", "Start state", true));

        // Start workflow with issue ID
        let run = executor
            .start_workflow_with_issue_id(workflow, Some("test-issue-123".to_string()))
            .unwrap();

        // Verify cost session ID was stored in metadata
        assert!(run.metadata.contains_key("cost_session_id"));

        // Verify cost session was created in tracker
        let tracker = cost_tracker.lock().unwrap();
        assert_eq!(tracker.session_count(), 1);
        assert_eq!(tracker.active_session_count(), 1);

        // Verify metrics were updated with cost tracking
        let run_metrics = executor.get_metrics().get_run_metrics(&run.id).unwrap();
        assert!(run_metrics.cost_metrics.is_some());
    }

    #[test]
    fn test_workflow_execution_without_cost_tracker() {
        let mut executor = WorkflowExecutor::new();
        // No cost tracker set

        let mut workflow = create_workflow("Test Workflow", "Test workflow", "start");
        workflow.add_state(create_state("start", "Start state", true));

        let run = executor
            .start_workflow_with_issue_id(workflow, Some("test-issue-123".to_string()))
            .unwrap();

        // Verify no cost session ID in metadata
        assert!(!run.metadata.contains_key("cost_session_id"));

        // Verify no cost metrics in run metrics
        let run_metrics = executor.get_metrics().get_run_metrics(&run.id).unwrap();
        assert!(run_metrics.cost_metrics.is_none());
    }

    #[tokio::test]
    async fn test_workflow_completion_with_cost_tracking() {
        let mut executor = WorkflowExecutor::new();
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
        executor.set_cost_tracker(cost_tracker.clone());

        // Create simple workflow that will complete immediately
        let mut workflow = create_workflow("Test Workflow", "Test workflow", "start");
        workflow.add_state(create_state("start", "Start state", true));

        // Execute workflow
        let run = executor
            .start_and_execute_workflow_with_issue_id(workflow, Some("test-issue-123".to_string()))
            .await
            .unwrap();

        // Verify workflow completed
        assert_eq!(run.status, WorkflowRunStatus::Completed);

        // Verify cost session was completed
        let tracker = cost_tracker.lock().unwrap();
        assert_eq!(tracker.session_count(), 1);
        assert_eq!(tracker.completed_session_count(), 1);
        assert_eq!(tracker.active_session_count(), 0);

        // Verify cost metrics were completed
        let run_metrics = executor.get_metrics().get_run_metrics(&run.id).unwrap();
        let cost_metrics = run_metrics.cost_metrics.as_ref().unwrap();
        assert!(cost_metrics.is_completed());
    }

    #[tokio::test]
    async fn test_workflow_failure_with_cost_tracking() {
        let mut executor = WorkflowExecutor::new();
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
        executor.set_cost_tracker(cost_tracker.clone());

        // Create valid workflow that starts but will fail during execution
        let mut workflow = create_workflow("Test Workflow", "Test workflow", "start");
        workflow.add_state(create_state("start", "log: start", false));
        workflow.add_state(create_state(
            "non_existent_state",
            "This state doesn't exist",
            true,
        ));
        workflow.add_transition(create_transition(
            "start",
            "non_existent_state",
            ConditionType::Always,
        ));

        // Execute workflow - this should start successfully but might fail during execution
        let _result = executor
            .start_and_execute_workflow_with_issue_id(
                workflow,
                Some("test-issue-failed".to_string()),
            )
            .await;

        // Verify cost session was created and handled (either completed or failed)
        let tracker = cost_tracker.lock().unwrap();
        // Session should have been created when workflow started
        assert!(
            tracker.session_count() >= 1,
            "Expected at least 1 session, got {}",
            tracker.session_count()
        );

        // The session should be either completed or failed (not active)
        assert_eq!(
            tracker.active_session_count(),
            0,
            "Expected no active sessions"
        );
    }

    #[test]
    fn test_cost_session_id_parsing() {
        let mut executor = WorkflowExecutor::new();
        let cost_tracker = Arc::new(Mutex::new(CostTracker::new()));
        executor.set_cost_tracker(cost_tracker);

        let mut workflow = create_workflow("Test Workflow", "Test workflow", "start");
        workflow.add_state(create_state("start", "Start state", true));

        let run = executor
            .start_workflow_with_issue_id(workflow, Some("test-issue".to_string()))
            .unwrap();

        // Get session ID from metadata
        let session_id_str = run.metadata.get("cost_session_id").unwrap();

        // Verify it can be parsed as a ULID
        let ulid = ulid::Ulid::from_string(session_id_str).unwrap();
        let session_id = CostSessionId::from_ulid(ulid);

        // Should be able to convert back to string
        assert_eq!(session_id_str, &session_id.to_string());
    }
}
