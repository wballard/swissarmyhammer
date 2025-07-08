# Nested Workflow Example

This example demonstrates how to use the new workflow delegation feature in SwissArmyHammer.

## Main Workflow (main-workflow.md)

```mermaid
stateDiagram-v2
    [*] --> Initialize
    Initialize --> RunValidation: Always
    RunValidation --> ProcessResults: OnSuccess  
    RunValidation --> HandleError: OnFailure
    ProcessResults --> [*]
    HandleError --> [*]

    Initialize: Set input_data="${file_path}"
    RunValidation: Run workflow "validation-workflow" with data="${input_data}"
    ProcessResults: Log "Validation completed successfully"
    HandleError: Log error "Validation failed"
```

## Sub-Workflow (validation-workflow.md)

```mermaid
stateDiagram-v2
    [*] --> ValidateInput
    ValidateInput --> CheckFormat: Always
    CheckFormat --> RunTests: OnSuccess
    CheckFormat --> ReportError: OnFailure
    RunTests --> Complete: Always
    ReportError --> Complete: Always
    Complete --> [*]

    ValidateInput: Log "Starting validation for ${data}"
    CheckFormat: Execute prompt "check-file-format" with file="${data}"
    RunTests: Execute prompt "run-validation-tests" with file="${data}"
    ReportError: Log error "Invalid file format"
    Complete: Set validation_result="completed"
```

## Usage

To run this example:

1. Save the main workflow to `.swissarmyhammer/workflows/main-workflow.md`
2. Save the sub-workflow to `.swissarmyhammer/workflows/validation-workflow.md`
3. Run: `swissarmyhammer flow run main-workflow --var file_path=example.txt`

## Features Demonstrated

1. **Workflow Delegation**: The main workflow delegates to the validation workflow
2. **Variable Passing**: The `input_data` variable is passed to the sub-workflow as `data`
3. **Result Handling**: The main workflow handles success/failure from the sub-workflow
4. **Circular Dependency Protection**: If workflows try to call each other in a circle, it will be detected

## Advanced Example with Multiple Delegations

```mermaid
stateDiagram-v2
    [*] --> Start
    Start --> ProcessFiles: Always
    ProcessFiles --> AnalyzeCode: Always
    AnalyzeCode --> GenerateReport: OnSuccess
    AnalyzeCode --> Retry: OnFailure
    Retry --> AnalyzeCode: Always
    GenerateReport --> [*]

    Start: Set files="${project_files}"
    ProcessFiles: Run workflow "file-processor" with input="${files}"
    AnalyzeCode: Run workflow "code-analyzer" with files="${processed_files}"
    GenerateReport: Run workflow "report-generator" with results="${analysis_results}"
    Retry: Wait 5 seconds
```

This demonstrates:
- Sequential workflow delegation
- Passing results between workflows
- Retry logic with delegated workflows