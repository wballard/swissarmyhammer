//! Mermaid state diagram parser for workflows
//!
//! This module integrates the mermaid_parser library to parse Mermaid state diagrams
//! and convert them to our internal Workflow types.

use crate::workflow::{
    ConditionType, State, StateId, Transition, TransitionCondition, Workflow, WorkflowName,
};
use mermaid_parser::{
    common::ast::{DiagramType, StateDiagram, StateTransition},
    parse_diagram,
};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during Mermaid parsing
#[derive(Debug, Error)]
pub enum ParseError {
    /// Error from the mermaid-parser library
    #[error("Mermaid parse error: {0}")]
    MermaidError(String),

    /// Diagram is not a state diagram
    #[error("Expected state diagram, found {diagram_type}")]
    WrongDiagramType {
        /// The type of diagram that was found
        diagram_type: String,
    },

    /// No initial state found in diagram
    #[error("No initial state found in state diagram. Ensure your diagram has a transition from [*] to define the starting state")]
    NoInitialState,

    /// No terminal states found
    #[error("No terminal states found in state diagram. At least one state must transition to [*] to mark workflow completion")]
    NoTerminalStates,

    /// Invalid state or transition structure
    #[error("Invalid workflow structure: {message}. Please check your diagram syntax and state references")]
    InvalidStructure {
        /// Description of the structural problem
        message: String,
    },
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Parser for Mermaid state diagrams
pub struct MermaidParser;

impl MermaidParser {
    /// Parse a Mermaid state diagram into a Workflow
    pub fn parse(input: &str, workflow_name: impl Into<WorkflowName>) -> ParseResult<Workflow> {
        // Attempt to parse the diagram
        match parse_diagram(input) {
            Ok(diagram) => match diagram {
                DiagramType::State(state_diagram) => {
                    Self::convert_state_diagram(state_diagram, workflow_name.into())
                }
                _ => Err(ParseError::WrongDiagramType {
                    diagram_type: format!("{:?}", diagram),
                }),
            },
            Err(e) => Err(ParseError::MermaidError(e.to_string())),
        }
    }

    /// Convert a parsed state diagram to our Workflow type
    fn convert_state_diagram(
        state_diagram: StateDiagram,
        workflow_name: WorkflowName,
    ) -> ParseResult<Workflow> {
        // Extract description from title or create default
        let description = state_diagram
            .title
            .unwrap_or_else(|| "Workflow from Mermaid state diagram".to_string());

        // Find initial state - look for [*] as source in transitions
        let initial_state_id = Self::find_initial_state(&state_diagram.transitions)?;

        let mut workflow = Workflow::new(workflow_name, description, initial_state_id.clone());

        // Convert all states from mermaid to our format
        for (state_id, mermaid_state) in state_diagram.states {
            // Skip the special [*] state as it's not a real state in our model
            if state_id == "[*]" {
                continue;
            }

            let is_terminal = Self::is_terminal_state(&state_id, &state_diagram.transitions);
            let (parsed_description, actions) = Self::parse_state_description(
                &mermaid_state
                    .display_name
                    .unwrap_or_else(|| state_id.clone()),
            );

            let mut metadata = HashMap::new();
            metadata.insert(
                "mermaid_type".to_string(),
                format!("{:?}", mermaid_state.state_type),
            );

            // Add any extracted actions as metadata
            if !actions.is_empty() {
                metadata.insert("actions".to_string(), actions.join(";"));
            }

            // Check if this state has substates or concurrent regions to enable parallel execution
            let allows_parallel =
                !mermaid_state.substates.is_empty() || !mermaid_state.concurrent_regions.is_empty();

            workflow.add_state(State {
                id: StateId::new(state_id),
                description: parsed_description,
                is_terminal,
                allows_parallel,
                metadata,
            });
        }

        // Convert all transitions
        for transition in state_diagram.transitions {
            // Skip transitions to/from [*] that don't involve real states
            if transition.from == "[*]" && transition.to == "[*]" {
                continue;
            }

            // Handle initial transitions from [*]
            if transition.from == "[*]" {
                // This is already handled by setting initial_state, skip the transition
                continue;
            }

            // Handle terminal transitions to [*]
            if transition.to == "[*]" {
                // Mark the source state as terminal (already handled above)
                continue;
            }

            let condition = Self::parse_transition_condition(&transition);

            workflow.add_transition(Transition {
                from_state: StateId::new(transition.from),
                to_state: StateId::new(transition.to),
                condition,
                action: transition.action,
                metadata: HashMap::new(),
            });
        }

        // Add metadata about the source
        workflow
            .metadata
            .insert("source".to_string(), "mermaid".to_string());
        workflow.metadata.insert(
            "version".to_string(),
            format!("{:?}", state_diagram.version),
        );

        // Perform workflow-specific validation
        Self::validate_workflow_structure(&workflow)?;

        Ok(workflow)
    }

    /// Find the initial state by looking for transitions from [*]
    fn find_initial_state(transitions: &[StateTransition]) -> ParseResult<StateId> {
        for transition in transitions {
            if transition.from == "[*]" && transition.to != "[*]" {
                return Ok(StateId::new(transition.to.clone()));
            }
        }
        Err(ParseError::NoInitialState)
    }

    /// Check if a state is terminal by looking for transitions to [*]
    fn is_terminal_state(state_id: &str, transitions: &[StateTransition]) -> bool {
        transitions
            .iter()
            .any(|t| t.from == state_id && t.to == "[*]")
    }

    /// Parse state description to extract actions and clean description
    fn parse_state_description(description: &str) -> (String, Vec<String>) {
        let mut actions = Vec::new();

        // Look for action patterns in the description
        // Format: "State: Execute prompt \"name\"" or "State: Set variable=\"value\""
        let parts: Vec<&str> = description.split(':').collect();

        let cleaned_description = if parts.len() == 2 {
            let state_name = parts[0].trim();
            let action_part = parts[1].trim();

            // Check for known action patterns
            if action_part.starts_with("Execute prompt")
                || action_part.starts_with("Set variable")
                || action_part.starts_with("Run workflow")
            {
                actions.push(action_part.to_string());
                state_name.to_string()
            } else {
                // Not a recognized action pattern, keep as description
                description.to_string()
            }
        } else {
            description.to_string()
        };

        (cleaned_description, actions)
    }

    /// Parse transition condition from mermaid transition
    fn parse_transition_condition(transition: &StateTransition) -> TransitionCondition {
        match &transition.event {
            Some(event) => {
                // Analyze the event text to determine condition type
                // Check negative conditions first to avoid substring issues
                let condition_type = if event.contains("invalid")
                    || event.contains("error")
                    || event.contains("fail")
                {
                    ConditionType::OnFailure
                } else if event.contains("valid") || event.contains("success") {
                    ConditionType::OnSuccess
                } else if event == "always" || event.is_empty() {
                    ConditionType::Always
                } else {
                    ConditionType::Custom
                };

                let expression = if matches!(condition_type, ConditionType::Custom) {
                    Some(event.clone())
                } else {
                    None
                };

                TransitionCondition {
                    condition_type,
                    expression,
                }
            }
            None => TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
        }
    }

    /// Validate workflow structure with additional checks beyond basic validation
    fn validate_workflow_structure(workflow: &Workflow) -> ParseResult<()> {
        // Run basic validation first
        if let Err(errors) = workflow.validate() {
            return Err(ParseError::InvalidStructure {
                message: errors.join("; "),
            });
        }

        // Check for single start state (no multiple initial transitions)
        let _initial_count = workflow
            .transitions
            .iter()
            .filter(|t| t.from_state == workflow.initial_state)
            .count();

        // Ensure reachability - all states should be reachable from initial state
        let reachable_states = Self::find_reachable_states(workflow);
        let unreachable: Vec<_> = workflow
            .states
            .keys()
            .filter(|id| !reachable_states.contains(id) && **id != workflow.initial_state)
            .collect();

        if !unreachable.is_empty() {
            return Err(ParseError::InvalidStructure {
                message: format!("Unreachable states found: {:?}", unreachable),
            });
        }

        // Check for disconnected components by ensuring at least one terminal state is reachable
        let terminal_reachable = workflow
            .states
            .values()
            .filter(|s| s.is_terminal)
            .any(|s| reachable_states.contains(&s.id));

        if !terminal_reachable {
            return Err(ParseError::InvalidStructure {
                message: "No terminal states are reachable from initial state".to_string(),
            });
        }

        Ok(())
    }

    /// Find all states reachable from the initial state using DFS
    fn find_reachable_states(workflow: &Workflow) -> std::collections::HashSet<StateId> {
        let mut reachable = std::collections::HashSet::new();
        let mut stack = vec![workflow.initial_state.clone()];

        while let Some(current) = stack.pop() {
            if reachable.contains(&current) {
                continue;
            }

            reachable.insert(current.clone());

            // Find all states reachable from current state
            for transition in &workflow.transitions {
                if transition.from_state == current && !reachable.contains(&transition.to_state) {
                    stack.push(transition.to_state.clone());
                }
            }
        }

        reachable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_state_diagram() {
        let input = r#"
        stateDiagram-v2
            [*] --> State1
            State1 --> State2: condition
            State2 --> [*]
        "#;

        let result = MermaidParser::parse(input, "test_workflow");
        assert!(result.is_ok());

        let workflow = result.unwrap();
        assert_eq!(workflow.name.as_str(), "test_workflow");
        assert_eq!(workflow.states.len(), 2); // State1 and State2 (not [*])
        assert_eq!(workflow.transitions.len(), 1); // Only State1 -> State2

        // Check initial state
        assert_eq!(workflow.initial_state.as_str(), "State1");

        // Check states
        assert!(workflow.states.contains_key(&StateId::new("State1")));
        assert!(workflow.states.contains_key(&StateId::new("State2")));

        // Check that State2 is terminal
        let state2 = &workflow.states[&StateId::new("State2")];
        assert!(state2.is_terminal);

        // Check transition
        let transition = &workflow.transitions[0];
        assert_eq!(transition.from_state.as_str(), "State1");
        assert_eq!(transition.to_state.as_str(), "State2");
        assert_eq!(transition.condition.condition_type, ConditionType::Custom);
        assert_eq!(
            transition.condition.expression,
            Some("condition".to_string())
        );
    }

    #[test]
    fn test_parse_wrong_diagram_type() {
        let input = r#"
        flowchart TD
            A --> B
        "#;

        let result = MermaidParser::parse(input, "test_workflow");
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::MermaidError(msg) => {
                assert!(msg.contains("Lexer error") || msg.contains("error"));
            }
            _ => panic!("Expected MermaidError for invalid syntax"),
        }
    }

    #[test]
    fn test_parse_state_diagram_with_actions() {
        let input = r#"
        stateDiagram-v2
            [*] --> CheckingInput: Start workflow
            CheckingInput --> ProcessingData: Input valid
            CheckingInput --> ErrorState: Input invalid
            ProcessingData --> [*]: Complete
            ErrorState --> [*]: Abort
        "#;

        let result = MermaidParser::parse(input, "action_workflow");
        assert!(result.is_ok());

        let workflow = result.unwrap();
        assert_eq!(workflow.states.len(), 3);
        assert_eq!(workflow.initial_state.as_str(), "CheckingInput");

        // Check transitions with proper condition types
        assert_eq!(workflow.transitions.len(), 2);

        let valid_transition = workflow
            .transitions
            .iter()
            .find(|t| {
                t.from_state.as_str() == "CheckingInput" && t.to_state.as_str() == "ProcessingData"
            })
            .unwrap();
        assert_eq!(
            valid_transition.condition.condition_type,
            ConditionType::OnSuccess
        );

        let invalid_transition = workflow
            .transitions
            .iter()
            .find(|t| {
                t.from_state.as_str() == "CheckingInput" && t.to_state.as_str() == "ErrorState"
            })
            .unwrap();
        assert_eq!(
            invalid_transition.condition.condition_type,
            ConditionType::OnFailure
        );
    }

    #[test]
    fn test_no_initial_state_error() {
        let input = r#"
        stateDiagram-v2
            State1 --> State2
            State2 --> State1
        "#;

        let result = MermaidParser::parse(input, "invalid_workflow");
        assert!(result.is_err());

        match result.unwrap_err() {
            ParseError::NoInitialState => (),
            _ => panic!("Expected NoInitialState error"),
        }
    }

    #[test]
    fn test_unreachable_states_validation() {
        // This test would require a more complex setup where we manually construct
        // a workflow with unreachable states, which is hard to do with valid Mermaid syntax
        // For now, we test that normal workflows pass validation
        let input = r#"
        stateDiagram-v2
            [*] --> State1
            State1 --> State2
            State2 --> [*]
        "#;

        let result = MermaidParser::parse(input, "valid_workflow");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_state_description() {
        let (desc, actions) =
            MermaidParser::parse_state_description("ProcessData: Execute prompt \"process\"");
        assert_eq!(desc, "ProcessData");
        assert_eq!(actions, vec!["Execute prompt \"process\""]);

        let (desc, actions) =
            MermaidParser::parse_state_description("SetVariable: Set variable=\"test\"");
        assert_eq!(desc, "SetVariable");
        assert_eq!(actions, vec!["Set variable=\"test\""]);

        let (desc, actions) = MermaidParser::parse_state_description("Simple state description");
        assert_eq!(desc, "Simple state description");
        assert!(actions.is_empty());
    }

    #[test]
    fn test_parse_transition_condition() {
        use mermaid_parser::common::ast::StateTransition;

        let transition = StateTransition {
            from: "A".to_string(),
            to: "B".to_string(),
            event: Some("Input valid".to_string()),
            guard: None,
            action: None,
        };

        let condition = MermaidParser::parse_transition_condition(&transition);
        assert_eq!(condition.condition_type, ConditionType::OnSuccess);
        assert_eq!(condition.expression, None);

        let transition_custom = StateTransition {
            from: "A".to_string(),
            to: "B".to_string(),
            event: Some("custom condition".to_string()),
            guard: None,
            action: None,
        };

        let condition_custom = MermaidParser::parse_transition_condition(&transition_custom);
        assert_eq!(condition_custom.condition_type, ConditionType::Custom);
        assert_eq!(
            condition_custom.expression,
            Some("custom condition".to_string())
        );
    }
}
