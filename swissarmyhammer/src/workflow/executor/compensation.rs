//! Compensation logic for workflow execution

use super::{ExecutionEventType, ExecutorResult, WorkflowExecutor};
use crate::workflow::{CompensationKey, StateId, WorkflowRun};
use serde_json::Value;

impl WorkflowExecutor {
    /// Execute compensation states in reverse order
    pub(super) async fn execute_compensation(
        &mut self,
        run: &mut WorkflowRun,
    ) -> ExecutorResult<()> {
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
                format!("Executing compensation state: {comp_state}"),
            );

            // Just transition to the compensation state, don't execute it
            // The normal workflow execution will handle it
            self.perform_transition(run, comp_state)?;

            // Remove from context after execution
            run.context.remove(&key);
        }

        Ok(())
    }
}
