---
name: refactor-clean
title: Clean Up Code
description: Refactor code for better readability, maintainability, and adherence to best practices
arguments:
  - name: code
    description: The code to refactor
    required: true
  - name: language
    description: Programming language
    required: false
    default: "auto-detect"
  - name: focus_areas
    description: Specific areas to focus on (e.g., "naming, complexity, duplication")
    required: false
    default: "all"
  - name: style_guide
    description: Specific style guide to follow
    required: false
    default: "language defaults"
---

# Code Refactoring: Clean Code Principles

## Original Code
```{{language}}
{{ code }}
```

## Refactoring Focus
- **Language**: {{language}}
- **Focus Areas**: {{focus_areas}}
- **Style Guide**: {{style_guide}}

## Refactoring Analysis

### 1. Code Smells Identified
- Long methods/functions
- Poor naming conventions
- Code duplication
- Complex conditionals
- Magic numbers/strings
- Tight coupling
- Missing abstractions

### 2. Refactoring Strategies

#### Readability Improvements
- Use descriptive variable and function names
- Extract complex expressions into well-named variables
- Add appropriate whitespace and formatting
- Group related functionality

#### Structural Improvements
- Extract methods for reusable code blocks
- Apply DRY (Don't Repeat Yourself) principle
- Simplify complex conditionals
- Introduce appropriate design patterns

#### Maintainability Enhancements
- Add error handling where appropriate
- Include necessary comments for complex logic
- Ensure single responsibility principle
- Make dependencies explicit

### 3. Refactored Code
Provide the cleaned-up version with:
- Clear naming
- Proper structure
- Reduced complexity
- Better organization
- Consistent style

### 4. Explanation of Changes
Document what was changed and why, focusing on:
- Improved readability
- Better performance (if applicable)
- Enhanced maintainability
- Reduced technical debt