# Prompt Organization

Effective prompt organization is crucial for maintaining a scalable and manageable prompt library. This guide covers best practices for organizing your SwissArmyHammer prompts.

## Directory Structure

### Recommended Hierarchy

```
~/.swissarmyhammer/prompts/
├── development/
│   ├── languages/
│   │   ├── python/
│   │   ├── javascript/
│   │   └── rust/
│   ├── frameworks/
│   │   ├── react/
│   │   ├── django/
│   │   └── fastapi/
│   └── tools/
│       ├── git/
│       ├── docker/
│       └── ci-cd/
├── writing/
│   ├── technical/
│   ├── business/
│   └── creative/
├── data/
│   ├── analysis/
│   ├── transformation/
│   └── visualization/
├── productivity/
│   ├── planning/
│   ├── automation/
│   └── workflows/
└── _shared/
    ├── components/
    ├── templates/
    └── utilities/
```

### Directory Purposes

- **development/** - Programming and technical prompts
- **writing/** - Content creation and documentation
- **data/** - Data processing and analysis
- **productivity/** - Task management and workflows
- **_shared/** - Reusable components and utilities

## Naming Conventions

### File Names

Use consistent, descriptive file names:

```
# Good examples
code-review-python.md
api-documentation-generator.md
git-commit-message.md
database-migration-planner.md

# Avoid
review.md
doc.md
prompt1.md
temp.md
```

### Naming Rules

1. **Use kebab-case** for file names
2. **Be descriptive** - Include the purpose and context
3. **Add type suffix** when multiple variants exist
4. **Keep under 40 characters** for readability

### Prompt Names (in YAML)

```yaml
# Good - matches file name pattern
name: code-review-python

# Also good - hierarchical naming
name: development/code-review/python

# Avoid - too generic
name: review
```

## Categories and Tags

### Using Categories

Categories provide broad groupings:

```yaml
---
name: api-security-scanner
title: API Security Scanner
category: development
subcategory: security
---
```

### Effective Tagging

Tags enable fine-grained discovery:

```yaml
---
name: react-component-generator
tags:
  - react
  - javascript
  - frontend
  - component
  - generator
  - boilerplate
---
```

### Category vs Tags

- **Categories**: Single, broad classification
- **Tags**: Multiple, specific attributes

```yaml
# Category for primary classification
category: development

# Tags for detailed attributes
tags:
  - python
  - testing
  - pytest
  - unit-tests
  - tdd
```

## Modular Design

### Base Templates

Create base templates in `_shared/templates/`:

```markdown
<!-- _shared/templates/code-review-base.md -->
---
name: code-review-base
title: Base Code Review Template
abstract: true  # Indicates this is a template
---

# Code Review

## Overview
Review the following code for:
- Best practices
- Potential bugs
- Performance issues
- Security concerns

## Code
```{{language}}
{{code}}
```

## Analysis
{{analysis_content}}
```

### Extending Base Templates

```markdown
<!-- development/languages/python/code-review-python.md -->
---
name: code-review-python
title: Python Code Review
extends: code-review-base
---

{% capture analysis_content %}
### Python-Specific Checks
- PEP 8 compliance
- Type hints usage
- Pythonic idioms
- Import organization
{% endcapture %}

{% include "code-review-base" %}
```

### Shared Components

Store reusable components in `_shared/components/`:

```markdown
<!-- _shared/components/security-checks.md -->
---
name: security-checks-component
component: true
---

### Security Analysis
- Input validation
- SQL injection risks
- XSS vulnerabilities
- Authentication flaws
- Data exposure
```

Use in prompts:

```markdown
---
name: web-app-review
---

# Web Application Review

{{code}}

{% include "_shared/components/security-checks.md" %}

## Additional Checks
...
```

## Versioning

### Version in Metadata

Track prompt versions:

```yaml
---
name: api-generator
version: 2.1.0
updated: 2024-03-20
changelog:
  - 2.1.0: Added GraphQL support
  - 2.0.0: Breaking change - new argument structure
  - 1.0.0: Initial release
---
```

### Version Directories

For major versions with breaking changes:

```
prompts/
├── development/
│   ├── api-generator/
│   │   ├── v1/
│   │   │   └── api-generator.md
│   │   ├── v2/
│   │   │   └── api-generator.md
│   │   └── latest -> v2/api-generator.md
```

### Migration Guides

Document version changes:

```markdown
<!-- development/api-generator/MIGRATION.md -->
# API Generator Migration Guide

## v1 to v2

### Breaking Changes
- `endpoint_list` argument renamed to `endpoints`
- `auth_method` now requires specific values

### Migration Steps
1. Update argument names in your scripts
2. Validate auth_method values
3. Test with new version
```

## Collections

### Prompt Collections

Group related prompts:

```markdown
<!-- collections/fullstack-development.md -->
---
name: fullstack-collection
title: Full-Stack Development Collection
type: collection
---

# Full-Stack Development Prompts

## Frontend
- `frontend/react-component` - React component generator
- `frontend/vue-template` - Vue.js templates
- `frontend/css-optimizer` - CSS optimization

## Backend
- `backend/api-design` - API design assistant
- `backend/database-schema` - Schema designer
- `backend/auth-implementation` - Authentication setup

## DevOps
- `devops/docker-config` - Docker configuration
- `devops/ci-pipeline` - CI/CD pipeline setup
- `devops/deployment-guide` - Deployment strategies
```

### Collection Metadata

```yaml
---
name: data-science-toolkit
type: collection
prompts:
  - data/eda-assistant
  - data/feature-engineering
  - data/model-evaluation
  - data/visualization-guide
dependencies:
  - python
  - pandas
  - scikit-learn
---
```

## Search and Discovery

### Metadata for Search

Optimize prompts for discovery:

```yaml
---
name: code-documenter
title: Intelligent Code Documentation Generator
description: |
  Generates comprehensive documentation for code including:
  - Function/method documentation
  - Class documentation
  - Module overview
  - Usage examples
  - API references
keywords:
  - documentation
  - docstring
  - comments
  - api docs
  - code docs
  - jsdoc
  - sphinx
  - rustdoc
search_terms:
  - "generate documentation"
  - "add comments to code"
  - "create api docs"
  - "document functions"
---
```

### Aliases

Support multiple names:

```yaml
---
name: git-commit-message
aliases:
  - commit-message
  - git-message
  - commit-generator
---
```

### Related Prompts

Link related prompts:

```yaml
---
name: code-review-security
related:
  - code-review-general
  - security-audit
  - vulnerability-scanner
  - penetration-test-guide
---
```

## Team Collaboration

### Shared Conventions

Document team conventions in `CONVENTIONS.md`:

```markdown
# Prompt Conventions

## Naming
- Use `project-` prefix for project-specific prompts
- Use `team-` prefix for team-wide prompts
- Use `personal-` prefix for individual prompts

## Categories
- `project` - Project-specific
- `team` - Team standards
- `experimental` - Under development

## Required Metadata
All prompts must include:
- name
- title
- description
- author
- created date
- category
```

### Ownership

Track prompt ownership:

```yaml
---
name: deployment-checklist
author: jane.doe@company.com
team: platform-engineering
maintainers:
  - jane.doe@company.com
  - john.smith@company.com
review_required: true
last_reviewed: 2024-03-15
---
```

### Review Process

Implement prompt review:

```yaml
---
name: api-contract-generator
status: draft  # draft, review, approved, deprecated
reviewers:
  - senior-dev-team
approved_by: tech-lead@company.com
approval_date: 2024-03-10
---
```

## Import/Export Strategies

### Partial Exports

Export specific categories:

```bash
# Export only development prompts
swissarmyhammer export dev-prompts.tar.gz --filter "category:development"

# Export by tags
swissarmyhammer export python-prompts.tar.gz --filter "tag:python"

# Export by date
swissarmyhammer export recent-prompts.tar.gz --filter "updated:>2024-01-01"
```

### Collection Bundles

Create installable bundles:

```yaml
# bundle.yaml
name: web-development-bundle
version: 1.0.0
description: Complete web development prompt collection
prompts:
  include:
    - category: frontend
    - category: backend
    - tag: web
  exclude:
    - tag: experimental
dependencies:
  - swissarmyhammer: ">=0.1.0"
install_to: ~/.swissarmyhammer/prompts/bundles/web-dev/
```

### Sync Strategies

Keep prompts synchronized:

```bash
#!/bin/bash
# sync-prompts.sh

# Backup local changes
swissarmyhammer export local-backup-$(date +%Y%m%d).tar.gz

# Pull team prompts
git pull origin main

# Import team updates
swissarmyhammer import team-prompts.tar.gz --merge

# Export for distribution
swissarmyhammer export team-bundle.tar.gz --filter "team:approved"
```

## Best Practices

### 1. Start Simple

Begin with basic organization:

```
prompts/
├── work/
├── personal/
└── learning/
```

Then evolve as needed.

### 2. Use Meaningful Hierarchies

```
# Good - clear hierarchy
development/testing/unit-test-generator.md
development/testing/integration-test-builder.md

# Avoid - flat structure
unit-test-generator.md
integration-test-builder.md
```

### 3. Document Your System

Create `prompts/README.md`:

```markdown
# Prompt Library Organization

## Structure
- `development/` - Programming prompts
- `data/` - Data analysis prompts
- `writing/` - Content creation
- `_shared/` - Reusable components

## Naming Convention
- Files: `purpose-context-type.md`
- Prompts: Match file names

## How to Contribute
1. Choose appropriate category
2. Follow naming conventions
3. Include all required metadata
4. Test before committing
```

### 4. Regular Maintenance

```bash
# Find unused prompts
swissarmyhammer list --unused --days 90

# Find duplicates
swissarmyhammer list --duplicates

# Validate all prompts
swissarmyhammer doctor --check prompts
```

### 5. Progressive Enhancement

Start with basic prompts and enhance:

```yaml
# Version 1 - Basic
name: code-review
description: Reviews code

# Version 2 - Enhanced
name: code-review
description: Reviews code for quality, security, and performance
category: development
tags: [review, quality, security]
version: 2.0.0
```

## Examples

### Enterprise Setup

```
company-prompts/
├── departments/
│   ├── engineering/
│   │   ├── standards/
│   │   ├── templates/
│   │   └── tools/
│   ├── product/
│   │   ├── specs/
│   │   ├── research/
│   │   └── documentation/
│   └── data-science/
│       ├── analysis/
│       ├── models/
│       └── reporting/
├── projects/
│   ├── project-alpha/
│   ├── project-beta/
│   └── _archived/
├── shared/
│   ├── components/
│   ├── templates/
│   └── utilities/
└── personal/
    └── [username]/
```

### Open Source Project

```
oss-prompts/
├── contribution/
│   ├── issue-templates/
│   ├── pr-templates/
│   └── code-review/
├── documentation/
│   ├── api-docs/
│   ├── user-guide/
│   └── examples/
├── maintenance/
│   ├── release/
│   ├── changelog/
│   └── security/
└── community/
    ├── support/
    └── onboarding/
```

## Automation

### Auto-Organization Script

```python
#!/usr/bin/env python3
# organize-prompts.py

import os
import yaml
from pathlib import Path

def organize_prompts(source_dir, target_dir):
    """Auto-organize prompts based on metadata."""
    for prompt_file in Path(source_dir).glob("**/*.md"):
        with open(prompt_file) as f:
            content = f.read()
            
        # Extract front matter
        if content.startswith("---"):
            _, fm, _ = content.split("---", 2)
            metadata = yaml.safe_load(fm)
            
            # Determine target path
            category = metadata.get("category", "uncategorized")
            subcategory = metadata.get("subcategory", "")
            
            target_path = Path(target_dir) / category
            if subcategory:
                target_path = target_path / subcategory
                
            # Move file
            target_path.mkdir(parents=True, exist_ok=True)
            target_file = target_path / prompt_file.name
            prompt_file.rename(target_file)
            
            print(f"Moved {prompt_file} -> {target_file}")
```

### Validation Script

```bash
#!/bin/bash
# validate-organization.sh

echo "Validating prompt organization..."

# Check for prompts without categories
echo "Prompts without categories:"
grep -L "category:" prompts/**/*.md

# Check for duplicate names
echo "Duplicate prompt names:"
swissarmyhammer list --format json | jq -r '.[] | .name' | sort | uniq -d

# Check naming conventions
echo "Files not following naming convention:"
find prompts -name "*.md" | grep -v "[a-z0-9-]*.md"
```

## Next Steps

- See [Creating Prompts](./creating-prompts.md) for prompt creation guidelines
- Learn about [Advanced Prompts](./advanced-prompts.md) for complex scenarios
- Explore [Examples](./examples.md) for organization patterns
- Read [Configuration](./configuration.md) for system-wide settings