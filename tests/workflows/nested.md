---
name: nested-test-workflow
title: Nested Test Workflow
description: A workflow that executes other workflows for testing nested execution
category: test
tags:
  - test
  - nested
  - orchestration
arguments:
  - name: orchestration_mode
    description: How to execute child workflows (sequential or parallel)
    required: false
    default: "sequential"
    type_hint: string
  - name: child_timeout
    description: Timeout in seconds for child workflow execution
    required: false
    default: "30"
    type_hint: string
---

# Nested Test Workflow

Tests nested workflow execution with data passing between workflows.

```mermaid
stateDiagram-v2
    [*] --> Setup
    Setup --> ExecuteSimple: Always
    ExecuteSimple --> CheckMode: Always
    
    CheckMode --> ExecuteParallel: mode == "parallel"
    CheckMode --> ExecuteConditional: mode == "sequential"
    CheckMode --> SkipRemaining: default
    
    ExecuteParallel --> AggregateResults: Always
    ExecuteConditional --> AggregateResults: Always
    SkipRemaining --> AggregateResults: Always
    
    AggregateResults --> ValidateResults: Always
    ValidateResults --> Success: all_passed == true
    ValidateResults --> PartialSuccess: some_passed == true
    ValidateResults --> Failed: default
    
    Success --> Cleanup: Always
    PartialSuccess --> Cleanup: Always
    Failed --> Cleanup: Always
    Cleanup --> [*]

    Setup: Setup orchestration
    Setup: action: set_variables
    Setup: variables:
    Setup:   start_time: "{{ now() }}"
    Setup:   mode: "{{ orchestration_mode }}"
    Setup:   workflows_to_run: ["simple", "conditional", "parallel"]
    
    ExecuteSimple: Execute simple workflow
    ExecuteSimple: action: run_workflow
    ExecuteSimple: workflow: simple
    ExecuteSimple: variables:
    ExecuteSimple:   message: "Hello from parent"
    ExecuteSimple:   delay: "1"
    ExecuteSimple: timeout: "{{ child_timeout }}"
    ExecuteSimple: wait: true
    
    CheckMode: Check orchestration mode
    CheckMode: type: choice
    CheckMode: description: Determine how to run remaining workflows
    
    ExecuteParallel: Execute workflows in parallel
    ExecuteParallel: action: parallel_workflows
    ExecuteParallel: workflows:
    ExecuteParallel:   - workflow: conditional
    ExecuteParallel:     variables:
    ExecuteParallel:       input_value: "{{ ExecuteSimple.output.value }}"
    ExecuteParallel:       threshold: "30"
    ExecuteParallel:   - workflow: error_handling
    ExecuteParallel:     variables:
    ExecuteParallel:       fail_at_step: "0"
    ExecuteParallel:       max_retries: "2"
    ExecuteParallel: wait_for_all: true
    ExecuteParallel: fail_fast: false
    
    ExecuteConditional: Execute conditional workflow
    ExecuteConditional: action: conditional_workflow
    ExecuteConditional: condition: "{{ ExecuteSimple.status == 'completed' }}"
    ExecuteConditional: workflow: conditional
    ExecuteConditional: variables:
    ExecuteConditional:   input_value: "{{ ExecuteSimple.output.value }}"
    ExecuteConditional:   threshold: "{{ ExecuteSimple.output.threshold }}"
    ExecuteConditional:   operation: "validate"
    
    SkipRemaining: Skip remaining workflows
    SkipRemaining: action: log
    SkipRemaining: message: "Skipping remaining workflows due to unknown mode: {{ mode }}"
    
    AggregateResults: Aggregate workflow results
    AggregateResults: action: execute_prompt
    AggregateResults: prompt: test/aggregate_workflow_results
    AggregateResults: variables:
    AggregateResults:   results:
    AggregateResults:     simple: "{{ ExecuteSimple.status }}"
    AggregateResults:     parallel: "{{ ExecuteParallel.results }}"
    AggregateResults:     conditional: "{{ ExecuteConditional.status }}"
    
    ValidateResults: Validate aggregated results
    ValidateResults: type: choice
    ValidateResults: description: Check if all workflows passed
    
    Success: All workflows succeeded
    Success: action: log
    Success: message: "All nested workflows completed successfully"
    Success: output:
    Success:   status: "success"
    Success:   completed_workflows: "{{ completed_count }}"
    Success:   execution_time: "{{ elapsed_time }}"
    
    PartialSuccess: Some workflows succeeded
    PartialSuccess: action: log
    PartialSuccess: message: "{{ passed_count }} of {{ total_count }} workflows succeeded"
    PartialSuccess: output:
    PartialSuccess:   status: "partial_success"
    PartialSuccess:   passed: "{{ passed_workflows }}"
    PartialSuccess:   failed: "{{ failed_workflows }}"
    
    Failed: All workflows failed
    Failed: action: log
    Failed: message: "All nested workflows failed"
    Failed: output:
    Failed:   status: "failed"
    Failed:   errors: "{{ error_summary }}"
    
    Cleanup: Cleanup nested execution
    Cleanup: terminal: true
    Cleanup: action: execute_prompt
    Cleanup: prompt: test/cleanup_nested
    Cleanup: variables:
    Cleanup:   execution_id: "{{ execution_id }}"
    Cleanup:   final_status: "{{ final_status }}"
```