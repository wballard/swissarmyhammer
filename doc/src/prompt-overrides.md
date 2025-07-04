# Prompt Overrides

SwissArmyHammer supports a hierarchical override system that allows you to customize prompts at different levels without modifying the original files.

## Override Hierarchy

Prompts are loaded and merged in this order (later overrides earlier):

```
1. Built-in prompts (system)
   ↓
2. User prompts (~/.swissarmyhammer/prompts)
   ↓
3. Project prompts (./.swissarmyhammer/prompts)
   ↓
4. Runtime overrides (CLI/API)
```

## How Overrides Work

### Complete Override

Replace an entire prompt by using the same name:

```markdown
<!-- Built-in: /usr/share/swissarmyhammer/prompts/code-review.md -->
---
name: code-review
title: Code Review
description: Reviews code for issues
arguments:
  - name: code
    required: true
---
Please review this code.

<!-- User override: ~/.swissarmyhammer/prompts/code-review.md -->
---
name: code-review
title: Enhanced Code Review
description: Comprehensive code analysis with security focus
arguments:
  - name: code
    required: true
  - name: security_check
    required: false
    default: true
---
Perform a detailed security-focused code review.
```

### Partial Override

Override specific fields while inheriting others:

```yaml
# ~/.swissarmyhammer/overrides/code-review.yaml
name: code-review
extends: true  # Inherit from lower level
title: "Code Review (Company Standards)"
# Only override specific arguments
arguments:
  merge: true  # Merge with parent arguments
  items:
    - name: style_guide
      default: "company-style-guide.md"
```

## Override Methods

### 1. File-Based Overrides

Create a prompt file with the same name at a higher level:

```
# Original
/usr/share/swissarmyhammer/prompts/development/python-analyzer.md

# User override
~/.swissarmyhammer/prompts/development/python-analyzer.md

# Project override
./.swissarmyhammer/prompts/development/python-analyzer.md
```

### 2. Override Configuration

Use override files to modify prompts without duplicating:

```yaml
# ~/.swissarmyhammer/overrides.yaml
overrides:
  - name: code-review
    # Override just the description
    description: "Code review with company standards"
    
  - name: api-generator
    # Add new arguments
    arguments:
      append:
        - name: auth_type
          default: "oauth2"
    
  - name: test-writer
    # Modify template content
    template:
      prepend: |
        # Company Test Standards
        Follow these guidelines:
        - Use pytest exclusively
        - Include docstrings
        
      append: |
        
        ## Additional Requirements
        - Minimum 80% coverage
        - Include integration tests
```

### 3. Runtime Overrides

Override prompts at runtime via CLI or API:

```bash
# Override prompt arguments
swissarmyhammer test code-review \
  --override title="Security Review" \
  --override description="Focus on security vulnerabilities"

# Override template content
swissarmyhammer test api-docs \
  --template-override prepend="# CONFIDENTIAL\n\n" \
  --template-override append="\n\n© 2024 Acme Corp"
```

## Advanced Override Patterns

### Inheritance Chain

Create a chain of inherited prompts:

```yaml
# base-analyzer.yaml
name: base-analyzer
abstract: true  # Can't be used directly
title: Base Code Analyzer
arguments:
  - name: code
    required: true
  - name: language
    required: false

# python-analyzer.yaml
name: python-analyzer
extends: base-analyzer
title: Python Code Analyzer
arguments:
  merge: true
  items:
    - name: check_types
      default: true

# security-python-analyzer.yaml
name: security-python-analyzer
extends: python-analyzer
title: Security-Focused Python Analyzer
template:
  inherit: true
  prepend: |
    ## Security Analysis
    Focus on OWASP Top 10 vulnerabilities.
```

### Conditional Overrides

Apply overrides based on conditions:

```yaml
# overrides.yaml
conditional_overrides:
  - condition:
      environment: production
    overrides:
      - name: all
        arguments:
          - name: verbose
            default: false
            
  - condition:
      user: qa-team
    overrides:
      - name: test-generator
        template:
          append: |
            Include edge case testing.
            
  - condition:
      project_type: web
    overrides:
      - name: security-scan
        arguments:
          - name: check_xss
            default: true
```

### Template Merging

Control how templates are merged:

```yaml
# Override with template merging
name: api-docs
extends: true
template_merge:
  strategy: smart  # smart, prepend, append, replace
  sections:
    - match: "## Authentication"
      action: replace
      content: |
        ## Authentication
        Use OAuth 2.0 with PKCE flow.
    
    - match: "## Error Handling"
      action: append
      content: |
        
        ### Company Error Codes
        - 4001: Invalid API key
        - 4002: Rate limit exceeded
```

## Project-Specific Overrides

### Directory Structure

Organize project overrides:

```
.swissarmyhammer/
├── prompts/           # Complete prompt overrides
│   └── code-review.md
├── overrides.yaml     # Partial overrides
├── templates/         # Template snippets
│   ├── header.md
│   └── footer.md
└── config.yaml       # Override configuration
```

### Override Configuration

Configure override behavior:

```yaml
# .swissarmyhammer/config.yaml
overrides:
  # Enable/disable overrides
  enabled: true
  
  # Override precedence
  precedence:
    - runtime      # Highest priority
    - project
    - user
    - system      # Lowest priority
  
  # Merge strategies
  merge:
    arguments: deep     # deep, shallow, replace
    template: smart     # smart, simple, replace
    metadata: shallow   # deep, shallow, replace
  
  # Validation
  validation:
    strict: true
    require_base: false
    allow_new_fields: true
```

## Use Cases

### 1. Company Standards

Enforce company-wide standards:

```yaml
# ~/.swissarmyhammer/company-overrides.yaml
global_overrides:
  all_prompts:
    template:
      prepend: |
        # {{company}} Standards
        This output follows {{company}} guidelines.
        
    globals:
      company: "Acme Corp"
      support_email: "ai-support@acme.com"
```

### 2. Environment-Specific

Different behavior per environment:

```yaml
# Development overrides
development:
  overrides:
    - name: code-review
      arguments:
        - name: verbose
          default: true
        - name: include_suggestions
          default: true

# Production overrides
production:
  overrides:
    - name: code-review
      arguments:
        - name: verbose
          default: false
        - name: security_scan
          default: true
```

### 3. Team Customization

Team-specific modifications:

```yaml
# Frontend team overrides
team: frontend
overrides:
  - pattern: "*-component"
    template:
      prepend: |
        Use React 18+ features.
        Follow Material-UI guidelines.
        
  - name: test-writer
    arguments:
      - name: framework
        default: "jest"
      - name: include_snapshots
        default: true
```

## Override Resolution

### Name Matching

How prompts are matched for override:

1. **Exact match**: `code-review` matches `code-review`
2. **Pattern match**: `*-review` matches `code-review`, `security-review`
3. **Category match**: `category:development` matches all development prompts

### Conflict Resolution

When multiple overrides apply:

```yaml
# Resolution rules
conflict_resolution:
  # Strategy: first, last, merge, error
  strategy: merge
  
  # Priority (higher wins)
  priorities:
    exact_match: 100
    pattern_match: 50
    category_match: 10
    global: 1
```

### Debugging Overrides

See what overrides are applied:

```bash
# Show override chain for a prompt
swissarmyhammer debug code-review --show-overrides

# Output:
Override chain for 'code-review':
1. System: /usr/share/swissarmyhammer/prompts/code-review.md
2. User: ~/.swissarmyhammer/prompts/code-review.md (extends)
3. Project: ./.swissarmyhammer/overrides.yaml (partial)
4. Runtime: --override title="Custom Review"

# Test with override preview
swissarmyhammer test code-review --preview-overrides
```

## Best Practices

### 1. Minimal Overrides

Override only what needs to change:

```yaml
# Good: Override specific fields
name: code-review
extends: true
description: "Code review with security focus"

# Avoid: Duplicating entire prompt
name: code-review
title: Code Review  # Unchanged
description: "Code review with security focus"  # Only this changed
arguments: [...]  # Duplicated
template: |       # Duplicated
  ...
```

### 2. Document Overrides

Always document why overrides exist:

```yaml
# overrides.yaml
overrides:
  - name: api-generator
    # OVERRIDE REASON: Company requires OAuth2 for all APIs
    # JIRA: SECURITY-123
    # Date: 2024-01-15
    arguments:
      - name: auth_type
        default: "oauth2"
        locked: true  # Prevent further overrides
```

### 3. Version Control

Track override changes:

```bash
# .swissarmyhammer/.gitignore
# Don't ignore override files
!overrides.yaml
!prompts/

# Track override history
git add .swissarmyhammer/overrides.yaml
git commit -m "Add security requirements to code-review prompt"
```

### 4. Testing Overrides

Test overrides thoroughly:

```bash
# Test override application
swissarmyhammer test code-review --test-overrides

# Compare with and without overrides
swissarmyhammer test code-review --no-overrides > without.txt
swissarmyhammer test code-review > with.txt
diff without.txt with.txt
```

## Security Considerations

### Lock Overrides

Prevent certain overrides:

```yaml
# System prompt with locked fields
---
name: security-scan
locked_fields:
  - title
  - core_checks
no_override: false  # Can't be overridden at all
```

### Validate Overrides

Ensure overrides meet requirements:

```yaml
# Override validation rules
validation:
  rules:
    - field: arguments
      required_items:
        - name: code
        - name: language
    
    - field: template
      must_contain:
        - "SECURITY WARNING"
        - "Confidential"
      
    - field: description
      min_length: 50
      pattern: ".*security.*"
```

## Troubleshooting

### Common Issues

1. **Override not applying**:
   ```bash
   # Check override precedence
   swissarmyhammer config get overrides.precedence
   
   # Verify file locations
   swissarmyhammer debug --show-paths
   ```

2. **Merge conflicts**:
   ```bash
   # Show merge details
   swissarmyhammer debug code-review --trace-merge
   ```

3. **Validation errors**:
   ```bash
   # Validate overrides
   swissarmyhammer validate --overrides
   ```

## Next Steps

- Learn about [Prompt Organization](./prompt-organization.md)
- Understand [Configuration](./configuration.md) options
- Read about [Testing](./testing-guide.md) override scenarios
- See [Examples](./examples.md) of override patterns