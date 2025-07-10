---
name: Code Review Workflow
description: Automated code review process with analysis, feedback, and approval steps
category: workflows
tags:
  - code-review
  - quality
  - automation
  - example
variables:
  code_path: "src/"
  review_depth: "comprehensive"
  language: "auto-detect"
---

# Code Review Workflow

This workflow demonstrates a linear code review process that analyzes code,
provides feedback, and tracks review status.

```mermaid
stateDiagram-v2
    [*] --> InitializeReview: Start Review
    InitializeReview --> AnalyzeCode: Code ready
    AnalyzeCode --> CheckQuality: Analysis complete
    CheckQuality --> GenerateFeedback: Issues found
    CheckQuality --> Approved: No issues
    GenerateFeedback --> RequestChanges: Feedback ready
    RequestChanges --> WaitForFixes: Changes requested
    WaitForFixes --> AnalyzeCode: Changes made
    Approved --> [*]: Review complete
    
    InitializeReview: Initialize Review
    InitializeReview: action: set_variable
    InitializeReview: variable: review_id
    InitializeReview: value: "review_{{ timestamp }}"
    
    AnalyzeCode: Analyze Code
    AnalyzeCode: action: execute_prompt
    AnalyzeCode: prompt: code/analyze-codebase
    AnalyzeCode: variables:
    AnalyzeCode:   path: "{{ code_path }}"
    AnalyzeCode:   depth: "{{ review_depth }}"
    
    CheckQuality: Check Quality Standards
    CheckQuality: action: execute_prompt
    CheckQuality: prompt: code/check-quality
    CheckQuality: variables:
    CheckQuality:   analysis: "{{ AnalyzeCode.output }}"
    CheckQuality:   standards: "high"
    
    GenerateFeedback: Generate Review Feedback
    GenerateFeedback: action: execute_prompt
    GenerateFeedback: prompt: review/generate-feedback
    GenerateFeedback: variables:
    GenerateFeedback:   issues: "{{ CheckQuality.issues }}"
    GenerateFeedback:   severity: "{{ CheckQuality.severity }}"
    
    RequestChanges: Request Changes
    RequestChanges: action: execute_prompt
    RequestChanges: prompt: review/format-feedback
    RequestChanges: variables:
    RequestChanges:   feedback: "{{ GenerateFeedback.output }}"
    RequestChanges:   review_id: "{{ review_id }}"
    
    WaitForFixes: Wait for Fixes
    WaitForFixes: action: manual_review
    WaitForFixes: prompt: "Please address the review feedback and confirm when changes are complete."
    
    Approved: Mark as Approved
    Approved: action: set_variable
    Approved: variable: review_status
    Approved: value: "approved"
    Approved: output: "Code review completed successfully! Review ID: {{ review_id }}"
```

## Usage

Run this workflow with:

```bash
swissarmyhammer workflow run code-review --set code_path=src/main --set language=rust
```

## Customization

- **code_path**: Directory or file to review
- **review_depth**: "basic", "comprehensive", or "security-focused"
- **language**: Programming language or "auto-detect"