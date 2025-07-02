---
title: Array Data Processor  
description: Process arrays with flexible filtering and loop control
arguments:
  - name: items
    description: Comma-separated list of items to process
    required: true
  - name: skip_pattern
    description: Pattern to skip items (items containing this will be skipped)
    required: false
    default: ""
  - name: stop_pattern
    description: Pattern to stop processing (processing stops when this is found)
    required: false
    default: ""
  - name: max_items
    description: Maximum number of items to process
    required: false
    default: "100"
  - name: show_skipped
    description: Show skipped items in a separate section
    required: false
    default: "false"
  - name: format
    description: Output format (list, table, json)
    required: false
    default: "list"
---

# Array Processing Report

{% assign item_array = items | split: "," %}
{% assign max_count = max_items | plus: 0 %}

## Processing Items

{% for item in item_array %}
{% assign item_clean = item | strip %}

{% comment %} Check if we should stop processing {% endcomment %}
{% if stop_pattern != "" and item_clean contains stop_pattern %}
### ⛔ Processing stopped at: "{{ item_clean }}"
{% break %}
{% endif %}

{% comment %} Check if we should skip this item {% endcomment %}
{% if skip_pattern != "" and item_clean contains skip_pattern %}
{% continue %}
{% endif %}

{% comment %} Process the item - using forloop.index for numbering {% endcomment %}
{% case format %}
{% when "table" %}
| {{ forloop.index }} | {{ item_clean }} | {{ item_clean | size }} chars |
{% when "json" %}
  {
    "index": {{ forloop.index }},
    "value": "{{ item_clean }}",
    "length": {{ item_clean | size }}
  }{% unless forloop.last %},{% endunless %}
{% else %}
{{ forloop.index }}. {{ item_clean }}
{% endcase %}
{% endfor %}

## Summary

- **Total items provided**: {{ item_array | size }}
- **Items processed**: See list above
- **Processing stopped**: {% if stop_pattern != "" %}Yes (stop pattern found){% else %}No{% endif %}

{% if show_skipped == "true" %}
### Skipped Items
{% for item in item_array %}
{% assign item_clean = item | strip %}
{% if skip_pattern != "" and item_clean contains skip_pattern %}
- {{ item_clean }} (matched pattern: "{{ skip_pattern }}")
{% endif %}
{% endfor %}
{% endif %}

## Processing Rules Applied

{% if skip_pattern != "" %}
✓ Skip pattern: "{{ skip_pattern }}"
{% endif %}
{% if stop_pattern != "" %}
✓ Stop pattern: "{{ stop_pattern }}"
{% endif %}
✓ Maximum items: {{ max_items }}