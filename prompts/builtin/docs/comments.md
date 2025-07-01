---
name: docs-comments
title: Generate Code Comments
description: Add comprehensive comments and documentation to code
arguments:
  - name: code
    description: The code to document
    required: true
  - name: comment_style
    description: Comment style (inline, block, jsdoc, docstring, rustdoc)
    required: false
    default: "auto-detect"
  - name: detail_level
    description: Level of detail (minimal, standard, comprehensive)
    required: false
    default: "standard"
  - name: audience
    description: Target audience for the comments
    required: false
    default: "developers"
---

# Code Documentation: {{comment_style}}

## Code to Document
```
{{{code}}}
```

## Documentation Parameters
- **Style**: {{comment_style}}
- **Detail Level**: {{detail_level}}
- **Audience**: {{audience}}

## Documentation Strategy

### 1. Comment Types

#### File/Module Level
- Purpose and responsibility
- Author and maintenance info
- Dependencies and requirements
- Usage examples

#### Class/Interface Level
- Design decisions
- Invariants and contracts
- Relationships to other components
- Thread safety considerations

#### Method/Function Level
- Purpose and behavior
- Parameters and return values
- Side effects and exceptions
- Usage examples
- Complexity notes

#### Implementation Comments
- Non-obvious logic explanation
- Algorithm choices
- Performance considerations
- Bug workarounds

### 2. Documentation Standards

{{#if (eq comment_style "jsdoc")}}
#### JSDoc Format
```javascript
/**
 * Brief description of the function.
 * 
 * @param {Type} paramName - Parameter description
 * @returns {Type} Return value description
 * @throws {ErrorType} When this error occurs
 * @example
 * // Example usage
 * functionName(args);
 */
```
{{else if (eq comment_style "docstring")}}
#### Python Docstring Format
```python
"""Brief description of the function.

Longer description if needed.

Args:
    param_name (Type): Parameter description
    
Returns:
    Type: Return value description
    
Raises:
    ErrorType: When this error occurs
    
Examples:
    >>> function_name(args)
    expected_output
"""
```
{{else if (eq comment_style "rustdoc")}}
#### Rust Documentation Format
```rust
/// Brief description of the function.
/// 
/// Longer description if needed.
/// 
/// # Arguments
/// 
/// * `param_name` - Parameter description
/// 
/// # Returns
/// 
/// Return value description
/// 
/// # Examples
/// 
/// ```
/// let result = function_name(args);
/// ```
```
{{/if}}

### 3. Best Practices

#### What to Document
- Public APIs thoroughly
- Complex algorithms
- Non-obvious decisions
- Workarounds and hacks
- Performance considerations

#### What NOT to Document
- Obvious code
- Language features
- Self-documenting code
- Redundant information

#### Writing Style
- Clear and concise
- Active voice
- Present tense
- Consistent terminology

### 4. {{detail_level}} Level Documentation

{{#if (eq detail_level "minimal")}}
Focus on:
- Public API documentation
- Critical warnings
- Non-obvious behavior
{{else if (eq detail_level "comprehensive")}}
Include:
- Detailed parameter descriptions
- Multiple examples
- Edge cases
- Performance notes
- Related references
{{else}}
Balance between clarity and completeness:
- Clear purpose statements
- Parameter/return documentation
- Key examples
- Important notes
{{/if}}

### 5. Generated Documentation
Provide the code with appropriate comments added according to the specified style and detail level.