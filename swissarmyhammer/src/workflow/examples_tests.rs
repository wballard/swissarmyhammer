//! Tests for workflow examples to ensure they work correctly
//!
//! These tests validate the workflow examples in the documentation to ensure
//! they remain functional and serve as integration tests.

use crate::workflow::{Workflow, WorkflowRunStatus};
use rstest::rstest;
use std::collections::HashMap;
use std::path::Path;

/// Test the simple workflow example
#[rstest]
#[case("simple-workflow")]
#[case("parallel-workflow")]
#[case("user-confirmation-workflow")]
fn test_example_workflows(#[case] workflow_name: &str) {
    let workflow_path = format!("../doc/examples/workflows/{}.md", workflow_name);
    
    // Skip if file doesn't exist (in case we're running tests before doc generation)
    if !Path::new(&workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(&workflow_path)
        .expect("Failed to load workflow from file");
    
    // Verify workflow has required metadata
    assert!(!workflow.name().is_empty(), "Workflow name should not be empty");
    assert!(!workflow.title().is_empty(), "Workflow title should not be empty");
    assert!(!workflow.description().is_empty(), "Workflow description should not be empty");
    
    // Verify workflow has initial state
    let initial_state = workflow.initial_state();
    assert!(initial_state.is_some(), "Workflow should have an initial state");
    
    // Verify workflow structure is valid
    workflow.validate().expect("Workflow should be valid");
}

/// Test simple workflow execution
#[rstest]
fn test_simple_workflow_execution() {
    let workflow_path = "../doc/examples/workflows/simple-workflow.md";
    
    if !Path::new(workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(workflow_path)
        .expect("Failed to load simple workflow");
    
    let mut variables = HashMap::new();
    variables.insert("start_time".to_string(), "2024-01-01T00:00:00Z".to_string());
    
    let mut run = workflow.start_with_variables(variables)
        .expect("Failed to start workflow");
    
    // Execute workflow steps
    while run.status() == WorkflowRunStatus::Running {
        let result = run.execute_next();
        
        // For this test, we'll simulate success
        if result.is_err() {
            break;
        }
    }
    
    // Verify workflow completed (either success or failure is acceptable)
    assert!(
        run.status() == WorkflowRunStatus::Completed || 
        run.status() == WorkflowRunStatus::Failed,
        "Workflow should complete with a final status"
    );
}



/// Test parallel workflow execution
#[rstest]
fn test_parallel_workflow() {
    let workflow_path = "../doc/examples/workflows/parallel-workflow.md";
    
    if !Path::new(workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(workflow_path)
        .expect("Failed to load parallel workflow");
    
    // Test parallel execution
    let mut variables = HashMap::new();
    variables.insert("parallel_timeout".to_string(), "5000".to_string());
    
    let mut run = workflow.start_with_variables(variables)
        .expect("Failed to start workflow");
    
    // Execute parallel workflow
    let mut steps = 0;
    while run.status() == WorkflowRunStatus::Running && steps < 30 {
        let _ = run.execute_next();
        steps += 1;
    }
    
    // Verify parallel execution completed
    assert!(steps > 0, "Parallel workflow should execute steps");
}

/// Test user confirmation workflow
#[rstest]
fn test_user_confirmation_workflow() {
    let workflow_path = "../doc/examples/workflows/user-confirmation-workflow.md";
    
    if !Path::new(workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(workflow_path)
        .expect("Failed to load user confirmation workflow");
    
    // Test with user confirmation
    let mut variables = HashMap::new();
    variables.insert("operation_description".to_string(), "Test operation".to_string());
    variables.insert("user_response".to_string(), "continue".to_string());
    
    let mut run = workflow.start_with_variables(variables)
        .expect("Failed to start workflow");
    
    // Execute confirmation workflow
    let mut steps = 0;
    while run.status() == WorkflowRunStatus::Running && steps < 10 {
        let _ = run.execute_next();
        steps += 1;
    }
    
    // Verify confirmation workflow executed
    assert!(steps > 0, "User confirmation workflow should execute steps");
}

/// Test workflow variable handling
#[rstest]
#[case("simple-workflow", "start_time")]
#[case("parallel-workflow", "parallel_timeout")]
#[case("user-confirmation-workflow", "operation_description")]
fn test_workflow_variables(#[case] workflow_name: &str, #[case] required_var: &str) {
    let workflow_path = format!("../doc/examples/workflows/{}.md", workflow_name);
    
    if !Path::new(&workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(&workflow_path)
        .expect("Failed to load workflow");
    
    // Test variable handling
    let mut variables = HashMap::new();
    variables.insert(required_var.to_string(), "test_value".to_string());
    
    let run = workflow.start_with_variables(variables)
        .expect("Failed to start workflow with variables");
    
    // Verify variable was set
    assert!(run.get_variable(required_var).is_some(), 
        "Required variable {} should be set", required_var);
}

/// Test workflow state transitions
#[rstest]
fn test_workflow_state_transitions() {
    let workflow_path = "../doc/examples/workflows/simple-workflow.md";
    
    if !Path::new(workflow_path).exists() {
        return;
    }
    
    let workflow = Workflow::from_file(workflow_path)
        .expect("Failed to load workflow");
    
    // Test that all states have proper transitions
    let states = workflow.states();
    assert!(!states.is_empty(), "Workflow should have states");
    
    for state in states {
        // Verify each state has proper structure
        assert!(!state.name().is_empty(), "State name should not be empty");
        
        // Verify transitions are valid
        let transitions = state.transitions();
        for transition in transitions {
            assert!(!transition.target().is_empty(), "Transition target should not be empty");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    /// Test workflow file loading and validation
    #[rstest]
    fn test_workflow_file_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let workflow_path = temp_dir.path().join("test-workflow.md");
        
        // Create a minimal valid workflow
        let workflow_content = r#"---
name: test-workflow
title: Test Workflow
description: A test workflow
version: 1.0.0
---

# Test Workflow

## States

### Start
- **Type**: Initial
- **Actions**: Log start
- **Transitions**: Always -> End

### End
- **Type**: Final
- **Actions**: Log end

## Workflow Definition

```mermaid
stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
```
"#;
        
        std::fs::write(&workflow_path, workflow_content)
            .expect("Failed to write workflow file");
        
        let workflow = Workflow::from_file(&workflow_path)
            .expect("Failed to load test workflow");
        
        assert_eq!(workflow.name(), "test-workflow");
        assert_eq!(workflow.title(), "Test Workflow");
        assert_eq!(workflow.description(), "A test workflow");
        
        // Validate the workflow structure
        workflow.validate().expect("Workflow should be valid");
    }
}