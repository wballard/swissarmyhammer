---
name: git-commit-message
title: Git Commit Message Generator
description: Generate conventional commit messages from changes
arguments:
  - name: changes
    description: Description of changes made
    required: true
  - name: type
    description: Type of change (feat, fix, docs, etc.)
    required: false
    default: feat
  - name: scope
    description: Scope of the change
    required: false
    default: ""
---

# Git Commit Message

Based on the changes: {{changes}}

Generate a conventional commit message:

Type: {{type}}
{% if scope %}Scope: {{scope}}{% endif %}

Format: `{{type}}{% if scope %}({{scope}}){% endif %}: <subject>`

Subject should be:
- 50 characters or less
- Present tense
- No period at the end
- Clear and descriptive