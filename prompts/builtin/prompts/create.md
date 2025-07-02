---
name: prompts-create
title: Create New Prompt
description: Help create effective prompts for swissarmyhammer
arguments:
  - name: purpose
    description: What the prompt should accomplish
    required: true
  - name: category
    description: Category for the prompt (debug, refactor, review, docs, test, etc.)
    required: false
    default: "general"
  - name: inputs_needed
    description: What information the prompt needs from users
    required: false
    default: ""
  - name: complexity
    description: Complexity level (simple, moderate, advanced)
    required: false
    default: "moderate"
---

# Create Prompt: {{purpose}}

## Prompt Requirements
- **Purpose**: {{purpose}}
- **Category**: {{category}}
- **Complexity**: {{complexity}}
{% if inputs_needed %}
- **Required Inputs**: {{inputs_needed}}
{% endif %}

## Prompt Design Guide

### 1. YAML Front Matter Structure
```yaml
---
name: {{category}}-descriptive-name
title: Clear Title for {{purpose}}
description: One-line description of what this prompt does
arguments:
  - name: primary_input
    description: Main input needed
    required: true
  - name: optional_input
    description: Additional configuration
    required: false
    default: "sensible default"
---
```

### 2. Prompt Content Guidelines

#### Opening Context
- Set clear expectations
- Define the task scope
- Mention any constraints

#### Input Integration
- Use `{{{{variable}}}}` for simple substitution
- Use `{{{% if variable %}}}` for conditional content
- Use `{{{{{variable}}}}}` for triple-brace (preserves one set of braces)

#### Structure Patterns
For {{complexity}} complexity:
{% if complexity == "simple" %}
- Direct instruction format
- Single-purpose focus
- Minimal configuration
{% elsif complexity == "moderate" %}
- Multi-step process
- Clear sections
- Flexible options
{% else %}
- Comprehensive framework
- Multiple pathways
- Advanced features
{% endif %}

### 3. Effective Prompt Techniques

#### Clarity
- Use specific, actionable language
- Break complex tasks into steps
- Provide examples where helpful

#### Flexibility
- Accommodate different use cases
- Provide sensible defaults
- Allow customization

#### Context Awareness
- Consider user's expertise level
- Adapt to different environments
- Handle edge cases gracefully

### 4. Example Prompt

Based on your requirements, here's a template:

```markdown
---
name: {{category}}-{{purpose | slugify}}
title: {{purpose | titlecase}}
description: A prompt that {{purpose | lowercase}}
arguments:
  - name: input
    description: The main input for {{purpose}}
    required: true
{% if inputs_needed %}
{{#each (split inputs_needed ",")}}
  - name: {{this | trim | slugify}}
    description: {{this | trim}}
    required: false
    default: ""
{{/each}}
{% endif %}
---

# {{purpose}}

## Overview
This prompt helps you {{purpose | lowercase}}.

## Input
- **Main Input**: {{{{input}}}}
{% if inputs_needed %}
## Additional Configuration
{{#each (split inputs_needed ",")}}
- **{{this | trim}}**: {{{{{{this | trim | slugify}}}}}}
{{/each}}
{% endif %}

## Process

1. **Analysis Phase**
   - Understand the requirements
   - Identify key components
   - Plan the approach

2. **Implementation**
   - Apply best practices
   - Consider edge cases
   - Ensure quality

3. **Validation**
   - Verify correctness
   - Check completeness
   - Confirm expectations met

## Output
Provide the result with:
- Clear structure
- Detailed explanation
- Next steps if applicable
```

### 5. Testing Your Prompt
- Try with various inputs
- Check edge cases
- Verify output quality
- Get user feedback

### 6. Best Practices
- Keep prompts focused
- Use consistent terminology
- Provide helpful examples
- Document assumptions
- Version your prompts