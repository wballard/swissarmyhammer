//! State-related types for workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Types of workflow states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StateType {
    /// Normal workflow state
    #[default]
    Normal,
    /// Fork state for parallel execution
    Fork,
    /// Join state for merging parallel branches
    Join,
    /// Choice state for conditional branching
    Choice,
}

impl StateType {
    /// Get the string representation of the state type
    pub fn as_str(&self) -> &'static str {
        match self {
            StateType::Normal => "Normal",
            StateType::Fork => "Fork",
            StateType::Join => "Join",
            StateType::Choice => "Choice",
        }
    }
}

/// Errors that can occur when creating state-related types
#[derive(Debug, Error)]
pub enum StateError {
    /// State ID cannot be empty or whitespace only
    #[error("State ID cannot be empty or whitespace only")]
    EmptyStateId,
}

/// Result type for state operations
pub type StateResult<T> = Result<T, StateError>;

/// Unique identifier for workflow states
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(String);

impl StateId {
    /// Create a new state ID
    ///
    /// # Panics
    /// Panics if the ID is empty or whitespace only. For non-panicking creation,
    /// use `try_new` instead.
    pub fn new(id: impl Into<String>) -> Self {
        Self::try_new(id).expect("State ID cannot be empty or whitespace only")
    }

    /// Create a new state ID, returning an error for invalid input
    pub fn try_new(id: impl Into<String>) -> StateResult<Self> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(StateError::EmptyStateId);
        }
        Ok(Self(id))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for StateId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StateId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a state in the workflow
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Unique identifier for the state
    pub id: StateId,
    /// Description of what should happen in this state
    pub description: String,
    /// Type of state (normal, fork, join)
    pub state_type: StateType,
    /// Whether this is a terminal state
    pub is_terminal: bool,
    /// Whether this state allows parallel execution
    pub allows_parallel: bool,
    /// Metadata for debugging and monitoring
    pub metadata: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_id_creation() {
        let id1 = StateId::new("start");
        let id2 = StateId::from("start");
        let id3: StateId = "start".into();

        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert_eq!(id1.as_str(), "start");
    }

    #[test]
    fn test_state_id_try_new_success() {
        let id = StateId::try_new("valid_id").unwrap();
        assert_eq!(id.as_str(), "valid_id");
    }

    #[test]
    fn test_state_id_try_new_empty_error() {
        assert!(StateId::try_new("").is_err());
        assert!(StateId::try_new("   ").is_err());
        assert!(StateId::try_new("\t\n").is_err());
    }

    #[test]
    #[should_panic(expected = "State ID cannot be empty or whitespace only")]
    fn test_state_id_new_panics_on_empty() {
        StateId::new("");
    }

    #[test]
    fn test_state_creation() {
        let state = State {
            id: StateId::new("start"),
            description: "Initial state of the workflow".to_string(),
            state_type: StateType::Normal,
            is_terminal: false,
            allows_parallel: false,
            metadata: HashMap::new(),
        };

        assert_eq!(state.id.as_str(), "start");
        assert!(!state.is_terminal);
        assert_eq!(state.state_type, StateType::Normal);
    }

    #[test]
    fn test_state_serialization() {
        let state = State {
            id: StateId::new("test"),
            description: "A test state".to_string(),
            state_type: StateType::Fork,
            is_terminal: false,
            allows_parallel: true,
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: State = serde_json::from_str(&serialized).unwrap();

        assert_eq!(state, deserialized);
        assert_eq!(deserialized.state_type, StateType::Fork);
    }
}
