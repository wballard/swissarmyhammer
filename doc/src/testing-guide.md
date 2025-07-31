# Testing and Debugging Guide

This guide covers testing strategies, debugging techniques, and best practices for working with SwissArmyHammer prompts.

## Interactive Testing

### Basic Testing Workflow

The `test` command provides an interactive environment for testing prompts:

```bash
# Start interactive testing
swissarmyhammer test code-review
```

This will:
1. Load the specified prompt
2. Prompt for required arguments
3. Show optional arguments with defaults
4. Render the template
5. Display the result
6. Offer additional actions (copy, save, retry)

### Testing with Predefined Arguments

```bash
# Test with known arguments
swissarmyhammer test code-review \
  --arg code="fn main() { println!(\"Hello\"); }" \
  --arg language="rust"

# Copy result directly to clipboard
swissarmyhammer test email-template \
  --arg recipient="John" \
  --arg subject="Meeting" \
  --copy
```

## Debugging Template Issues

### Common Template Problems

#### Missing Variables
```liquid
<!-- Problem: undefined variable -->
Hello {{name}}

<!-- Solution: provide default -->
Hello {{ name | default: "Guest" }}
```

#### Type Mismatches
```liquid
<!-- Problem: trying to use string methods on numbers -->
{{ count | upcase }}

<!-- Solution: convert types -->
{{ count | append: " items" }}
```

#### Loop Issues
```liquid
<!-- Problem: not checking for empty arrays -->
{% for item in items %}
  - {{ item }}
{% endfor %}

<!-- Solution: check array exists and has items -->
{% if items and items.size > 0 %}
  {% for item in items %}
    - {{ item }}
  {% endfor %}
{% else %}
  No items found.
{% endif %}
```

### Debug Mode

Use debug mode to see detailed template processing:

```bash
swissarmyhammer test prompt-name --debug
```

Debug output includes:
- Variable resolution steps
- Filter application results
- Conditional evaluation
- Loop iteration details
- Performance timing

## Validation Strategies

### Argument Validation

Test with different argument combinations:

```bash
# Test required arguments only
swissarmyhammer test prompt-name --arg required_arg="value"

# Test with all arguments
swissarmyhammer test prompt-name \
  --arg required_arg="value" \
  --arg optional_arg="optional_value"

# Test with edge cases
swissarmyhammer test prompt-name \
  --arg text="" \
  --arg number="0" \
  --arg array="[]"
```

### Template Edge Cases

Create test cases for common scenarios:

1. **Empty inputs**
2. **Very long inputs**
3. **Special characters**
4. **Unicode content**
5. **Null/undefined values**

### Automated Testing

For prompt libraries, create test scripts:

```bash
#!/bin/bash
# test-all-prompts.sh

PROMPTS=$(swissarmyhammer search --json "" --limit 100 | jq -r '.results[].id')

for prompt in $PROMPTS; do
    echo "Testing $prompt..."
    if swissarmyhammer test "$prompt" --arg placeholder="test" 2>/dev/null; then
        echo "✓ $prompt"
    else
        echo "✗ $prompt"
    fi
done
```

## Performance Testing

### Measuring Render Time

```bash
# Time a complex template
time swissarmyhammer test complex-template \
  --arg large_data="$(cat large-file.json)"

# Use debug mode for detailed timing
swissarmyhammer test template-name --debug | grep "Performance:"
```

### Memory Usage Testing

For large templates or data:

```bash
# Monitor memory usage during rendering
/usr/bin/time -v swissarmyhammer test large-template \
  --arg big_data="$(cat massive-dataset.json)"
```

## Best Practices

### Writing Testable Prompts

1. **Provide sensible defaults** for optional arguments
2. **Handle empty/null inputs** gracefully
3. **Use meaningful argument names**
4. **Include example values** in descriptions
5. **Test with realistic data sizes**

### Testing Workflow

1. **Start simple**: Test with minimal arguments
2. **Add complexity**: Test with full argument sets
3. **Test edge cases**: Empty, null, large inputs
4. **Validate output**: Ensure rendered content makes sense
5. **Performance check**: Verify reasonable render times

### Debugging Tips

1. **Use debug mode** for complex templates
2. **Test filters individually** in simple templates
3. **Validate JSON/YAML** with external tools
4. **Check argument types** match expectations
5. **Use raw mode** to see unprocessed templates

## Integration with Development

### IDE Integration

Many editors support SwissArmyHammer testing:

```bash
# VS Code task example
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Test Current Prompt",
      "type": "shell",
      "command": "swissarmyhammer",
      "args": ["test", "${fileBasenameNoExtension}"],
      "group": "test",
      "presentation": {
        "echo": true,
        "reveal": "always",
        "focus": false,
        "panel": "shared"
      }
    }
  ]
}
```

### Continuous Integration

Add prompt testing to CI pipelines:

```yaml
# .github/workflows/test-prompts.yml
name: Test Prompts
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install SwissArmyHammer
        run: cargo install --git https://github.com/swissarmyhammer/swissarmyhammer.git swissarmyhammer-cli
      - name: Test all prompts
        run: |
          for prompt in prompts/*.md; do
            name=$(basename "$prompt" .md)
            echo "Testing $name..."
            swissarmyhammer test "$name" --arg test="ci_validation"
          done
```

## See Also

- [`test` command](./cli-test.md) - Command reference
- [Template Variables](./template-variables.md) - Template syntax
- [Custom Filters](./custom-filters.md) - Filter reference
