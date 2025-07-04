# Creating Prompts

SwissArmyHammer prompts are markdown files with YAML front matter that define reusable AI prompts. This guide walks you through creating effective prompts.

## Basic Structure

Every prompt file has two parts:

1. **YAML Front Matter** - Metadata about the prompt
2. **Markdown Content** - The actual prompt template

```markdown
---
name: my-prompt
title: My Awesome Prompt
description: Does something useful
arguments:
  - name: input
    description: What to process
    required: true
---

# My Prompt

Please help me with {{input}}.

Provide a detailed response.
```

## YAML Front Matter

The YAML front matter defines the prompt's metadata:

### Required Fields

- `name` - Unique identifier for the prompt
- `title` - Human-readable name
- `description` - What the prompt does

### Optional Fields

- `category` - Group related prompts (e.g., "development", "writing")
- `tags` - List of keywords for discovery
- `arguments` - Input parameters (see [Arguments](#arguments))
- `author` - Prompt creator
- `version` - Version number
- `license` - License information

### Example

```yaml
---
name: code-review
title: Code Review Assistant
description: Reviews code for best practices, bugs, and improvements
category: development
tags: ["code", "review", "quality", "best-practices"]
author: SwissArmyHammer Team
version: 1.0.0
arguments:
  - name: code
    description: The code to review
    required: true
  - name: language
    description: Programming language
    required: false
    default: auto-detect
  - name: focus
    description: Specific areas to focus on
    required: false
    default: all aspects
---
```

## Arguments

Arguments define the inputs your prompt accepts. Each argument has:

### Argument Properties

- `name` - Parameter name (used in template as `{{name}}`)
- `description` - What this parameter is for
- `required` - Whether the argument is mandatory
- `default` - Default value if not provided
- `type_hint` - Expected data type (documentation only)

### Example Arguments

```yaml
arguments:
  - name: text
    description: Text to analyze
    required: true
    type_hint: string
  
  - name: format
    description: Output format
    required: false
    default: markdown
    type_hint: string
  
  - name: max_length
    description: Maximum response length
    required: false
    default: 500
    type_hint: integer
  
  - name: include_examples
    description: Include code examples
    required: false
    default: true
    type_hint: boolean
```

## Template Content

The markdown content is your prompt template using [Liquid templating](./template-variables.md).

### Basic Variables

Use `{{variable}}` to insert argument values:

```markdown
Please review this {{language}} code:

```{{code}}```

Focus on {{focus}}.
```

### Conditional Logic

Use `{% if %}` blocks for conditional content:

```markdown
{% if language == "python" %}
Pay special attention to:
- PEP 8 style guidelines
- Type hints
- Error handling
{% elsif language == "javascript" %}
Pay special attention to:
- ESLint rules
- Async/await usage
- Error handling
{% endif %}
```

### Loops

Use `{% for %}` to iterate over lists:

```markdown
{% if tags %}
Tags: {% for tag in tags %}#{{tag}}{% unless forloop.last %}, {% endunless %}{% endfor %}
{% endif %}
```

### Filters

Apply filters to transform data:

```markdown
Language: {{language | capitalize}}
Code length: {{code | length}} characters
Summary: {{description | truncate: 100}}
```

## Organization

### Directory Structure

Organize prompts in logical directories:

```
prompts/
├── development/
│   ├── code-review.md
│   ├── debug-helper.md
│   └── api-docs.md
├── writing/
│   ├── blog-post.md
│   ├── email-draft.md
│   └── summary.md
└── analysis/
    ├── data-insights.md
    └── competitor-analysis.md
```

### Naming Conventions

- Use kebab-case for filenames: `code-review.md`
- Make names descriptive: `debug-python-errors.md` not `debug.md`
- Include the category in the path, not the filename

### Categories and Tags

Use categories for broad groupings:
- `development` - Code-related prompts
- `writing` - Content creation
- `analysis` - Data and research
- `productivity` - Task management

Use tags for specific features:
- `["python", "debugging", "error-handling"]`
- `["marketing", "email", "b2b"]`
- `["data", "visualization", "charts"]`

## Best Practices

### 1. Write Clear Descriptions

```yaml
# Good
description: Reviews Python code for PEP 8 compliance, type hints, and common bugs

# Bad
description: Code review
```

### 2. Provide Helpful Defaults

```yaml
arguments:
  - name: style_guide
    description: Coding style guide to follow
    required: false
    default: PEP 8  # Sensible default
```

### 3. Use Descriptive Variable Names

```markdown
# Good
Please analyze this {{source_code}} for {{security_vulnerabilities}}.

# Bad
Please analyze this {{input}} for {{stuff}}.
```

### 4. Include Examples in Descriptions

```yaml
arguments:
  - name: format
    description: Output format (markdown, json, html)
    required: false
    default: markdown
```

### 5. Structure Your Prompts

Use clear sections and formatting:

```markdown
# Code Review

## Overview
Please review the following {{language}} code for:

## Focus Areas
1. **Best Practices** - Follow {{style_guide}} guidelines
2. **Security** - Identify potential vulnerabilities
3. **Performance** - Suggest optimizations
4. **Maintainability** - Assess code clarity

## Code to Review

```{{code}}```

## Instructions
{{#if include_suggestions}}
Please provide specific improvement suggestions.
{{/if}}
```

### 6. Test Your Prompts

Use the CLI to test prompts:

```bash
# Test with required arguments
swissarmyhammer test code-review --code "def hello(): print('hi')" --language python

# Test with defaults
swissarmyhammer test code-review --code "function hello() { console.log('hi'); }"
```

## Common Patterns

### Code Analysis

```markdown
---
name: analyze-code
title: Code Analyzer
description: Analyzes code for issues and improvements
arguments:
  - name: code
    description: Code to analyze
    required: true
  - name: language
    description: Programming language
    required: false
    default: auto-detect
---

# Code Analysis

Analyze this {{language}} code:

```{{code}}```

Provide feedback on:
- Code quality and best practices
- Potential bugs or issues
- Performance optimizations
- Readability improvements
```

### Document Generation

```markdown
---
name: api-docs
title: API Documentation Generator
description: Generates API documentation from code
arguments:
  - name: code
    description: API code to document
    required: true
  - name: format
    description: Documentation format
    required: false
    default: markdown
---

# API Documentation

Generate {{format}} documentation for this API:

```{{code}}```

Include:
- Endpoint descriptions
- Parameter details
- Response examples
- Error codes
```

### Text Processing

```markdown
---
name: summarize
title: Text Summarizer
description: Creates concise summaries of text
arguments:
  - name: text
    description: Text to summarize
    required: true
  - name: length
    description: Target summary length
    required: false
    default: 3 sentences
---

# Text Summary

Create a {{length}} summary of this text:

{{text}}

Focus on the key points and main ideas.
```

## Next Steps

- Learn about [YAML Front Matter](./yaml-front-matter.md) in detail
- Explore [Template Variables](./template-variables.md) and Liquid syntax
- Check out [Custom Filters](./custom-filters.md) for advanced transformations
- See [Examples](./examples.md) for real-world prompt templates
- Read about [Prompt Organization](./prompt-organization.md) strategies