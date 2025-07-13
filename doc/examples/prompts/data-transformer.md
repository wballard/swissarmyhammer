---
name: data-transformer
title: Data Transformation Pipeline
description: Transform data using custom filters
arguments:
  - name: data
    description: Input data (JSON or CSV)
    required: true
  - name: transformations
    description: Comma-separated list of transformations
    required: true
---

# Data Transformation

Input data:
```
{{data}}
```

Apply transformations: {{transformations}}

{% assign transform_list = transformations | split: "," %}
{% for transform in transform_list %}
  {% case transform | strip %}
  {% when "uppercase" %}
    - Convert all text fields to uppercase
  {% when "normalize" %}
    - Normalize whitespace and formatting
  {% when "validate" %}
    - Validate data types and constraints
  {% when "aggregate" %}
    - Aggregate numeric fields
  {% endcase %}
{% endfor %}

Provide:
1. Transformed data
2. Transformation log
3. Any validation errors
4. Summary statistics