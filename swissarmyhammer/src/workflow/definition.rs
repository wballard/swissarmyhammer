//! Main workflow type and validation

use crate::validation::{Validatable, ValidationIssue, ValidationLevel};
use crate::workflow::{State, StateId, Transition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur when creating workflow-related types
#[derive(Debug, Error)]
pub enum WorkflowError {
    /// Workflow name cannot be empty or whitespace only
    #[error("Workflow name cannot be empty or whitespace only")]
    EmptyWorkflowName,
}

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Unique identifier for workflows
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowName(String);

impl WorkflowName {
    /// Create a new workflow name
    ///
    /// # Panics
    /// Panics if the name is empty or whitespace only. For non-panicking creation,
    /// use `try_new` instead.
    pub fn new(name: impl Into<String>) -> Self {
        Self::try_new(name).expect("Workflow name cannot be empty or whitespace only")
    }

    /// Create a new workflow name, returning an error for invalid input
    pub fn try_new(name: impl Into<String>) -> WorkflowResult<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorkflowError::EmptyWorkflowName);
        }
        Ok(Self(name))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for WorkflowName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for WorkflowName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for WorkflowName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Main workflow representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow name
    pub name: WorkflowName,
    /// Workflow description
    pub description: String,
    /// All states in the workflow
    pub states: HashMap<StateId, State>,
    /// All transitions in the workflow
    pub transitions: Vec<Transition>,
    /// Initial state ID
    pub initial_state: StateId,
    /// Metadata for debugging and monitoring
    pub metadata: HashMap<String, String>,
}

impl Workflow {
    /// Create a new workflow with basic validation
    pub fn new(name: WorkflowName, description: String, initial_state: StateId) -> Self {
        Self {
            name,
            description,
            states: Default::default(),
            transitions: Vec::new(),
            initial_state,
            metadata: Default::default(),
        }
    }

    /// Validate the workflow structure
    pub fn validate_structure(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check if workflow name is not empty
        if self.name.as_str().trim().is_empty() {
            errors.push("Workflow name cannot be empty".to_string());
        }

        // Check if initial state exists
        if !self.states.contains_key(&self.initial_state) {
            errors.push(format!(
                "Initial state '{}' not found in workflow states. Available states: {:?}",
                self.initial_state,
                self.states.keys().map(|k| k.as_str()).collect::<Vec<_>>()
            ));
        }

        // Check if all transitions reference existing states
        for transition in &self.transitions {
            // Check for empty state IDs in transitions
            if transition.from_state.as_str().trim().is_empty() {
                errors.push(format!("Transition #{} has empty source state ID. All transitions must have valid non-empty state IDs", self.transitions.iter().position(|t| t == transition).unwrap_or(0)));
            }
            if transition.to_state.as_str().trim().is_empty() {
                errors.push(format!("Transition #{} has empty target state ID. All transitions must have valid non-empty state IDs", self.transitions.iter().position(|t| t == transition).unwrap_or(0)));
            }

            if !self.states.contains_key(&transition.from_state) {
                errors.push(format!(
                    "Transition references non-existent source state: '{}'",
                    transition.from_state
                ));
            }
            if !self.states.contains_key(&transition.to_state) {
                errors.push(format!(
                    "Transition references non-existent target state: '{}'",
                    transition.to_state
                ));
            }
        }

        // Check for at least one terminal state
        let has_terminal = self.states.values().any(|s| s.is_terminal);
        if !has_terminal {
            errors.push("Workflow must have at least one terminal state. Add 'is_terminal: true' to at least one state or create a transition to [*]".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Add a state to the workflow
    pub fn add_state(&mut self, state: State) {
        self.states.insert(state.id.clone(), state);
    }

    /// Add a transition to the workflow
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }
}

impl Validatable for Workflow {
    fn validate(&self, source_path: Option<&Path>) -> Vec<ValidationIssue> {
        match self.validate_structure() {
            Ok(()) => Vec::new(),
            Err(error_messages) => {
                let workflow_path = source_path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
                    std::path::PathBuf::from(format!("workflow:{}", self.name.as_str()))
                });

                error_messages
                    .into_iter()
                    .map(|message| ValidationIssue {
                        level: ValidationLevel::Error,
                        file_path: workflow_path.clone(),
                        content_title: Some(self.name.to_string()),
                        line: None,
                        column: None,
                        message,
                        suggestion: None,
                    })
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::test_helpers::*;

    #[test]
    fn test_workflow_validation_success() {
        let workflow = create_basic_workflow();
        assert!(workflow.validate_structure().is_ok());
    }

    #[test]
    fn test_workflow_validation_missing_initial_state() {
        let workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );

        let result = workflow.validate_structure();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Initial state")));
    }

    #[test]
    fn test_workflow_validation_no_terminal_state() {
        let mut workflow = Workflow::new(
            WorkflowName::new("Test Workflow"),
            "A test workflow".to_string(),
            StateId::new("start"),
        );

        workflow.add_state(create_state("start", "Start state", false));

        let result = workflow.validate_structure();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("terminal state")));
    }
}
