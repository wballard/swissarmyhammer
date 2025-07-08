//! Condition evaluation and validation functionality

use crate::workflow::{
    ConditionType, StateId, TransitionCondition, WorkflowRun,
};
use serde_json::Value;
use std::collections::HashMap;
use super::{ExecutorError, ExecutorResult, ExecutionEventType, LAST_ACTION_RESULT_KEY};
use super::core::WorkflowExecutor;

impl WorkflowExecutor {
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

    /// Helper function to evaluate action-based conditions (success/failure)
    pub fn evaluate_action_condition(
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
    pub fn evaluate_condition(
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
}