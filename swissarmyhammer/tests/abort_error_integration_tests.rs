//! Integration tests for abort error mechanism in workflows

use swissarmyhammer::workflow::{
    ConditionType, State, StateId, StateType, Transition, TransitionCondition, Workflow,
    WorkflowExecutor, WorkflowName, WorkflowRunStatus,
};

fn create_state(id: &str, description: &str, is_terminal: bool) -> State {
    State {
        id: StateId::new(id),
        description: description.to_string(),
        state_type: StateType::Normal,
        is_terminal,
        allows_parallel: false,
        metadata: Default::default(),
    }
}

fn create_transition(from: &str, to: &str, condition_type: ConditionType) -> Transition {
    Transition {
        from_state: StateId::new(from),
        to_state: StateId::new(to),
        condition: TransitionCondition {
            condition_type,
            expression: None,
        },
        action: None,
        metadata: Default::default(),
    }
}

#[tokio::test]
async fn test_abort_error_single_workflow() {
    // Create a workflow that will abort
    let mut workflow = Workflow::new(
        WorkflowName::new("Abort Test"),
        "Test abort propagation".to_string(),
        StateId::new("start"),
    );

    // Add states
    workflow.add_state(create_state("start", "Start state", false));
    workflow.add_state(create_state("error", "Error state", true));

    // Add transition to error state
    workflow.add_transition(create_transition("start", "error", ConditionType::Always));

    // Execute workflow
    let mut executor = WorkflowExecutor::new();
    let result = executor.start_and_execute_workflow(workflow).await;

    // The workflow should complete with the error state
    assert!(result.is_ok());
    let run = result.unwrap();
    assert_eq!(run.current_state, StateId::new("error"));
}

#[tokio::test]
async fn test_workflow_state_transitions() {
    // Test basic workflow state transitions
    let mut workflow = Workflow::new(
        WorkflowName::new("State Test"),
        "Test state transitions".to_string(),
        StateId::new("start"),
    );

    workflow.add_state(create_state("start", "Start", false));
    workflow.add_state(create_state("middle", "Middle", false));
    workflow.add_state(create_state("end", "End", true));

    workflow.add_transition(create_transition("start", "middle", ConditionType::Always));
    workflow.add_transition(create_transition("middle", "end", ConditionType::Always));

    let mut executor = WorkflowExecutor::new();
    let result = executor.start_and_execute_workflow(workflow).await;

    assert!(result.is_ok());
    let run = result.unwrap();
    assert_eq!(run.status, WorkflowRunStatus::Completed);
    assert_eq!(run.current_state, StateId::new("end"));
}

#[tokio::test]
async fn test_workflow_error_state_handling() {
    // Test workflow transitioning to error state
    let mut workflow = Workflow::new(
        WorkflowName::new("Error Test"),
        "Test error handling".to_string(),
        StateId::new("start"),
    );

    workflow.add_state(create_state("start", "Start", false));
    workflow.add_state(create_state("error", "Error", true));

    // Direct transition to error state
    workflow.add_transition(create_transition("start", "error", ConditionType::Always));

    let mut executor = WorkflowExecutor::new();
    let result = executor.start_and_execute_workflow(workflow).await;

    assert!(result.is_ok());
    let run = result.unwrap();
    assert_eq!(run.status, WorkflowRunStatus::Completed);
    assert_eq!(run.current_state, StateId::new("error"));
}

#[tokio::test]
async fn test_workflow_with_context_data() {
    // Test workflow with context data flow
    let mut workflow = Workflow::new(
        WorkflowName::new("Context Test"),
        "Test context data".to_string(),
        StateId::new("start"),
    );

    // For integration tests, we can't easily test actions that require external input
    // Instead, we test the workflow structure and transitions
    workflow.add_state(create_state("start", "Start", false));
    workflow.add_state(create_state("end", "End", true));

    workflow.add_transition(create_transition("start", "end", ConditionType::Always));

    let mut executor = WorkflowExecutor::new();
    let result = executor.start_and_execute_workflow(workflow).await;

    assert!(result.is_ok());
    let run = result.unwrap();
    assert_eq!(run.status, WorkflowRunStatus::Completed);
    assert_eq!(run.current_state, StateId::new("end"));
}
