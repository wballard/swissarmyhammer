---
name: docs-readme
title: Generate README Documentation
description: Create comprehensive README documentation for a project
arguments:
  - name: project_name
    description: Name of the project
    required: true
  - name: project_description
    description: Brief description of what the project does
    required: true
  - name: language
    description: Primary programming language
    required: false
    default: "auto-detect"
  - name: features
    description: Key features of the project (comma-separated)
    required: false
    default: ""
  - name: target_audience
    description: Who this project is for
    required: false
    default: "developers"
---

# Generate README for {{project_name}}

## Project Overview
- **Name**: {{project_name}}
- **Description**: {{project_description}}
- **Language**: {{language}}
- **Target Audience**: {{target_audience}}
{% if features %}
- **Key Features**: {{features}}
{% endif %}

## README Structure

### 1. Title and Badges
- Project name with appropriate styling
- Relevant badges (build status, version, license)
- Brief tagline

### 2. Description
- Clear explanation of what the project does
- Why it exists and what problems it solves
- Key differentiators

### 3. Table of Contents
For longer READMEs, include navigation

### 4. Installation
- Prerequisites
- Step-by-step installation instructions
- Platform-specific considerations
- Docker instructions (if applicable)

### 5. Usage
- Quick start guide
- Basic examples
- Common use cases
- API reference (if applicable)

### 6. Features
- Detailed feature list
- Screenshots or demos (if UI project)
- Configuration options

### 7. Contributing
- How to contribute
- Code of conduct
- Development setup
- Testing guidelines
- Pull request process

### 8. License
- License type and link
- Copyright information

### 9. Additional Sections
Consider adding:
- FAQ
- Troubleshooting
- Changelog
- Credits/Acknowledgments
- Related projects

## Best Practices
- Keep it concise but comprehensive
- Use clear headings and formatting
- Include code examples
- Add visuals where helpful
- Update regularly
- Consider internationalization