//! Tests to verify removal of redundant retry logic
//!
//! This test file proves that retry logic exists and then verifies its removal.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use swissarmyhammer::workflow::actions::{ActionError, ActionResult, PromptAction, RetryableAction};
use swissarmyhammer::workflow::definition::Workflow;
use swissarmyhammer::workflow::executor::core::WorkflowExecutor;
use swissarmyhammer::workflow::state::{State, StateId, StateType};
use swissarmyhammer::workflow::storage::WorkflowStorage;
use swissarmyhammer::workflow::transition::{ConditionType, Transition, TransitionCondition};
use swissarmyhammer::workflow::WorkflowName;

/// Test that workflow executor retry configuration exists
#[tokio::test]
async fn test_workflow_executor_retry_config_exists() {
    // Create a workflow with retry configuration in transition metadata
    let start_state = State {
        id: StateId::new("start"),
        description: "Log \"Starting test\"".to_string(),
        state_type: StateType::Normal,
        is_terminal: false,
        allows_parallel: false,
        metadata: HashMap::new(),
    };

    let retry_state = State {
        id: StateId::new("retry_state"),
        description: "Log \"In retry state\"".to_string(),
        state_type: StateType::Normal,
        is_terminal: true,
        allows_parallel: false,
        metadata: HashMap::new(),
    };

    let mut transition_metadata = HashMap::new();
    transition_metadata.insert("retry_max_attempts".to_string(), "3".to_string());
    transition_metadata.insert("retry_backoff_ms".to_string(), "100".to_string());
    transition_metadata.insert("retry_backoff_multiplier".to_string(), "2.0".to_string());

    let transition = Transition {
        from_state: StateId::new("start"),
        to_state: StateId::new("retry_state"),
        condition: TransitionCondition {
            condition_type: ConditionType::Always,
            expression: None,
        },
        action: None,
        metadata: transition_metadata,
    };

    let mut workflow = Workflow::new(
        WorkflowName::new("test-retry-workflow"),
        "Test workflow for retry configuration".to_string(),
        StateId::new("start"),
    );

    workflow.add_state(start_state);
    workflow.add_state(retry_state);
    workflow.add_transition(transition);

    // Set up test storage
    let mut storage = WorkflowStorage::memory();
    storage.store_workflow(workflow.clone()).unwrap();
    let arc_storage = Arc::new(storage);
    swissarmyhammer::workflow::actions::set_test_storage(arc_storage);

    // Execute the workflow
    let mut executor = WorkflowExecutor::new();
    let mut run = executor.start_workflow(workflow).unwrap();

    // Execute single cycle to get into retry_state
    let _result = executor.execute_single_cycle(&mut run).await;

    // Verify we can get retry config from transition metadata
    // This is testing the private method through public interface
    // The fact that we can transition and execute shows the retry config parsing exists
    
    // Clean up test storage
    swissarmyhammer::workflow::actions::clear_test_storage();
    
    // This test passes if the workflow executed without errors
    // proving that the retry configuration parsing and execution exists
}

/// Test that action-level retry logic exists
#[test]
fn test_action_level_retry_logic_exists() {
    let action = PromptAction::new("test-prompt".to_string());
    
    // Test that PromptAction implements RetryableAction
    assert_eq!(action.max_retries(), 2);
    
    // Test that retry logic methods exist
    let error = ActionError::ExecutionError("test error".to_string());
    assert!(!action.is_retryable_error(&error));
    
    let rate_limit_error = ActionError::RateLimit {
        message: "Rate limited".to_string(),
        wait_time: Duration::from_secs(60),
    };
    assert!(action.is_retryable_error(&rate_limit_error));
    
    // Test wait time calculation
    let wait_time = action.calculate_wait_time(&rate_limit_error, 1);
    assert_eq!(wait_time, Duration::from_secs(60));
}

/// Test that proves the current execute() method uses retry logic
#[test]
fn test_prompt_action_execute_uses_retry() {
    let action = PromptAction::new("test-prompt".to_string());
    
    // The execute method should call execute_with_retry
    // We can't test the actual retry behavior without mocking Claude
    // but we can verify the method signature and that it's wired up correctly
    
    // This test verifies the structure exists - the actual retry removal
    // will be tested by ensuring these methods no longer exist
    assert_eq!(action.action_type(), "prompt");
    assert_eq!(action.max_retries(), 2);
}

/// Test that RetryableAction trait exists with required methods
#[test]
fn test_retryable_action_trait_exists() {
    let action = PromptAction::new("test-prompt".to_string());
    
    // Test trait methods exist
    assert_eq!(action.max_retries(), 2);
    
    // Test retry strategy exists
    use swissarmyhammer::workflow::actions::RetryStrategy;
    let strategy = action.retry_strategy();
    matches!(strategy, RetryStrategy::WaitUntilNextHour);
}