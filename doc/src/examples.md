# Examples

This page provides real-world examples of using SwissArmyHammer for various development tasks. Each example demonstrates practical usage patterns and best practices for prompt engineering.

## Overview

SwissArmyHammer excels at organizing and managing prompts for:
- **Development workflows** - Code review, testing, debugging
- **Documentation tasks** - API docs, user guides, technical writing
- **Security analysis** - Code audits, vulnerability assessment
- **Content creation** - Blog posts, marketing copy, educational materials
- **Data processing** - Analysis, transformation, reporting

## New Advanced Prompts

### AI Code Assistant
Intelligent code assistance with context awareness:
```markdown
{{#include ../examples/prompts/ai-code-assistant.md}}
```

### Security Audit
Comprehensive security analysis following OWASP guidelines:
```markdown
{{#include ../examples/prompts/security-audit.md}}
```

### Technical Writer
Professional technical documentation generation:
```markdown
{{#include ../examples/prompts/technical-writer.md}}
```

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

## Real-World Scenarios

### Onboarding New Team Members

Create an interactive onboarding workflow:

```bash
# Create onboarding checklist
swissarmyhammer test onboarding/checklist \
  --team_name "Backend Team" \
  --role "Senior Engineer" \
  --project_stack "Rust, PostgreSQL, Docker"

# Generate personalized learning path
swissarmyhammer test onboarding/learning-path \
  --experience_level "senior" \
  --background "Go, MongoDB" \
  --target_skills "Rust, async programming"
```

### Code Migration Project

Systematic approach to migrating codebases:

```bash
# Analyze legacy code
swissarmyhammer test migration/analyze \
  --source_language "Python 2.7" \
  --target_language "Python 3.11" \
  --codebase_size "50k LOC"

# Generate migration plan
swissarmyhammer test migration/plan \
  --analysis_results "$(cat analysis.md)" \
  --timeline "3 months" \
  --team_size "4 developers"

# Create migration checklist per module
for module in $(find src -name "*.py"); do
  swissarmyhammer test migration/module-checklist \
    --module_path "$module" \
    --complexity "$(wc -l < $module)" \
    >> migration-plan.md
done
```

### Technical Debt Assessment

Comprehensive debt analysis workflow:

```bash
# Assess technical debt across codebase
swissarmyhammer test debt/assessment \
  --project_age "2 years" \
  --team_turnover "high" \
  --test_coverage "$(pytest --cov=. --cov-report=term | grep TOTAL | awk '{print $4}')"

# Prioritize debt items
swissarmyhammer test debt/prioritize \
  --business_impact "high" \
  --development_velocity "slowing" \
  --upcoming_features "user dashboard, payments"
```

### Performance Optimization Campaign

Systematic performance improvement:

```bash
# Identify bottlenecks
swissarmyhammer test performance/analyze \
  --profile_data "$(cat profile.json)" \
  --target_improvement "50% faster" \
  --budget "2 weeks"

# Generate optimization roadmap
swissarmyhammer test performance/roadmap \
  --current_metrics "$(cat metrics.json)" \
  --constraints "no breaking changes" \
  --priority "database queries, API latency"
```

## Advanced Patterns

### Team Collaboration Workflows

```bash
# Daily standup preparation
swissarmyhammer test standup/prepare \
  --yesterday_commits "$(git log --oneline --since='1 day ago' --author="$(git config user.email)")" \
  --current_branch "$(git branch --show-current)" \
  --blockers "waiting for API keys"

# Sprint retrospective insights
swissarmyhammer test retro/insights \
  --sprint_goals "$(cat sprint-goals.md)" \
  --completed_stories "8/12" \
  --team_feedback "$(cat feedback.json)"
```

### Dynamic Prompt Selection

```bash
{{#include ../examples/scripts/smart-review.sh}}
```

### Batch Processing

```python
{{#include ../examples/scripts/batch_analyze.py}}
```

### Enterprise Integration Patterns

```bash
# Compliance audit preparation
swissarmyhammer test compliance/audit \
  --standards "SOC2, GDPR, HIPAA" \
  --audit_date "2024-03-15" \
  --evidence_path "./compliance-docs"

# Risk assessment for new features
swissarmyhammer test risk/assessment \
  --feature_description "$(cat feature-spec.md)" \
  --security_requirements "PII handling, payment processing" \
  --timeline "Q2 2024"
```

### Multi-Repository Management

```bash
# Synchronize standards across repos
for repo in frontend backend mobile; do
  cd "../$repo"
  swissarmyhammer test standards/sync \
    --repo_type "$repo" \
    --base_standards "$(cat ../standards/base.md)" \
    --output_file "CODING_STANDARDS.md"
done

# Generate cross-repo dependency analysis
swissarmyhammer test deps/analyze \
  --repositories "frontend,backend,mobile" \
  --focus "security,performance,maintainability"
```

### Custom Filter Integration

Create a prompt that uses custom filters:

```markdown
{{#include ../examples/prompts/data-transformer.md}}
```

### Workflow Automation with Issue Management

```bash
# Create issues for code quality improvements
swissarmyhammer test quality/issues \
  --analysis_report "$(cat code-analysis.json)" \
  --severity_threshold "medium" | \
while read issue_title; do
  swissarmyhammer issue create quality \
    --content "# Code Quality Issue\n\n$issue_title\n\n## Analysis\n$(cat details.md)"
done

# Generate release notes from completed issues
swissarmyhammer test release/notes \
  --version "v2.1.0" \
  --completed_issues "$(ls issues/complete/*.md)" \
  --target_audience "technical users"
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