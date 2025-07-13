---
title: Code Review Assistant
description: Comprehensive code review with focus on best practices, security, and performance
arguments:
  - name: code
    description: The code to review (can be a function, class, or entire file)
    required: true
  - name: language
    description: Programming language (helps with language-specific advice)
    required: false
    default: "auto-detect"
  - name: focus
    description: Areas to focus on (security, performance, readability, etc.)
    required: false
    default: "general best practices"
---

# Code Review

I need a thorough code review for the following {{language}} code.

## Code to Review

```{{language}}
{{code}}
```

## Review Focus

Please focus on: {{focus}}

## Review Criteria

Please analyze the code for:

### 🔒 Security
- Potential security vulnerabilities
- Input validation issues
- Authentication/authorization concerns

### 🚀 Performance
- Inefficient algorithms or operations
- Memory usage concerns
- Potential bottlenecks

### 📖 Readability & Maintainability
- Code clarity and organization
- Naming conventions
- Documentation needs

### 🧪 Testing & Reliability
- Error handling
- Edge cases
- Testability

### 🏗️ Architecture & Design
- SOLID principles adherence
- Design patterns usage
- Code structure

## Output Format

Please provide:

1. **Overall Assessment** - Brief summary of code quality
2. **Specific Issues** - List each issue with:
   - Severity (High/Medium/Low)
   - Location (line numbers if applicable)
   - Explanation of the problem
   - Suggested fix
3. **Positive Aspects** - What's done well
4. **Recommendations** - Broader suggestions for improvement

Focus especially on {{focus}} in your analysis.