# Built-in Prompts

SwissArmyHammer includes a comprehensive set of built-in prompts designed to assist with various development tasks. These prompts use Liquid templating for dynamic, customizable assistance and are organized by category for easy discovery.

## Overview

All built-in prompts:
- Support customizable arguments with sensible defaults
- Use Liquid syntax for variable substitution and control flow
- Are organized into logical categories for easy discovery
- Follow a standardized YAML front matter format

## Categories

### System Management

#### are_issues_complete
Check if the plan is complete.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test are_issues_complete
```

#### are_reviews_done
Check if all the code review items are complete.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test are_reviews_done
```

#### are_tests_passing
Check if all tests are passing.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test are_tests_passing
```

### Issue Management

#### branch (issue_branch)
Create an issue work branch for the next issue to work

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test branch
```

#### issue_complete
Mark an issue as complete

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test issue_complete
```

### Code Development

#### code/issue (do_issue)
Code up an issue

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test code/issue
```

#### code/review
Do Code Review

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test code/review
```

### Version Control

#### commit
Commit your work to git.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test commit
```

#### merge
Merge your work into the main branch.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test merge
```

### Testing & Quality

#### coverage
Improve coverage.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test coverage
```

#### test
Iterate to correct test failures in the codebase.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test test
```

### Debugging

#### debug/error
Analyze error messages and provide debugging guidance with potential solutions.

**Arguments:**
- `error_message` (required) - The error message or stack trace to analyze
- `language` (default: "auto-detect") - The programming language
- `context` (optional) - Additional context about when the error occurs

**Example:**
```bash
swissarmyhammer prompt test debug/error --arg error_message="TypeError: cannot read property 'name' of undefined" --arg language="javascript"
```

#### debug/logs
Analyze log files to identify issues and patterns.

**Arguments:**
- `log_content` (required) - The log content to analyze
- `issue_description` (default: "general analysis") - Description of the issue you're investigating
- `time_range` (default: "all") - Specific time range to focus on
- `log_format` (default: "auto-detect") - Log format (json, plaintext, syslog, etc.)

**Example:**
```bash
swissarmyhammer prompt test debug/logs --arg log_content="$(cat application.log)" --arg issue_description="API timeout errors"
```

### Documentation

#### document
Create documentation for the project

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test document
```

#### docs/comments
Add comprehensive comments and documentation to code.

**Arguments:**
- `code` (required) - The code to document
- `comment_style` (default: "auto-detect") - Comment style (inline, block, jsdoc, docstring, rustdoc)
- `detail_level` (default: "standard") - Level of detail (minimal, standard, comprehensive)
- `audience` (default: "developers") - Target audience for the comments

**Example:**
```bash
swissarmyhammer prompt test docs/comments --arg code="$(cat utils.js)" --arg comment_style="jsdoc" --arg detail_level="comprehensive"
```

#### docs/readme
Create comprehensive README documentation for a project.

**Arguments:**
- `project_name` (required) - Name of the project
- `project_description` (required) - Brief description of what the project does
- `language` (default: "auto-detect") - Primary programming language
- `features` (default: "") - Key features of the project (comma-separated)
- `target_audience` (default: "developers") - Who this project is for

**Example:**
```bash
swissarmyhammer prompt test docs/readme --arg project_name="MyLib" --arg project_description="A library for awesome things" --arg features="fast,reliable,easy"
```

### Planning

#### plan
Generate a step by step development plan from a specification.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test plan
```

### Code Review

#### review/accessibility
Review code for accessibility compliance and best practices.

**Arguments:**
- `code` (required) - The UI/frontend code to review
- `wcag_level` (default: "AA") - WCAG compliance level target (A, AA, AAA)
- `component_type` (default: "general") - Type of component (form, navigation, content, interactive)
- `target_users` (default: "all users") - Specific user needs to consider

**Example:**
```bash
swissarmyhammer prompt test review/accessibility --arg code="$(cat form.html)" --arg component_type="form" --arg wcag_level="AA"
```

#### review/branch
Review code

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test review/branch
```

#### review/code
Review code for quality, bugs, and improvements.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test review/code
```

#### review/documentation
Review documentation

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test review/documentation
```

#### review/patterns
Perform a comprehensive review of the code to improve pattern use.

**Arguments:** None

**Example:**
```bash
swissarmyhammer prompt test review/patterns
```

#### review/security
Perform a comprehensive security review of code to identify vulnerabilities.

**Arguments:**
- `code` (required) - The code to review for security issues
- `context` (default: "general purpose code") - Context about the code
- `language` (default: "auto-detect") - Programming language
- `severity_threshold` (default: "low") - Minimum severity to report (critical, high, medium, low)

**Example:**
```bash
swissarmyhammer prompt test review/security --arg code="$(cat login.php)" --arg context="handles user authentication"
```

### Prompt Management

#### prompts/create
Help create effective prompts for swissarmyhammer.

**Arguments:**
- `purpose` (required) - What the prompt should accomplish
- `category` (default: "general") - Category for the prompt
- `inputs_needed` (default: "") - What information the prompt needs from users
- `complexity` (default: "moderate") - Complexity level (simple, moderate, advanced)

**Example:**
```bash
swissarmyhammer prompt test prompts/create --arg purpose="Generate database migrations" --arg category="database"
```

#### prompts/improve
Analyze and enhance existing prompts for better effectiveness.

**Arguments:**
- `prompt_content` (required) - The current prompt content (including YAML front matter)
- `improvement_goals` (default: "overall enhancement") - What aspects to improve
- `user_feedback` (default: "") - Any feedback or issues users have reported

**Example:**
```bash
swissarmyhammer prompt test prompts/improve --arg prompt_content="$(cat my-prompt.md)" --arg improvement_goals="clarity,flexibility"
```

### General Purpose

#### help
A prompt for providing helpful assistance and guidance to users.

**Arguments:**
- `topic` (default: "general assistance") - The topic to get help about
- `detail_level` (default: "normal") - How detailed the help should be (basic, normal, detailed)

**Example:**
```bash
swissarmyhammer prompt test help --arg topic="git workflows" --arg detail_level="detailed"
```

#### example
An example prompt for testing.

**Arguments:**
- `topic` (default: "general topic") - The topic to ask about

**Example:**
```bash
swissarmyhammer prompt test example --arg topic="testing prompts"
```

#### say-hello
A simple greeting prompt that can be customized with name and language.

**Arguments:**
- `name` (default: "friend") - Name to greet
- `language` (default: "english") - Language for the greeting

**Example:**
```bash
swissarmyhammer prompt test say-hello --arg name="Alice" --arg language="spanish"
```

## Template Partials

These `.md` files are partial templates used by other prompts for consistency:

- `code.md` - Partial template for reuse in other prompts
- `coding_standards.md` - Coding Standards template
- `documentation.md` - Documentation template  
- `empty.md` - Empty template
- `principals.md` - Principals template
- `review_format.md` - Review Format template
- `todo.md` - Todo template

## Usage Patterns

### Basic Usage
```bash
# Use a prompt with default arguments
swissarmyhammer prompt test review/code

# Specify custom arguments
swissarmyhammer prompt test debug/error --arg error_message="NullPointerException" --arg language="java"
```

### Piping Content
```bash
# Use command substitution to pass file content
swissarmyhammer prompt test docs/comments --arg code="$(cat utils.py)" --arg comment_style="docstring"

# Pipe log content for analysis
cat error.log | xargs -I {} swissarmyhammer prompt test debug/logs --arg log_content="{}" --arg issue_description="memory leak"
```

### Non-Interactive Mode
```bash
# Run prompts non-interactively with all arguments specified
swissarmyhammer prompt test help --arg topic="git" --arg detail_level="detailed"
```

### Debugging Prompts
```bash
# Use debug mode to see template processing
swissarmyhammer prompt test debug/error --debug --arg error_message="Sample error"
```

## Best Practices

1. **Choose the Right Prompt** - Select prompts that match your specific task
2. **Provide Context** - Use optional arguments to give more context when available
3. **Combine Prompts** - Use multiple prompts in sequence for comprehensive workflows
4. **Test Arguments** - Use the test command to verify prompt behavior before using in workflows
5. **Review Output** - Always review and validate generated content before using it

## Creating Custom Prompts

If the built-in prompts don't meet your needs:

1. Use the `prompts/create` prompt to generate a template
2. Save it in your `~/.swissarmyhammer/prompts/` directory
3. Follow the YAML front matter format for consistency
4. Test with various inputs to ensure reliability

For more information on creating custom prompts, see [Creating Prompts](./creating-prompts.md).