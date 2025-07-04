# Built-in Prompts

SwissArmyHammer includes a comprehensive set of built-in prompts designed to assist with various development tasks. These prompts are organized by category and leverage Liquid templating for dynamic, customizable assistance.

## Overview

All built-in prompts:
- Support customizable arguments with sensible defaults
- Use Liquid syntax for variable substitution and control flow
- Are organized into logical categories for easy discovery
- Follow a standardized YAML front matter format

## Categories

### Analysis

#### statistics-calculator
Calculate statistics on numeric data using math filters and array operations.

**Arguments:**
- `numbers` (required) - Comma-separated list of numbers
- `precision` (default: "2") - Decimal precision for calculations
- `show_outliers` (default: "true") - Identify outliers in the dataset
- `percentiles` (default: "25,50,75") - Calculate percentiles (comma-separated)
- `visualization` (default: "true") - Show ASCII visualization

**Example:**
```bash
swissarmyhammer test statistics-calculator --numbers "10,20,30,40,50" --percentiles "10,50,90"
```

### Communication

#### email-composer
Compose professional emails with dynamic content using capture blocks.

**Arguments:**
- `recipient_name` (required) - Name of the email recipient
- `sender_name` (required) - Name of the sender
- `email_type` (required) - Type of email (welcome, followup, reminder, thank_you)
- `context` (default: "") - Additional context for the email
- `formal` (default: "false") - Use formal tone
- `include_signature` (default: "true") - Include email signature
- `time_of_day` (default: "morning") - Current time of day

**Example:**
```bash
swissarmyhammer test email-composer --recipient_name "John Doe" --sender_name "Jane Smith" --email_type "followup" --formal "true"
```

### Data Processing

#### array-processor
Process arrays with flexible filtering and loop control.

**Arguments:**
- `items` (required) - Comma-separated list of items to process
- `skip_pattern` (default: "") - Pattern to skip items containing this text
- `stop_pattern` (default: "") - Pattern to stop processing
- `max_items` (default: "100") - Maximum number of items to process
- `show_skipped` (default: "false") - Show skipped items separately
- `format` (default: "list") - Output format (list, table, json)

**Example:**
```bash
swissarmyhammer test array-processor --items "apple,banana,cherry" --skip_pattern "berry" --format "table"
```

### Debug

#### debug/error
Analyze error messages and provide debugging guidance with potential solutions.

**Arguments:**
- `error_message` (required) - The error message or stack trace to analyze
- `language` (default: "auto-detect") - The programming language
- `context` (default: "") - Additional context about when the error occurs

**Example:**
```bash
swissarmyhammer test debug/error --error_message "TypeError: cannot read property 'name' of undefined" --language "javascript"
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
swissarmyhammer test debug/logs --log_content "$(cat application.log)" --issue_description "API timeout errors"
```

#### debug/performance
Analyze performance problems and suggest optimization strategies.

**Arguments:**
- `problem_description` (required) - Description of the performance issue
- `metrics` (default: "not provided") - Performance metrics
- `code_snippet` (default: "") - Relevant code that might be causing the issue
- `environment` (default: "development") - Environment details

**Example:**
```bash
swissarmyhammer test debug/performance --problem_description "Database queries taking 5+ seconds" --environment "production"
```

### Documentation

#### docs/api
Create comprehensive API documentation from code.

**Arguments:**
- `code` (required) - The API code to document
- `api_type` (default: "REST") - Type of API (REST, GraphQL, gRPC, library)
- `format` (default: "markdown") - Documentation format (markdown, openapi, swagger)
- `include_examples` (default: "true") - Whether to include usage examples

**Example:**
```bash
swissarmyhammer test docs/api --code "$(cat api.py)" --api_type "REST" --include_examples "true"
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
swissarmyhammer test docs/comments --code "$(cat utils.js)" --comment_style "jsdoc" --detail_level "comprehensive"
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
swissarmyhammer test docs/readme --project_name "MyLib" --project_description "A library for awesome things" --features "fast,reliable,easy"
```

### Formatting

#### table-generator
Generate formatted tables with alternating row styles.

**Arguments:**
- `headers` (required) - Comma-separated list of table headers
- `rows` (required) - Semicolon-separated rows, with comma-separated values
- `style` (default: "markdown") - Table style (markdown, html, ascii)
- `zebra` (default: "true") - Enable zebra striping for rows
- `row_numbers` (default: "false") - Add row numbers

**Example:**
```bash
swissarmyhammer test table-generator --headers "Name,Age,City" --rows "John,30,NYC;Jane,25,LA" --style "markdown"
```

### Planning & Productivity

#### plan
Create structured plans and break down complex tasks.

**Arguments:**
- `task` (required) - The task to plan for
- `context` (default: "") - Additional context for the planning
- `constraints` (default: "none") - Any constraints or limitations to consider

**Example:**
```bash
swissarmyhammer test plan --task "Implement user authentication" --constraints "Must use OAuth2"
```

#### task-formatter
Format and organize tasks with priorities and grouping.

**Arguments:**
- `tasks` (required) - Comma-separated list of tasks
- `group_by` (default: "none") - How to group tasks (priority, status, category, none)
- `show_index` (default: "true") - Show task numbers
- `show_status` (default: "true") - Include status checkboxes
- `date_format` (default: "%B %d, %Y") - Date format for due dates

**Example:**
```bash
swissarmyhammer test task-formatter --tasks "Write tests,Fix bug,Update docs" --group_by "priority"
```

### Prompt Management

#### prompts/create
Help create effective prompts for SwissArmyHammer.

**Arguments:**
- `purpose` (required) - What the prompt should accomplish
- `category` (default: "general") - Category for the prompt
- `inputs_needed` (default: "") - What information the prompt needs from users
- `complexity` (default: "moderate") - Complexity level (simple, moderate, advanced)

**Example:**
```bash
swissarmyhammer test prompts/create --purpose "Generate database migrations" --category "database"
```

#### prompts/improve
Analyze and enhance existing prompts for better effectiveness.

**Arguments:**
- `prompt_content` (required) - The current prompt content (including YAML front matter)
- `improvement_goals` (default: "overall enhancement") - What aspects to improve
- `user_feedback` (default: "") - Any feedback or issues users have reported

**Example:**
```bash
swissarmyhammer test prompts/improve --prompt_content "$(cat my-prompt.md)" --improvement_goals "clarity,flexibility"
```

### Refactoring

#### refactor/clean
Refactor code for better readability, maintainability, and adherence to best practices.

**Arguments:**
- `code` (required) - The code to refactor
- `language` (default: "auto-detect") - Programming language
- `focus_areas` (default: "all") - Specific areas to focus on
- `style_guide` (default: "language defaults") - Specific style guide to follow

**Example:**
```bash
swissarmyhammer test refactor/clean --code "$(cat messy_code.py)" --focus_areas "naming,complexity"
```

#### refactor/extract
Extract code into well-named, reusable methods or functions.

**Arguments:**
- `code` (required) - The code containing logic to extract
- `extract_purpose` (required) - What the extracted method should do
- `method_name` (default: "auto-suggest") - Suggested name for the extracted method
- `scope` (default: "method") - Scope for the extraction (method, function, class, module)

**Example:**
```bash
swissarmyhammer test refactor/extract --code "$(cat complex.js)" --extract_purpose "validate user input"
```

#### refactor/patterns
Refactor code to match a target pattern or improve structure.

**Arguments:**
- `code` (required) - The code to refactor
- `target_pattern` (required) - The pattern or style to refactor towards

**Example:**
```bash
swissarmyhammer test refactor/patterns --code "$(cat service.py)" --target_pattern "Repository pattern"
```

### Code Review

#### review/code
Review code for quality, bugs, and improvements.

**Arguments:**
- `file_path` (required) - Path to the file being reviewed
- `context` (default: "general review") - Additional context about the code review focus

**Example:**
```bash
swissarmyhammer test review/code --file_path "src/auth.py" --context "focus on security"
```

#### review/code-dynamic
Language-specific code review with conditional logic.

**Arguments:**
- `file_path` (required) - Path to the file being reviewed
- `language` (required) - Programming language (python, javascript, rust, etc.)
- `focus_areas` (default: "style,bugs,performance") - Comma-separated list of areas to focus on
- `severity_level` (default: "warning") - Minimum severity level to report (info, warning, error)
- `include_suggestions` (default: "true") - Include code improvement suggestions

**Example:**
```bash
swissarmyhammer test review/code-dynamic --file_path "app.js" --language "javascript" --focus_areas "security,performance"
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
swissarmyhammer test review/security --code "$(cat login.php)" --context "handles user authentication"
```

#### review/accessibility
Review code for accessibility compliance and best practices.

**Arguments:**
- `code` (required) - The UI/frontend code to review
- `wcag_level` (default: "AA") - WCAG compliance level target (A, AA, AAA)
- `component_type` (default: "general") - Type of component (form, navigation, content, interactive)
- `target_users` (default: "all users") - Specific user needs to consider

**Example:**
```bash
swissarmyhammer test review/accessibility --code "$(cat form.html)" --component_type "form" --wcag_level "AA"
```

### Testing

#### test/unit
Create comprehensive unit tests for code with good coverage.

**Arguments:**
- `code` (required) - The code to generate tests for
- `framework` (default: "auto-detect") - Testing framework to use
- `style` (default: "BDD") - Testing style (BDD, TDD, classical)
- `coverage_target` (default: "80") - Target test coverage percentage

**Example:**
```bash
swissarmyhammer test test/unit --code "$(cat calculator.py)" --framework "pytest" --style "BDD"
```

#### test/integration
Create integration tests to verify component interactions.

**Arguments:**
- `system_description` (required) - Description of the system/components to test
- `test_scenarios` (default: "basic flow") - Specific scenarios to test (comma-separated)
- `framework` (default: "auto-detect") - Testing framework to use
- `environment` (default: "local") - Test environment setup requirements

**Example:**
```bash
swissarmyhammer test test/integration --system_description "User service and database" --test_scenarios "user creation,user update"
```

#### test/property
Create property-based tests to find edge cases automatically.

**Arguments:**
- `code` (required) - The code to test with properties
- `framework` (default: "auto-detect") - Property testing framework
- `properties_to_test` (default: "common properties") - Specific properties or invariants to verify
- `num_examples` (default: "100") - Number of random examples to generate

**Example:**
```bash
swissarmyhammer test test/property --code "$(cat sort.js)" --properties_to_test "output length equals input length"
```

### General Purpose

#### help
A prompt for providing helpful assistance and guidance to users.

**Arguments:**
- `topic` (default: "general assistance") - The topic to get help about
- `detail_level` (default: "normal") - How detailed the help should be

**Example:**
```bash
swissarmyhammer test help --topic "git workflows" --detail_level "detailed"
```

#### example
An example prompt for testing.

**Arguments:**
- `topic` (default: "general topic") - The topic to ask about

**Example:**
```bash
swissarmyhammer test example --topic "testing prompts"
```

## Usage Patterns

### Basic Usage
```bash
# Use a prompt with default arguments
swissarmyhammer test review/code --file_path "main.py"

# Specify custom arguments
swissarmyhammer test test/unit --code "$(cat utils.js)" --framework "jest" --coverage_target "90"
```

### Piping Content
```bash
# Pipe file content to a prompt
cat error.log | xargs -I {} swissarmyhammer test debug/logs --log_content "{}" --issue_description "memory leak"

# Use command substitution
swissarmyhammer test docs/api --code "$(cat api.py)" --api_type "REST"
```

### Combining Multiple Prompts
```bash
# First analyze the code
swissarmyhammer test review/code --file_path "service.py" > review.md

# Then generate tests based on the review
swissarmyhammer test test/unit --code "$(cat service.py)" --style "TDD"
```

### Custom Workflows
```bash
# Create a security-focused workflow
swissarmyhammer test review/security --code "$(cat auth.js)" --severity_threshold "medium" > security.md
swissarmyhammer test test/unit --code "$(cat auth.js)" --focus "security edge cases"
```

## Best Practices

1. **Choose the Right Prompt** - Select prompts that match your specific task
2. **Provide Context** - Use optional arguments to give more context
3. **Combine Prompts** - Use multiple prompts in sequence for comprehensive workflows
4. **Customize Arguments** - Override defaults when you need specific behavior
5. **Review Output** - Always review and validate generated content before using it

## Creating Custom Prompts

If the built-in prompts don't meet your needs:

1. Use the `prompts/create` prompt to generate a template
2. Save it in your `~/.swissarmyhammer/prompts/` directory
3. Follow the YAML front matter format for consistency
4. Test with various inputs to ensure reliability

For more information on creating custom prompts, see [Creating Prompts](./creating-prompts.md).