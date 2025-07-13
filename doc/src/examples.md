# Examples

This page provides real-world examples of using SwissArmyHammer for various development tasks.

## Basic Prompt Usage

### Simple Code Review

```bash
# Review a Python file
swissarmyhammer test review/code --file_path "src/main.py"

# Review with specific focus
swissarmyhammer test review/code --file_path "api/auth.py" --context "focus on security and error handling"
```

### Generate Unit Tests

```bash
# Generate tests for a function
swissarmyhammer test test/unit --code "$(cat calculator.py)" --framework "pytest"

# Generate tests with high coverage target
swissarmyhammer test test/unit --code "$(cat utils.js)" --framework "jest" --coverage_target "95"
```

### Debug an Error

```bash
# Analyze an error message
swissarmyhammer test debug/error \
  --error_message "TypeError: Cannot read property 'name' of undefined" \
  --language "javascript" \
  --context "Happens when user submits form"
```

## Creating Custom Prompts

### Basic Prompt Structure

Create `~/.swissarmyhammer/prompts/my-prompt.md`:

```markdown
{{#include ../examples/prompts/git-commit-message.md}}
```

Use it:

```bash
swissarmyhammer test git-commit-message \
  --changes "Added user authentication with OAuth2" \
  --type "feat" \
  --scope "auth"
```

### Advanced Template with Conditionals

Create `~/.swissarmyhammer/prompts/database-query.md`:

```markdown
{{#include ../examples/prompts/database-query-optimizer.md}}
```

### Using Arrays and Loops

Create `~/.swissarmyhammer/prompts/api-client.md`:

```markdown
{{#include ../examples/prompts/api-client-generator.md}}
```

## Complex Workflows

### Multi-Step Code Analysis

```bash
{{#include ../examples/scripts/analyze-codebase.sh}}
```

### Automated PR Review

```bash
{{#include ../examples/scripts/pr-review.sh}}
```

### Project Setup Automation

```bash
{{#include ../examples/scripts/setup-project.sh}}
```

## Integration Examples

### Git Hooks

`.git/hooks/pre-commit`:

```bash
{{#include ../examples/scripts/pre-commit}}
```

### CI/CD Integration

`.github/workflows/code-quality.yml`:

```yaml
{{#include ../examples/configs/github-workflow.yml}}
```

### VS Code Task

`.vscode/tasks.json`:

```json
{{#include ../examples/configs/vscode-tasks.json}}
```

## Advanced Patterns

### Dynamic Prompt Selection

```bash
{{#include ../examples/scripts/smart-review.sh}}
```

### Batch Processing

```python
{{#include ../examples/scripts/batch_analyze.py}}
```

### Custom Filter Integration

Create a prompt that uses custom filters:

```markdown
{{#include ../examples/prompts/data-transformer.md}}
```

## Tips and Best Practices

### 1. Use Command Substitution

```bash
# Good - passes file content directly
swissarmyhammer test review/code --code "$(cat main.py)"

# Less efficient - requires file path handling
swissarmyhammer test review/code --file_path main.py
```

### 2. Chain Commands

```bash
# Review then test
swissarmyhammer test review/code --file_path app.py && \
swissarmyhammer test test/unit --code "$(cat app.py)"
```

### 3. Save Common Workflows

Create `~/.swissarmyhammer/scripts/full-review.sh`:

```bash
{{#include ../examples/scripts/full-review.sh}}
```

### 4. Use Environment Variables

```bash
export SAH_DEFAULT_LANGUAGE=python
export SAH_DEFAULT_FRAMEWORK=pytest

# Now these defaults apply
swissarmyhammer test test/unit --code "$(cat app.py)"
```

### 5. Create Project Templates

Store in `~/.swissarmyhammer/templates/`:

```bash
# Create new project with templates
cp -r ~/.swissarmyhammer/templates/webapp-template my-new-app
cd my-new-app
swissarmyhammer test docs/readme \
  --project_name "my-new-app" \
  --project_description "My awesome web app"
```

## Next Steps

- Explore [Built-in Prompts](./builtin-prompts.md) for more capabilities
- Learn about [Creating Prompts](./creating-prompts.md) for custom workflows
- Check [CLI Reference](./cli-reference.md) for all available commands
- See [Library Usage](./library-usage.md) for programmatic integration