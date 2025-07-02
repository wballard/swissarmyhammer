# Your First Prompt

Let's create your first custom prompt with SwissArmyHammer! This guide will walk you through creating a useful code review prompt.

## Understanding Prompt Structure

SwissArmyHammer prompts are markdown files with YAML front matter. Here's the basic structure:

```markdown
---
title: Your Prompt Title
description: What this prompt does
arguments:
  - name: argument_name
    description: What this argument is for
    required: true/false
    default: "optional default value"
---

# Your Prompt Content

Use {{argument_name}} to insert variables into your prompt.
```

## Creating a Code Review Prompt

Let's create a practical code review prompt step by step.

### Step 1: Create the File

First, create a new prompt file in your prompts directory:

```bash
# Create the file
touch ~/.swissarmyhammer/prompts/code-review.md

# Or create a category directory
mkdir -p ~/.swissarmyhammer/prompts/development
touch ~/.swissarmyhammer/prompts/development/code-review.md
```

### Step 2: Add the YAML Front Matter

Open the file in your favorite editor and add the front matter:

```yaml
---
title: Code Review Assistant
description: Comprehensive code review with focus on best practices, security, and performance
arguments:
  - name: code
    description: The code to review (can be a function, class, or entire file)
    required: true
  - name: language
    description: Programming language (helps with language-specific advice)
    required: false
    default: "auto-detect"
  - name: focus
    description: Areas to focus on (security, performance, readability, etc.)
    required: false
    default: "general best practices"
---
```

### Step 3: Write the Prompt Content

Below the front matter, add the prompt content:

```markdown
# Code Review

I need a thorough code review for the following {{language}} code.

## Code to Review

```{{language}}
{{code}}
```

## Review Focus

Please focus on: {{focus}}

## Review Criteria

Please analyze the code for:

### 🔒 Security
- Potential security vulnerabilities
- Input validation issues
- Authentication/authorization concerns

### 🚀 Performance
- Inefficient algorithms or operations
- Memory usage concerns
- Potential bottlenecks

### 📖 Readability & Maintainability
- Code clarity and organization
- Naming conventions
- Documentation needs

### 🧪 Testing & Reliability
- Error handling
- Edge cases
- Testability

### 🏗️ Architecture & Design
- SOLID principles adherence
- Design patterns usage
- Code structure

## Output Format

Please provide:

1. **Overall Assessment** - Brief summary of code quality
2. **Specific Issues** - List each issue with:
   - Severity (High/Medium/Low)
   - Location (line numbers if applicable)
   - Explanation of the problem
   - Suggested fix
3. **Positive Aspects** - What's done well
4. **Recommendations** - Broader suggestions for improvement

Focus especially on {{focus}} in your analysis.
```

### Step 4: Complete File Example

Here's the complete prompt file:

```markdown
---
title: Code Review Assistant
description: Comprehensive code review with focus on best practices, security, and performance
arguments:
  - name: code
    description: The code to review (can be a function, class, or entire file)
    required: true
  - name: language
    description: Programming language (helps with language-specific advice)
    required: false
    default: "auto-detect"
  - name: focus
    description: Areas to focus on (security, performance, readability, etc.)
    required: false
    default: "general best practices"
---

# Code Review

I need a thorough code review for the following {{language}} code.

## Code to Review

```{{language}}
{{code}}
```

## Review Focus

Please focus on: {{focus}}

## Review Criteria

Please analyze the code for:

### 🔒 Security
- Potential security vulnerabilities
- Input validation issues
- Authentication/authorization concerns

### 🚀 Performance
- Inefficient algorithms or operations
- Memory usage concerns
- Potential bottlenecks

### 📖 Readability & Maintainability
- Code clarity and organization
- Naming conventions
- Documentation needs

### 🧪 Testing & Reliability
- Error handling
- Edge cases
- Testability

### 🏗️ Architecture & Design
- SOLID principles adherence
- Design patterns usage
- Code structure

## Output Format

Please provide:

1. **Overall Assessment** - Brief summary of code quality
2. **Specific Issues** - List each issue with:
   - Severity (High/Medium/Low)
   - Location (line numbers if applicable)
   - Explanation of the problem
   - Suggested fix
3. **Positive Aspects** - What's done well
4. **Recommendations** - Broader suggestions for improvement

Focus especially on {{focus}} in your analysis.
```

## Step 5: Test Your Prompt

Save the file and test that SwissArmyHammer can load it:

```bash
# Check if your prompt loads correctly
swissarmyhammer doctor
```

The doctor command will validate your YAML syntax and confirm the prompt is loaded.

## Step 6: Use Your Prompt

1. **Open Claude Code**
2. **Start a new conversation**
3. **Look for your prompt** in the prompt picker - it should appear as "Code Review Assistant"
4. **Fill in the parameters**:
   - `code`: Paste some code you want reviewed
   - `language`: Specify the programming language (optional)
   - `focus`: Specify what to focus on (optional)

## Understanding What Happened

When you created this prompt, SwissArmyHammer:

1. **Detected the new file** using its file watcher
2. **Parsed the YAML front matter** to understand the prompt structure
3. **Made it available** to Claude Code via the MCP protocol
4. **Prepared for template substitution** when the prompt is used

## Best Practices for Your First Prompt

### ✅ Do's

- **Use descriptive titles and descriptions**
- **Document your arguments clearly**
- **Provide sensible defaults** for optional arguments
- **Structure your prompt content** with clear sections
- **Use template variables** to make prompts flexible

### ❌ Don'ts

- **Don't use required arguments unless necessary**
- **Don't make prompts too rigid** - allow for flexibility
- **Don't forget to test** your YAML syntax
- **Don't use overly complex template logic** in your first prompts

## Next Steps

Now that you've created your first prompt, you can:

1. **Create more prompts** for different use cases
2. **Organize prompts** into directories by category
3. **Learn advanced template features** like conditionals and loops
4. **Share prompts** with your team or the community

### Recommended Reading

- [Creating Prompts](./creating-prompts.md) - Comprehensive guide to prompt creation
- [Template Variables](./template-variables.md) - Advanced template features
- [Prompt Organization](./prompt-organization.md) - How to organize your prompt library
- [Built-in Prompts](./builtin-prompts.md) - Examples from the built-in library

## Troubleshooting

If your prompt isn't working:

1. **Check YAML syntax** - Make sure your front matter is valid YAML
2. **Run doctor** - `swissarmyhammer doctor` will catch common issues
3. **Check file permissions** - Make sure SwissArmyHammer can read the file
4. **Restart Claude Code** - Sometimes needed after creating new prompts

Common issues:
- **YAML indentation errors** - Use spaces, not tabs
- **Missing required fields** - Title and description are required
- **Invalid argument structure** - Check the argument format
- **File encoding** - Use UTF-8 encoding for your markdown files