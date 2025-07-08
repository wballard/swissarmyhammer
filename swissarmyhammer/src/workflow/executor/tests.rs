//! Tests for the workflow executor module

use super::*;
use crate::workflow::test_helpers::*;
use crate::workflow::{Transition, WorkflowName, ConditionType, TransitionCondition, StateType, Workflow, WorkflowRun, WorkflowRunStatus};
use std::collections::HashMap;

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
        executor.set_max_history_size(10); // Set small limit for testing

        // Add many events to trigger trimming
        for i in 0..20 {
            executor.log_event(ExecutionEventType::Started, format!("Event {}", i));
        }

        // History should be trimmed to stay under limit
        assert!(executor.get_history().len() <= 10);
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
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), serde_json::Value::Bool(true));

        let condition = TransitionCondition {
            condition_type: ConditionType::OnSuccess,
            expression: None,
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);

        // Test with false result
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), serde_json::Value::Bool(false));
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_on_failure_condition_with_context() {
        let executor = WorkflowExecutor::new();
        let mut context = HashMap::new();
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), serde_json::Value::Bool(false));

        let condition = TransitionCondition {
            condition_type: ConditionType::OnFailure,
            expression: None,
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);

        // Test with true result
        context.insert(LAST_ACTION_RESULT_KEY.to_string(), serde_json::Value::Bool(true));
        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_cel_expression_evaluation() {
        let executor = WorkflowExecutor::new();
        let mut context = HashMap::new();
        context.insert("result".to_string(), serde_json::Value::String("ok".to_string()));

        // Test simple string comparison
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("result == \"ok\"".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);

        // Test default condition
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("default".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_cel_expression_with_variables() {
        let executor = WorkflowExecutor::new();
        let mut context = HashMap::new();
        context.insert("count".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
        context.insert("status".to_string(), serde_json::Value::String("active".to_string()));

        // Test numeric comparison
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("count > 3".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);

        // Test string comparison
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("status == \"active\"".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);

        // Test complex expression
        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("count > 3 && status == \"active\"".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context).unwrap();
        assert!(result);
    }

    #[test]
    fn test_cel_expression_invalid_syntax() {
        let executor = WorkflowExecutor::new();
        let context = HashMap::new();

        let condition = TransitionCondition {
            condition_type: ConditionType::Custom,
            expression: Some("invalid == == syntax".to_string()),
        };

        let result = executor.evaluate_condition(&condition, &context);
        assert!(matches!(result, Err(ExecutorError::ExpressionError(_))));
    }

    #[tokio::test]
    async fn test_choice_state_execution() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with a choice state
        let mut workflow = Workflow::new(
            WorkflowName::new("Choice State Test"),
            "Test choice state execution".to_string(),
            StateId::new("start"),
        );

        // Add states
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state_with_type(
            "choice1",
            "Choice state",
            StateType::Choice,
            false,
        ));
        workflow.add_state(create_state("success", "Success state", true));
        workflow.add_state(create_state("failure", "Failure state", true));

        // Add transitions
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("choice1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        // Choice state with success condition first
        workflow.add_transition(Transition {
            from_state: StateId::new("choice1"),
            to_state: StateId::new("success"),
            condition: TransitionCondition {
                condition_type: ConditionType::OnSuccess,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        // Choice state with default condition as fallback
        workflow.add_transition(Transition {
            from_state: StateId::new("choice1"),
            to_state: StateId::new("failure"),
            condition: TransitionCondition {
                condition_type: ConditionType::Custom,
                expression: Some("default".to_string()),
            },
            action: None,
            metadata: HashMap::new(),
        });

        let run = executor.start_workflow(workflow).await.unwrap();

        // Should go to success state since OnSuccess defaults to true
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "success");
    }

    #[tokio::test]
    async fn test_choice_state_with_cel_conditions() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with a choice state using CEL expressions
        let mut workflow = Workflow::new(
            WorkflowName::new("Choice State CEL Test"),
            "Test choice state with CEL conditions".to_string(),
            StateId::new("start"),
        );

        // Add states
        workflow.add_state(create_state("start", "Set result=\"ok\"", false));
        workflow.add_state(create_state_with_type(
            "choice1",
            "Choice state with CEL",
            StateType::Choice,
            false,
        ));
        workflow.add_state(create_state("success", "Success state", true));
        workflow.add_state(create_state("failure", "Failure state", true));

        // Add transitions
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("choice1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        // Choice state with CEL condition that checks result
        workflow.add_transition(Transition {
            from_state: StateId::new("choice1"),
            to_state: StateId::new("success"),
            condition: TransitionCondition {
                condition_type: ConditionType::Custom,
                expression: Some("result == \"ok\"".to_string()),
            },
            action: None,
            metadata: HashMap::new(),
        });

        // Choice state with default condition as fallback
        workflow.add_transition(Transition {
            from_state: StateId::new("choice1"),
            to_state: StateId::new("failure"),
            condition: TransitionCondition {
                condition_type: ConditionType::Custom,
                expression: Some("default".to_string()),
            },
            action: None,
            metadata: HashMap::new(),
        });

        let run = executor.start_workflow(workflow).await.unwrap();

        // Should go to success state since start state sets result="ok"
        assert_eq!(run.status, WorkflowRunStatus::Completed);
        assert_eq!(run.current_state.as_str(), "success");
    }

    #[tokio::test]
    async fn test_choice_state_no_matching_conditions() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with a choice state where no conditions match
        let mut workflow = Workflow::new(
            WorkflowName::new("Choice State No Match"),
            "Test choice state with no matching conditions".to_string(),
            StateId::new("start"),
        );

        // Add states
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state_with_type(
            "choice1",
            "Choice state",
            StateType::Choice,
            false,
        ));
        workflow.add_state(create_state("success", "Success state", true));

        // Add transitions
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("choice1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        // Choice state with condition that will never match
        workflow.add_transition(Transition {
            from_state: StateId::new("choice1"),
            to_state: StateId::new("success"),
            condition: TransitionCondition {
                condition_type: ConditionType::Never,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        let result = executor.start_workflow(workflow).await;
        assert!(matches!(result, Err(ExecutorError::ExecutionFailed(_))));
    }

    #[tokio::test]
    async fn test_choice_state_no_transitions() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with a choice state that has no outgoing transitions
        let mut workflow = Workflow::new(
            WorkflowName::new("Choice State No Transitions"),
            "Test choice state with no transitions".to_string(),
            StateId::new("start"),
        );

        // Add states
        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state_with_type(
            "choice1",
            "Choice state",
            StateType::Choice,
            false,
        ));
        workflow.add_state(create_state("success", "Success state", true));

        // Add transition to choice state but no transitions from it
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("choice1"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        let result = executor.start_workflow(workflow).await;
        assert!(matches!(result, Err(ExecutorError::ExecutionFailed(_))));
    }

    #[test]
    fn test_transition_order_evaluation() {
        let mut executor = WorkflowExecutor::new();

        // Create a workflow with multiple transitions from the same state
        let mut workflow = Workflow::new(
            WorkflowName::new("Transition Order Test"),
            "Test transition order evaluation".to_string(),
            StateId::new("start"),
        );

        workflow.add_state(create_state("start", "Start state", false));
        workflow.add_state(create_state("first", "First state", true));
        workflow.add_state(create_state("second", "Second state", true));

        // Add transitions in specific order - first should always win
        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("first"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        workflow.add_transition(Transition {
            from_state: StateId::new("start"),
            to_state: StateId::new("second"),
            condition: TransitionCondition {
                condition_type: ConditionType::Always,
                expression: None,
            },
            action: None,
            metadata: HashMap::new(),
        });

        let run = WorkflowRun::new(workflow);
        let next_state = executor.evaluate_transitions(&run).unwrap();

        // Should select the first transition (to "first" state)
        assert_eq!(next_state, Some(StateId::new("first")));
    }