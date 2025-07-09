---
name: Data Processing Pipeline
description: Parallel data processing workflow for analyzing multiple data sources simultaneously
category: workflows
tags:
  - data-processing
  - parallel
  - analytics
  - example
variables:
  data_sources: "logs,metrics,events"
  output_format: "json"
  processing_mode: "parallel"
---

# Data Processing Pipeline

This workflow demonstrates parallel execution by processing multiple data sources
simultaneously and aggregating results.

```mermaid
stateDiagram-v2
    [*] --> Initialize: Start Pipeline
    Initialize --> PrepareJobs: Initialized
    PrepareJobs --> ProcessLogs: Start parallel
    PrepareJobs --> ProcessMetrics: Start parallel
    PrepareJobs --> ProcessEvents: Start parallel
    ProcessLogs --> WaitForCompletion: Logs processed
    ProcessMetrics --> WaitForCompletion: Metrics processed
    ProcessEvents --> WaitForCompletion: Events processed
    WaitForCompletion --> AggregateResults: All complete
    AggregateResults --> ValidateData: Aggregated
    ValidateData --> GenerateReport: Valid
    ValidateData --> HandleErrors: Invalid
    GenerateReport --> PublishResults: Report ready
    HandleErrors --> GenerateReport: Errors logged
    PublishResults --> [*]: Pipeline complete
    
    Initialize: Initialize Pipeline
    Initialize: action: set_variable
    Initialize: variable: pipeline_id
    Initialize: value: "pipeline_{{ timestamp }}"
    Initialize: parallel:
    Initialize:   - action: set_variable
    Initialize:     variable: start_time
    Initialize:     value: "{{ timestamp }}"
    Initialize:   - action: execute_prompt
    Initialize:     prompt: data/validate-sources
    Initialize:     variables:
    Initialize:       sources: "{{ data_sources }}"
    
    PrepareJobs: Prepare Processing Jobs
    PrepareJobs: action: execute_prompt
    PrepareJobs: prompt: data/prepare-jobs
    PrepareJobs: variables:
    PrepareJobs:   sources: "{{ data_sources }}"
    PrepareJobs:   mode: "{{ processing_mode }}"
    
    ProcessLogs: Process Log Data
    ProcessLogs: action: execute_prompt
    ProcessLogs: prompt: data/process-logs
    ProcessLogs: variables:
    ProcessLogs:   source: "logs"
    ProcessLogs:   filters: "{{ PrepareJobs.log_filters }}"
    ProcessLogs: parallel: true
    ProcessLogs: timeout: 300
    
    ProcessMetrics: Process Metrics Data
    ProcessMetrics: action: execute_prompt
    ProcessMetrics: prompt: data/process-metrics
    ProcessMetrics: variables:
    ProcessMetrics:   source: "metrics"
    ProcessMetrics:   aggregations: "{{ PrepareJobs.metric_aggregations }}"
    ProcessMetrics: parallel: true
    ProcessMetrics: timeout: 300
    
    ProcessEvents: Process Event Data
    ProcessEvents: action: execute_prompt
    ProcessEvents: prompt: data/process-events
    ProcessEvents: variables:
    ProcessEvents:   source: "events"
    ProcessEvents:   transformations: "{{ PrepareJobs.event_transformations }}"
    ProcessEvents: parallel: true
    ProcessEvents: timeout: 300
    
    WaitForCompletion: Wait for All Processing
    WaitForCompletion: action: wait_for_parallel
    WaitForCompletion: states:
    WaitForCompletion:   - ProcessLogs
    WaitForCompletion:   - ProcessMetrics
    WaitForCompletion:   - ProcessEvents
    WaitForCompletion: timeout: 600
    
    AggregateResults: Aggregate Results
    AggregateResults: action: execute_prompt
    AggregateResults: prompt: data/aggregate-results
    AggregateResults: variables:
    AggregateResults:   log_data: "{{ ProcessLogs.output }}"
    AggregateResults:   metric_data: "{{ ProcessMetrics.output }}"
    AggregateResults:   event_data: "{{ ProcessEvents.output }}"
    AggregateResults:   format: "{{ output_format }}"
    
    ValidateData: Validate Processed Data
    ValidateData: action: execute_prompt
    ValidateData: prompt: data/validate-output
    ValidateData: variables:
    ValidateData:   data: "{{ AggregateResults.output }}"
    ValidateData:   schema: "{{ AggregateResults.schema }}"
    ValidateData: parallel:
    ValidateData:   - action: execute_prompt
    ValidateData:     prompt: data/check-completeness
    ValidateData:     variables:
    ValidateData:       data: "{{ AggregateResults.output }}"
    ValidateData:   - action: execute_prompt
    ValidateData:     prompt: data/check-quality
    ValidateData:     variables:
    ValidateData:       data: "{{ AggregateResults.output }}"
    
    GenerateReport: Generate Report
    GenerateReport: action: execute_prompt
    GenerateReport: prompt: data/generate-report
    GenerateReport: variables:
    GenerateReport:   data: "{{ AggregateResults.output }}"
    GenerateReport:   validation: "{{ ValidateData.results }}"
    GenerateReport:   format: "{{ output_format }}"
    GenerateReport:   pipeline_id: "{{ pipeline_id }}"
    
    HandleErrors: Handle Validation Errors
    HandleErrors: action: execute_prompt
    HandleErrors: prompt: data/handle-errors
    HandleErrors: variables:
    HandleErrors:   errors: "{{ ValidateData.errors }}"
    HandleErrors:   severity: "warning"
    
    PublishResults: Publish Results
    PublishResults: action: parallel_execute
    PublishResults: tasks:
    PublishResults:   - action: execute_prompt
    PublishResults:     prompt: data/save-to-storage
    PublishResults:     variables:
    PublishResults:       report: "{{ GenerateReport.output }}"
    PublishResults:       location: "reports/{{ pipeline_id }}"
    PublishResults:   - action: execute_prompt
    PublishResults:     prompt: notifications/send-completion
    PublishResults:     variables:
    PublishResults:       pipeline_id: "{{ pipeline_id }}"
    PublishResults:       duration: "{{ elapsed_time }}"
    PublishResults:   - action: execute_prompt
    PublishResults:     prompt: metrics/record-pipeline
    PublishResults:     variables:
    PublishResults:       id: "{{ pipeline_id }}"
    PublishResults:       status: "completed"
```

## Usage

Run this workflow with:

```bash
# Process all data sources in parallel
swissarmyhammer workflow run data-processing-pipeline

# Process specific sources with custom format
swissarmyhammer workflow run data-processing-pipeline \
  --set data_sources=logs,metrics \
  --set output_format=csv
```

## Parallel Execution Features

1. **Multiple Parallel States**: ProcessLogs, ProcessMetrics, and ProcessEvents run simultaneously
2. **Wait for Completion**: Synchronization point that waits for all parallel tasks
3. **Parallel Actions within States**: Initialize and ValidateData show parallel actions
4. **Parallel Final Tasks**: PublishResults executes multiple tasks in parallel
5. **Timeout Management**: Each parallel task has configurable timeouts