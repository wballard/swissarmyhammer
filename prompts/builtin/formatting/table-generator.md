---
title: Table Generator with Alternating Rows
description: Generate formatted tables with alternating row styles
arguments:
  - name: headers
    description: Comma-separated list of table headers
    required: true
  - name: rows
    description: Semicolon-separated rows, with comma-separated values
    required: true
  - name: style
    description: Table style (markdown, html, ascii)
    default: "markdown"
  - name: zebra
    description: Enable zebra striping for rows
    default: "true"
  - name: row_numbers
    description: Add row numbers
    default: "false"
---

{% assign header_list = headers | split: "," %}
{% assign row_list = rows | split: ";" %}

{% case style %}
{% when "markdown" %}
# Table Output

{% if row_numbers == "true" %}| # {% endif %}{% for header in header_list %}| {{ header | strip }} {% endfor %}|
{% if row_numbers == "true" %}|---{% endif %}{% for header in header_list %}|---{% endfor %}|
{% for row in row_list %}
{% assign cells = row | split: "," %}
{% if row_numbers == "true" %}| {{ forloop.index }} {% endif %}{% for cell in cells %}| {{ cell | strip }} {% endfor %}|
{% endfor %}

{% when "html" %}
<table>
  <thead>
    <tr>
      {% if row_numbers == "true" %}<th>#</th>{% endif %}
      {% for header in header_list %}
      <th>{{ header | strip }}</th>
      {% endfor %}
    </tr>
  </thead>
  <tbody>
    {% for row in row_list %}
    {% assign cells = row | split: "," %}
    <tr class="{% if zebra == "true" %}{% cycle 'even', 'odd' %}{% else %}normal{% endif %}">
      {% if row_numbers == "true" %}<td>{{ forloop.index }}</td>{% endif %}
      {% for cell in cells %}
      <td>{{ cell | strip }}</td>
      {% endfor %}
    </tr>
    {% endfor %}
  </tbody>
</table>

{% if zebra == "true" %}
<style>
.even { background-color: #f2f2f2; }
.odd { background-color: #ffffff; }
</style>
{% endif %}

{% when "ascii" %}
{% comment %} Calculate column widths {% endcomment %}
{% assign max_widths = "" %}
{% for header in header_list %}
{% assign max_widths = max_widths | append: header.size | append: "," %}
{% endfor %}

{% comment %} ASCII art table with cycle for row separators {% endcomment %}
{% for header in header_list %}{{ header | strip | append: "    " | truncate: 15, "" }}{% endfor %}
{% for header in header_list %}{% cycle '--------------', '==============' %}{% endfor %}
{% for row in row_list %}
{% assign cells = row | split: "," %}
{% assign row_marker = "" %}
{% if zebra == "true" %}
{% assign row_marker = "" %}
{% endif %}
{{ row_marker }}{% for cell in cells %}{{ cell | strip | append: "    " | truncate: 15, "" }}{% endfor %}
{% endfor %}
{% endcase %}

## Table Statistics

- **Headers**: {{ header_list | size }}
- **Rows**: {{ row_list | size }}
- **Total cells**: {{ row_list | size | times: header_list.size }}

{% comment %} Demonstrate nested cycles {% endcomment %}
## Cell Analysis

{% for row in row_list %}
{% assign cells = row | split: "," %}
### Row {{ forloop.index }} - {% cycle 'Primary', 'Secondary', 'Tertiary' %} Type
{% for cell in cells %}
- Cell {{ forloop.index }}: "{{ cell | strip }}" {% cycle 'group1': '[Important]', '[Normal]' %} {% cycle 'group2': '🟢', '🔵', '🟡' %}
{% endfor %}
{% endfor %}

## Legend

{% if zebra == "true" %}
- Even rows: Light background
- Odd rows: White background
{% endif %}

Row Types:
- Primary: First type in rotation
- Secondary: Second type in rotation  
- Tertiary: Third type in rotation

Cell Indicators:
- 🟢 Green: First in cycle
- 🔵 Blue: Second in cycle
- 🟡 Yellow: Third in cycle