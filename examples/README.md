# SwissArmyHammer Example Prompts

This directory contains a collection of example prompts that demonstrate the power and flexibility of SwissArmyHammer. These prompts cover various use cases and can serve as inspiration for creating your own prompt library.

## Categories

### 📱 Development
Prompts for software development tasks:
- **[api-design.md](prompts/development/api-design.md)** - REST API design assistant
- **[database-schema.md](prompts/development/database-schema.md)** - Database schema designer

### ✍️ Writing
Prompts for content creation and writing:
- **[blog-post.md](prompts/writing/blog-post.md)** - Blog post generator

### 🔬 Research
Prompts for academic and professional research:
- **[literature-review.md](prompts/research/literature-review.md)** - Literature review assistant

### 📋 Productivity
Prompts for workplace productivity and organization:
- **[meeting-agenda.md](prompts/productivity/meeting-agenda.md)** - Meeting agenda creator

## How to Use These Examples

### 1. Copy to Your Prompt Library

```bash
# Copy specific prompts
cp examples/prompts/development/api-design.md ~/.swissarmyhammer/prompts/

# Copy entire categories
cp -r examples/prompts/development/ ~/.swissarmyhammer/prompts/

# Copy all examples
cp -r examples/prompts/* ~/.swissarmyhammer/prompts/
```

### 2. Customize for Your Needs

Each prompt can be customized by:
- Modifying the arguments to match your specific requirements
- Adjusting the prompt content for your domain or style
- Adding additional template variables
- Changing the structure or format

### 3. Create Your Own Categories

Organize prompts into categories that make sense for your workflow:

```bash
mkdir -p ~/.swissarmyhammer/prompts/{marketing,design,analysis,planning}
```

## Prompt Structure

Each example follows the SwissArmyHammer prompt format:

```markdown
---
title: Prompt Title
description: What this prompt does
arguments:
  - name: argument_name
    description: What this argument is for
    required: true/false
    default: "optional default value"
---

# Prompt Content

Use {{argument_name}} to insert variables into your prompt content.
```

## Contributing Examples

We welcome contributions of new example prompts! If you create useful prompts, consider:

1. **Sharing with the community** - Submit a pull request with new examples
2. **Following the format** - Use consistent YAML front matter and clear descriptions
3. **Including documentation** - Add clear descriptions and use cases
4. **Testing thoroughly** - Make sure your prompts work as expected

## Best Practices

When creating your own prompts based on these examples:

### ✅ Do's
- Use descriptive titles and descriptions
- Provide sensible defaults for optional arguments
- Structure content with clear sections and formatting
- Include examples and context in your prompts
- Test with different argument combinations

### ❌ Don'ts
- Make all arguments required unless truly necessary
- Create overly rigid prompts without flexibility
- Forget to validate your YAML syntax
- Use unclear or ambiguous argument names

## Getting Started

1. **Install SwissArmyHammer** - See the [installation guide](https://wballard.github.io/swissarmyhammer/installation.html)
2. **Copy example prompts** to your prompt directory
3. **Configure Claude Code** to use SwissArmyHammer
4. **Start using the prompts** in your daily workflow
5. **Customize and create** your own prompts

## More Resources

- **[Documentation](https://wballard.github.io/swissarmyhammer)** - Complete SwissArmyHammer guide
- **[Built-in Prompts](https://wballard.github.io/swissarmyhammer/builtin-prompts.html)** - Prompts included with SwissArmyHammer
- **[Creating Prompts](https://wballard.github.io/swissarmyhammer/creating-prompts.html)** - Detailed prompt creation guide
- **[GitHub Repository](https://github.com/wballard/swissarmyhammer)** - Source code and issues

---

**Happy prompting! 🔨**