# Workflow Examples

This guide showcases practical workflow examples demonstrating various features and patterns available in SwissArmyHammer's workflow system.

## Example Workflows

All example workflows are located in `prompts/builtin/workflows/` and can be run directly or used as templates for your own workflows.

### 1. Code Review Workflow

**File**: `prompts/builtin/workflows/code-review.yaml`  
**Type**: Linear workflow

A straightforward sequential workflow for automated code review:

```bash
swissarmyhammer workflow run code-review --set code_path=src/main
```

**Features demonstrated**:
- Linear state progression
- State variables
- Conditional transitions based on quality checks
- Feedback loops for addressing issues

**Use cases**:
- Automated code quality checks
- Pre-commit validations
- Continuous integration pipelines

### 2. Deployment Pipeline

**File**: `prompts/builtin/workflows/deployment-pipeline.yaml`  
**Type**: Interactive workflow with choices

An interactive deployment workflow with environment selection and rollback options:

```bash
swissarmyhammer workflow run deployment-pipeline --set app_name=myservice
```

**Features demonstrated**:
- User choice actions for environment selection
- Conditional transitions based on build/test results
- Rollback mechanisms
- Different deployment paths for different environments
- Auto-rollback configuration

**Use cases**:
- Application deployments
- Infrastructure provisioning
- Release management

### 3. Data Processing Pipeline

**File**: `prompts/builtin/workflows/data-processing-pipeline.yaml`  
**Type**: Parallel execution workflow

A high-performance data processing workflow that handles multiple data sources concurrently:

```bash
swissarmyhammer workflow run data-processing-pipeline \
  --set data_sources=logs,metrics,events
```

**Features demonstrated**:
- Parallel state execution
- Synchronization points with `wait_for_parallel`
- Parallel actions within states
- Timeout management for long-running tasks
- Aggregation of parallel results

**Use cases**:
- ETL pipelines
- Data analysis workflows
- Batch processing systems
- Report generation

### 4. Database Migration

**File**: `prompts/builtin/workflows/database-migration.yaml`  
**Type**: Error handling workflow

A robust database migration workflow with comprehensive error handling:

```bash
swissarmyhammer workflow run database-migration --set target_version=v2.0
```

**Features demonstrated**:
- Multiple error states for different failure types
- Retry logic with configurable attempts
- Multi-level rollback (migration rollback, backup restore)
- Emergency mode for critical failures
- Validation at every step
- Graceful degradation

**Use cases**:
- Database schema updates
- Data migrations
- System upgrades
- Critical operations requiring rollback capability

### 5. Multi-Step Refactoring

**File**: `prompts/builtin/workflows/multi-step-refactoring.yaml`  
**Type**: Nested workflow orchestration

A complex refactoring workflow that coordinates multiple sub-workflows:

```bash
swissarmyhammer workflow run multi-step-refactoring \
  --set project_path=src/core \
  --set refactoring_scope=full
```

**Features demonstrated**:
- Single workflow execution with `run_workflow`
- Sequential workflow execution
- Parallel workflow execution
- Conditional workflow execution
- Data passing between workflows
- Complex orchestration patterns

**Use cases**:
- Large-scale refactoring projects
- Multi-phase deployments
- Complex automation pipelines
- Orchestrating microservices

## Running Example Workflows

### Basic Execution

Run any example workflow:

```bash
swissarmyhammer workflow run <workflow-name>
```

### With Custom Variables

Override default variables:

```bash
swissarmyhammer workflow run code-review \
  --set code_path=lib/ \
  --set review_depth=security-focused
```

### Interactive Mode

For workflows with user choices:

```bash
swissarmyhammer workflow run deployment-pipeline --interactive
```

### Dry Run Mode

Test workflow logic without executing actions:

```bash
swissarmyhammer workflow run database-migration \
  --set dry_run=true
```

## Learning from Examples

### How to Study the Examples

1. **Read the workflow definition**: Understand the state flow and transitions
2. **Examine the actions**: See how different action types are used
3. **Look at error handling**: Note how errors are caught and handled
4. **Study variable usage**: See how data flows through the workflow
5. **Run with debugging**: Use `--debug` to see detailed execution logs

### Adapting Examples

To create your own workflow based on an example:

1. Copy the example workflow to your prompts directory
2. Modify the metadata (name, description, tags)
3. Adjust the state diagram to match your needs
4. Update actions and variables
5. Test incrementally with `--dry-run`

### Common Modifications

**Adding error handling** to the code review workflow:
```yaml
AnalyzeCode: Analyze Code
AnalyzeCode: action: execute_prompt
AnalyzeCode: prompt: code/analyze-codebase
AnalyzeCode: error_handler: continue_and_log
AnalyzeCode: retry:
AnalyzeCode:   attempts: 3
AnalyzeCode:   delay: 30
```

**Making deployment pipeline fully automated**:
```yaml
variables:
  auto_deploy: "true"
  target_env: "staging"
  skip_confirmations: "true"
```

**Adding notifications** to data processing:
```yaml
PublishResults: action: parallel_execute
PublishResults: tasks:
PublishResults:   - action: execute_prompt
PublishResults:     prompt: notifications/slack
PublishResults:     variables:
PublishResults:       channel: "#data-team"
PublishResults:       message: "Pipeline completed"
```

## Best Practices from Examples

### 1. State Naming
- Use descriptive, action-oriented names
- Keep names concise but clear
- Use consistent naming patterns

### 2. Error Handling
- Always include error states for critical operations
- Provide meaningful error messages
- Design rollback paths for reversible operations

### 3. Variable Management
- Define sensible defaults
- Document variable purposes
- Validate inputs early in the workflow

### 4. User Interaction
- Provide clear choice descriptions
- Include help text for complex decisions
- Allow bypassing interaction for automation

### 5. Performance
- Use parallel execution where possible
- Set appropriate timeouts
- Design for idempotency

## Troubleshooting Examples

### Common Issues

1. **Workflow not found**:
   ```bash
   swissarmyhammer workflow list  # Check available workflows
   ```

2. **Variable errors**:
   ```bash
   swissarmyhammer workflow show code-review  # View required variables
   ```

3. **State transition failures**:
   - Check condition syntax
   - Verify variable values
   - Use `--debug` for detailed logs

4. **Action failures**:
   - Ensure referenced prompts exist
   - Check variable interpolation
   - Verify action syntax

### Debugging Tips

1. Use verbose output:
   ```bash
   swissarmyhammer workflow run deployment-pipeline -v
   ```

2. Enable debug mode:
   ```bash
   swissarmyhammer workflow run data-processing-pipeline --debug
   ```

3. Test with minimal data:
   ```bash
   swissarmyhammer workflow run data-processing-pipeline \
     --set data_sources=logs \
     --set processing_mode=sequential
   ```

## Next Steps

- Explore [Workflow Patterns](./workflow-patterns.md) for advanced techniques
- Read the main [Workflows Documentation](./workflows.md) for detailed reference
- Create your own workflows based on these examples
- Share your workflows with the community