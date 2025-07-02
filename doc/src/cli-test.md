# test - Interactive Prompt Testing

The `test` command allows you to test prompts interactively, providing argument values and seeing the rendered output before using them with AI models.

## Synopsis

```bash
swissarmyhammer test [OPTIONS] <PROMPT_ID>
```

## Description

Test prompts interactively by providing arguments and viewing the rendered output. This is essential for debugging template issues, validating arguments, and refining prompts before deployment.

## Arguments

- `PROMPT_ID` - The ID of the prompt to test (required)

## Options

### Argument Specification
- `--arg KEY=VALUE` - Provide argument values directly (can be used multiple times)

### Output Control
- `--raw` - Show raw template without rendering
- `--copy` - Copy rendered result to clipboard
- `--save FILE` - Save rendered result to file
- `--debug` - Show detailed debug information including variable resolution

## Interactive Mode

When no `--arg` options are provided, the command enters interactive mode:

1. **Prompt Selection**: If prompt ID is ambiguous, presents a fuzzy selector
2. **Argument Collection**: Prompts for each required and optional argument
3. **Template Rendering**: Shows the rendered output
4. **Actions**: Offers to copy to clipboard or save to file

## Examples

### Interactive Testing
```bash
# Test a prompt interactively
swissarmyhammer test code-review

# The command will prompt for arguments:
# ? Enter value for 'code' (required): fn main() { println!("Hello"); }
# ? Enter value for 'language' (optional, default: auto-detect): rust
# 
# [Rendered output shows here]
# 
# ? What would you like to do?
#   > View output
#     Copy to clipboard
#     Save to file
#     Test with different arguments
#     Exit
```

### Non-Interactive Testing
```bash
# Test with predefined arguments
swissarmyhammer test code-review \
  --arg code="fn main() { println!(\"Hello\"); }" \
  --arg language="rust"

# Test and copy to clipboard
swissarmyhammer test debug-helper \
  --arg error="compiler error" \
  --copy

# Test and save output
swissarmyhammer test api-docs \
  --arg code="$(cat src/api.rs)" \
  --save generated-docs.md
```

### Debug Mode
```bash
# Show debug information
swissarmyhammer test template-complex --debug

# Output includes:
# Variables resolved:
#   user_input: "example text"
#   timestamp: "2024-01-15T10:30:00Z"
#   
# Template processing:
#   Line 5: Variable 'user_input' resolved to "example text"
#   Line 12: Filter 'capitalize' applied
#   Line 18: Conditional block evaluated to true
#
# Final output:
# [rendered template]
```

### Raw Template View
```bash
# View the raw template without rendering
swissarmyhammer test email-template --raw

# Shows:
# ---
# title: Email Template
# arguments:
#   - name: recipient
#     required: true
# ---
# 
# Dear {{recipient | capitalize}},
# 
# {% if urgent %}
# **URGENT:** 
# {% endif %}
# {{message}}
```

## Output Format

### Default Output
```
Testing prompt: code-review

Arguments:
  code: "fn main() { println!(\"Hello\"); }"
  language: "rust" (default: auto-detect)

Rendered Output:
┌─────────────────────────────────────────────────────────────┐
│ # Code Review                                               │
│                                                             │
│ Please review the following rust code:                     │
│                                                             │
│ ```rust                                                     │
│ fn main() { println!("Hello"); }                           │
│ ```                                                         │
│                                                             │
│ Focus on:                                                   │
│ - Code quality and readability                             │
│ - Potential bugs or security issues                        │
│ - Performance considerations                                │
│ - Best practices adherence                                  │
└─────────────────────────────────────────────────────────────┘

✓ Template rendered successfully (247 characters)
```

### Debug Output
```
Testing prompt: code-review (debug mode)

Prompt loaded from: ~/.swissarmyhammer/prompts/review/code.md
Arguments defined: 2 (1 required, 1 optional)

Argument Resolution:
✓ code: "fn main() { println!(\"Hello\"); }" [user provided]
✓ language: "rust" [user provided, overrides default "auto-detect"]

Template Processing:
→ Line 8: Variable 'language' resolved and capitalized
→ Line 12-14: Code block with 'code' variable substitution
→ Line 16-20: Static bullet list rendered

Filters Applied:
- capitalize: "rust" → "Rust"

Rendered Output:
[... same as above ...]

Performance:
- Template parsing: 2ms
- Variable resolution: 1ms
- Rendering: 3ms
- Total: 6ms
```

## Error Handling

The test command provides helpful error messages for common issues:

### Missing Arguments
```bash
$ swissarmyhammer test code-review
Error: Missing required argument 'code'

Available arguments:
  code (required) - The code to review
  language (optional) - Programming language (default: auto-detect)

Use --arg KEY=VALUE to provide arguments, or run without --arg for interactive mode.
```

### Template Errors
```bash
$ swissarmyhammer test broken-template --arg data="test"
Error: Template rendering failed at line 15

  13 | {% for item in items %}
  14 |   - {{item.name}}
> 15 |   - {{item.invalid_field | unknown_filter}}
     |                           ^^^^^^^^^^^^^^
  16 | {% endfor %}

Unknown filter: unknown_filter
Available filters: capitalize, lower, upper, truncate, ...

Fix the template and try again.
```

## Integration with Development Workflow

### Testing Before Deployment
```bash
# Test a prompt before adding to Claude Code
swissarmyhammer test new-prompt --debug

# Validate all prompts in a directory
for prompt in $(ls prompts/*.md); do
  swissarmyhammer test "${prompt%.md}" --arg placeholder="test"
done
```

### Clipboard Integration
```bash
# Test and copy for immediate use
swissarmyhammer test quick-note \
  --arg content="Meeting notes" \
  --copy

# Now paste into your editor or Claude Code
```

### Script Integration
```bash
#!/bin/bash
# test-and-deploy.sh

PROMPT_ID="$1"
if swissarmyhammer test "$PROMPT_ID" --arg test="validation"; then
  echo "✓ Prompt test passed, deploying..."
  swissarmyhammer export "$PROMPT_ID" --format directory deployment/
else
  echo "✗ Prompt test failed, fix issues before deploying"
  exit 1
fi
```

## See Also

- [`search`](./cli-search.md) - Find prompts to test
- [Template Variables](./template-variables.md) - Template syntax reference
- [Testing Guide](./testing-guide.md) - Advanced testing strategies
- [Custom Filters](./custom-filters.md) - Available template filters