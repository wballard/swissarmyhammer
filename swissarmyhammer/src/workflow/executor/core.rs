//! Core workflow execution logic

use super::{
    ExecutionEvent, ExecutionEventType, ExecutorError, ExecutorResult, DEFAULT_MAX_HISTORY_SIZE,
    LAST_ACTION_RESULT_KEY, MAX_TRANSITIONS,
};
use crate::workflow::{
    metrics::{MemoryMetrics, WorkflowMetrics},
    parse_action_from_description_with_context, ActionError, CompensationKey, ErrorContext,
    StateId, TransitionKey, TransitionPath, Workflow, WorkflowCacheManager, WorkflowRun,
    WorkflowRunStatus,
};
use cel_interpreter::Program;
use serde_json::Value;
use std::time::{Duration, Instant};

/// Configuration for retry behavior
#[derive(Debug, Clone)]
struct RetryConfig {
    /// Maximum number of retry attempts
    max_attempts: usize,
    /// Initial backoff duration in milliseconds
    backoff_ms: u64,
    /// Multiplier for exponential backoff
    backoff_multiplier: f64,
}

impl RetryConfig {
    /// Maximum allowed retry attempts
    const MAX_RETRY_ATTEMPTS: usize = 100;
    /// Maximum allowed initial backoff in milliseconds
    const MAX_BACKOFF_MS: u64 = 60_000; // 1 minute
    /// Maximum allowed backoff multiplier
    const MAX_BACKOFF_MULTIPLIER: f64 = 10.0;
    /// Default initial backoff in milliseconds
    const DEFAULT_BACKOFF_MS: u64 = 100;
    /// Default backoff multiplier for exponential backoff
    const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;
}

/// Workflow execution engine
pub struct WorkflowExecutor {
    /// Execution history for debugging
    execution_history: Vec<ExecutionEvent>,
    /// Maximum size of execution history to prevent unbounded growth
    max_history_size: usize,
    /// Metrics collector for workflow execution
    metrics: WorkflowMetrics,
    /// Cache manager for performance optimizations
    cache_manager: WorkflowCacheManager,
}

impl WorkflowExecutor {
    /// Create a new workflow executor
    pub fn new() -> Self {
        Self {
            execution_history: Vec::new(),
            max_history_size: DEFAULT_MAX_HISTORY_SIZE,
            metrics: WorkflowMetrics::new(),
            cache_manager: WorkflowCacheManager::new(),
        }
    }

    /// Start a new workflow run (initializes but doesn't execute)
    pub fn start_workflow(&mut self, workflow: Workflow) -> ExecutorResult<WorkflowRun> {
        // Validate workflow before starting
        workflow
            .validate()
            .map_err(|errors| ExecutorError::ValidationFailed(errors.join("; ")))?;

        let run = WorkflowRun::new(workflow);

        // Start metrics tracking for this run
        self.metrics.start_run(run.id, run.workflow.name.clone());

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
        let mut run = self.start_workflow(workflow)?;

        // Execute the initial state with transition limit
        let result = self
            .execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await;

        // Complete metrics tracking
        match &result {
            Ok(_) => {
                self.metrics.complete_run(&run.id, run.status, None);
            }
            Err(e) => {
                self.metrics
                    .complete_run(&run.id, WorkflowRunStatus::Failed, Some(e.to_string()));
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
        let result = self
            .execute_state_with_limit(&mut run, MAX_TRANSITIONS)
            .await;

        // Complete metrics tracking
        match &result {
            Ok(_) => {
                self.metrics.complete_run(&run.id, run.status, None);
            }
            Err(e) => {
                self.metrics
                    .complete_run(&run.id, WorkflowRunStatus::Failed, Some(e.to_string()));
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
                format!("State {} requires manual intervention", current_state_id),
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
                    "State {} requires manual approval",
                    current_state_id
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
            return Err(ExecutorError::StateNotFound(next_state.clone()));
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

        tracing::debug!(
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

    /// Get retry configuration from current transition metadata
    fn get_retry_config(&mut self, run: &WorkflowRun) -> Option<RetryConfig> {
        // Find transitions TO the current state (the transition that brought us here)
        let transitions = self.find_transitions_to_state(run, &run.current_state);

        // Look for retry configuration in transition metadata
        for transition in transitions {
            if let Some(max_attempts) = transition.metadata.get("retry_max_attempts") {
                // Parse configuration values safely
                let max_attempts = match max_attempts.parse::<usize>() {
                    Ok(n) if n <= RetryConfig::MAX_RETRY_ATTEMPTS => n,
                    Ok(n) => {
                        self.log_event(
                            ExecutionEventType::Failed,
                            format!(
                                "Retry attempts {} exceeds maximum allowed {}",
                                n,
                                RetryConfig::MAX_RETRY_ATTEMPTS
                            ),
                        );
                        RetryConfig::MAX_RETRY_ATTEMPTS
                    }
                    Err(_) => {
                        self.log_event(
                            ExecutionEventType::Failed,
                            format!("Invalid retry_max_attempts value: {}", max_attempts),
                        );
                        continue;
                    }
                };

                let backoff_ms = transition
                    .metadata
                    .get("retry_backoff_ms")
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(|ms| ms.min(RetryConfig::MAX_BACKOFF_MS))
                    .unwrap_or(RetryConfig::DEFAULT_BACKOFF_MS);

                let backoff_multiplier = transition
                    .metadata
                    .get("retry_backoff_multiplier")
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|m| m.clamp(1.0, RetryConfig::MAX_BACKOFF_MULTIPLIER))
                    .unwrap_or(RetryConfig::DEFAULT_BACKOFF_MULTIPLIER);

                let config = RetryConfig {
                    max_attempts,
                    backoff_ms,
                    backoff_multiplier,
                };

                if config.max_attempts > 0 {
                    return Some(config);
                }
            }
        }

        None
    }

    /// Execute action with retry logic
    async fn execute_action_with_retry(
        &mut self,
        run: &mut WorkflowRun,
        action: Box<dyn crate::workflow::Action>,
        retry_config: &RetryConfig,
    ) -> Result<Value, ActionError> {
        let mut last_error = None;
        let mut backoff_ms = retry_config.backoff_ms;

        for attempt in 1..=retry_config.max_attempts {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Retry attempt {} of {}", attempt, retry_config.max_attempts),
            );

            match action.execute(&mut run.context).await {
                Ok(result) => {
                    self.handle_retry_success(run, attempt);
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error);

                    if attempt < retry_config.max_attempts {
                        self.handle_retry_failure(backoff_ms).await;
                        backoff_ms = self.calculate_next_backoff(backoff_ms, retry_config);
                    }
                }
            }
        }

        // All retries exhausted
        run.context.insert(
            "retry_attempts".to_string(),
            Value::Number(retry_config.max_attempts.into()),
        );
        Err(last_error.unwrap())
    }

    /// Handle successful retry attempt
    fn handle_retry_success(&mut self, run: &mut WorkflowRun, attempt: usize) {
        if attempt > 1 {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Action succeeded on retry attempt {}", attempt),
            );
        }

        // Update error context with retry attempts if it exists
        if let Some(Value::Object(error_obj)) = run.context.get_mut(ErrorContext::CONTEXT_KEY) {
            error_obj.insert("retry_attempts".to_string(), Value::Number(attempt.into()));
        }
    }

    /// Handle failed retry attempt
    async fn handle_retry_failure(&mut self, backoff_ms: u64) {
        self.log_event(
            ExecutionEventType::Failed,
            format!("Action failed, waiting {}ms before retry", backoff_ms),
        );

        // Wait with exponential backoff
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
    }

    /// Calculate next backoff duration
    fn calculate_next_backoff(&self, current_backoff: u64, retry_config: &RetryConfig) -> u64 {
        (current_backoff as f64 * retry_config.backoff_multiplier) as u64
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
            let result = self.execute_action_with_config(run, action).await;
            self.handle_action_result(run, result).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Execute action with retry configuration if available
    async fn execute_action_with_config(
        &mut self,
        run: &mut WorkflowRun,
        action: Box<dyn crate::workflow::Action>,
    ) -> Result<Value, ActionError> {
        let retry_config = self.get_retry_config(run);

        if let Some(config) = retry_config {
            self.execute_action_with_retry(run, action, &config).await
        } else {
            action.execute(&mut run.context).await
        }
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
                    format!(
                        "Action completed successfully with result: {}",
                        result_value
                    ),
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
                format!("Compensation failed: {}", comp_error),
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
        let retry_attempts = run
            .context
            .get("retry_attempts")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let error_context = if let Some(attempts) = retry_attempts {
            ErrorContext::with_retries(
                action_error.to_string(),
                run.current_state.clone(),
                attempts,
            )
        } else {
            ErrorContext::new(action_error.to_string(), run.current_state.clone())
        };

        let error_context_json = serde_json::to_value(&error_context).unwrap_or(Value::Null);
        run.context
            .insert(ErrorContext::CONTEXT_KEY.to_string(), error_context_json);
    }

    /// Format action error for logging
    fn format_action_error(&self, action_error: &ActionError) -> String {
        match action_error {
            ActionError::Timeout { timeout } => {
                format!("Action timed out after {:?}", timeout)
            }
            ActionError::ClaudeError(msg) => format!("Claude command failed: {}", msg),
            ActionError::VariableError(msg) => {
                format!("Variable operation failed: {}", msg)
            }
            ActionError::IoError(io_err) => format!("IO operation failed: {}", io_err),
            ActionError::JsonError(json_err) => {
                format!("JSON parsing failed: {}", json_err)
            }
            ActionError::ParseError(msg) => format!("Action parsing failed: {}", msg),
            ActionError::ExecutionError(msg) => {
                format!("Action execution failed: {}", msg)
            }
            ActionError::RateLimit { message, wait_time } => {
                format!(
                    "Rate limit reached: {}. Please wait {:?} before retrying.",
                    message, wait_time
                )
            }
            ActionError::AbortError(msg) => format!("ABORT ERROR: {}", msg),
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
            Value::String(format!("Max retries exhausted: {}", action_error)),
        );

        // Transition to dead letter state
        self.log_event(
            ExecutionEventType::StateTransition,
            format!("Transitioning to dead letter state: {}", dead_letter_state),
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

    /// Execute compensation states in reverse order
    async fn execute_compensation(&mut self, run: &mut WorkflowRun) -> ExecutorResult<()> {
        self.log_event(
            ExecutionEventType::StateExecution,
            "Starting compensation/rollback".to_string(),
        );

        // Find all compensation states stored in context
        let mut compensation_states: Vec<(String, StateId)> = Vec::new();

        for (key, value) in &run.context {
            if CompensationKey::is_compensation_key(key) {
                if let Value::String(comp_state) = value {
                    compensation_states.push((key.clone(), StateId::new(comp_state)));
                }
            }
        }

        // Execute compensation states
        if let Some((key, comp_state)) = compensation_states.into_iter().next() {
            self.log_event(
                ExecutionEventType::StateExecution,
                format!("Executing compensation state: {}", comp_state),
            );

            // Just transition to the compensation state, don't execute it
            // The normal workflow execution will handle it
            self.perform_transition(run, comp_state)?;

            // Remove from context after execution
            run.context.remove(&key);
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

    /// Check if a CEL program is cached
    pub fn is_cel_program_cached(&self, expression: &str) -> bool {
        self.cache_manager.cel_cache.get(expression).is_some()
    }

    /// Get CEL program cache statistics
    pub fn get_cel_cache_stats(&self) -> (usize, usize) {
        let stats = self.cache_manager.cel_cache.stats();
        (stats.size, stats.capacity)
    }

    /// Get cache manager for advanced cache operations
    pub fn get_cache_manager(&self) -> &WorkflowCacheManager {
        &self.cache_manager
    }

    /// Get mutable cache manager for advanced cache operations
    pub fn get_cache_manager_mut(&mut self) -> &mut WorkflowCacheManager {
        &mut self.cache_manager
    }

    /// Cache a transition path for optimization
    pub fn cache_transition_path(
        &mut self,
        from_state: StateId,
        to_state: StateId,
        conditions: Vec<String>,
    ) {
        let key = TransitionKey::new(from_state.clone(), to_state.clone());
        let path = TransitionPath::new(from_state, to_state, conditions);
        self.cache_manager.transition_cache.put(key, path);
    }

    /// Get cached transition path if available
    pub fn get_cached_transition_path(
        &self,
        from_state: &StateId,
        to_state: &StateId,
    ) -> Option<TransitionPath> {
        let key = TransitionKey::new(from_state.clone(), to_state.clone());
        self.cache_manager.transition_cache.get(&key)
    }

    /// Clear all caches
    pub fn clear_all_caches(&mut self) {
        self.cache_manager.clear_all();
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}
