---
name: parallel-test-workflow
title: Parallel Test Workflow
description: A workflow with fork/join for testing parallel execution
category: test
tags:
  - test
  - parallel
  - fork-join
arguments:
  - name: task_count
    description: Number of parallel tasks to execute
    required: false
    default: "3"
    type_hint: string
  - name: timeout
    description: Timeout in seconds for parallel execution
    required: false
    default: "10"
    type_hint: string
---

# Parallel Test Workflow

Tests parallel execution with fork and join states.

```mermaid
stateDiagram-v2
    [*] --> Initialize
    Initialize --> Fork: Always
    Fork --> TaskA: Always
    Fork --> TaskB: Always
    Fork --> TaskC: Always
    TaskA --> Join: Always
    TaskB --> Join: Always
    TaskC --> Join: Always
    Join --> Aggregate: All Complete
    Aggregate --> Complete: Always
    Complete --> [*]

    Initialize: Initialize parallel tasks
    Initialize: action: set_variable
    Initialize: name: start_time
    Initialize: value: "{{ now() }}"
    
    Fork: Fork state
    Fork: type: fork
    Fork: description: Split into parallel branches
    
    TaskA: Task A
    TaskA: action: execute_prompt
    TaskA: prompt: test/delay
    TaskA: variables:
    TaskA:   task_name: "A"
    TaskA:   delay_seconds: "2"
    
    TaskB: Task B
    TaskB: action: execute_prompt
    TaskB: prompt: test/delay
    TaskB: variables:
    TaskB:   task_name: "B"
    TaskB:   delay_seconds: "3"
    
    TaskC: Task C
    TaskC: action: execute_prompt
    TaskC: prompt: test/delay
    TaskC: variables:
    TaskC:   task_name: "C"
    TaskC:   delay_seconds: "1"
    
    Join: Join state
    Join: type: join
    Join: description: Wait for all tasks to complete
    
    Aggregate: Aggregate results
    Aggregate: action: execute_prompt
    Aggregate: prompt: test/aggregate
    Aggregate: variables:
    Aggregate:   results:
    Aggregate:     - "{{ TaskA.result }}"
    Aggregate:     - "{{ TaskB.result }}"
    Aggregate:     - "{{ TaskC.result }}"
    
    Complete: Complete workflow
    Complete: terminal: true
    Complete: action: log
    Complete: message: "Parallel execution completed in {{ elapsed_time }} seconds"
```