---
name: simple-test-workflow
title: Simple Test Workflow
description: A simple workflow for integration testing
category: test
tags:
  - test
  - simple
  - integration
arguments:
  - name: message
    description: Test message to process
    required: false
    default: "Hello from test"
    type_hint: string
  - name: delay
    description: Delay in seconds for processing
    required: false
    default: "1"
    type_hint: string
---

# Simple Test Workflow

This is a basic workflow used for integration testing.

```mermaid
stateDiagram-v2
    [*] --> Start
    Start --> Process: Always
    Process --> Success: On Success
    Process --> Failed: On Failure
    Success --> End: Always
    Failed --> End: Always
    End --> [*]

    Start: Start state
    Start: description: Initialize workflow
    
    Process: Process data
    Process: action: execute_prompt
    Process: prompt: test/echo
    Process: variables:
    Process:   message: "{{ message }}"
    Process:   delay: "{{ delay }}"
    
    Success: Success state
    Success: action: log
    Success: message: "Processing completed successfully"
    
    Failed: Failed state
    Failed: action: log
    Failed: message: "Processing failed"
    
    End: End state
    End: terminal: true
```