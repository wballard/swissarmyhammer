---
title: Task List Formatter
description: Format and organize tasks with priorities and grouping
arguments:
  - name: tasks
    description: Comma-separated list of tasks
    required: true
  - name: group_by
    description: How to group tasks (priority, status, category, none)
    required: false
    default: "none"
  - name: show_index
    description: Show task numbers
    required: false
    default: "true"
  - name: show_status
    description: Include status checkboxes
    required: false
    default: "true"
  - name: date_format
    description: Date format for due dates
    required: false
    default: "%B %d, %Y"
---

# Task List

{% assign task_list = tasks | split: "," %}
{% assign total_tasks = task_list | size %}
{% assign show_numbers = show_index | default: "true" %}
{% assign show_checks = show_status | default: "true" %}

**Total Tasks**: {{ total_tasks }}
**Generated**: {{ "now" | date: date_format }}

---

{% if group_by == "none" %}
## All Tasks

{% for task in task_list %}
{% assign task_clean = task | strip %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}{% if show_checks == "true" %}[ ] {% endif %}{{ task_clean }}
{% endfor %}

{% elsif group_by == "priority" %}
## Tasks by Priority

{% comment %} Simulate priority grouping by looking for keywords {% endcomment %}
### 🔴 High Priority
{% for task in task_list %}
{% assign task_lower = task | downcase %}
{% if task_lower contains "urgent" or task_lower contains "critical" or task_lower contains "high" %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}{% if show_checks == "true" %}[ ] {% endif %}{{ task | strip }}
{% endif %}
{% endfor %}

### 🟡 Medium Priority
{% for task in task_list %}
{% assign task_lower = task | downcase %}
{% unless task_lower contains "urgent" or task_lower contains "critical" or task_lower contains "high" or task_lower contains "low" %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}{% if show_checks == "true" %}[ ] {% endif %}{{ task | strip }}
{% endunless %}
{% endfor %}

### 🟢 Low Priority
{% for task in task_list %}
{% assign task_lower = task | downcase %}
{% if task_lower contains "low" %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}{% if show_checks == "true" %}[ ] {% endif %}{{ task | strip }}
{% endif %}
{% endfor %}

{% elsif group_by == "status" %}
## Tasks by Status

### 📋 To Do
{% for task in task_list %}
{% assign task_clean = task | strip %}
{% unless task_clean contains "[x]" or task_clean contains "[X]" or task_clean contains "✓" %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}[ ] {{ task_clean | remove: "[ ]" | strip }}
{% endunless %}
{% endfor %}

### ✅ Completed
{% for task in task_list %}
{% assign task_clean = task | strip %}
{% if task_clean contains "[x]" or task_clean contains "[X]" or task_clean contains "✓" %}
{% if show_numbers == "true" %}{{ forloop.index }}. {% endif %}[x] {{ task_clean | remove: "[x]" | remove: "[X]" | remove: "✓" | strip }}
{% endif %}
{% endfor %}

{% else %}
## Tasks

{% for task in task_list %}
{% if forloop.first %}
### First Task
{% elsif forloop.last %}
### Last Task
{% else %}
### Task {{ forloop.index }}
{% endif %}

{% if show_checks == "true" %}[ ] {% endif %}{{ task | strip }}

{% comment %} Add metadata for each task {% endcomment %}
- Position: {{ forloop.index }} of {{ forloop.length }}
- {% cycle "Priority: High", "Priority: Medium", "Priority: Low" %}
- Status: {% assign mod_val = forloop.index0 | modulo: 3 %}{% if mod_val == 0 %}New{% elsif mod_val == 1 %}In Progress{% else %}Review{% endif %}
{% endfor %}
{% endif %}

---

## Summary

- **Total Tasks**: {{ total_tasks }}
- **Average Length**: {{ task_list | join: "" | size | divided_by: total_tasks }} characters
{% assign first_task = task_list | first | strip %}
{% assign last_task = task_list | last | strip %}
- **First Task**: {{ first_task | truncate: 30 }}
- **Last Task**: {{ last_task | truncate: 30 }}

{% if total_tasks > 10 %}
⚠️ **Note**: You have {{ total_tasks }} tasks. Consider breaking them down into smaller groups or categories.
{% elsif total_tasks == 0 %}
✨ **Congratulations**: No pending tasks!
{% else %}
💪 **Manageable**: {{ total_tasks }} tasks is a good amount to focus on.
{% endif %}