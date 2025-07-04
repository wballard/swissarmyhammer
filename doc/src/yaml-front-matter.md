# YAML Front Matter

YAML front matter is the metadata section at the beginning of each prompt file. It defines the prompt's properties, arguments, and other configuration details.

## Structure

Front matter appears between triple dashes (`---`) at the start of your markdown file:

```markdown
---
name: my-prompt
title: My Prompt
description: What this prompt does
---

# Prompt content starts here
```

## Required Fields

Every prompt must have these fields:

### `name`
- **Type**: String
- **Description**: Unique identifier for the prompt
- **Format**: kebab-case recommended
- **Example**: `code-review`, `debug-helper`, `api-docs`

```yaml
name: code-review
```

### `title`
- **Type**: String  
- **Description**: Human-readable name displayed in UIs
- **Format**: Title Case
- **Example**: `Code Review Assistant`, `Debug Helper`

```yaml
title: Code Review Assistant
```

### `description`
- **Type**: String
- **Description**: Brief explanation of what the prompt does
- **Format**: Sentence or short paragraph
- **Example**: Clear, actionable description

```yaml
description: Reviews code for best practices, bugs, and potential improvements
```

## Optional Fields

### `category`
- **Type**: String
- **Description**: Groups related prompts together
- **Common Values**: `development`, `writing`, `analysis`, `productivity`

```yaml
category: development
```

### `tags`
- **Type**: Array of strings
- **Description**: Keywords for discovery and filtering
- **Format**: Lowercase, descriptive terms

```yaml
tags: ["python", "debugging", "error-handling"]
```

### `arguments`
- **Type**: Array of argument objects
- **Description**: Input parameters the prompt accepts
- **See**: [Arguments section](#arguments) below

```yaml
arguments:
  - name: code
    description: Code to review
    required: true
```

### `author`
- **Type**: String
- **Description**: Creator of the prompt
- **Format**: Name or organization

```yaml
author: SwissArmyHammer Team
```

### `version`
- **Type**: String
- **Description**: Version number for tracking changes
- **Format**: Semantic versioning recommended

```yaml
version: 1.2.0
```

### `license`
- **Type**: String
- **Description**: License for the prompt
- **Common Values**: `MIT`, `Apache-2.0`, `GPL-3.0`

```yaml
license: MIT
```

### `created`
- **Type**: Date string
- **Description**: When the prompt was created
- **Format**: ISO 8601 date

```yaml
created: 2024-01-15
```

### `updated`
- **Type**: Date string
- **Description**: Last modification date
- **Format**: ISO 8601 date

```yaml
updated: 2024-03-20
```

### `keywords`
- **Type**: Array of strings
- **Description**: Alternative to tags for SEO/discovery
- **Note**: Similar to tags but more formal

```yaml
keywords: ["code quality", "static analysis", "best practices"]
```

## Arguments

Arguments define the inputs your prompt can accept. Each argument is an object with these properties:

### Argument Properties

#### `name` (required)
- **Type**: String
- **Description**: Parameter name used in template
- **Format**: snake_case or kebab-case
- **Usage**: Referenced as `{{name}}` in template

```yaml
arguments:
  - name: source_code
    # Used as {{source_code}} in template
```

#### `description` (required)
- **Type**: String
- **Description**: What this argument is for
- **Format**: Clear, helpful explanation

```yaml
arguments:
  - name: language
    description: Programming language of the code
```

#### `required` (optional, default: false)
- **Type**: Boolean
- **Description**: Whether this argument must be provided
- **Default**: `false`

```yaml
arguments:
  - name: code
    description: Code to analyze
    required: true  # Must be provided
  - name: style_guide
    description: Coding style to follow
    required: false  # Optional
```

#### `default` (optional)
- **Type**: String
- **Description**: Default value if not provided
- **Note**: Only used when `required: false`

```yaml
arguments:
  - name: format
    description: Output format
    required: false
    default: markdown
```

#### `type_hint` (optional)
- **Type**: String
- **Description**: Expected data type (documentation only)
- **Common Values**: `string`, `integer`, `boolean`, `array`, `object`

```yaml
arguments:
  - name: max_length
    description: Maximum output length
    required: false
    default: 500
    type_hint: integer
```

### Argument Examples

#### Simple Text Input
```yaml
arguments:
  - name: text
    description: Text to process
    required: true
    type_hint: string
```

#### Optional with Default
```yaml
arguments:
  - name: format
    description: Output format (markdown, json, html)
    required: false
    default: markdown
    type_hint: string
```

#### Boolean Flag
```yaml
arguments:
  - name: include_examples
    description: Include code examples in output
    required: false
    default: true
    type_hint: boolean
```

#### Multiple Arguments
```yaml
arguments:
  - name: code
    description: Source code to review
    required: true
    type_hint: string
  
  - name: language
    description: Programming language
    required: false
    default: auto-detect
    type_hint: string
    
  - name: severity
    description: Minimum issue severity to report
    required: false
    default: medium
    type_hint: string
    
  - name: include_suggestions
    description: Include improvement suggestions
    required: false
    default: true
    type_hint: boolean
```

## Complete Example

Here's a comprehensive example showing all available fields:

```yaml
---
name: comprehensive-code-review
title: Comprehensive Code Review Assistant
description: Performs detailed code review focusing on best practices, security, and performance
category: development
tags: ["code-review", "security", "performance", "best-practices"]
author: SwissArmyHammer Team
version: 2.1.0
license: MIT
created: 2024-01-15
updated: 2024-03-20
keywords: ["static analysis", "code quality", "security audit"]

arguments:
  - name: code
    description: Source code to review
    required: true
    type_hint: string
    
  - name: language
    description: Programming language (python, javascript, rust, etc.)
    required: false
    default: auto-detect
    type_hint: string
    
  - name: focus_areas
    description: Specific areas to focus on (security, performance, style)
    required: false
    default: all
    type_hint: string
    
  - name: severity_threshold
    description: Minimum severity level to report (low, medium, high)
    required: false
    default: medium
    type_hint: string
    
  - name: include_examples
    description: Include code examples in suggestions
    required: false
    default: true
    type_hint: boolean
    
  - name: max_suggestions
    description: Maximum number of suggestions to provide
    required: false
    default: 10
    type_hint: integer
---

# Comprehensive Code Review

I'll perform a detailed review of your {{language}} code, focusing on {{focus_areas}}.

## Code Analysis

```{{code}}```

## Review Criteria

I'll evaluate the code for:

{% if focus_areas contains "security" or focus_areas == "all" %}
- **Security vulnerabilities** and best practices
{% endif %}

{% if focus_areas contains "performance" or focus_areas == "all" %}
- **Performance optimizations** and efficiency
{% endif %}

{% if focus_areas contains "style" or focus_areas == "all" %}
- **Code style** and formatting consistency
{% endif %}

{% if language != "auto-detect" %}
- **{{language | capitalize}}-specific** best practices and idioms
{% endif %}

## Reporting

- Minimum severity: {{severity_threshold}}
- Maximum suggestions: {{max_suggestions}}
{% if include_examples %}
- Including code examples and fixes
{% endif %}

Please provide detailed feedback with specific line references where applicable.
```

## Validation Rules

SwissArmyHammer validates your YAML front matter:

### Required Field Validation
- `name`, `title`, and `description` must be present
- `name` must be unique within the prompt library
- `name` must not contain spaces or special characters

### Argument Validation
- Each argument must have `name` and `description`
- Argument names must be valid template variables
- Required arguments cannot have default values
- Argument names must be unique within the prompt

### Type Validation
- Arrays must contain valid elements
- Dates must be in ISO format
- Booleans must be `true` or `false`

## Best Practices

### 1. Use Descriptive Names
```yaml
# Good
name: python-code-review
title: Python Code Review Assistant

# Bad
name: review
title: Review
```

### 2. Write Clear Descriptions
```yaml
# Good
description: Analyzes Python code for PEP 8 compliance, type hints, security issues, and performance optimizations

# Bad
description: Reviews code
```

### 3. Organize with Categories and Tags
```yaml
category: development
tags: ["python", "pep8", "security", "performance", "code-quality"]
```

### 4. Provide Sensible Defaults
```yaml
arguments:
  - name: style_guide
    description: Python style guide to follow
    required: false
    default: PEP 8  # Most common choice
```

### 5. Use Type Hints
```yaml
arguments:
  - name: max_issues
    description: Maximum number of issues to report
    required: false
    default: 20
    type_hint: integer  # Helps users understand expected format
```

### 6. Keep Versions Updated
```yaml
version: 1.2.0  # Update when you make changes
updated: 2024-03-20  # Track modification date
```

## Common Patterns

### Code Processing Prompt
```yaml
---
name: code-processor
title: Code Processor
description: Processes and transforms code
category: development
arguments:
  - name: code
    description: Source code to process
    required: true
  - name: language
    description: Programming language
    required: false
    default: auto-detect
  - name: output_format
    description: Desired output format
    required: false
    default: markdown
---
```

### Text Analysis Prompt
```yaml
---
name: text-analyzer
title: Text Analyzer
description: Analyzes text for various metrics
category: analysis
arguments:
  - name: text
    description: Text to analyze
    required: true
  - name: analysis_type
    description: Type of analysis to perform
    required: false
    default: comprehensive
  - name: include_stats
    description: Include statistical analysis
    required: false
    default: true
    type_hint: boolean
---
```

### Document Generator
```yaml
---
name: doc-generator
title: Document Generator
description: Generates documentation from code or specifications
category: documentation
arguments:
  - name: source
    description: Source material to document
    required: true
  - name: format
    description: Output format
    required: false
    default: markdown
  - name: include_examples
    description: Include usage examples
    required: false
    default: true
    type_hint: boolean
---
```

## Troubleshooting

### Common Errors

#### Invalid YAML Syntax
```yaml
# Error: Missing quotes around string with special characters
description: This won't work: because of the colon

# Fixed: Quote strings with special characters
description: "This works: because it's quoted"
```

#### Missing Required Fields
```yaml
# Error: Missing required fields
title: My Prompt

# Fixed: Include all required fields
name: my-prompt
title: My Prompt
description: What this prompt does
```

#### Invalid Argument Structure
```yaml
# Error: Missing required argument properties
arguments:
  - name: code

# Fixed: Include required properties
arguments:
  - name: code
    description: Code to process
    required: true
```

### Validation Tips

1. **Use a YAML validator** - Many online tools can check syntax
2. **Test with CLI** - Use `swissarmyhammer test` to validate prompts
3. **Check the logs** - SwissArmyHammer provides detailed error messages
4. **Start simple** - Begin with minimal front matter and add complexity

## Next Steps

- Learn about [Template Variables](./template-variables.md) to use your arguments
- Explore [Custom Filters](./custom-filters.md) for advanced text processing
- See [Creating Prompts](./creating-prompts.md) for the complete workflow
- Check [Examples](./examples.md) for real-world YAML configurations