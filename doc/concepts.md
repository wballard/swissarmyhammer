# Basic Concepts

Understanding the core concepts of SwissArmyHammer will help you use it effectively.

## What is a Prompt?

A prompt in SwissArmyHammer is a reusable template for interacting with AI models. It consists of:

1. **Metadata** - Information about the prompt (title, description, arguments)
2. **Template** - The actual prompt text with variable placeholders
3. **Arguments** - Variables that can be substituted into the template

### Example Prompt File

```markdown
---
title: Code Review Assistant
description: Reviews code for best practices
category: development
tags:
  - code
  - review
  - quality
arguments:
  - name: code
    description: The code to review
    required: true
  - name: language
    description: Programming language
    default: "auto-detect"
  - name: focus
    description: Specific areas to focus on
    required: false
---

Please review this {{ language }} code:

```{{ language }}
{{ code }}
```

{% if focus %}
Focus specifically on: {{ focus }}
{% endif %}

Provide feedback on:
- Code quality and best practices
- Potential bugs or issues
- Performance considerations
- Security concerns
- Suggestions for improvement
```

## YAML Front Matter

The YAML front matter (between `---` markers) defines the prompt's metadata:

### Required Fields

- **title**: Human-readable name for the prompt
- **description**: What the prompt does

### Optional Fields

- **category**: Organizational category (e.g., "development", "writing")
- **tags**: List of tags for searching and filtering
- **arguments**: List of variables used in the template
- **model**: Suggested AI model to use
- **temperature**: Suggested temperature setting
- **max_tokens**: Suggested max tokens

### Arguments Structure

Each argument can have:

```yaml
arguments:
  - name: variable_name      # Required: variable name in template
    description: What it is  # Optional: helps users understand
    required: true/false     # Optional: default is false
    default: "value"        # Optional: default value if not provided
    type_hint: "string"     # Optional: type hint for validation
```

## Liquid Templates

SwissArmyHammer uses the [Liquid template engine](https://shopify.github.io/liquid/) with custom extensions.

### Variables

Basic variable substitution:

```liquid
Hello {{ name }}!
Your age is {{ age }}.
```

### Filters

Transform variables with filters:

```liquid
{{ name | capitalize }}
{{ text | truncate: 50 }}
{{ code | format_lang: "python" }}
```

### Control Flow

Conditional logic:

```liquid
{% if premium_user %}
  Access granted to premium features
{% else %}
  Please upgrade your account
{% endif %}

{% case language %}
  {% when "python" %}
    Use pytest for testing
  {% when "javascript" %}
    Use Jest for testing
  {% else %}
    Use appropriate testing framework
{% endcase %}
```

### Loops

Iterate over collections:

```liquid
{% for item in items %}
  - {{ item }}
{% endfor %}

{% for key, value in metadata %}
  {{ key }}: {{ value }}
{% endfor %}
```

## Prompt Organization

SwissArmyHammer uses a hierarchical directory structure with override capabilities.

### Directory Hierarchy

1. **Built-in prompts**: Shipped with SwissArmyHammer
   - Location: Embedded in binary
   - Purpose: Common, general-purpose prompts

2. **User prompts**: Personal prompt collection
   - Location: `~/.swissarmyhammer/prompts/`
   - Purpose: User's custom prompts available globally

3. **Local prompts**: Project-specific prompts
   - Location: `./.swissarmyhammer/prompts/`
   - Purpose: Prompts specific to current project

### Override Behavior

Prompts are loaded in order, with later locations overriding earlier ones:

```
Built-in → User → Local
```

Example:
- Built-in has `code-review` prompt
- User creates `~/.swissarmyhammer/prompts/code-review.md`
- User's version overrides the built-in version

### Naming Conventions

- Use kebab-case: `code-review`, not `CodeReview` or `code_review`
- Be descriptive: `python-unittest-generator` not `test-gen`
- Use categories: Put related prompts in subdirectories

### Categories and Tags

Organize prompts with categories and tags:

```yaml
category: development
tags:
  - python
  - testing
  - tdd
```

This enables:
- Filtering: `swissarmyhammer list --category development`
- Searching: `swissarmyhammer search --tag python`

## Template Variables

Variables are placeholders in your template that get replaced with actual values.

### Simple Variables

```liquid
Hello {{ name }}!
```

### Variable with Filters

```liquid
Hello {{ name | capitalize }}!
```

### Default Values

In the template:
```liquid
Hello {{ name | default: "Friend" }}!
```

Or in arguments:
```yaml
arguments:
  - name: name
    default: "Friend"
```

### Special Variables

SwissArmyHammer provides some special variables:

- `{{ env.USER }}` - Current username
- `{{ env.HOME }}` - Home directory
- `{{ env.PWD }}` - Current directory
- `{{ timestamp }}` - Current timestamp
- `{{ date }}` - Current date

## Model Context Protocol (MCP)

MCP is the protocol that allows SwissArmyHammer to communicate with Claude Code and other AI tools.

### How it Works

1. **Server**: SwissArmyHammer runs as an MCP server
2. **Discovery**: Claude Code discovers available prompts
3. **Selection**: User selects a prompt in Claude Code
4. **Arguments**: Claude Code collects required arguments
5. **Rendering**: SwissArmyHammer renders the final prompt
6. **Execution**: Claude Code sends the rendered prompt to the AI

### Benefits

- **Integration**: Seamless integration with AI tools
- **Management**: Centralized prompt management
- **Versioning**: Version control for prompts
- **Sharing**: Easy sharing of prompt libraries

## Custom Filters

SwissArmyHammer extends Liquid with domain-specific filters.

### Categories

1. **Code Filters**: Working with source code
2. **Text Filters**: Text processing and formatting
3. **Data Filters**: Data transformation
4. **Utility Filters**: General utilities

### Examples

```liquid
# Format code with syntax highlighting
{{ code | format_lang: "python" }}

# Extract function names
{{ code | extract_functions | join: ", " }}

# Create URL-friendly slug
{{ title | slugify }}

# Parse JSON data
{% assign data = json_string | from_json %}
{{ data.name }}
```

## Best Practices

### 1. Clear Naming

Use descriptive names that indicate the prompt's purpose:
- ✅ `python-docstring-generator`
- ❌ `doc-gen-2`

### 2. Comprehensive Metadata

Provide complete metadata:
```yaml
title: Python Docstring Generator
description: Generates Google-style docstrings for Python functions
category: development/python
tags: [python, documentation, docstring]
```

### 3. Flexible Templates

Make templates adaptable:
```liquid
{% if detailed %}
  # Detailed analysis requested
  Provide comprehensive documentation including:
  - Parameters with types
  - Return values
  - Exceptions
  - Examples
{% else %}
  # Generate concise docstring
{% endif %}
```

### 4. Sensible Defaults

Provide defaults for optional arguments:
```yaml
arguments:
  - name: style
    description: Docstring style
    default: "google"
```

### 5. Validation

Validate inputs when possible:
```liquid
{% unless languages contains language %}
  Error: Unsupported language '{{ language }}'
  Supported: {{ languages | join: ", " }}
{% endunless %}
```

## Performance Considerations

1. **Template Complexity**: Complex templates with many includes impact performance
2. **Directory Size**: Large prompt directories take longer to scan
3. **Search Indexing**: Initial indexing of large libraries takes time
4. **Filter Performance**: Some filters (like `format_lang`) are computationally expensive

## Security Considerations

1. **Template Injection**: Be careful with user-provided content in templates
2. **File Access**: Templates cannot access files outside allowed directories
3. **Environment Variables**: Only specific env vars are exposed
4. **Command Execution**: Templates cannot execute system commands