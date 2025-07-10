---
name: conditional-test-workflow
title: Conditional Test Workflow
description: A workflow with choice states for testing conditional execution
category: test
tags:
  - test
  - conditional
  - choice
arguments:
  - name: input_value
    description: Value to be processed and validated
    required: false
    default: "50"
    type_hint: string
  - name: threshold
    description: Threshold value for comparison operations
    required: false
    default: "30"
    type_hint: string
  - name: operation
    description: Type of operation to perform (validate, process, etc.)
    required: false
    default: "validate"
    type_hint: string
---

# Conditional Test Workflow

Tests conditional execution with choice states.

```mermaid
stateDiagram-v2
    [*] --> Validate
    Validate --> CheckValue: Always
    
    CheckValue --> HighValue: value > 75
    CheckValue --> MediumValue: value > 25 && value <= 75
    CheckValue --> LowValue: value <= 25
    CheckValue --> InvalidValue: default
    
    HighValue --> ProcessHigh: Always
    MediumValue --> ProcessMedium: Always
    LowValue --> ProcessLow: Always
    InvalidValue --> HandleError: Always
    
    ProcessHigh --> FinalCheck: Always
    ProcessMedium --> FinalCheck: Always
    ProcessLow --> FinalCheck: Always
    HandleError --> Failed: Always
    
    FinalCheck --> Success: result == "ok"
    FinalCheck --> Retry: result == "retry"
    FinalCheck --> Failed: default
    
    Retry --> Validate: retry_count < 3
    Retry --> Failed: retry_count >= 3
    
    Success --> Complete: Always
    Failed --> Complete: Always
    Complete --> [*]

    Validate: Validate input
    Validate: action: execute_prompt
    Validate: prompt: test/validate
    Validate: variables:
    Validate:   value: "{{ input_value }}"
    Validate:   operation: "{{ operation }}"
    
    CheckValue: Check value range
    CheckValue: type: choice
    CheckValue: description: Determine processing path based on value
    
    HighValue: High value path
    HighValue: action: set_variable
    HighValue: name: processing_mode
    HighValue: value: "high"
    
    MediumValue: Medium value path
    MediumValue: action: set_variable
    MediumValue: name: processing_mode
    MediumValue: value: "medium"
    
    LowValue: Low value path
    LowValue: action: set_variable
    LowValue: name: processing_mode
    LowValue: value: "low"
    
    InvalidValue: Invalid value path
    InvalidValue: action: set_variable
    InvalidValue: name: processing_mode
    InvalidValue: value: "invalid"
    
    ProcessHigh: Process high value
    ProcessHigh: action: execute_prompt
    ProcessHigh: prompt: test/process
    ProcessHigh: variables:
    ProcessHigh:   mode: "{{ processing_mode }}"
    ProcessHigh:   value: "{{ input_value }}"
    ProcessHigh:   threshold: "{{ threshold }}"
    
    ProcessMedium: Process medium value
    ProcessMedium: action: execute_prompt
    ProcessMedium: prompt: test/process
    ProcessMedium: variables:
    ProcessMedium:   mode: "{{ processing_mode }}"
    ProcessMedium:   value: "{{ input_value }}"
    ProcessMedium:   threshold: "{{ threshold }}"
    
    ProcessLow: Process low value
    ProcessLow: action: execute_prompt
    ProcessLow: prompt: test/process
    ProcessLow: variables:
    ProcessLow:   mode: "{{ processing_mode }}"
    ProcessLow:   value: "{{ input_value }}"
    ProcessLow:   threshold: "{{ threshold }}"
    
    HandleError: Handle error
    HandleError: action: log
    HandleError: message: "Invalid value detected: {{ input_value }}"
    
    FinalCheck: Final validation
    FinalCheck: type: choice
    FinalCheck: description: Check final result
    
    Success: Success state
    Success: action: log
    Success: message: "Processing completed successfully for {{ processing_mode }} value"
    
    Retry: Retry state
    Retry: action: increment_counter
    Retry: counter: retry_count
    Retry: message: "Retrying... attempt {{ retry_count }}"
    
    Failed: Failed state
    Failed: action: log
    Failed: message: "Processing failed after {{ retry_count }} attempts"
    
    Complete: Complete workflow
    Complete: terminal: true
```