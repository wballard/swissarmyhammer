//! Condition evaluation and validation functionality

use crate::workflow::{
    ConditionType, StateId, TransitionCondition, WorkflowRun,
};
use serde_json::Value;
use std::collections::HashMap;
use super::{ExecutorError, ExecutorResult, ExecutionEventType, LAST_ACTION_RESULT_KEY};
use super::core::WorkflowExecutor;
use cel_interpreter::{Context, Program, Value as CelValue};

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

        // Check if this is a choice state and validate it has transitions
        let is_choice_state = run
            .workflow
            .states
            .get(current_state)
            .map(|state| state.state_type == crate::workflow::StateType::Choice)
            .unwrap_or(false);

        if is_choice_state && transitions.is_empty() {
            return Err(ExecutorError::ExecutionFailed(
                format!(
                    "Choice state '{}' has no outgoing transitions. Choice states must have at least one outgoing transition",
                    current_state
                ),
            ));
        }

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

        // If this is a choice state and no conditions matched, it's an error
        if is_choice_state {
            return Err(ExecutorError::ExecutionFailed(
                format!(
                    "Choice state '{}' has no matching conditions. All transition conditions evaluated to false",
                    current_state
                ),
            ));
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
                    self.evaluate_cel_expression(expression, context)
                } else {
                    Err(ExecutorError::ExpressionError(
                        "Custom condition requires an expression".to_string(),
                    ))
                }
            }
        }
    }

    /// Evaluate a CEL expression with the given context
    fn evaluate_cel_expression(
        &self,
        expression: &str,
        context: &HashMap<String, Value>,
    ) -> ExecutorResult<bool> {
        // Parse the CEL expression
        let program = Program::compile(expression)
            .map_err(|e| ExecutorError::ExpressionError(format!("Failed to compile CEL expression '{}': {}", expression, e)))?;

        // Create CEL context with workflow variables
        let mut cel_context = Context::default();
        
        // Add 'default' variable that is always true
        cel_context.add_variable("default", true)
            .map_err(|e| ExecutorError::ExpressionError(format!("Failed to add 'default' variable: {}", e)))?;
        
        // Add 'result' variable from the final response
        let result_text = self.extract_result_text(context);
        cel_context.add_variable("result", result_text)
            .map_err(|e| ExecutorError::ExpressionError(format!("Failed to add 'result' variable: {}", e)))?;
        
        // Add other context variables
        for (key, value) in context {
            self.add_json_variable_to_cel_context(&mut cel_context, key, value)
                .map_err(|e| ExecutorError::ExpressionError(format!("Failed to add variable '{}': {}", key, e)))?;
        }

        // Execute the expression
        let result = program.execute(&cel_context)
            .map_err(|e| ExecutorError::ExpressionError(format!("Failed to execute CEL expression '{}': {}", expression, e)))?;

        // Convert result to boolean
        self.cel_value_to_bool(&result, expression)
    }

    /// Extract result text from context for CEL evaluation
    fn extract_result_text(&self, context: &HashMap<String, Value>) -> String {
        // Look for common result keys
        let result_keys = ["result", "output", "response", "claude_result"];
        
        for key in &result_keys {
            if let Some(value) = context.get(*key) {
                return match value {
                    Value::String(s) => s.clone(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };
            }
        }
        
        // Default empty string if no result found
        String::new()
    }

    /// Add JSON variable to CEL context
    fn add_json_variable_to_cel_context(&self, cel_context: &mut Context, key: &str, value: &Value) -> Result<(), Box<dyn std::error::Error>> {
        match value {
            Value::Bool(b) => {
                cel_context.add_variable(key, *b)?;
            }
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    cel_context.add_variable(key, i)?;
                } else if let Some(f) = n.as_f64() {
                    cel_context.add_variable(key, f)?;
                }
            }
            Value::String(s) => {
                cel_context.add_variable(key, s.clone())?;
            }
            Value::Null => {
                // Skip null values for now
            }
            _ => {
                // Arrays and objects not supported for now
            }
        }
        Ok(())
    }

    /// Convert CEL value to boolean
    fn cel_value_to_bool(&self, value: &CelValue, expression: &str) -> ExecutorResult<bool> {
        match value {
            CelValue::Bool(b) => Ok(*b),
            CelValue::Int(i) => Ok(*i != 0),
            CelValue::Float(f) => Ok(*f != 0.0),
            CelValue::String(s) => Ok(!s.is_empty()),
            CelValue::Null => Ok(false),
            _ => Err(ExecutorError::ExpressionError(format!(
                "CEL expression '{}' returned non-boolean result: {:?}",
                expression, value
            ))),
        }
    }
}