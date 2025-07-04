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
---
name: database-query-optimizer
title: Database Query Optimizer
description: Optimize SQL queries for better performance
arguments:
  - name: query
    description: The SQL query to optimize
    required: true
  - name: database
    description: Database type (postgres, mysql, sqlite)
    required: false
    default: postgres
  - name: table_sizes
    description: Approximate table sizes (small, medium, large)
    required: false
    default: medium
  - name: indexes
    description: Available indexes (comma-separated)
    required: false
    default: ""
---

# SQL Query Optimization

## Original Query
```sql
{{query}}
```

## Database: {{database | capitalize}}

{% if database == "postgres" %}
### PostgreSQL Specific Optimizations
- Consider using EXPLAIN ANALYZE
- Check for missing indexes on JOIN columns
- Use CTEs for complex queries
- Consider partial indexes for WHERE conditions
{% elsif database == "mysql" %}
### MySQL Specific Optimizations
- Use EXPLAIN to check execution plan
- Consider covering indexes
- Optimize GROUP BY queries
- Check buffer pool size
{% else %}
### SQLite Specific Optimizations
- Use EXPLAIN QUERY PLAN
- Consider table order in JOINs
- Minimize use of LIKE with wildcards
{% endif %}

## Table Size Considerations
{% case table_sizes %}
{% when "small" %}
- Full table scans might be acceptable
- Focus on query simplicity
{% when "large" %}
- Indexes are critical
- Consider partitioning
- Avoid SELECT *
{% else %}
- Balance between indexes and write performance
- Monitor query execution time
{% endcase %}

{% if indexes %}
## Available Indexes
{% assign index_list = indexes | split: "," %}
{% for index in index_list %}
- {{ index | strip }}
{% endfor %}
{% endif %}

Provide:
1. Optimized query
2. Explanation of changes
3. Expected performance improvement
4. Additional index recommendations
```

### Using Arrays and Loops

Create `~/.swissarmyhammer/prompts/api-client.md`:

```markdown
---
name: api-client-generator
title: API Client Generator
description: Generate API client code from endpoint specifications
arguments:
  - name: endpoints
    description: Comma-separated list of endpoints (method:path)
    required: true
  - name: base_url
    description: Base URL for the API
    required: true
  - name: language
    description: Target language for the client
    required: false
    default: javascript
  - name: auth_type
    description: Authentication type (none, bearer, basic, apikey)
    required: false
    default: none
---

# API Client Generator

Generate a {{language}} API client for:
- Base URL: {{base_url}}
- Authentication: {{auth_type}}

## Endpoints
{% assign endpoint_list = endpoints | split: "," %}
{% for endpoint in endpoint_list %}
  {% assign parts = endpoint | split: ":" %}
  {% assign method = parts[0] | strip | upcase %}
  {% assign path = parts[1] | strip %}
- {{method}} {{path}}
{% endfor %}

{% if language == "javascript" %}
Generate a modern JavaScript client using:
- Fetch API for requests
- Async/await syntax
- Proper error handling
- TypeScript interfaces if applicable
{% elsif language == "python" %}
Generate a Python client using:
- requests library
- Type hints
- Proper exception handling
- Docstrings for all methods
{% endif %}

{% if auth_type != "none" %}
Include authentication handling for {{auth_type}}:
{% case auth_type %}
{% when "bearer" %}
- Accept token in constructor
- Add Authorization: Bearer header
{% when "basic" %}
- Accept username/password
- Encode credentials properly
{% when "apikey" %}
- Accept API key
- Add to headers or query params as needed
{% endcase %}
{% endif %}

Include:
1. Complete client class
2. Error handling
3. Usage examples
4. Any necessary types/interfaces
```

## Complex Workflows

### Multi-Step Code Analysis

```bash
#!/bin/bash
# analyze-codebase.sh

# Step 1: Get overview of the codebase
echo "=== Codebase Overview ==="
swissarmyhammer test help --topic "codebase structure" --detail_level "detailed" > analysis/overview.md

# Step 2: Review critical files
echo "=== Security Review ==="
for file in auth.py payment.py user.py; do
  echo "Reviewing $file..."
  swissarmyhammer test review/security \
    --code "$(cat src/$file)" \
    --context "handles sensitive data" \
    --severity_threshold "medium" > "analysis/security-$file.md"
done

# Step 3: Generate tests for uncovered code
echo "=== Test Generation ==="
swissarmyhammer test test/unit \
  --code "$(cat src/utils.py)" \
  --framework "pytest" \
  --style "BDD" \
  --coverage_target "90" > tests/test_utils_generated.py

# Step 4: Create documentation
echo "=== Documentation ==="
swissarmyhammer test docs/api \
  --code "$(cat src/api.py)" \
  --api_type "REST" \
  --format "openapi" > docs/api-spec.yaml

echo "Analysis complete! Check the analysis/ directory for results."
```

### Automated PR Review

```bash
#!/bin/bash
# pr-review.sh

# Get changed files
CHANGED_FILES=$(git diff --name-only main...HEAD)

echo "# Pull Request Review" > pr-review.md
echo "" >> pr-review.md

for file in $CHANGED_FILES; do
  if [[ $file == *.py ]] || [[ $file == *.js ]] || [[ $file == *.ts ]]; then
    echo "## Review: $file" >> pr-review.md
    
    # Dynamic code review
    swissarmyhammer test review/code-dynamic \
      --file_path "$file" \
      --language "${file##*.}" \
      --focus_areas "bugs,security,performance" \
      --severity_level "info" >> pr-review.md
    
    echo "" >> pr-review.md
  fi
done

# Check for accessibility issues in UI files
for file in $CHANGED_FILES; do
  if [[ $file == *.html ]] || [[ $file == *.jsx ]] || [[ $file == *.tsx ]]; then
    echo "## Accessibility: $file" >> pr-review.md
    swissarmyhammer test review/accessibility \
      --code "$(cat $file)" \
      --wcag_level "AA" >> pr-review.md
    echo "" >> pr-review.md
  fi
done

echo "Review complete! See pr-review.md"
```

### Project Setup Automation

```bash
#!/bin/bash
# setup-project.sh

PROJECT_NAME=$1
PROJECT_TYPE=$2  # api, webapp, library

# Create project structure
mkdir -p $PROJECT_NAME/{src,tests,docs}
cd $PROJECT_NAME

# Generate README
swissarmyhammer test docs/readme \
  --project_name "$PROJECT_NAME" \
  --project_description "A $PROJECT_TYPE project" \
  --language "$PROJECT_TYPE" > README.md

# Create initial prompts
mkdir -p prompts/project

# Generate project-specific code review prompt
cat > prompts/project/code-review.md << 'EOF'
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
EOF

# Configure SwissArmyHammer for this project
claude mcp add ${PROJECT_NAME}_sah swissarmyhammer serve --prompts ./prompts

echo "Project $PROJECT_NAME setup complete!"
```

## Integration Examples

### Git Hooks

`.git/hooks/pre-commit`:

```bash
#!/bin/bash
# Check code quality before commit

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(py|js|ts)$')

if [ -z "$STAGED_FILES" ]; then
  exit 0
fi

echo "Running pre-commit checks..."

for FILE in $STAGED_FILES; do
  # Run security review on staged content
  git show ":$FILE" | swissarmyhammer test review/security \
    --code "$(cat)" \
    --severity_threshold "high" \
    --language "${FILE##*.}"
  
  if [ $? -ne 0 ]; then
    echo "Security issues found in $FILE"
    exit 1
  fi
done

echo "Pre-commit checks passed!"
```

### CI/CD Integration

`.github/workflows/code-quality.yml`:

```yaml
name: Code Quality

on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install SwissArmyHammer
        run: |
          curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | bash
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Run Code Reviews
        run: |
          for file in $(find src -name "*.py"); do
            swissarmyhammer test review/code-dynamic \
              --file_path "$file" \
              --language "python" \
              --focus_areas "bugs,security" \
              --severity_level "warning"
          done
      
      - name: Generate Missing Tests
        run: |
          swissarmyhammer test test/unit \
            --code "$(cat src/core.py)" \
            --framework "pytest" \
            --coverage_target "80" > tests/test_core_generated.py
      
      - name: Update Documentation
        run: |
          swissarmyhammer test docs/api \
            --code "$(cat src/api.py)" \
            --api_type "REST" \
            --format "markdown" > docs/api.md
```

### VS Code Task

`.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Review Current File",
      "type": "shell",
      "command": "swissarmyhammer",
      "args": [
        "test",
        "review/code",
        "--file_path",
        "${file}"
      ],
      "group": {
        "kind": "test",
        "isDefault": true
      },
      "presentation": {
        "reveal": "always",
        "panel": "new"
      }
    },
    {
      "label": "Generate Tests",
      "type": "shell",
      "command": "swissarmyhammer",
      "args": [
        "test",
        "test/unit",
        "--code",
        "$(cat ${file})",
        "--framework",
        "auto-detect"
      ],
      "group": "test"
    }
  ]
}
```

## Advanced Patterns

### Dynamic Prompt Selection

```bash
#!/bin/bash
# smart-review.sh

FILE=$1
EXTENSION="${FILE##*.}"

case $EXTENSION in
  py)
    PROMPT="review/code-dynamic"
    ARGS="--language python --focus_areas style,typing"
    ;;
  js|ts)
    PROMPT="review/code-dynamic"
    ARGS="--language javascript --focus_areas async,security"
    ;;
  html)
    PROMPT="review/accessibility"
    ARGS="--wcag_level AA"
    ;;
  sql)
    PROMPT="database-query-optimizer"
    ARGS="--database postgres"
    ;;
  *)
    PROMPT="review/code"
    ARGS=""
    ;;
esac

swissarmyhammer test $PROMPT --file_path "$FILE" $ARGS
```

### Batch Processing

```python
#!/usr/bin/env python3
# batch_analyze.py

import subprocess
import json
import glob

def analyze_file(filepath):
    """Run SwissArmyHammer analysis on a file."""
    result = subprocess.run([
        'swissarmyhammer', 'test', 'review/code',
        '--file_path', filepath,
        '--context', 'batch analysis'
    ], capture_output=True, text=True)
    
    return {
        'file': filepath,
        'output': result.stdout,
        'errors': result.stderr
    }

# Analyze all Python files
files = glob.glob('**/*.py', recursive=True)
results = [analyze_file(f) for f in files]

# Save results
with open('analysis_results.json', 'w') as f:
    json.dump(results, f, indent=2)

print(f"Analyzed {len(files)} files. Results saved to analysis_results.json")
```

### Custom Filter Integration

Create a prompt that uses custom filters:

```markdown
---
name: data-transformer
title: Data Transformation Pipeline
description: Transform data using custom filters
arguments:
  - name: data
    description: Input data (JSON or CSV)
    required: true
  - name: transformations
    description: Comma-separated list of transformations
    required: true
---

# Data Transformation

Input data:
```
{{data}}
```

Apply transformations: {{transformations}}

{% assign transform_list = transformations | split: "," %}
{% for transform in transform_list %}
  {% case transform | strip %}
  {% when "uppercase" %}
    - Convert all text fields to uppercase
  {% when "normalize" %}
    - Normalize whitespace and formatting
  {% when "validate" %}
    - Validate data types and constraints
  {% when "aggregate" %}
    - Aggregate numeric fields
  {% endcase %}
{% endfor %}

Provide:
1. Transformed data
2. Transformation log
3. Any validation errors
4. Summary statistics
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
#!/bin/bash
FILE=$1
echo "=== Code Review ==="
swissarmyhammer test review/code --file_path "$FILE"

echo -e "\n=== Security Check ==="
swissarmyhammer test review/security --code "$(cat $FILE)"

echo -e "\n=== Test Generation ==="
swissarmyhammer test test/unit --code "$(cat $FILE)"
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