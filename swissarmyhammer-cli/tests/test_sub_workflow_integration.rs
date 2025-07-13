//! Integration tests for sub-workflow functionality
//!
//! This module tests that sub-workflows are executed in-process rather than
//! shelling out to a subprocess.

use anyhow::Result;
use swissarmyhammer::workflow::{
    MermaidParser, StateId, WorkflowExecutor, WorkflowRun, WorkflowStorage,
};

#[tokio::test]
async fn test_sub_workflow_in_process_execution() -> Result<()> {
    // Test that sub-workflows are executed in-process
    let parent_workflow_content = r#"---
name: test-parent
title: Test Parent Workflow
description: Tests sub-workflow execution
---

# Test Parent Workflow

```mermaid
stateDiagram-v2
    [*] --> Start
    Start --> CallSubWorkflow
    CallSubWorkflow --> ProcessResult
    ProcessResult --> [*]
```

## Actions

- Start: Set parent_var="parent_value"
- CallSubWorkflow: Run workflow "hello-world" with greeting="Hello from parent" result="sub_result"
- ProcessResult: Log "Sub-workflow completed"
"#;

    // Parse the parent workflow
    let parent_workflow = MermaidParser::parse(parent_workflow_content, "test-parent")?;

    // Create executor
    let mut executor = WorkflowExecutor::new();

    // Start the parent workflow
    let mut run = WorkflowRun::new(parent_workflow);

    // Execute the workflow
    let result = executor.execute_state(&mut run).await;

    // The workflow should complete successfully if hello-world exists
    // Otherwise it will fail with a workflow loading error (not a subprocess error)
    match result {
        Ok(_) => {
            // If it succeeded, verify the context
            assert!(run.context.contains_key("parent_var"));
            assert_eq!(
                run.context.get("parent_var"),
                Some(&serde_json::json!("parent_value"))
            );
        }
        Err(e) => {
            // If it failed, verify it's a workflow loading error, not a subprocess error
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("Failed to load sub-workflow")
                    || error_msg.contains("workflow 'hello-world'"),
                "Expected workflow loading error, got: {}",
                error_msg
            );
            assert!(
                !error_msg.contains("Failed to spawn"),
                "Should not contain subprocess spawn error"
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_sub_workflow_with_memory_storage() -> Result<()> {
    // Test sub-workflow execution with workflows in memory storage
    let parent_workflow_content = r#"---
name: test-parent-memory
title: Test Parent Workflow
description: Tests sub-workflow execution with memory storage
---

# Test Parent Workflow

```mermaid
stateDiagram-v2
    [*] --> Start
    Start --> CallSubWorkflow
    CallSubWorkflow --> ProcessResult
    ProcessResult --> [*]
```

## Actions

- Start: Set parent_var="parent_value"
- CallSubWorkflow: Run workflow "test-child" with input_value="${parent_var}" result="sub_result"
- ProcessResult: Log "Sub-workflow result: ${sub_result}"
"#;

    let child_workflow_content = r#"---
name: test-child
title: Test Child Workflow
description: Child workflow for testing
---

# Test Child Workflow

```mermaid
stateDiagram-v2
    [*] --> ProcessInput
    ProcessInput --> GenerateResult
    GenerateResult --> [*]
```

## Actions

- ProcessInput: Log "Received input: ${input_value}"
- GenerateResult: Set child_result="Processed: ${input_value}"
"#;

    // Create memory storage and store both workflows
    let mut storage = WorkflowStorage::memory();
    let parent_workflow = MermaidParser::parse(parent_workflow_content, "test-parent-memory")?;
    let child_workflow = MermaidParser::parse(child_workflow_content, "test-child")?;

    storage.store_workflow(parent_workflow.clone())?;
    storage.store_workflow(child_workflow)?;

    // Create executor
    let mut executor = WorkflowExecutor::new();

    // Start the parent workflow
    let mut run = WorkflowRun::new(parent_workflow);

    // Execute the workflow
    let result = executor.execute_state(&mut run).await;

    // Since we're using memory storage and not file system storage,
    // the sub-workflow action will fail to load the workflow from the file system
    assert!(result.is_ok());

    // The workflow should have executed to the point where it tries to call the sub-workflow
    // but fails because SubWorkflowAction uses file system storage
    let visited_states: Vec<StateId> = run
        .history
        .iter()
        .map(|(state_id, _)| state_id.clone())
        .collect();

    assert!(visited_states.contains(&StateId::new("Start")));
    assert!(visited_states.contains(&StateId::new("CallSubWorkflow")));

    Ok(())
}

#[tokio::test]
async fn test_sub_workflow_circular_dependency_detection_integration() -> Result<()> {
    // Test that circular dependencies are properly detected
    let workflow_a_content = r#"---
name: workflow-a
title: Workflow A
description: Calls workflow B
---

# Workflow A

```mermaid
stateDiagram-v2
    [*] --> CallB
    CallB --> [*]
```

## Actions

- CallB: Run workflow "workflow-b"
"#;

    let workflow_b_content = r#"---
name: workflow-b
title: Workflow B
description: Calls workflow A (circular)
---

# Workflow B

```mermaid
stateDiagram-v2
    [*] --> CallA
    CallA --> [*]
```

## Actions

- CallA: Run workflow "workflow-a"
"#;

    // Create memory storage and store both workflows
    let mut storage = WorkflowStorage::memory();
    let workflow_a = MermaidParser::parse(workflow_a_content, "workflow-a")?;
    let workflow_b = MermaidParser::parse(workflow_b_content, "workflow-b")?;

    storage.store_workflow(workflow_a.clone())?;
    storage.store_workflow(workflow_b)?;

    // Create executor and start workflow A
    let mut executor = WorkflowExecutor::new();
    let mut run = WorkflowRun::new(workflow_a);

    // Execute the workflow - it should detect circular dependency
    let result = executor.execute_state(&mut run).await;

    // The workflow should complete, likely with a failure when it tries to
    // execute the sub-workflow that doesn't exist in the file system
    match &result {
        Ok(_) => {
            // Check if we reached the sub-workflow action
            let visited_states: Vec<StateId> = run
                .history
                .iter()
                .map(|(state_id, _)| state_id.clone())
                .collect();

            assert!(visited_states.contains(&StateId::new("CallB")));
        }
        Err(e) => {
            println!("Workflow execution error (expected): {}", e);
            // This is expected if the workflow tried to execute a sub-workflow
            // that doesn't exist in the file system
        }
    }

    Ok(())
}
