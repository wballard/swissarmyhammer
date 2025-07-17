---
name: error-handling-test-workflow
title: Error Handling Test Workflow
description: A workflow for testing error handling and recovery mechanisms
category: test
tags:
  - test
  - error-handling
  - recovery
arguments:
  - name: fail_at_step
    description: Step number where failure should be simulated
    required: false
    default: "2"
    type_hint: string
  - name: max_retries
    description: Maximum number of retry attempts for failed steps
    required: false
    default: "3"
    type_hint: string
  - name: enable_fallback
    description: Whether to enable fallback recovery paths
    required: false
    default: "true"
    type_hint: string
---

# Error Handling Test Workflow

Tests error handling, fallback paths, and recovery.

**Note**: This is a legacy test workflow that demonstrates error handling patterns. In production workflows, retry logic is handled automatically by Claude's infrastructure and should not be implemented at the application level.

```mermaid
stateDiagram-v2
    [*] --> Initialize
    Initialize --> Step1: Always
    
    Step1 --> Step2: On Success
    Step1 --> RetryStep1: On Failure
    RetryStep1 --> Step1: retry_count < max_retries
    RetryStep1 --> Fallback1: retry_count >= max_retries
    
    Step2 --> Step3: On Success
    Step2 --> RetryStep2: On Failure
    RetryStep2 --> Step2: retry_count < max_retries
    RetryStep2 --> Fallback2: retry_count >= max_retries
    
    Step3 --> Finalize: On Success
    Step3 --> RetryStep3: On Failure
    RetryStep3 --> Step3: retry_count < max_retries
    RetryStep3 --> Fallback3: retry_count >= max_retries
    
    Fallback1 --> RecoveryPath: enable_fallback == "true"
    Fallback1 --> ErrorHandler: enable_fallback != "true"
    
    Fallback2 --> RecoveryPath: enable_fallback == "true"
    Fallback2 --> ErrorHandler: enable_fallback != "true"
    
    Fallback3 --> RecoveryPath: enable_fallback == "true"
    Fallback3 --> ErrorHandler: enable_fallback != "true"
    
    RecoveryPath --> Compensate: Always
    Compensate --> Cleanup: Always
    ErrorHandler --> Cleanup: Always
    
    Finalize --> Complete: Always
    Cleanup --> Complete: Always
    Complete --> [*]

    Initialize: Initialize workflow
    Initialize: action: set_variable
    Initialize: name: step_count
    Initialize: value: "0"
    Initialize: metadata:
    
    Step1: Execute Step 1
    Step1: action: execute_prompt
    Step1: prompt: test/step
    Step1: variables:
    Step1:   step_number: "1"
    Step1:   fail_at: "{{ fail_at_step }}"
    Step1:   message: "Processing step 1"
    
    Step2: Execute Step 2
    Step2: action: execute_prompt
    Step2: prompt: test/step
    Step2: variables:
    Step2:   step_number: "2"
    Step2:   fail_at: "{{ fail_at_step }}"
    Step2:   message: "Processing step 2"
    Step2: metadata:
    Step2:   compensation_state: "CompensateStep2"
    
    Step3: Execute Step 3
    Step3: action: execute_prompt
    Step3: prompt: test/step
    Step3: variables:
    Step3:   step_number: "3"
    Step3:   fail_at: "{{ fail_at_step }}"
    Step3:   message: "Processing step 3"
    
    RetryStep1: Retry Step 1
    RetryStep1: type: choice
    RetryStep1: action: increment_retry
    RetryStep1: step: "Step1"
    
    RetryStep2: Retry Step 2
    RetryStep2: type: choice
    RetryStep2: action: increment_retry
    RetryStep2: step: "Step2"
    
    RetryStep3: Retry Step 3
    RetryStep3: type: choice
    RetryStep3: action: increment_retry
    RetryStep3: step: "Step3"
    
    Fallback1: Fallback for Step 1
    Fallback1: type: choice
    Fallback1: action: log
    Fallback1: message: "Step 1 failed after {{ max_retries }} retries"
    
    Fallback2: Fallback for Step 2
    Fallback2: type: choice
    Fallback2: action: log
    Fallback2: message: "Step 2 failed after {{ max_retries }} retries"
    
    Fallback3: Fallback for Step 3
    Fallback3: type: choice
    Fallback3: action: log
    Fallback3: message: "Step 3 failed after {{ max_retries }} retries"
    
    RecoveryPath: Recovery path
    RecoveryPath: action: execute_prompt
    RecoveryPath: prompt: test/recover
    RecoveryPath: variables:
    RecoveryPath:   failed_step: "{{ last_failed_step }}"
    RecoveryPath:   error_context: "{{ error_context }}"
    
    Compensate: Compensate completed steps
    Compensate: action: execute_prompt
    Compensate: prompt: test/compensate
    Compensate: variables:
    Compensate:   completed_steps: "{{ completed_steps }}"
    
    ErrorHandler: Handle unrecoverable error
    ErrorHandler: action: execute_prompt
    ErrorHandler: prompt: test/error_handler
    ErrorHandler: variables:
    ErrorHandler:   error: "{{ last_error }}"
    ErrorHandler:   state: "{{ error_state }}"
    ErrorHandler: metadata:
    ErrorHandler:   dead_letter_state: "DeadLetter"
    
    Cleanup: Cleanup resources
    Cleanup: action: log
    Cleanup: message: "Cleaning up resources after error"
    
    Finalize: Finalize successful execution
    Finalize: action: log
    Finalize: message: "All steps completed successfully"
    
    Complete: Complete workflow
    Complete: terminal: true
    Complete: action: log
    Complete: message: "Workflow completed with status: {{ workflow_status }}"
```