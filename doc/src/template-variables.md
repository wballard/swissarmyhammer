# Template Variables

SwissArmyHammer uses the [Liquid template engine](https://shopify.github.io/liquid/) for processing prompts. This provides powerful templating features including variables, conditionals, loops, and filters.

## Basic Variable Substitution

Variables are inserted using double curly braces:

```liquid
Hello {{ name }}!
Your email is {{ email }}.
```

For backward compatibility, variables without spaces also work:

```liquid
Hello {{name}}!
```

## Conditionals

### If Statements

Use `if` statements to conditionally include content:

```liquid
{% if user_type == "admin" %}
  Welcome, administrator!
{% elsif user_type == "moderator" %}
  Welcome, moderator!
{% else %}
  Welcome, user!
{% endif %}
```

### Unless Statements

`unless` is the opposite of `if`:

```liquid
{% unless error_count == 0 %}
  Warning: {{ error_count }} errors found.
{% endunless %}
```

### Comparison Operators

- `==` - equals
- `!=` - not equals
- `>` - greater than
- `<` - less than
- `>=` - greater or equal
- `<=` - less or equal
- `contains` - string/array contains
- `and` - logical AND
- `or` - logical OR

Example:
```liquid
{% if age >= 18 and country == "US" %}
  You are eligible to vote.
{% endif %}

{% if tags contains "urgent" %}
  🚨 This is urgent!
{% endif %}
```

## Case Statements

For multiple conditions, use `case`:

```liquid
{% case status %}
  {% when "pending" %}
    ⏳ Waiting for approval
  {% when "approved" %}
    ✅ Approved and ready
  {% when "rejected" %}
    ❌ Rejected
  {% else %}
    ❓ Unknown status
{% endcase %}
```

## Loops

### Basic For Loops

Iterate over arrays:

```liquid
{% for item in items %}
  - {{ item }}
{% endfor %}
```

### Range Loops

Loop over a range of numbers:

```liquid
{% for i in (1..5) %}
  Step {{ i }} of 5
{% endfor %}
```

### Loop Variables

Inside loops, you have access to special variables:

```liquid
{% for item in items %}
  {% if forloop.first %}First item: {% endif %}
  {{ forloop.index }}. {{ item }}
  {% if forloop.last %}(last item){% endif %}
{% endfor %}
```

Available loop variables:
- `forloop.index` - current iteration (1-based)
- `forloop.index0` - current iteration (0-based)
- `forloop.first` - true on first iteration
- `forloop.last` - true on last iteration
- `forloop.length` - total number of items

### Loop Control

Use `break` and `continue` for flow control:

```liquid
{% for item in items %}
  {% if item == "skip" %}
    {% continue %}
  {% endif %}
  {% if item == "stop" %}
    {% break %}
  {% endif %}
  Processing: {{ item }}
{% endfor %}
```

### Cycle

Alternate between values:

```liquid
{% for row in data %}
  <tr class="{% cycle 'odd', 'even' %}">
    <td>{{ row }}</td>
  </tr>
{% endfor %}
```

## Filters

Filters modify variables using the pipe (`|`) character:

### String Filters

```liquid
{{ name | upcase }}           # ALICE
{{ name | downcase }}         # alice
{{ name | capitalize }}       # Alice
{{ text | strip }}            # removes whitespace
{{ text | truncate: 20 }}     # truncates to 20 chars
{{ text | truncate: 20, "..." }} # custom ellipsis
{{ text | append: "!" }}      # adds to end
{{ text | prepend: "Hello " }} # adds to beginning
{{ text | remove: "bad" }}    # removes all occurrences
{{ text | replace: "old", "new" }} # replaces all
{{ text | split: "," }}       # splits into array
```

### Array Filters

```liquid
{{ array | first }}           # first element
{{ array | last }}            # last element
{{ array | join: ", " }}      # joins with delimiter
{{ array | sort }}            # sorts array
{{ array | reverse }}         # reverses array
{{ array | size }}            # number of elements
{{ array | uniq }}            # removes duplicates
```

### Math Filters

```liquid
{{ number | plus: 5 }}        # addition
{{ number | minus: 3 }}       # subtraction
{{ number | times: 2 }}       # multiplication
{{ number | divided_by: 4 }}  # division
{{ number | modulo: 3 }}      # remainder
{{ number | ceil }}           # round up
{{ number | floor }}          # round down
{{ number | round }}          # round to nearest
{{ number | round: 2 }}       # round to 2 decimals
{{ number | abs }}            # absolute value
```

### Default Filter

Provide fallback values:

```liquid
Hello {{ name | default: "Guest" }}!
Score: {{ score | default: 0 }}
```

### Date Filters

```liquid
{{ date | date: "%Y-%m-%d" }}         # 2024-01-15
{{ date | date: "%B %d, %Y" }}        # January 15, 2024
{{ "now" | date: "%Y" }}              # current year
```

## Advanced Features

### Comments

Comments are not rendered in output:

```liquid
{% comment %}
  This is a comment that won't appear in the output.
  Useful for documentation or temporarily disabling code.
{% endcomment %}
```

### Raw Blocks

Prevent Liquid processing:

```liquid
{% raw %}
  This {{ variable }} won't be processed.
  Useful for showing Liquid syntax examples.
{% endraw %}
```

### Assign Variables

Create new variables:

```liquid
{% assign full_name = first_name | append: " " | append: last_name %}
Welcome, {{ full_name }}!

{% assign item_count = items | size %}
You have {{ item_count }} items.
```

### Capture Blocks

Capture content into a variable:

```liquid
{% capture greeting %}
  {% if time_of_day == "morning" %}
    Good morning
  {% elsif time_of_day == "evening" %}
    Good evening
  {% else %}
    Hello
  {% endif %}
{% endcapture %}

{{ greeting }}, {{ name }}!
```

## Environment Variables

Access environment variables through the `env` object:

```liquid
Current user: {{ env.USER }}
Home directory: {{ env.HOME }}
Custom setting: {{ env.MY_APP_CONFIG | default: "not set" }}
```

## Object Access

Access nested objects and arrays:

```liquid
{{ user.name }}
{{ user.address.city }}
{{ items[0] }}
{{ items[index] }}
{{ data["dynamic_key"] }}
```

## Truthy and Falsy Values

In Liquid conditions:
- **Falsy**: `false`, `nil`
- **Truthy**: everything else (including `0`, `""`, `[]`)

```liquid
{% if value %}
  This shows unless value is false or nil
{% endif %}
```

## Error Handling

When a variable is undefined:
- In backward-compatible mode: `{{ undefined }}` renders as `{{ undefined }}`
- With validation: An error is raised for missing required arguments

Use the `default` filter to handle missing values gracefully:

```liquid
{{ optional_var | default: "fallback value" }}
```

## Partials

SwissArmyHammer supports partials (template fragments) that can be included in other templates. This allows you to create reusable components and organize your templates better.

### Creating Partials

To create a partial, add the `{% partial %}` tag at the beginning of your template file:

```liquid
{% partial %}

## Code Review Guidelines

Please review the following code for:
- Syntax errors
- Logic issues
- Best practices
- Performance concerns
```

### Using Partials

Use the `{% render %}` tag to include partials in your templates:

```liquid
# Main Template

{% render "code-review-guidelines" %}

## Code to Review

```{{ language }}
{{ code }}
```

Please focus on {{ focus_area }}.
```

### Partial Resolution

SwissArmyHammer automatically resolves partials based on your prompt library:

1. **Exact name match**: `{% render "my-partial" %}` looks for `my-partial`
2. **With extensions**: Also tries `my-partial.md`, `my-partial.liquid`, `my-partial.md.liquid`, `my-partial.liquid.markdown`
3. **Without extensions**: If you have `my-partial.md.liquid`, you can reference it as `{% render "my-partial" %}`

### Supported File Extensions

Partials can use any of these extensions:
- `.md` - Markdown files
- `.liquid` - Liquid template files
- `.md.liquid` - Markdown with Liquid processing
- `.liquid.markdown` - Liquid with Markdown processing

### Organizing Partials

You can organize partials in subdirectories:

```
prompts/
├── main-template.md.liquid
├── partials/
│   ├── header.liquid
│   ├── footer.liquid
│   └── code-review/
│       ├── guidelines.md.liquid
│       └── checklist.liquid
└── shared/
    └── common-footer.md
```

Reference them with their relative path:

```liquid
{% render "partials/header" %}
{% render "partials/code-review/guidelines" %}
{% render "shared/common-footer" %}
```

### Partials with Context

Partials have access to the same variables as the parent template:

**Parent template:**
```liquid
{% assign project_name = "SwissArmyHammer" %}
{% assign language = "Rust" %}

{% render "project-info" %}
```

**Partial (project-info.liquid):**
```liquid
{% partial %}

## Project: {{ project_name }}

This {{ language }} project follows strict coding standards.
```

### Examples

#### Basic Partial Usage

**footer.liquid:**
```liquid
{% partial %}

---
Generated by SwissArmyHammer
Report issues at: https://github.com/project/issues
```

**main-template.md.liquid:**
```liquid
# Code Review for {{ file_name }}

Please review the following code:

```{{ language }}
{{ code }}
```

{% render "footer" %}
```

#### Conditional Partials

```liquid
{% if include_security_check %}
  {% render "security-checklist" %}
{% endif %}

{% if language == "rust" %}
  {% render "rust-specific-guidelines" %}
{% elsif language == "python" %}
  {% render "python-specific-guidelines" %}
{% endif %}
```

#### Nested Partials

Partials can include other partials:

**main-header.liquid:**
```liquid
{% partial %}

{% render "project-info" %}
{% render "timestamp" %}

---
```

**project-info.liquid:**
```liquid
{% partial %}

# {{ project_name | default: "Unknown Project" }}
Version: {{ version | default: "1.0.0" }}
```

#### Partials in Loops

```liquid
{% for task in tasks %}
  {% render "task-template" %}
{% endfor %}
```

**task-template.liquid:**
```liquid
{% partial %}

## Task: {{ task.title }}
Status: {{ task.status }}
Priority: {{ task.priority | default: "normal" }}

{% if task.description %}
Description: {{ task.description }}
{% endif %}
```

### Best Practices

1. **Use the `{% partial %}` tag**: Always start partial files with `{% partial %}` to clearly mark them as partials
2. **Meaningful names**: Use descriptive names for partials (`code-review-guidelines` instead of `guidelines`)
3. **Organize by function**: Group related partials in subdirectories
4. **Keep partials focused**: Each partial should have a single responsibility
5. **Document dependencies**: If a partial expects certain variables, document them
6. **Test partials**: Test partials with different variable contexts

### Common Use Cases

#### Shared Headers and Footers

Create consistent headers and footers across multiple prompts:

```liquid
{% render "shared/header" %}

{{ main_content }}

{% render "shared/footer" %}
```

#### Language-Specific Templates

```liquid
{% case language %}
  {% when "rust" %}
    {% render "languages/rust-template" %}
  {% when "python" %}
    {% render "languages/python-template" %}
  {% when "javascript" %}
    {% render "languages/js-template" %}
  {% else %}
    {% render "languages/generic-template" %}
{% endcase %}
```

#### Conditional Content

```liquid
{% if debug_mode %}
  {% render "debug-info" %}
{% endif %}

{% if include_examples %}
  {% render "code-examples" %}
{% endif %}
```

### Troubleshooting

**Partial not found**: Check that the partial file exists in your prompt library and that the name matches exactly.

**Variables not available**: Partials use the same context as the parent template. Make sure variables are defined before rendering the partial.

**Infinite recursion**: Avoid having partials that include themselves or create circular dependencies.

## Migration from Basic Templates

If you're migrating from basic `{{variable}}` syntax:

1. **Your existing templates still work** - backward compatibility is maintained
2. **Add spaces for clarity**: `{{var}}` → `{{ var }}`
3. **Use filters for transformation**: `{{ name | upcase }}` instead of post-processing
4. **Add conditions for dynamic content**: Use `{% if %}` blocks
5. **Use loops for repetitive content**: Replace manual duplication with `{% for %}`

### Migration Examples

#### Before: Basic Variable Substitution
```
Please review the {{language}} code in {{file}}.
Focus on {{focus_area}}.
```

#### After: Enhanced with Liquid Features
```liquid
Please review the {{ language | capitalize }} code in {{ file }}.

{% if focus_area %}
Focus on {{ focus_area }}.
{% else %}
Perform a general code review.
{% endif %}

{% if language == "python" %}
Pay special attention to PEP 8 compliance.
{% elsif language == "javascript" %}
Check for ESLint rule violations.
{% endif %}
```

#### Before: Manual List Creation
```
Files to review:
- {{file1}}
- {{file2}}
- {{file3}}
```

#### After: Dynamic Lists with Loops
```liquid
Files to review:
{% for file in files %}
- {{ file }}{% if forloop.last %} (final file){% endif %}
{% endfor %}

Total: {{ files | size }} files
```

#### Before: Fixed Templates
```
Status: {{status}}
```

#### After: Conditional Formatting
```liquid
Status: {% case status %}
  {% when "success" %}✅ {{ status | upcase }}
  {% when "error" %}❌ {{ status | upcase }}
  {% when "warning" %}⚠️ {{ status | capitalize }}
  {% else %}{{ status }}
{% endcase %}
```

### Differences from Handlebars/Mustache

If you're familiar with Handlebars or Mustache templating:

| Feature | Handlebars/Mustache | Liquid |
|---------|---------------------|---------|
| Variables | `{{variable}}` | `{{ variable }}` |
| Conditionals | `{{#if}}...{{/if}}` | `{% if %}...{% endif %}` |
| Loops | `{{#each}}...{{/each}}` | `{% for %}...{% endfor %}` |
| Comments | `{{! comment }}` | `{% comment %}...{% endcomment %}` |
| Filters | Limited | Extensive built-in filters |
| Logic | Minimal | Full comparison operators |

### Common Migration Patterns

1. **Variable with Default**
   - Before: Handle missing variables in code
   - After: `{{ variable | default: "fallback" }}`

2. **Conditional Sections**
   - Before: Generate different templates
   - After: Single template with `{% if %}` blocks

3. **Repeated Content**
   - Before: Manual duplication
   - After: `{% for %}` loops with `forloop` variables

4. **String Transformation**
   - Before: Transform in application code
   - After: Use Liquid filters directly

### Backward Compatibility Notes

- Simple `{{variable}}` syntax continues to work
- Undefined variables are preserved as `{{ variable }}` in output
- No breaking changes to existing templates
- Gradual migration is supported - mix old and new syntax

## Examples

### Dynamic Code Review

```liquid
{% if language == "python" %}
  Please review this Python code for PEP 8 compliance.
{% elsif language == "javascript" %}
  Please review this JavaScript code for ESLint rules.
{% else %}
  Please review this {{ language }} code for best practices.
{% endif %}

{% if include_security %}
  Also check for security vulnerabilities.
{% endif %}
```

### Formatted List

```liquid
{% for item in tasks %}
  {{ forloop.index }}. {{ item.title }}
  {% if item.completed %}✓{% else %}○{% endif %}
  Priority: {{ item.priority | default: "normal" }}
  {% unless item.completed %}
    Due: {{ item.due_date | date: "%B %d" }}
  {% endunless %}
{% endfor %}
```

### Conditional Debugging

```liquid
{% if debug_mode %}
  === Debug Information ===
  Variables: {{ arguments | json }}
  Environment: {{ env.NODE_ENV | default: "development" }}
  {% for key in api_keys %}
    {{ key }}: {{ key | truncate: 8 }}...
  {% endfor %}
{% endif %}
```

## Best Practices

1. **Use meaningful variable names**: `{{ user_email }}` instead of `{{ ue }}`
2. **Provide defaults**: `{{ value | default: "N/A" }}` for optional values
3. **Format output**: Use filters to ensure consistent formatting
4. **Comment complex logic**: Use `{% comment %}` blocks
5. **Test edge cases**: Empty arrays, nil values, missing variables
6. **Keep it readable**: Break complex templates into sections

## Custom Filters

SwissArmyHammer includes specialized custom filters designed for prompt engineering:

### Code Filters
```liquid
{{ code | format_lang: "rust" }}      # Format code with language
{{ code | extract_functions }}        # Extract function signatures
{{ path | basename }}                 # Get filename from path
{{ path | dirname }}                  # Get directory from path
{{ text | count_lines }}              # Count number of lines
{{ code | dedent }}                   # Remove common indentation
```

### Text Processing Filters
```liquid
{{ text | extract_urls }}             # Extract URLs from text
{{ title | slugify }}                 # Convert to URL-friendly slug
{{ text | word_wrap: 80 }}            # Wrap text at 80 characters
{{ text | indent: 2 }}                # Indent all lines by 2 spaces
{{ items | bullet_list }}             # Convert array to bullet list
{{ text | highlight: "keyword" }}     # Highlight specific terms
```

### Data Transformation Filters
```liquid
{{ json_string | from_json }}         # Parse JSON string
{{ data | to_json }}                  # Convert to JSON string
{{ csv_string | from_csv }}           # Parse CSV string
{{ array | to_csv }}                  # Convert to CSV string
{{ yaml_string | from_yaml }}         # Parse YAML string
{{ data | to_yaml }}                  # Convert to YAML string
```

### Utility Filters
```liquid
{{ text | md5 }}                      # Generate MD5 hash
{{ text | sha1 }}                     # Generate SHA1 hash
{{ text | sha256 }}                   # Generate SHA256 hash
{{ number | ordinal }}                # Convert to ordinal (1st, 2nd, 3rd)
{{ 100 | lorem_words }}               # Generate lorem ipsum words
{{ date | format_date: "%Y-%m-%d" }}  # Advanced date formatting
```

For complete documentation of custom filters, see the [Custom Filters Reference](./custom-filters.md).

## Limitations

1. **No custom tags**: Only standard Liquid tags (plus SwissArmyHammer's `{% partial %}` and `{% render %}`) are supported
2. **Performance**: Very large loops may impact performance

## Further Reading

- [Official Liquid Documentation](https://shopify.github.io/liquid/)
- [Liquid Playground](https://liquidjs.com/playground.html) - Test templates online
- [Liquid Cheat Sheet](https://github.com/Shopify/liquid/wiki/Liquid-for-Designers)