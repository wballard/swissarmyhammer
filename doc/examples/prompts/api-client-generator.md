---
name: api-client-generator
title: API Client Generator
description: Generate API client code from endpoint specifications
arguments:
  - name: endpoints
    description: Comma-separated list of endpoints (method:path)
    required: true
  - name: base_url
    description: Base URL for the API
    required: true
  - name: language
    description: Target language for the client
    required: false
    default: javascript
  - name: auth_type
    description: Authentication type (none, bearer, basic, apikey)
    required: false
    default: none
---

# API Client Generator

Generate a {{language}} API client for:
- Base URL: {{base_url}}
- Authentication: {{auth_type}}

## Endpoints
{% assign endpoint_list = endpoints | split: "," %}
{% for endpoint in endpoint_list %}
  {% assign parts = endpoint | split: ":" %}
  {% assign method = parts[0] | strip | upcase %}
  {% assign path = parts[1] | strip %}
- {{method}} {{path}}
{% endfor %}

{% if language == "javascript" %}
Generate a modern JavaScript client using:
- Fetch API for requests
- Async/await syntax
- Proper error handling
- TypeScript interfaces if applicable
{% elsif language == "python" %}
Generate a Python client using:
- requests library
- Type hints
- Proper exception handling
- Docstrings for all methods
{% endif %}

{% if auth_type != "none" %}
Include authentication handling for {{auth_type}}:
{% case auth_type %}
{% when "bearer" %}
- Accept token in constructor
- Add Authorization: Bearer header
{% when "basic" %}
- Accept username/password
- Encode credentials properly
{% when "apikey" %}
- Accept API key
- Add to headers or query params as needed
{% endcase %}
{% endif %}

Include:
1. Complete client class
2. Error handling
3. Usage examples
4. Any necessary types/interfaces