# Workflows

SwissArmyHammer provides a powerful workflow system that allows you to define and execute complex multi-step processes using Mermaid state diagrams. This guide covers creating, running, and managing workflows.

## Overview

Workflows in SwissArmyHammer are defined using Mermaid state diagrams in markdown files. Each workflow consists of states (actions) and transitions that control the flow of execution. Workflows can:

- Execute prompts or other workflows
- Make decisions based on outputs
- Run actions in parallel
- Handle errors gracefully
- Resume from failures

## Creating Workflows

Workflows are stored in `.swissarmyhammer/workflows/` directories and use the `.md` file extension. Each workflow file consists of:

1. **YAML Front Matter** - Metadata about the workflow
2. **Mermaid State Diagram** - The workflow structure
3. **Actions Section** - Mappings of states to their actions

Here's a basic workflow structure:

```markdown
---
name: my-workflow
title: My Example Workflow
description: A workflow that demonstrates basic functionality
category: user
tags:
  - example
  - automation
---

# My Example Workflow

This workflow processes data through multiple stages.

For a complete example, see: [Simple Workflow](../examples/workflows/simple-workflow.md)

## Actions

- Start: Execute prompt "setup" with input="${data}"
- Process: Execute prompt "main-task"
- Success: Log "Task completed successfully"
- Failure: Log error "Task failed: ${error}"
```

## Workflow Components

### Front Matter

The YAML front matter contains workflow metadata:

```yaml
---
name: workflow-id           # Unique identifier for the workflow
title: Workflow Title       # Human-readable title
description: Description    # What the workflow does
category: builtin          # Category (builtin, user, etc.)
tags:                      # Tags for organization
  - automation
  - data-processing
---
```

### States

States represent steps in your workflow. They are defined in the Mermaid diagram and their actions are specified in the Actions section:

- **Execute a prompt**: `Execute prompt "prompt-name" with var="value"`
- **Run another workflow**: `Run workflow "workflow-name" with data="${input}"`
- **Set variables**: `Set result="${output}"`
- **Log messages**: `Log "Processing complete"`
- **Wait**: `Wait 5 seconds`

### Actions Section

The Actions section maps state names to their actions using the format:

```markdown
## Actions

- StateName: Action description
- AnotherState: Execute prompt "example" with param="value"
```

### Transitions

Transitions control flow between states:

- **Always**: Unconditional transition
- **OnSuccess**: Transition when action succeeds
- **OnFailure**: Transition when action fails
- **Conditional**: Based on regex matching or CEL expressions
  - Regex patterns: `"pattern"` or `/regex/`
  - CEL expressions: Complex conditions like `var.startsWith('Hello')` or `is_error == true`

### Special States

- `[*]`: Start and end states
- Fork (`<<fork>>`) and Join (`<<join>>`): For parallel execution

## Mermaid Syntax Guide

SwissArmyHammer uses standard Mermaid state diagram syntax. The diagram defines the workflow structure, while actions are defined separately in the Actions section:

### Basic Flow

```mermaid
stateDiagram-v2
    [*] --> StateA
    StateA --> StateB
    StateB --> [*]
```

With corresponding actions:

```markdown
## Actions

- StateA: Log "Starting process"
- StateB: Execute prompt "process-data"
```

### Conditional Branching

```mermaid
stateDiagram-v2
    [*] --> Check
    Check --> OptionA: "pattern_a"
    Check --> OptionB: "pattern_b"
    Check --> Default: Always
```

#### Choice State Detection

SwissArmyHammer automatically detects choice states based on their transition patterns. A state is identified as a choice state when it has:

- Multiple outgoing transitions
- At least one transition with a condition (regex pattern or CEL expression)
- Different transition types (e.g., conditional and "always" transitions)

This automatic detection ensures that branching logic works correctly without requiring explicit choice state declarations in the Mermaid diagram.

### Parallel Execution

See: [Parallel Workflow](../examples/workflows/parallel-workflow.md)

## Action Reference

### Execute Prompt

Execute a SwissArmyHammer prompt:

```
Execute prompt "prompt-name" with var1="value1" var2="${variable}"
```

### Run Workflow

Delegate to another workflow:

```
Run workflow "workflow-name" with input="${data}"
```

### Set Variable

Store values for later use:

```
Set variable_name="value"
Set result="${output}"
```

### Log Messages

Output information:

```
Log "Information message"
Log error "Error message"
Log warning "Warning message"
```

### Wait

Pause execution:

```
Wait 5 seconds
Wait 1 minute
```

### System Commands

Execute shell commands:

```
Execute command "ls -la"
Execute command "npm test"
```

## Variables and Context

Workflows have access to:

- Input variables passed via `--var`
- Template variables passed via `--set` (for liquid template rendering)
- Variables set in previous states
- Output from executed prompts (`${output}`)
- Error messages (`${error}`)
- Workflow metadata (`${workflow_name}`, `${run_id}`)

### Variable Interpolation

Use `${variable_name}` syntax to reference workflow variables:

```
Execute prompt "analyze" with file="${input_file}"
Set result="Analysis of ${input_file}: ${output}"
```

### Liquid Template Support

Workflows support Liquid template rendering in action strings when using the `--set` parameter. This allows dynamic parameterization of workflows at runtime:

```markdown
## Actions

- start: Log "Starting workflow for {{ user_name | default: 'Guest' }}"
- greet: Execute prompt "say-hello" with name="{{ name }}" language="{{ language | default: 'English' }}"
- process: Set message="{{ greeting_type }} for {{ user_name }}!"
- farewell: Log "Goodbye, {{ name }}!"
```

Run the workflow with template variables:

```bash
# Pass template variables with --set
swissarmyhammer flow run greeting --set name=Alice --set language=French

# Template variables with default values
swissarmyhammer flow run greeting --set name=Bob
# The language will default to 'English'

# Complex template variables
swissarmyhammer flow run data-processor --set user.name=Alice --set user.role=admin
```

#### Template Features in Workflows

You can use all Liquid template features in workflow action strings:

**Filters:**
```markdown
- log_user: Log "Processing user: {{ username | upcase }}"
- set_path: Set output_file="/tmp/{{ filename | slugify }}.json"
```

**Conditionals:**
```markdown
- notify: Log "{% if priority == 'high' %}🚨 URGENT: {% endif %}{{ message }}"
```

**Default Values:**
```markdown
- configure: Set timeout="{{ timeout | default: '30' }}"
- log_mode: Log "Running in {{ mode | default: 'development' }} mode"
```

**Complex Objects:**
```markdown
- process_user: Execute prompt "user-handler" with name="{{ user.name }}" role="{{ user.role }}"
```

#### Combining --var and --set

You can use both `--var` (for workflow variables) and `--set` (for template variables) together:

```bash
swissarmyhammer flow run my-workflow \
  --var input_file=data.json \
  --var output_dir=/tmp \
  --set user_name=Alice \
  --set environment=production
```

- `--var` variables are available as `${variable}` in the workflow
- `--set` variables are available as `{{ variable }}` in liquid templates

#### Template Rendering Behavior

- Templates are rendered before action parsing
- If a template variable is not provided, the original template syntax is preserved
- Template rendering errors are logged as warnings but don't stop workflow execution
- Use the `default` filter to provide fallback values for optional variables

## Error Handling

Workflows provide robust error handling:

### Try-Catch Pattern

```mermaid
stateDiagram-v2
    [*] --> Try
    Try --> Success: OnSuccess
    Try --> Catch: OnFailure
    Catch --> Recovery
    Success --> [*]
    Recovery --> [*]
```

```markdown
## Actions

- Try: Execute prompt "risky-operation"
- Catch: Log error "Operation failed: ${error}"
- Recovery: Execute prompt "cleanup"
- Success: Log "Operation completed successfully"
```

### Retry Pattern

See: [Retry Workflow](../examples/workflows/retry-workflow.md)

```markdown
## Actions

- Attempt: Execute prompt "network-call"
- Wait: Wait 5 seconds
- Success: Log "Network call succeeded"
```

### Abort Error Handling

Workflows support immediate termination through abort errors. When a prompt action's result begins with `ABORT ERROR:`, the workflow immediately exits all the way back to the root workflow with an error.

#### How Abort Errors Work

1. **Detection**: When a prompt returns a response starting with `ABORT ERROR:`, it triggers immediate termination
2. **Propagation**: The error bypasses all normal error handling (retries, compensation, transitions)
3. **Root Exit**: In nested workflows, the abort error propagates through all parent workflows to the root

#### Example Usage

See: [User Confirmation Workflow](../examples/workflows/user-confirmation-workflow.md)

```markdown
## Actions

- UserConfirmation: Execute prompt "confirm-destructive-action"
- ProcessData: Execute prompt "process-user-data"
- Complete: Log "Processing completed"
```

If the `confirm-destructive-action` prompt returns `ABORT ERROR: User cancelled the operation`, the workflow immediately terminates without executing ProcessData or Complete states.

#### Use Cases

- **User Cancellation**: Allow users to cancel long-running operations
- **Critical Failures**: Immediately stop on unrecoverable errors
- **Safety Checks**: Abort when safety conditions are not met

#### Important Notes

- Abort errors only trigger when the response **starts with** `ABORT ERROR:` (case-sensitive)
- The error message after `ABORT ERROR:` is propagated in the error
- Abort errors cannot be caught or handled within the workflow
- In sub-workflows, abort errors bubble up to terminate the parent workflow

## Best Practices

### 1. Keep States Focused

Each state should perform one clear action:

```
Good:
ValidateInput: Execute prompt "validate-json" with file="${input}"

Bad:
DoEverything: Execute prompt "validate-and-transform-and-save"
```

### 2. Use Meaningful State Names

State names should describe what happens:

```
Good: ValidateConfiguration, ProcessUserData, GenerateReport
Bad: Step1, Step2, DoStuff
```

### 3. Handle All Paths

Ensure all states have clear exit paths:

```mermaid
stateDiagram-v2
    [*] --> Process
    Process --> Success: OnSuccess
    Process --> Failure: OnFailure
    Success --> [*]
    Failure --> Cleanup: Always
    Cleanup --> [*]
```

### 4. Use Variables Effectively

Pass data between states using variables:

```
ExtractData: Execute prompt "parse-file" with file="${input_file}"
ProcessData: Execute prompt "transform" with data="${output}"
SaveResults: Execute prompt "save" with content="${output}" path="${output_file}"
```

### 5. Document Complex Logic

Add comments to explain complex workflows:

```mermaid
stateDiagram-v2
    %% This workflow processes user uploads
    %% It validates, transforms, and stores the data
    
    [*] --> Validate
    %% Validation ensures file format is correct
    Validate --> Transform: OnSuccess
```

## Complete Example

Here's a complete workflow file showing all components including liquid template support:

```markdown
---
name: data-processor
title: Data Processing Workflow
description: Validates and processes incoming data files
category: user
tags:
  - data-processing
  - validation
  - automation
---

# Data Processing Workflow

This workflow validates incoming data files, transforms them to the required format,
and stores the results. It includes error handling and retry logic.

```mermaid
stateDiagram-v2
    [*] --> Initialize
    Initialize --> ValidateFormat
    ValidateFormat --> Transform: OnSuccess
    ValidateFormat --> LogError: OnFailure
    Transform --> StoreData: OnSuccess
    Transform --> RetryTransform: OnFailure
    RetryTransform --> Transform
    StoreData --> NotifyComplete
    LogError --> NotifyError
    NotifyComplete --> [*]
    NotifyError --> [*]
```

## Actions

- Initialize: Log "Starting {{ environment | default: 'development' }} data processing for file: ${input_file}"
- ValidateFormat: Execute prompt "validate-json-schema" with file="${input_file}" schema="{{ schema | default: 'default-schema.json' }}"
- Transform: Execute prompt "transform-data" with input="${output}" format="{{ format | default: 'json' }}"
- StoreData: Execute prompt "store-to-database" with data="${output}" table="{{ db_table | default: 'processed_data' }}"
- RetryTransform: Wait {{ retry_delay | default: '5' }} seconds
- NotifyComplete: Log "Successfully processed ${input_file} in {{ environment }} environment"
- LogError: Log error "Validation failed for ${input_file}: ${error}"
- NotifyError: Execute prompt "send-notification" with message="Processing failed: ${error}" channel="{{ alert_channel | default: 'errors' }}"
```

### Running the Example

```bash
# Basic run with defaults
swissarmyhammer flow run data-processor --var input_file=data.json

# Production run with custom settings
swissarmyhammer flow run data-processor \
  --var input_file=data.json \
  --set environment=production \
  --set schema=production-schema.json \
  --set db_table=prod_data \
  --set alert_channel=prod-alerts

# Development run with custom retry
swissarmyhammer flow run data-processor \
  --var input_file=test.json \
  --set environment=development \
  --set retry_delay=10
```

## Running Workflows

Execute workflows using the `flow` command:

```bash
# Run a workflow
swissarmyhammer flow run workflow-name

# Pass variables
swissarmyhammer flow run workflow-name --var input_file=data.json --var mode=production

# Resume from failure
swissarmyhammer flow run workflow-name --resume <run_id>
```

## Monitoring and Debugging

### View Workflow Runs

```bash
# List recent runs
swissarmyhammer flow list

# Show run details
swissarmyhammer flow show <run_id>
```

### Debug Output

Workflows create detailed logs in `.swissarmyhammer/workflows/runs/<run_id>.jsonl`:

- State entries and exits
- Variable values
- Prompt outputs
- Error messages
- Timing information

### Visualization

Generate workflow diagrams:

```bash
# Visualize workflow structure
swissarmyhammer flow visualize workflow-name

# Show run path
swissarmyhammer flow visualize workflow-name --run <run_id>
```

## Advanced Features

### Nested Workflows

Workflows can call other workflows, enabling modular design:

```mermaid
stateDiagram-v2
    [*] --> Initialize
    Initialize --> RunSubWorkflow
    RunSubWorkflow --> ProcessResults: OnSuccess
    ProcessResults --> [*]
```

```markdown
## Actions

- Initialize: Log "Starting main workflow"
- RunSubWorkflow: Run workflow "data-processor" with input="${raw_data}"
- ProcessResults: Log "Processed ${output}"
```

### Dynamic Workflow Selection

Choose workflows at runtime:

```mermaid
stateDiagram-v2
    [*] --> DetermineType
    DetermineType --> RunTypeA: "type:A"
    DetermineType --> RunTypeB: "type:B"
    RunTypeA --> [*]
    RunTypeB --> [*]
```

```markdown
## Actions

- DetermineType: Execute prompt "detect-type" with data="${input}"
- RunTypeA: Run workflow "process-type-a" with data="${input}"
- RunTypeB: Run workflow "process-type-b" with data="${input}"
```

### CEL Expression Branching

Use CEL expressions for complex conditional logic:

```mermaid
stateDiagram-v2
    [*] --> BranchDecision
    BranchDecision --> ProcessNormal: example_var.startsWith('Hello')
    BranchDecision --> ProcessError: is_error == true
    BranchDecision --> ProcessSpecial: count > 10 && status == 'active'
    BranchDecision --> ProcessDefault: always
    ProcessNormal --> [*]
    ProcessError --> [*]
    ProcessSpecial --> [*]
    ProcessDefault --> [*]
```

```markdown
## Actions

- BranchDecision: Execute prompt "analyze-data" with input="${data}"
- ProcessNormal: Log "Processing normal flow for ${example_var}"
- ProcessError: Execute prompt "handle-error" with error="${error}"
- ProcessSpecial: Execute prompt "special-handler" with count="${count}"
- ProcessDefault: Log "No special conditions met"
```

The choice state (BranchDecision) is automatically detected and will evaluate each condition in order, taking the first matching transition.

### Parallel Processing

Process multiple items concurrently:

```mermaid
stateDiagram-v2
    [*] --> Split
    Split --> fork1: Always
    
    state fork1 <<fork>>
    fork1 --> ProcessItem1
    fork1 --> ProcessItem2
    fork1 --> ProcessItem3
    
    ProcessItem1 --> join1: Always
    ProcessItem2 --> join1: Always
    ProcessItem3 --> join1: Always
    
    state join1 <<join>>
    join1 --> Aggregate
    Aggregate --> [*]
```

## Troubleshooting

### Common Issues

1. **Workflow not found**: Ensure workflow is in `.swissarmyhammer/workflows/`
2. **Variable undefined**: Check variable names and initialization
3. **Infinite loops**: Add proper exit conditions
4. **Prompt not found**: Verify prompt paths and names

### Validation

Always validate workflows before running:

```bash
swissarmyhammer validate
```

This checks:
- Mermaid syntax
- State connectivity
- Action syntax
- Variable usage
- Circular dependencies

## Next Steps

- Explore [Example Workflows](./workflow-examples.md) for practical patterns
- Read about [Workflow Patterns](./workflow-patterns.md) for common solutions
- Check the [CLI Reference](./cli-reference.md#flow) for all flow commands
- Learn about [Testing Workflows](./testing-guide.md#workflows)