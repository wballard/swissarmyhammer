---
name: plan
title: Task Planning Assistant
description: A prompt for creating structured plans and breaking down complex tasks
arguments:
  - name: task
    description: The task to plan for
    required: true
  - name: context
    description: Additional context for the planning
    required: false
    default: ""
  - name: constraints
    description: Any constraints or limitations to consider
    required: false
    default: none
---

# Planning for {{task}}

{% if context and context != "" %}
## Context
{{context}}
{% endif %}

{% if constraints and constraints != "none" %}
## Constraints
{{constraints}}
{% endif %}

Create a detailed plan for: {{task}}