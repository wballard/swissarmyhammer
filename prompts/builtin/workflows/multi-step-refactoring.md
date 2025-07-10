---
name: Multi-Step Refactoring
description: Complex refactoring workflow that orchestrates multiple sub-workflows
category: workflows
tags:
  - refactoring
  - nested-workflows
  - orchestration
  - example
variables:
  project_path: "src/"
  refactoring_scope: "full"
  create_pr: "true"
---

# Multi-Step Refactoring Workflow

This workflow demonstrates nested workflow execution by orchestrating a complete
refactoring process through multiple specialized sub-workflows.

```mermaid
stateDiagram-v2
    [*] --> AnalyzeCodebase: Start Refactoring
    AnalyzeCodebase --> PlanRefactoring: Analysis complete
    PlanRefactoring --> CreateBranch: Plan approved
    PlanRefactoring --> AdjustPlan: Plan needs revision
    AdjustPlan --> PlanRefactoring: Revised
    CreateBranch --> ExecuteRefactoring: Branch created
    ExecuteRefactoring --> RunCodeReview: Refactoring complete
    RunCodeReview --> RunTests: Review passed
    RunCodeReview --> FixIssues: Issues found
    FixIssues --> RunCodeReview: Fixed
    RunTests --> CreatePR: Tests passed
    RunTests --> FixTests: Tests failed
    FixTests --> RunTests: Fixed
    CreatePR --> MonitorPR: PR created
    MonitorPR --> MergePR: Approved
    MonitorPR --> UpdatePR: Changes requested
    UpdatePR --> MonitorPR: Updated
    MergePR --> Cleanup: Merged
    Cleanup --> [*]: Complete
    
    AnalyzeCodebase: Analyze Codebase
    AnalyzeCodebase: action: run_workflow
    AnalyzeCodebase: workflow: code-analysis-workflow
    AnalyzeCodebase: variables:
    AnalyzeCodebase:   path: "{{ project_path }}"
    AnalyzeCodebase:   analysis_type: "refactoring"
    AnalyzeCodebase:   include_metrics: "true"
    AnalyzeCodebase:   include_dependencies: "true"
    
    PlanRefactoring: Plan Refactoring Strategy
    PlanRefactoring: action: execute_prompt
    PlanRefactoring: prompt: refactoring/create-plan
    PlanRefactoring: variables:
    PlanRefactoring:   analysis: "{{ AnalyzeCodebase.output }}"
    PlanRefactoring:   scope: "{{ refactoring_scope }}"
    PlanRefactoring:   priorities:
    PlanRefactoring:     - "code_quality"
    PlanRefactoring:     - "performance"
    PlanRefactoring:     - "maintainability"
    
    AdjustPlan: Adjust Refactoring Plan
    AdjustPlan: action: user_input
    AdjustPlan: prompt: "Please review and adjust the refactoring plan:\n{{ PlanRefactoring.plan }}"
    AdjustPlan: type: "multiline"
    
    CreateBranch: Create Feature Branch
    CreateBranch: action: execute_prompt
    CreateBranch: prompt: git/create-branch
    CreateBranch: variables:
    CreateBranch:   branch_name: "refactor/{{ PlanRefactoring.refactoring_id }}"
    CreateBranch:   from_branch: "main"
    
    ExecuteRefactoring: Execute Refactoring Steps
    ExecuteRefactoring: action: sequential_workflows
    ExecuteRefactoring: workflows:
    ExecuteRefactoring:   - workflow: rename-symbols-workflow
    ExecuteRefactoring:     variables:
    ExecuteRefactoring:       renames: "{{ PlanRefactoring.symbol_renames }}"
    ExecuteRefactoring:   - workflow: extract-methods-workflow
    ExecuteRefactoring:     variables:
    ExecuteRefactoring:       extractions: "{{ PlanRefactoring.method_extractions }}"
    ExecuteRefactoring:   - workflow: restructure-modules-workflow
    ExecuteRefactoring:     variables:
    ExecuteRefactoring:       structure: "{{ PlanRefactoring.module_structure }}"
    ExecuteRefactoring:   - workflow: update-patterns-workflow
    ExecuteRefactoring:     variables:
    ExecuteRefactoring:       patterns: "{{ PlanRefactoring.pattern_updates }}"
    
    RunCodeReview: Run Automated Code Review
    RunCodeReview: action: run_workflow
    RunCodeReview: workflow: code-review
    RunCodeReview: variables:
    RunCodeReview:   code_path: "{{ project_path }}"
    RunCodeReview:   review_depth: "comprehensive"
    RunCodeReview:   compare_branch: "main"
    
    FixIssues: Fix Review Issues
    FixIssues: action: run_workflow
    FixIssues: workflow: fix-code-issues-workflow
    FixIssues: variables:
    FixIssues:   issues: "{{ RunCodeReview.issues }}"
    FixIssues:   auto_fix: "true"
    FixIssues:   preserve_logic: "true"
    
    RunTests: Run Test Suite
    RunTests: action: parallel_workflows
    RunTests: workflows:
    RunTests:   - workflow: unit-test-workflow
    RunTests:     variables:
    RunTests:       path: "{{ project_path }}"
    RunTests:   - workflow: integration-test-workflow
    RunTests:     variables:
    RunTests:       path: "{{ project_path }}"
    RunTests:   - workflow: performance-test-workflow
    RunTests:     variables:
    RunTests:       baseline: "{{ AnalyzeCodebase.performance_baseline }}"
    RunTests: wait_for_all: true
    RunTests: fail_fast: false
    
    FixTests: Fix Failing Tests
    FixTests: action: run_workflow
    FixTests: workflow: fix-tests-workflow
    FixTests: variables:
    FixTests:   test_results: "{{ RunTests.results }}"
    FixTests:   refactoring_changes: "{{ ExecuteRefactoring.changes }}"
    
    CreatePR: Create Pull Request
    CreatePR: action: conditional_workflow
    CreatePR: condition: "{{ create_pr == 'true' }}"
    CreatePR: workflow: create-pr-workflow
    CreatePR: variables:
    CreatePR:   title: "Refactor: {{ PlanRefactoring.title }}"
    CreatePR:   description: "{{ PlanRefactoring.description }}"
    CreatePR:   branch: "{{ CreateBranch.branch_name }}"
    CreatePR:   reviewers: "{{ PlanRefactoring.suggested_reviewers }}"
    CreatePR:   labels:
    CreatePR:     - "refactoring"
    CreatePR:     - "automated"
    
    MonitorPR: Monitor Pull Request
    MonitorPR: action: run_workflow
    MonitorPR: workflow: monitor-pr-workflow
    MonitorPR: variables:
    MonitorPR:   pr_number: "{{ CreatePR.pr_number }}"
    MonitorPR:   timeout: "7200"
    MonitorPR:   check_interval: "300"
    
    UpdatePR: Update Pull Request
    UpdatePR: action: run_workflow
    UpdatePR: workflow: update-pr-workflow
    UpdatePR: variables:
    UpdatePR:   pr_number: "{{ CreatePR.pr_number }}"
    UpdatePR:   feedback: "{{ MonitorPR.feedback }}"
    UpdatePR:   auto_resolve: "true"
    
    MergePR: Merge Pull Request
    MergePR: action: run_workflow
    MergePR: workflow: merge-pr-workflow
    MergePR: variables:
    MergePR:   pr_number: "{{ CreatePR.pr_number }}"
    MergePR:   merge_method: "squash"
    MergePR:   delete_branch: "true"
    
    Cleanup: Cleanup and Report
    Cleanup: action: parallel_workflows
    Cleanup: workflows:
    Cleanup:   - workflow: generate-refactoring-report
    Cleanup:     variables:
    Cleanup:       refactoring_id: "{{ PlanRefactoring.refactoring_id }}"
    Cleanup:       metrics_before: "{{ AnalyzeCodebase.metrics }}"
    Cleanup:       metrics_after: "{{ current_metrics }}"
    Cleanup:   - workflow: update-documentation
    Cleanup:     variables:
    Cleanup:       changes: "{{ ExecuteRefactoring.changes }}"
    Cleanup:   - workflow: notify-team
    Cleanup:     variables:
    Cleanup:       message: "Refactoring {{ PlanRefactoring.title }} completed"
    Cleanup: output: "Refactoring completed successfully! See report: {{ generate-refactoring-report.report_url }}"
```

## Usage

Run this workflow with:

```bash
# Full automated refactoring
swissarmyhammer workflow run multi-step-refactoring \
  --set project_path=src/core \
  --set refactoring_scope=full

# Refactoring without PR creation
swissarmyhammer workflow run multi-step-refactoring \
  --set project_path=src/utils \
  --set create_pr=false
```

## Nested Workflow Features

1. **Single Workflow Execution**: `run_workflow` action for running individual workflows
2. **Sequential Workflows**: `sequential_workflows` for running workflows in order
3. **Parallel Workflows**: `parallel_workflows` for concurrent workflow execution
4. **Conditional Workflows**: `conditional_workflow` based on variables
5. **Workflow Data Passing**: Output from one workflow used as input to another
6. **Complex Orchestration**: Multiple levels of workflow nesting
7. **Error Propagation**: Errors in sub-workflows handled by parent workflow