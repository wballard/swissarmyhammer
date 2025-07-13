---
name: project-code-review
title: Project Code Review
description: Review code according to our project standards
arguments:
  - name: file_path
    description: File to review
    required: true
---

Review {{file_path}} for:
- Our naming conventions (camelCase for JS, snake_case for Python)
- Error handling patterns we use
- Project-specific security requirements
- Performance considerations for our scale