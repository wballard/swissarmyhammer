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