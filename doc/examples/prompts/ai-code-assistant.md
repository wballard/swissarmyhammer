---
title: AI Code Assistant
description: Intelligent code assistance with context awareness
category: development
tags: ["coding", "assistance", "refactoring", "optimization"]
arguments:
  - name: code
    description: The code to analyze and improve
    required: true
  - name: language
    description: Programming language
    required: true
  - name: task
    description: Specific task to perform
    required: true
    default: "improve"
  - name: context
    description: Additional context about the code
    required: false
---

# AI Code Assistant

You are an expert {{ language }} developer. Please help me with the following task: **{{ task }}**

## Code to analyze:

```{{ language }}
{{ code }}
```

{% if context %}
## Additional context:
{{ context }}
{% endif %}

## Instructions:

{% if task == "improve" %}
Please improve this code by:
1. Identifying potential issues or inefficiencies
2. Suggesting optimizations
3. Ensuring best practices are followed
4. Improving readability and maintainability
{% elsif task == "refactor" %}
Please refactor this code to:
1. Improve structure and organization
2. Extract reusable components
3. Reduce complexity
4. Follow {{ language }} conventions
{% elsif task == "optimize" %}
Please optimize this code for:
1. Performance improvements
2. Memory efficiency
3. Algorithmic complexity
4. Resource usage
{% elsif task == "debug" %}
Please help debug this code by:
1. Identifying potential bugs
2. Analyzing logic errors
3. Suggesting fixes
4. Explaining the root cause
{% elsif task == "document" %}
Please document this code by:
1. Adding clear docstrings/comments
2. Explaining complex logic
3. Providing usage examples
4. Describing parameters and return values
{% else %}
Please analyze this code and provide assistance with: {{ task }}
{% endif %}

## Response format:
- **Analysis**: What you found in the code
- **Recommendations**: Specific improvements to make
- **Updated Code**: The improved version
- **Explanation**: Why these changes improve the code