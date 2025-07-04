# Advanced Prompt Techniques

This guide covers advanced techniques for creating sophisticated and powerful prompts with SwissArmyHammer.

## Composable Prompts

### Prompt Chaining

Chain multiple prompts together for complex workflows:

```markdown
---
name: full-analysis
title: Complete Code Analysis Pipeline
description: Runs multiple analysis steps on code
arguments:
  - name: file_path
    description: File to analyze
    required: true
  - name: output_format
    description: Format for results
    default: markdown
---

# Complete Analysis for {{file_path}}

## Step 1: Code Review
{% capture review_output %}
Run code review on {{file_path}} focusing on:
- Code quality
- Best practices
- Potential bugs
{% endcapture %}

## Step 2: Security Analysis
{% capture security_output %}
Analyze {{file_path}} for security vulnerabilities:
- Input validation
- Authentication issues
- Data exposure risks
{% endcapture %}

## Step 3: Performance Analysis
{% capture performance_output %}
Check {{file_path}} for performance issues:
- Algorithm complexity
- Resource usage
- Optimization opportunities
{% endcapture %}

{% if output_format == "markdown" %}
## Analysis Results

### Code Review
{{ review_output }}

### Security
{{ security_output }}

### Performance
{{ performance_output }}
{% elsif output_format == "json" %}
{
  "code_review": "{{ review_output | escape }}",
  "security": "{{ security_output | escape }}",
  "performance": "{{ performance_output | escape }}"
}
{% endif %}
```

### Modular Prompt Components

Create reusable prompt components:

```markdown
---
name: code-analyzer-base
title: Base Code Analyzer
description: Reusable base for code analysis prompts
arguments:
  - name: code
    description: Code to analyze
    required: true
  - name: analysis_type
    description: Type of analysis
    required: true
---

{% comment %} Base analysis template {% endcomment %}
{% assign lines = code | split: "\n" %}
{% assign line_count = lines | size %}

# {{analysis_type | capitalize}} Analysis

## Code Metrics
- Lines of code: {{line_count}}
- Language: {% if code contains "def " %}Python{% elsif code contains "function" %}JavaScript{% else %}Unknown{% endif %}

## Analysis Focus
{% case analysis_type %}
{% when "security" %}
  {% include "security-checks.liquid" %}
{% when "performance" %}
  {% include "performance-checks.liquid" %}
{% when "style" %}
  {% include "style-checks.liquid" %}
{% endcase %}

## Detailed Analysis
Analyze the following code for {{analysis_type}} issues:

```
{{code}}
```
```

## Advanced Templating

### Dynamic Content Generation

Generate content based on complex conditions:

```markdown
---
name: api-documentation-generator
title: Dynamic API Documentation
description: Generates API docs with dynamic sections
arguments:
  - name: api_spec
    description: API specification (JSON)
    required: true
  - name: include_examples
    description: Include code examples
    default: "true"
  - name: languages
    description: Example languages (comma-separated)
    default: "curl,python,javascript"
---

{% assign api = api_spec | parse_json %}
{% assign lang_list = languages | split: "," %}

# {{api.title}} API Documentation

{{api.description}}

Base URL: `{{api.base_url}}`
Version: {{api.version}}

## Authentication

{% if api.auth.type == "bearer" %}
This API uses Bearer token authentication. Include your API token in the Authorization header:

```
Authorization: Bearer YOUR_API_TOKEN
```
{% elsif api.auth.type == "oauth2" %}
This API uses OAuth 2.0. See [Authentication Guide](#auth-guide) for details.
{% endif %}

## Endpoints

{% for endpoint in api.endpoints %}
### {{endpoint.method}} {{endpoint.path}}

{{endpoint.description}}

{% if endpoint.parameters.size > 0 %}
#### Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
{% for param in endpoint.parameters %}
| {{param.name}} | {{param.type}} | {{param.required | default: false}} | {{param.description}} |
{% endfor %}
{% endif %}

{% if include_examples == "true" %}
#### Examples

{% for lang in lang_list %}
{% case lang %}
{% when "curl" %}
```bash
curl -X {{endpoint.method}} \
  {{api.base_url}}{{endpoint.path}} \
  {% if api.auth.type == "bearer" %}-H "Authorization: Bearer $API_TOKEN" \{% endif %}
  {% for param in endpoint.parameters %}{% if param.in == "header" %}-H "{{param.name}}: value" \{% endif %}{% endfor %}
  {% if endpoint.method == "POST" or endpoint.method == "PUT" %}-H "Content-Type: application/json" \
  -d '{"key": "value"}'{% endif %}
```

{% when "python" %}
```python
import requests

response = requests.{{endpoint.method | downcase}}(
    "{{api.base_url}}{{endpoint.path}}",
    {% if api.auth.type == "bearer" %}headers={"Authorization": f"Bearer {api_token}"},{% endif %}
    {% if endpoint.method == "POST" or endpoint.method == "PUT" %}json={"key": "value"}{% endif %}
)
print(response.json())
```

{% when "javascript" %}
```javascript
const response = await fetch('{{api.base_url}}{{endpoint.path}}', {
  method: '{{endpoint.method}}',
  {% if api.auth.type == "bearer" %}headers: {
    'Authorization': `Bearer ${apiToken}`,
    {% if endpoint.method == "POST" or endpoint.method == "PUT" %}'Content-Type': 'application/json'{% endif %}
  },{% endif %}
  {% if endpoint.method == "POST" or endpoint.method == "PUT" %}body: JSON.stringify({ key: 'value' }){% endif %}
});
const data = await response.json();
```
{% endcase %}
{% endfor %}
{% endif %}

---
{% endfor %}
```

### Complex Conditionals

Use advanced conditional logic:

```markdown
---
name: smart-optimizer
title: Smart Code Optimizer
description: Applies context-aware optimizations
arguments:
  - name: code
    description: Code to optimize
    required: true
  - name: metrics
    description: Performance metrics (JSON)
    required: false
  - name: constraints
    description: Optimization constraints
    default: "balanced"
---

{% if metrics %}
  {% assign perf = metrics | parse_json %}
  {% assign needs_memory_opt = false %}
  {% assign needs_cpu_opt = false %}
  
  {% if perf.memory_usage > 80 %}
    {% assign needs_memory_opt = true %}
  {% endif %}
  
  {% if perf.cpu_usage > 70 %}
    {% assign needs_cpu_opt = true %}
  {% endif %}
{% endif %}

# Optimization Analysis

{% if needs_memory_opt and needs_cpu_opt %}
## Critical: Both Memory and CPU Optimization Needed

Your code is experiencing both memory and CPU pressure. This requires careful optimization to balance both concerns.

### Recommended Strategy: Hybrid Optimization
1. Profile to identify hotspots
2. Optimize algorithms first (reduces both CPU and memory)
3. Implement caching strategically
4. Consider async processing

{% elsif needs_memory_opt %}
## Memory Optimization Required

Current memory usage: {{perf.memory_usage}}%

### Memory Optimization Strategies:
1. Reduce object allocation
2. Use object pooling
3. Implement lazy loading
4. Clear unused references

{% elsif needs_cpu_opt %}
## CPU Optimization Required

Current CPU usage: {{perf.cpu_usage}}%

### CPU Optimization Strategies:
1. Algorithm optimization
2. Parallel processing
3. Caching computed results
4. Reduce unnecessary operations

{% else %}
## Performance is Acceptable

No immediate optimization needed. Consider:
- Code maintainability improvements
- Preemptive optimization for scale
- Documentation updates
{% endif %}

## Code Analysis

```
{{code}}
```

{% case constraints %}
{% when "memory-first" %}
Focus on reducing memory footprint, even at slight CPU cost.
{% when "cpu-first" %}
Optimize for CPU performance, memory usage is secondary.
{% when "balanced" %}
Balance both memory and CPU optimizations.
{% endcase %}
```

## State Management

### Using Captures for State

Manage complex state across prompt sections:

```markdown
---
name: migration-planner
title: Database Migration Planner
description: Plans complex database migrations
arguments:
  - name: current_schema
    description: Current database schema
    required: true
  - name: target_schema
    description: Target database schema
    required: true
  - name: strategy
    description: Migration strategy
    default: "safe"
---

{% comment %} Analyze schemas and capture findings {% endcomment %}

{% capture added_tables %}
{% assign current_tables = current_schema | parse_json | map: "name" %}
{% assign target_tables = target_schema | parse_json | map: "name" %}
{% for table in target_tables %}
  {% unless current_tables contains table %}
    - {{table}}
  {% endunless %}
{% endfor %}
{% endcapture %}

{% capture removed_tables %}
{% for table in current_tables %}
  {% unless target_tables contains table %}
    - {{table}}
  {% endunless %}
{% endfor %}
{% endcapture %}

{% capture migration_risk %}
{% if removed_tables contains "users" or removed_tables contains "auth" %}
HIGH - Critical tables being removed
{% elsif added_tables.size > 5 %}
MEDIUM - Large number of new tables
{% else %}
LOW - Minimal structural changes
{% endif %}
{% endcapture %}

# Database Migration Plan

## Risk Assessment: {{migration_risk | strip}}

## Changes Summary

### New Tables
{{added_tables | default: "None"}}

### Removed Tables
{{removed_tables | default: "None"}}

## Migration Strategy: {{strategy | upcase}}

{% if strategy == "safe" %}
### Safe Migration Steps
1. Create backup
2. Add new tables first
3. Migrate data with validation
4. Update application code
5. Remove old tables after verification

{% elsif strategy == "fast" %}
### Fast Migration Steps
1. Quick backup
2. Execute all changes in transaction
3. Minimal validation
4. Quick rollback if needed

{% elsif strategy == "zero-downtime" %}
### Zero-Downtime Migration Steps
1. Create new tables alongside old
2. Implement dual-write logic
3. Backfill data progressively
4. Switch reads to new tables
5. Remove old tables after stabilization
{% endif %}

{% if migration_risk contains "HIGH" %}
## ⚠️ High Risk Mitigation

Due to the high risk nature of this migration:
1. Schedule during maintenance window
2. Have rollback plan ready
3. Test in staging environment first
4. Monitor closely after deployment
{% endif %}
```

## Performance Optimization

### Lazy Evaluation

Use lazy evaluation for expensive operations:

```markdown
---
name: smart-analyzer
title: Smart Performance Analyzer
description: Analyzes code with lazy evaluation
arguments:
  - name: code
    description: Code to analyze
    required: true
  - name: quick_check
    description: Perform quick check only
    default: "false"
---

# Code Analysis

{% if quick_check == "true" %}
## Quick Analysis
- Lines: {{code | split: "\n" | size}}
- Complexity: {{code | size | divided_by: 100}} (estimated)

{% else %}
{% comment %} Full analysis only when needed {% endcomment %}

{% capture complexity_analysis %}
  {% assign lines = code | split: "\n" %}
  {% assign complexity = 0 %}
  {% for line in lines %}
    {% if line contains "if " or line contains "for " or line contains "while " %}
      {% assign complexity = complexity | plus: 1 %}
    {% endif %}
  {% endfor %}
  Cyclomatic Complexity: {{complexity}}
{% endcapture %}

{% capture pattern_analysis %}
  {% if code contains "TODO" or code contains "FIXME" %}
    - Contains pending work items
  {% endif %}
  {% if code contains "console.log" or code contains "print(" %}
    - Contains debug output
  {% endif %}
{% endcapture %}

## Full Analysis

### Metrics
{{complexity_analysis}}

### Code Patterns
{{pattern_analysis | default: "No issues found"}}

### Detailed Review
Analyze the code for:
1. Performance bottlenecks
2. Security vulnerabilities
3. Best practice violations

```
{{code}}
```
{% endif %}
```

### Caching Computed Values

Cache expensive computations:

```markdown
---
name: data-processor
title: Efficient Data Processor
description: Processes data with caching
arguments:
  - name: data
    description: Data to process (CSV or JSON)
    required: true
  - name: operations
    description: Operations to perform
    required: true
---

{% comment %} Cache parsed data {% endcomment %}
{% assign is_json = false %}
{% assign is_csv = false %}

{% if data contains "{" and data contains "}" %}
  {% assign is_json = true %}
  {% assign parsed_data = data | parse_json %}
{% elsif data contains "," %}
  {% assign is_csv = true %}
  {% comment %} Cache row count {% endcomment %}
  {% assign rows = data | split: "\n" %}
  {% assign row_count = rows | size %}
{% endif %}

# Data Processing

## Data Format: {% if is_json %}JSON{% elsif is_csv %}CSV ({{row_count}} rows){% else %}Unknown{% endif %}

{% comment %} Reuse cached values {% endcomment %}
{% for operation in operations %}
  {% case operation %}
  {% when "count" %}
    - Count: {% if is_json %}{{parsed_data | size}}{% else %}{{row_count}}{% endif %}
  {% when "validate" %}
    - Validation: {% if is_json %}Valid JSON{% elsif is_csv %}Valid CSV{% endif %}
  {% endcase %}
{% endfor %}
```

## Error Handling

### Graceful Degradation

Handle errors gracefully:

```markdown
---
name: robust-analyzer
title: Robust Code Analyzer
description: Analyzes code with error handling
arguments:
  - name: code
    description: Code to analyze
    required: true
  - name: language
    description: Programming language
    default: "auto"
---

# Code Analysis

{% comment %} Safe language detection {% endcomment %}
{% assign detected_language = "unknown" %}
{% if language == "auto" %}
  {% if code contains "def " and code contains ":" %}
    {% assign detected_language = "python" %}
  {% elsif code contains "function" or code contains "const " %}
    {% assign detected_language = "javascript" %}
  {% elsif code contains "fn " and code contains "->" %}
    {% assign detected_language = "rust" %}
  {% endif %}
{% else %}
  {% assign detected_language = language %}
{% endif %}

## Language: {{detected_language | capitalize}}

{% comment %} Safe parsing with fallbacks {% endcomment %}
{% assign parse_success = false %}
{% capture parsed_structure %}
  {% if detected_language == "python" %}
    {% comment %} Python-specific parsing {% endcomment %}
    {% assign functions = code | split: "def " | size | minus: 1 %}
    {% assign classes = code | split: "class " | size | minus: 1 %}
    Functions: {{functions}}, Classes: {{classes}}
    {% assign parse_success = true %}
  {% elsif detected_language == "javascript" %}
    {% comment %} JavaScript-specific parsing {% endcomment %}
    {% assign functions = code | split: "function" | size | minus: 1 %}
    {% assign arrows = code | split: "=>" | size | minus: 1 %}
    Functions: {{functions | plus: arrows}}
    {% assign parse_success = true %}
  {% endif %}
{% endcapture %}

{% if parse_success %}
## Structure Analysis
{{parsed_structure}}
{% else %}
## Basic Analysis
Unable to parse structure for {{detected_language}}.
Falling back to general analysis:
- Lines: {{code | split: "\n" | size}}
- Characters: {{code | size}}
{% endif %}

## Code Review
Analyze the following {{detected_language}} code:

```{{detected_language}}
{{code}}
```
```

### Input Validation

Validate and sanitize inputs:

```markdown
---
name: secure-processor
title: Secure Input Processor
description: Processes inputs with validation
arguments:
  - name: user_input
    description: User-provided input
    required: true
  - name: input_type
    description: Expected input type
    required: true
  - name: max_length
    description: Maximum allowed length
    default: "1000"
---

{% comment %} Input validation {% endcomment %}
{% assign is_valid = true %}
{% assign validation_errors = "" %}

{% comment %} Length check {% endcomment %}
{% assign input_length = user_input | size %}
{% if input_length > max_length %}
  {% assign is_valid = false %}
  {% capture validation_errors %}{{validation_errors}}
  - Input exceeds maximum length ({{input_length}} > {{max_length}}){% endcapture %}
{% endif %}

{% comment %} Type validation {% endcomment %}
{% case input_type %}
{% when "email" %}
  {% unless user_input contains "@" and user_input contains "." %}
    {% assign is_valid = false %}
    {% capture validation_errors %}{{validation_errors}}
    - Invalid email format{% endcapture %}
  {% endunless %}
{% when "number" %}
  {% assign test_number = user_input | plus: 0 %}
  {% if test_number == 0 and user_input != "0" %}
    {% assign is_valid = false %}
    {% capture validation_errors %}{{validation_errors}}
    - Input is not a valid number{% endcapture %}
  {% endif %}
{% when "json" %}
  {% capture json_test %}{{user_input | parse_json}}{% endcapture %}
  {% unless json_test %}
    {% assign is_valid = false %}
    {% capture validation_errors %}{{validation_errors}}
    - Invalid JSON format{% endcapture %}
  {% endunless %}
{% endcase %}

# Input Processing Result

## Validation: {% if is_valid %}✅ Passed{% else %}❌ Failed{% endif %}

{% unless is_valid %}
## Validation Errors:
{{validation_errors}}
{% endunless %}

{% if is_valid %}
## Processing Input

Type: {{input_type}}
Length: {{input_length}} characters

### Sanitized Input:
```
{{user_input | strip | escape}}
```

### Next Steps:
Process the validated {{input_type}} input according to business logic.
{% else %}
## Cannot Process Invalid Input

Please fix the validation errors and try again.
{% endif %}
```

## Integration Patterns

### External Tool Integration

Integrate with external tools and services:

```markdown
---
name: ci-cd-analyzer
title: CI/CD Pipeline Analyzer
description: Analyzes CI/CD configurations
arguments:
  - name: pipeline_config
    description: CI/CD configuration file
    required: true
  - name: platform
    description: CI/CD platform (github, gitlab, jenkins)
    required: true
  - name: recommendations
    description: Include recommendations
    default: "true"
---

# CI/CD Pipeline Analysis

Platform: {{platform | capitalize}}

{% assign config = pipeline_config %}

## Pipeline Structure

{% case platform %}
{% when "github" %}
  {% if config contains "on:" %}
    ### Triggers
    - Configured triggers found
    {% if config contains "push:" %}✓ Push events{% endif %}
    {% if config contains "pull_request:" %}✓ Pull request events{% endif %}
    {% if config contains "schedule:" %}✓ Scheduled runs{% endif %}
  {% endif %}
  
  {% if config contains "jobs:" %}
    ### Jobs
    {% assign job_count = config | split: "jobs:" | last | split: ":" | size %}
    - Number of jobs: ~{{job_count}}
  {% endif %}

{% when "gitlab" %}
  {% if config contains "stages:" %}
    ### Stages
    - Pipeline stages defined
  {% endif %}
  
  {% if config contains "before_script:" %}
    ### Global Configuration
    - Global before_script found
  {% endif %}

{% when "jenkins" %}
  {% if config contains "pipeline {" %}
    ### Pipeline Type
    - Declarative pipeline
  {% elsif config contains "node {" %}
    - Scripted pipeline
  {% endif %}
{% endcase %}

## Security Analysis

{% capture security_issues %}
{% if config contains "secrets." or config contains "${{" %}
  - ✓ Uses secure secret management
{% endif %}
{% if config contains "password" or config contains "api_key" %}
  - ⚠️ Possible hardcoded credentials
{% endif %}
{% if platform == "github" and config contains "actions/checkout" %}
  {% unless config contains "actions/checkout@v" %}
    - ⚠️ Using unpinned actions
  {% endunless %}
{% endif %}
{% endcapture %}

{{security_issues | default: "No security issues found"}}

{% if recommendations == "true" %}
## Recommendations

{% case platform %}
{% when "github" %}
1. Use specific action versions (e.g., `actions/checkout@v3`)
2. Implement job dependencies for efficiency
3. Use matrix builds for multiple versions
4. Cache dependencies for faster builds

{% when "gitlab" %}
1. Use DAG for job dependencies
2. Implement proper stage dependencies
3. Use artifacts for job communication
4. Enable pipeline caching

{% when "jenkins" %}
1. Use declarative pipeline syntax
2. Implement proper error handling
3. Use Jenkins shared libraries
4. Enable pipeline visualization
{% endcase %}

### General Best Practices
- Implement proper testing stages
- Add security scanning steps
- Use parallel execution where possible
- Monitor pipeline metrics
{% endif %}

## Raw Configuration

```yaml
{{pipeline_config}}
```
```

## Advanced Examples

### Multi-Stage Document Generator

```markdown
---
name: tech-doc-generator
title: Technical Documentation Generator
description: Generates comprehensive technical documentation
arguments:
  - name: project_info
    description: Project information (JSON)
    required: true
  - name: doc_sections
    description: Sections to include (comma-separated)
    default: "overview,architecture,api,deployment"
  - name: audience
    description: Target audience
    default: "developers"
---

{% assign project = project_info | parse_json %}
{% assign sections = doc_sections | split: "," %}

# {{project.name}} Technical Documentation

Version: {{project.version}}
Last Updated: {% assign date = 'now' | date: "%B %d, %Y" %}{{date}}

{% for section in sections %}
{% case section | strip %}
{% when "overview" %}
## Overview

{{project.description}}

### Key Features
{% for feature in project.features %}
- **{{feature.name}}**: {{feature.description}}
{% endfor %}

### Technology Stack
{% for tech in project.stack %}
- {{tech.name}} ({{tech.version}}) - {{tech.purpose}}
{% endfor %}

{% when "architecture" %}
## Architecture

### System Components
{% for component in project.components %}
#### {{component.name}}
- **Type**: {{component.type}}
- **Responsibility**: {{component.responsibility}}
- **Dependencies**: {% for dep in component.dependencies %}{{dep}}{% unless forloop.last %}, {% endunless %}{% endfor %}
{% endfor %}

### Data Flow
```
{% for flow in project.dataflows %}
{{flow.source}} --> {{flow.destination}}: {{flow.description}}
{% endfor %}
```

{% when "api" %}
## API Reference

Base URL: `{{project.api.base_url}}`

### Authentication
{{project.api.auth.description}}

### Endpoints
{% for endpoint in project.api.endpoints %}
#### {{endpoint.method}} {{endpoint.path}}
{{endpoint.description}}

**Parameters:**
{% for param in endpoint.parameters %}
- `{{param.name}}` ({{param.type}}{% if param.required %}, required{% endif %}) - {{param.description}}
{% endfor %}

**Response:** {{endpoint.response.description}}
{% endfor %}

{% when "deployment" %}
## Deployment Guide

### Prerequisites
{% for prereq in project.deployment.prerequisites %}
- {{prereq}}
{% endfor %}

### Environment Variables
| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
{% for env in project.deployment.env_vars %}
| {{env.name}} | {{env.description}} | {{env.required}} | {{env.default | default: "none"}} |
{% endfor %}

### Deployment Steps
{% for step in project.deployment.steps %}
{{forloop.index}}. {{step.description}}
   ```bash
   {{step.command}}
   ```
{% endfor %}
{% endcase %}
{% endfor %}

---
Generated for {{audience}} by SwissArmyHammer
```

### Intelligent Code Refactoring Assistant

```markdown
---
name: refactoring-assistant
title: Intelligent Refactoring Assistant
description: Provides context-aware refactoring suggestions
arguments:
  - name: code
    description: Code to refactor
    required: true
  - name: code_metrics
    description: Code metrics (JSON)
    required: false
  - name: refactor_goals
    description: Refactoring goals (comma-separated)
    default: "readability,maintainability,performance"
  - name: preserve_behavior
    description: Ensure behavior preservation
    default: "true"
---

{% if code_metrics %}
  {% assign metrics = code_metrics | parse_json %}
{% endif %}

# Refactoring Analysis

## Current Code Metrics
{% if metrics %}
- Complexity: {{metrics.complexity}}
- Lines: {{metrics.lines}}
- Duplication: {{metrics.duplication}}%
- Test Coverage: {{metrics.coverage}}%
{% else %}
- Lines: {{code | split: "\n" | size}}
{% endif %}

## Refactoring Goals
{% assign goals = refactor_goals | split: "," %}
{% for goal in goals %}
- {{goal | strip | capitalize}}
{% endfor %}

## Analysis

```
{{code}}
```

{% capture refactoring_plan %}
{% for goal in goals %}
{% case goal | strip %}
{% when "readability" %}
### Readability Improvements
1. Extract complex conditionals into well-named functions
2. Replace magic numbers with named constants
3. Improve variable and function names
4. Add clarifying comments for complex logic

{% when "maintainability" %}
### Maintainability Enhancements
1. Apply SOLID principles
2. Reduce coupling between components
3. Extract reusable components
4. Improve error handling

{% when "performance" %}
### Performance Optimizations
1. Identify and optimize bottlenecks
2. Reduce unnecessary iterations
3. Implement caching where appropriate
4. Optimize data structures

{% when "testability" %}
### Testability Improvements
1. Extract pure functions
2. Reduce dependencies
3. Implement dependency injection
4. Separate business logic from I/O
{% endcase %}
{% endfor %}
{% endcapture %}

{{refactoring_plan}}

{% if preserve_behavior == "true" %}
## Behavior Preservation Strategy

To ensure the refactoring preserves behavior:

1. **Write characterization tests** before refactoring
2. **Refactor in small steps** with tests passing
3. **Use automated refactoring tools** where possible
4. **Compare outputs** before and after changes

### Suggested Test Cases
Based on the code analysis, ensure tests cover:
- Edge cases and boundary conditions
- Error handling paths
- Main business logic flows
- Integration points
{% endif %}

## Refactoring Priority

{% if metrics %}
{% if metrics.complexity > 10 %}
**High Priority**: Reduce complexity first - current complexity of {{metrics.complexity}} is too high
{% elsif metrics.duplication > 20 %}
**High Priority**: Address code duplication - {{metrics.duplication}}% duplication detected
{% elsif metrics.coverage < 60 %}
**High Priority**: Improve test coverage before refactoring - only {{metrics.coverage}}% covered
{% else %}
**Normal Priority**: Code is in reasonable shape for refactoring
{% endif %}
{% else %}
Based on initial analysis, focus on readability and structure improvements.
{% endif %}

## Next Steps

1. Review the refactoring plan
2. Set up safety nets (tests, version control)
3. Apply refactorings incrementally
4. Validate behavior preservation
5. Update documentation
```

## Best Practices

### 1. Use Meaningful Variable Names

```liquid
{% comment %} Bad {% endcomment %}
{% assign x = data | split: "," %}

{% comment %} Good {% endcomment %}
{% assign csv_rows = data | split: "," %}
```

### 2. Cache Expensive Operations

```liquid
{% comment %} Cache parsed data {% endcomment %}
{% assign parsed_json = data | parse_json %}
{% comment %} Reuse parsed_json multiple times {% endcomment %}
```

### 3. Provide Fallbacks

```liquid
{{variable | default: "No value provided"}}
```

### 4. Use Comments for Complex Logic

```liquid
{% comment %} 
  Check if the code is Python by looking for specific syntax
  This is more reliable than file extension alone
{% endcomment %}
{% if code contains "def " and code contains ":" %}
  {% assign language = "python" %}
{% endif %}
```

### 5. Modularize with Captures

```liquid
{% capture header %}
  # {{title}}
  Generated on: {{date}}
{% endcapture %}

{% comment %} Reuse header in multiple places {% endcomment %}
{{header}}
```

## Next Steps

- Explore [Custom Filters](./custom-filters.md) for extending functionality
- Learn about [Prompt Organization](./prompt-organization.md) for managing complex prompts
- See [Examples](./examples.md) for more real-world scenarios
- Read [Template Variables](./template-variables.md) for Liquid syntax reference