---
name: refactor
title: Code Refactoring
description: Refactor code to match a target pattern or improve structure
arguments:
  - name: code
    description: The code to refactor
    required: true
  - name: target_pattern
    description: The pattern or style to refactor towards
    required: true
---

# Code Refactoring Request

## Original Code:
```
{{code}}
```

## Target Pattern:
{{target_pattern}}

Please refactor the provided code to match the target pattern while:

1. **Preserving Functionality**
   - Ensure all existing functionality remains intact
   - Maintain the same public API if applicable
   - Keep all tests passing

2. **Improving Structure**
   - Apply the target pattern consistently
   - Improve code organization
   - Enhance readability

3. **Following Best Practices**
   - Use appropriate design patterns
   - Apply SOLID principles where relevant
   - Ensure code is testable

4. **Documentation**
   - Update comments to reflect changes
   - Document any new abstractions
   - Explain significant architectural decisions

Please provide:
- The refactored code
- A summary of changes made
- Any potential risks or considerations
- Suggestions for further improvements