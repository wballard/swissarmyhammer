---
title: Dynamic Code Review
description: Language-specific code review with conditional logic
arguments:
  - name: file_path
    description: Path to the file being reviewed
    required: true
  - name: language
    description: Programming language (python, javascript, rust, etc.)
    required: true
  - name: focus_areas
    description: Comma-separated list of areas to focus on
    default: "style,bugs,performance"
  - name: severity_level
    description: Minimum severity level to report (info, warning, error)
    default: "warning"
  - name: include_suggestions
    description: Include code improvement suggestions
    default: "true"
---

# Code Review: {{ file_path }}

{% assign areas = focus_areas | split: "," %}
{% assign show_suggestions = include_suggestions | default: "true" %}

## Review Configuration

- **Language**: {{ language | capitalize }}
- **Focus Areas**: {% for area in areas %}{{ area | strip | capitalize }}{% unless forloop.last %}, {% endunless %}{% endfor %}
- **Severity Level**: {{ severity_level | upcase }}

{% case language %}
  {% when "python" %}
## Python-Specific Checks

Please review this Python code with focus on:
- PEP 8 style compliance
- Type hints usage
- Docstring completeness
- Python idioms and best practices
{% if areas contains "security" %}
- Security: SQL injection, command injection, unsafe pickle usage
{% endif %}

  {% when "javascript" %}
## JavaScript-Specific Checks

Please review this JavaScript code with focus on:
- ESLint rule compliance
- Modern ES6+ syntax usage
- Async/await patterns
- Error handling completeness
{% if areas contains "security" %}
- Security: XSS vulnerabilities, unsafe eval usage
{% endif %}

  {% when "rust" %}
## Rust-Specific Checks

Please review this Rust code with focus on:
- Clippy warnings
- Unsafe block usage justification
- Error handling with Result<T, E>
- Ownership and borrowing patterns
{% if areas contains "security" %}
- Security: Memory safety, unsafe operations
{% endif %}

  {% else %}
## {{ language | capitalize }} Code Review

Please review this {{ language }} code with focus on:
- Language-specific best practices
- Code style and formatting
- Error handling patterns
- Performance considerations
{% endcase %}

## Review Checklist

{% for area in areas %}
{% assign area_clean = area | strip | downcase %}
### {{ area | strip | capitalize }}

{% case area_clean %}
  {% when "style" %}
- [ ] Consistent naming conventions
- [ ] Proper indentation and formatting
- [ ] Clear and concise variable names
- [ ] Appropriate comments

  {% when "bugs" %}
- [ ] Logic errors and edge cases
- [ ] Null/undefined handling
- [ ] Array bounds checking
- [ ] Race conditions

  {% when "performance" %}
- [ ] Algorithm efficiency
- [ ] Database query optimization
- [ ] Caching opportunities
- [ ] Resource cleanup

  {% when "security" %}
- [ ] Input validation
- [ ] Authentication/authorization
- [ ] Sensitive data handling
- [ ] Dependency vulnerabilities

  {% when "testing" %}
- [ ] Test coverage
- [ ] Edge case testing
- [ ] Mock usage
- [ ] Test maintainability

  {% else %}
- [ ] Review {{ area_clean }} aspects
{% endcase %}
{% endfor %}

## Severity Levels

{% if severity_level == "info" %}
Report all findings including:
- ℹ️ **Info**: Minor suggestions and improvements
- ⚠️ **Warning**: Should be addressed but not critical
- ❌ **Error**: Must be fixed before merging
{% elsif severity_level == "warning" %}
Report only warnings and errors:
- ⚠️ **Warning**: Should be addressed but not critical
- ❌ **Error**: Must be fixed before merging
{% else %}
Report only critical errors:
- ❌ **Error**: Must be fixed before merging
{% endif %}

{% if show_suggestions == "true" %}
## Format for Suggestions

For each issue found, please provide:
1. **Location**: Line number and context
2. **Issue**: Clear description of the problem
3. **Suggestion**: Concrete fix or improvement
4. **Example**: Code snippet showing the fix

```{{ language }}
// Example format
// Line 42: Inefficient loop
// Suggestion: Use map() instead of forEach() with push()
// Before:
const result = [];
items.forEach(item => result.push(item * 2));

// After:
const result = items.map(item => item * 2);
```
{% endif %}

Please analyze the code and provide a comprehensive review based on the above criteria.