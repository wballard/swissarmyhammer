---
title: Statistics Calculator
description: Calculate statistics on numeric data using math filters and array operations
arguments:
  - name: numbers
    description: Comma-separated list of numbers
    required: true
  - name: precision
    description: Decimal precision for calculations
    required: false
    default: "2"
  - name: show_outliers
    description: Identify outliers in the dataset
    required: false
    default: "true"
  - name: percentiles
    description: Calculate percentiles (comma-separated, e.g., "25,50,75")
    required: false
    default: "25,50,75"
  - name: visualization
    description: Show ASCII visualization
    required: false
    default: "true"
---

# Statistical Analysis Report

{% assign num_array = numbers | split: "," %}
{% assign sorted_array = num_array | sort %}

## Dataset Overview

**Raw Data**: {{ numbers }}
**Count**: {{ num_array | size }} values
**Sorted**: {{ sorted_array | join: ", " }}

## Basic Statistics

{% comment %} Find min and max {% endcomment %}
**Range**: {{ sorted_array | first }} to {{ sorted_array | last }} (span: {{ sorted_array | last | minus: sorted_array.first }})

{% comment %} For demonstration, we'll show the array operations {% endcomment %}
**First Value**: {{ sorted_array | first }}
**Last Value**: {{ sorted_array | last }}
**Array Size**: {{ num_array | size }}

## Array Operations Demo

### Original vs Sorted
- Original: {{ num_array | join: " → " }}
- Sorted: {{ sorted_array | join: " → " }}

### Reverse Order
{{ sorted_array | reverse | join: " ← " }}

### With Index Numbers
{% for num in sorted_array %}
{{ forloop.index }}. Value: {{ num }}
{% endfor %}

## Math Filter Examples

Using the first number ({{ num_array | first }}) as base:
- Plus 10: {{ num_array.first | plus: 10 }}
- Times 2: {{ num_array.first | times: 2 }}
- Divided by 2: {{ num_array.first | divided_by: 2 }}
- Modulo 3: {{ num_array.first | modulo: 3 }}

## Percentile Positions

{% assign percentile_list = percentiles | split: "," %}
{% for p in percentile_list %}
{% assign p_num = p | strip | plus: 0 %}
{% assign pos = num_array.size | times: p_num | divided_by: 100 | round %}
**{{ p_num }}th Percentile Position**: {{ pos }} (value at this position: {{ sorted_array[pos] | default: sorted_array.last }})
{% endfor %}

{% if visualization == "true" %}
## ASCII Visualization

### Values as Bar Chart
{% for num in sorted_array %}
{{ num | prepend: "      " | truncate: 6, "" }}: {% for i in (1..num) %}█{% endfor %}
{% endfor %}
{% endif %}

## Summary

- **Data Points**: {{ num_array | size }}
- **Unique Values**: {{ sorted_array | uniq | size }}
- **First**: {{ sorted_array | first }}
- **Last**: {{ sorted_array | last }}