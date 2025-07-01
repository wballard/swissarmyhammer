---
name: code-review
title: Code Review
description: Review code for quality, bugs, and improvements
arguments:
  - name: file_path
    description: Path to the file being reviewed
    required: true
  - name: context
    description: Additional context about the code review focus
    required: false
    default: "general review"
---

# Code Review for {{file_path}}

Please review the following code file with a focus on: {{context}}

## Review Checklist:

1. **Code Quality**
   - Is the code readable and well-structured?
   - Are variable and function names descriptive?
   - Is the code properly documented?

2. **Potential Bugs**
   - Are there any logic errors?
   - Are edge cases handled?
   - Is error handling appropriate?

3. **Performance**
   - Are there any performance bottlenecks?
   - Can any algorithms be optimized?
   - Is memory usage efficient?

4. **Security**
   - Are there any security vulnerabilities?
   - Is input validation proper?
   - Are secrets handled correctly?

5. **Best Practices**
   - Does the code follow language idioms?
   - Is the code DRY (Don't Repeat Yourself)?
   - Are design patterns used appropriately?

Please provide specific feedback for each area of concern, including line numbers where applicable.