# Sharing and Collaboration

This guide covers how to share SwissArmyHammer prompts with your team, collaborate on prompt development, and manage shared prompt libraries.

## Overview

SwissArmyHammer supports multiple collaboration workflows:
- **File Sharing** - Share prompt files directly
- **Git Integration** - Version control for prompts
- **Team Directories** - Shared network folders
- **Package Management** - Distribute as packages

## Sharing Methods

### Direct File Sharing

#### Single Prompt Sharing

Share individual prompt files:

```bash
# Send a single prompt
cp ~/.swissarmyhammer/prompts/code-review.md /shared/prompts/

# Share via email/chat
# Attach the .md file directly
```

Recipients install by copying to their prompt directory:

```bash
# Install shared prompt
cp /downloads/code-review.md ~/.swissarmyhammer/prompts/
```

#### Prompt Collections

Share multiple related prompts:

```bash
# Create a collection
mkdir python-toolkit
cp ~/.swissarmyhammer/prompts/python-*.md python-toolkit/
cp ~/.swissarmyhammer/prompts/pytest-*.md python-toolkit/

# Share as zip
zip -r python-toolkit.zip python-toolkit/
```

## Git-Based Collaboration

### Repository Structure

Organize prompts in a Git repository:

```
prompt-library/
├── .git/
├── README.md
├── prompts/
│   ├── development/
│   │   ├── languages/
│   │   ├── frameworks/
│   │   └── tools/
│   ├── data/
│   │   ├── analysis/
│   │   └── visualization/
│   └── writing/
│       ├── technical/
│       └── creative/
├── shared/
│   ├── components/
│   └── templates/
├── scripts/
│   ├── validate.sh
│   └── install.sh
└── .github/
    └── workflows/
        └── validate-prompts.yml
```

### Team Workflow

#### Initial Setup

```bash
# Create prompt repository
git init prompt-library
cd prompt-library

# Add initial structure
mkdir -p prompts/{development,data,writing}
mkdir -p shared/{components,templates}

# Add README
cat > README.md << 'EOF'
# Team Prompt Library

Shared SwissArmyHammer prompts for our team.

## Installation

```bash
git clone https://github.com/ourteam/prompt-library
./scripts/install.sh
```

## Contributing

See CONTRIBUTING.md for guidelines.
EOF

# Initial commit
git add .
git commit -m "Initial prompt library structure"
```

#### Installation Script

Create `scripts/install.sh`:

```bash
#!/bin/bash
# install.sh - Install team prompts

PROMPT_DIR="$HOME/.swissarmyhammer/prompts/team"

# Create team namespace
mkdir -p "$PROMPT_DIR"

# Copy prompts
cp -r prompts/* "$PROMPT_DIR/"

# Copy shared components
cp -r shared/* "$HOME/.swissarmyhammer/prompts/_shared/"

echo "Team prompts installed to $PROMPT_DIR"
echo "Run 'swissarmyhammer list' to see available prompts"
```

#### Contributing Prompts

```bash
# Clone repository
git clone https://github.com/ourteam/prompt-library
cd prompt-library

# Create feature branch
git checkout -b add-docker-prompts

# Add new prompts
mkdir -p prompts/development/docker
vim prompts/development/docker/dockerfile-optimizer.md

# Test locally
swissarmyhammer doctor --check prompts

# Commit and push
git add prompts/development/docker/
git commit -m "Add Docker optimization prompts"
git push origin add-docker-prompts

# Create pull request
gh pr create --title "Add Docker optimization prompts" \
  --body "Adds prompts for Dockerfile optimization and best practices"
```

### Version Control Best Practices

#### Branching Strategy

```bash
# Main branches
main           # Stable, tested prompts
develop        # Integration branch
feature/*      # New prompts
fix/*          # Bug fixes
experimental/* # Experimental prompts
```

#### Commit Messages

Follow conventional commits:

```bash
# Adding prompts
git commit -m "feat(python): add async code review prompt"

# Fixing prompts
git commit -m "fix(api-design): correct OpenAPI template syntax"

# Updating prompts
git commit -m "refactor(test-writer): improve test case generation"

# Documentation
git commit -m "docs: add prompt writing guidelines"
```

#### Pull Request Template

`.github/pull_request_template.md`:

```markdown
## Description
Brief description of the prompts being added/modified

## Type of Change
- [ ] New prompt(s)
- [ ] Bug fix
- [ ] Enhancement
- [ ] Documentation

## Testing
- [ ] Tested with `swissarmyhammer doctor`
- [ ] Validated template syntax
- [ ] Checked for duplicates

## Checklist
- [ ] Follows naming conventions
- [ ] Includes required metadata
- [ ] Has meaningful description
- [ ] Includes usage examples
```

### Automated Validation

#### GitHub Actions Workflow

`.github/workflows/validate-prompts.yml`:

```yaml
name: Validate Prompts

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  validate:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install SwissArmyHammer
      run: |
        curl -sSL https://install.swissarmyhammer.dev | sh
        
    - name: Validate Prompts
      run: |
        swissarmyhammer doctor --check prompts
        
    - name: Check Duplicates
      run: |
        swissarmyhammer list --format json | \
          jq -r '.[].name' | sort | uniq -d > duplicates.txt
        if [ -s duplicates.txt ]; then
          echo "Duplicate prompts found:"
          cat duplicates.txt
          exit 1
        fi
        
    - name: Lint Markdown
      uses: DavidAnson/markdownlint-cli2-action@v11
      with:
        globs: 'prompts/**/*.md'
```

## Team Directories

### Network Share Setup

#### Windows Network Share

```powershell
# Create shared folder
New-Item -Path "\\server\prompts" -ItemType Directory

# Set permissions
$acl = Get-Acl "\\server\prompts"
$permission = "Domain\TeamMembers","ReadAndExecute","Allow"
$accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule $permission
$acl.SetAccessRule($accessRule)
Set-Acl "\\server\prompts" $acl
```

Configure SwissArmyHammer:

```toml
# ~/.swissarmyhammer/config.toml
[prompts]
directories = [
    "~/.swissarmyhammer/prompts",
    "//server/prompts"
]
```

#### NFS Share (Linux/Mac)

```bash
# Server setup
sudo mkdir -p /srv/prompts
sudo chown -R :team /srv/prompts
sudo chmod -R 775 /srv/prompts

# /etc/exports
/srv/prompts 192.168.1.0/24(ro,sync,no_subtree_check)

# Client mount
sudo mkdir -p /mnt/team-prompts
sudo mount -t nfs server:/srv/prompts /mnt/team-prompts
```

### Syncing Strategies

#### rsync Method

```bash
#!/bin/bash
# sync-prompts.sh

REMOTE="server:/srv/prompts"
LOCAL="$HOME/.swissarmyhammer/prompts/team"

# Sync from server (read-only)
rsync -avz --delete "$REMOTE/" "$LOCAL/"

# Watch for changes
while inotifywait -r -e modify,create,delete "$LOCAL"; do
    echo "Changes detected, syncing..."
    rsync -avz "$LOCAL/" "$REMOTE/"
done
```

#### Cloud Storage Sync

Using rclone:

```bash
# Configure rclone
rclone config

# Sync from cloud
rclone sync dropbox:team-prompts ~/.swissarmyhammer/prompts/team

# Bidirectional sync
rclone bisync dropbox:team-prompts ~/.swissarmyhammer/prompts/team
```

## Package Management

### Creating Packages

#### NPM Package

`package.json`:

```json
{
  "name": "@company/swissarmyhammer-prompts",
  "version": "1.0.0",
  "description": "Company SwissArmyHammer prompts",
  "files": [
    "prompts/**/*.md",
    "install.js"
  ],
  "scripts": {
    "postinstall": "node install.js"
  },
  "keywords": ["swissarmyhammer", "prompts", "ai"],
  "repository": {
    "type": "git",
    "url": "https://github.com/company/prompts.git"
  }
}
```

`install.js`:

```javascript
const fs = require('fs');
const path = require('path');
const os = require('os');

const sourceDir = path.join(__dirname, 'prompts');
const targetDir = path.join(
  os.homedir(), 
  '.swissarmyhammer', 
  'prompts', 
  'company'
);

// Copy prompts to user directory
fs.cpSync(sourceDir, targetDir, { recursive: true });
console.log(`Prompts installed to ${targetDir}`);
```

#### Python Package

`setup.py`:

```python
from setuptools import setup, find_packages
import os
from pathlib import Path

def get_prompt_files():
    """Get all prompt files for packaging."""
    prompt_files = []
    for root, dirs, files in os.walk('prompts'):
        for file in files:
            if file.endswith('.md'):
                prompt_files.append(os.path.join(root, file))
    return prompt_files

setup(
    name='company-swissarmyhammer-prompts',
    version='1.0.0',
    packages=find_packages(),
    data_files=[
        (f'.swissarmyhammer/prompts/company/{os.path.dirname(f)}', [f])
        for f in get_prompt_files()
    ],
    install_requires=[],
    entry_points={
        'console_scripts': [
            'install-company-prompts=scripts.install:main',
        ],
    },
)
```

### Distribution Channels

#### Internal Package Registry

```bash
# Publish to internal registry
npm publish --registry https://npm.company.com

# Install from registry
npm install @company/swissarmyhammer-prompts --registry https://npm.company.com
```

#### Container Registry

`Dockerfile`:

```dockerfile
FROM alpine:latest

# Install prompts
COPY prompts /prompts

# Create tarball
RUN tar -czf /prompts.tar.gz -C / prompts

# Export as artifact
FROM scratch
COPY --from=0 /prompts.tar.gz /
```

```bash
# Build and push
docker build -t registry.company.com/prompts:latest .
docker push registry.company.com/prompts:latest

# Pull and extract
docker create --name temp registry.company.com/prompts:latest
docker cp temp:/prompts.tar.gz .
docker rm temp
tar -xzf prompts.tar.gz -C ~/.swissarmyhammer/
```

## Access Control

### Git-Based Permissions

```bash
# Separate repositories by access level
prompt-library-public/    # All team members
prompt-library-internal/  # Internal team only
prompt-library-sensitive/ # Restricted access
```

### File System Permissions

```bash
# Create group-based access
sudo groupadd prompt-readers
sudo groupadd prompt-writers

# Set permissions
sudo chown -R :prompt-readers /srv/prompts
sudo chmod -R 750 /srv/prompts
sudo chmod -R 770 /srv/prompts/contributions

# Add users to groups
sudo usermod -a -G prompt-readers alice
sudo usermod -a -G prompt-writers bob
```

### Prompt Metadata

Mark prompts with access levels:

```yaml
---
name: sensitive-data-analyzer
title: Sensitive Data Analysis
access: restricted
allowed_users:
  - security-team
  - data-governance
tags:
  - sensitive
  - compliance
  - restricted
---
```

## Collaboration Tools

### Prompt Development Environment

VS Code workspace settings:

`.vscode/settings.json`:

```json
{
  "files.associations": {
    "*.md": "markdown"
  },
  "markdown.validate.enabled": true,
  "markdown.validate.rules": {
    "yaml-front-matter": true
  },
  "files.exclude": {
    "**/.git": true,
    "**/.DS_Store": true
  },
  "search.exclude": {
    "**/node_modules": true,
    "**/.git": true
  }
}
```

### Team Guidelines

Create `CONTRIBUTING.md`:

```markdown
# Contributing to Team Prompts

## Prompt Standards

### Naming Conventions
- Use kebab-case: `code-review-security.md`
- Be descriptive: `python-async-optimizer.md`
- Include context: `react-component-generator.md`

### Required Metadata
All prompts must include:
- `name` - Unique identifier
- `title` - Human-readable title
- `description` - What the prompt does
- `author` - Your email
- `category` - Primary category
- `tags` - At least 3 relevant tags

### Template Quality
- Use clear, concise language
- Include usage examples
- Test with various inputs
- Document edge cases

## Review Process

1. Create feature branch
2. Add/modify prompts
3. Run validation: `swissarmyhammer doctor`
4. Submit pull request
5. Address review feedback
6. Merge when approved

## Testing

Before submitting:
```bash
# Validate syntax
swissarmyhammer doctor --check prompts

# Test rendering
swissarmyhammer get your-prompt --args key=value

# Check for conflicts
swissarmyhammer list --format json | jq '.[] | select(.name=="your-prompt")'
```
```

### Communication

#### Slack Integration

```javascript
// slack-bot.js
const { WebClient } = require('@slack/web-api');
const { exec } = require('child_process');

const slack = new WebClient(process.env.SLACK_TOKEN);

// Notify on new prompts
async function notifyNewPrompt(promptName, author) {
  await slack.chat.postMessage({
    channel: '#prompt-library',
    text: `New prompt added: *${promptName}* by ${author}`,
    attachments: [{
      color: 'good',
      fields: [{
        title: 'View Prompt',
        value: `\`swissarmyhammer get ${promptName}\``,
        short: false
      }]
    }]
  });
}
```

#### Email Notifications

```bash
#!/bin/bash
# notify-updates.sh

RECIPIENTS="team@company.com"
SUBJECT="Prompt Library Updates"

# Get recent changes
CHANGES=$(git log --oneline --since="1 week ago" --grep="^feat\|^fix")

# Send email
echo "Weekly prompt library updates:

$CHANGES

To update your local prompts:
git pull origin main
./scripts/install.sh
" | mail -s "$SUBJECT" $RECIPIENTS
```

## Best Practices

### 1. Establish Standards

Define clear guidelines:
- Naming conventions
- Required metadata
- Quality standards
- Review process
- Version strategy

### 2. Use Namespaces

Organize prompts by team/project:

```
~/.swissarmyhammer/prompts/
├── personal/       # Your prompts
├── team/          # Team shared
├── company/       # Company wide
└── community/     # Open source
```

### 3. Document Everything

- README for each category
- Usage examples in prompts
- Change logs for versions
- Migration guides

### 4. Automate Validation

- Pre-commit hooks
- CI/CD validation
- Automated testing
- Quality metrics

### 5. Regular Maintenance

- Review unused prompts
- Update outdated content
- Consolidate duplicates
- Archive deprecated

## Examples

### Team Onboarding

Create onboarding bundle:

```bash
#!/bin/bash
# create-onboarding-bundle.sh

# Create directory structure
mkdir -p onboarding-prompts/prompts

# Copy essential prompts
cp ~/.swissarmyhammer/prompts/*onboarding*.md onboarding-prompts/prompts/
cp ~/.swissarmyhammer/prompts/*essential*.md onboarding-prompts/prompts/

# Add setup script
cat > onboarding-prompts/setup.sh << 'EOF'
#!/bin/bash
echo "Welcome to the team! Setting up your prompts..."
cp -r prompts/* ~/.swissarmyhammer/prompts/
echo "Run 'swissarmyhammer list' to see your new prompts!"
EOF

# Create welcome package
tar -czf welcome-pack.tar.gz onboarding-prompts/
```

### Project Templates

Share project-specific prompts:

```yaml
# project-manifest.yaml
name: microservice-toolkit
version: 1.0.0
description: Prompts for microservice development
prompts:
  - api-design
  - openapi-generator
  - dockerfile-creator
  - k8s-manifest-builder
  - test-suite-generator
dependencies:
  - base-toolkit: ">=1.0.0"
install_script: |
  mkdir -p ~/.swissarmyhammer/prompts/projects/microservices
  cp prompts/*.md ~/.swissarmyhammer/prompts/projects/microservices/
```

## Next Steps

- Read [Prompt Organization](./prompt-organization.md) for structure best practices
- See [Contributing](./contributing.md) for contribution guidelines
- Explore [Git Integration](./development.md) for version control workflows
- Learn about [Configuration](./configuration.md) for team setup