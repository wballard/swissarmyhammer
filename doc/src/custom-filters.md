# Custom Filters Reference

SwissArmyHammer includes a comprehensive set of custom Liquid filters designed specifically for prompt engineering and content processing.

## Code Filters

### format_lang

Formats code with language-specific syntax highlighting hints.

```liquid
{{ code | format_lang: "rust" }}
{{ code | format_lang: language_var }}
```

**Arguments:**
- `language` - Programming language identifier (e.g., "rust", "python", "javascript")

**Example:**
```liquid
<!-- Input -->
{{ "fn main() { println!(\"Hello\"); }" | format_lang: "rust" }}

<!-- Output -->
```rust
fn main() { println!("Hello"); }
```
```

### extract_functions

Extracts function signatures and definitions from code.

```liquid
{{ code | extract_functions }}
{{ code | extract_functions: "detailed" }}
```

**Arguments:**
- `mode` (optional) - "signatures" (default) or "detailed" for full function bodies

**Example:**
```liquid
<!-- Input -->
{{ rust_code | extract_functions }}

<!-- Output -->
- fn main()
- fn calculate(x: i32, y: i32) -> i32
- fn process_data(data: &Vec<String>) -> Result<(), Error>
```

### basename

Extracts the filename from a file path.

```liquid
{{ path | basename }}
```

**Example:**
```liquid
<!-- Input -->
{{ "/usr/local/bin/swissarmyhammer" | basename }}

<!-- Output -->
swissarmyhammer
```

### dirname

Extracts the directory path from a file path.

```liquid
{{ path | dirname }}
```

**Example:**
```liquid
<!-- Input -->
{{ "/usr/local/bin/swissarmyhammer" | dirname }}

<!-- Output -->
/usr/local/bin
```

### count_lines

Counts the number of lines in text.

```liquid
{{ text | count_lines }}
```

**Example:**
```liquid
<!-- Input -->
{{ "line 1\nline 2\nline 3" | count_lines }}

<!-- Output -->
3
```

### dedent

Removes common leading whitespace from all lines.

```liquid
{{ code | dedent }}
```

**Example:**
```liquid
<!-- Input -->
{{ "    fn main() {\n        println!(\"Hello\");\n    }" | dedent }}

<!-- Output -->
fn main() {
    println!("Hello");
}
```

## Text Processing Filters

### extract_urls

Extracts all URLs from text.

```liquid
{{ text | extract_urls }}
{{ text | extract_urls: "list" }}
```

**Arguments:**
- `format` (optional) - "array" (default) or "list" for bullet point list

**Example:**
```liquid
<!-- Input -->
{{ "Visit https://example.com and https://github.com" | extract_urls }}

<!-- Output -->
["https://example.com", "https://github.com"]

<!-- With list format -->
{{ "Visit https://example.com and https://github.com" | extract_urls: "list" }}

<!-- Output -->
- https://example.com
- https://github.com
```

### slugify

Converts text to a URL-friendly slug.

```liquid
{{ title | slugify }}
```

**Example:**
```liquid
<!-- Input -->
{{ "Advanced Code Review Helper!" | slugify }}

<!-- Output -->
advanced-code-review-helper
```

### word_wrap

Wraps text at specified column width.

```liquid
{{ text | word_wrap: 80 }}
{{ text | word_wrap: width_var }}
```

**Arguments:**
- `width` - Column width for wrapping (default: 80)

**Example:**
```liquid
<!-- Input -->
{{ "This is a very long line that should be wrapped at a specific column width to ensure readability." | word_wrap: 30 }}

<!-- Output -->
This is a very long line that
should be wrapped at a specific
column width to ensure
readability.
```

### indent

Indents all lines by specified number of spaces.

```liquid
{{ text | indent: 4 }}
{{ text | indent: spaces_var }}
```

**Arguments:**
- `spaces` - Number of spaces to indent (default: 2)

**Example:**
```liquid
<!-- Input -->
{{ "line 1\nline 2" | indent: 4 }}

<!-- Output -->
    line 1
    line 2
```

### bullet_list

Converts an array to a bullet point list.

```liquid
{{ array | bullet_list }}
{{ array | bullet_list: "*" }}
```

**Arguments:**
- `bullet` (optional) - Bullet character (default: "-")

**Example:**
```liquid
<!-- Input -->
{{ ["Item 1", "Item 2", "Item 3"] | bullet_list }}

<!-- Output -->
- Item 1
- Item 2
- Item 3

<!-- With custom bullet -->
{{ ["Item 1", "Item 2"] | bullet_list: "*" }}

<!-- Output -->
* Item 1
* Item 2
```

### highlight

Highlights specific terms in text.

```liquid
{{ text | highlight: "keyword" }}
{{ text | highlight: term_var }}
```

**Arguments:**
- `term` - Term to highlight

**Example:**
```liquid
<!-- Input -->
{{ "This is important text with keywords" | highlight: "important" }}

<!-- Output -->
This is **important** text with keywords
```

## Data Transformation Filters

### from_json

Parses JSON string into object/array.

```liquid
{{ json_string | from_json }}
```

**Example:**
```liquid
<!-- Input -->
{% assign data = '{"name": "John", "age": 30}' | from_json %}
Name: {{ data.name }}
Age: {{ data.age }}

<!-- Output -->
Name: John
Age: 30
```

### to_json

Converts object/array to JSON string.

```liquid
{{ data | to_json }}
{{ data | to_json: "pretty" }}
```

**Arguments:**
- `format` (optional) - "compact" (default) or "pretty" for formatted output

**Example:**
```liquid
<!-- Input -->
{% assign user = { "name": "John", "age": 30 } %}
{{ user | to_json: "pretty" }}

<!-- Output -->
{
  "name": "John",
  "age": 30
}
```

### from_csv

Parses CSV string into array of objects.

```liquid
{{ csv_string | from_csv }}
{{ csv_string | from_csv: ";" }}
```

**Arguments:**
- `delimiter` (optional) - Field delimiter (default: ",")

**Example:**
```liquid
<!-- Input -->
{% assign data = "name,age\nJohn,30\nJane,25" | from_csv %}
{% for row in data %}
- {{ row.name }} is {{ row.age }} years old
{% endfor %}

<!-- Output -->
- John is 30 years old
- Jane is 25 years old
```

### to_csv

Converts array of objects to CSV string.

```liquid
{{ array | to_csv }}
{{ array | to_csv: ";" }}
```

**Arguments:**
- `delimiter` (optional) - Field delimiter (default: ",")

**Example:**
```liquid
<!-- Input -->
{% assign users = [{"name": "John", "age": 30}, {"name": "Jane", "age": 25}] %}
{{ users | to_csv }}

<!-- Output -->
name,age
John,30
Jane,25
```

### from_yaml

Parses YAML string into object/array.

```liquid
{{ yaml_string | from_yaml }}
```

**Example:**
```liquid
<!-- Input -->
{% assign config = "database:\n  host: localhost\n  port: 5432" | from_yaml %}
Host: {{ config.database.host }}
Port: {{ config.database.port }}

<!-- Output -->
Host: localhost
Port: 5432
```

### to_yaml

Converts object/array to YAML string.

```liquid
{{ data | to_yaml }}
```

**Example:**
```liquid
<!-- Input -->
{% assign config = {"database": {"host": "localhost", "port": 5432}} %}
{{ config | to_yaml }}

<!-- Output -->
database:
  host: localhost
  port: 5432
```

## Hash Filters

### md5

Generates MD5 hash of input text.

```liquid
{{ text | md5 }}
```

**Example:**
```liquid
<!-- Input -->
{{ "hello world" | md5 }}

<!-- Output -->
5d41402abc4b2a76b9719d911017c592
```

### sha1

Generates SHA1 hash of input text.

```liquid
{{ text | sha1 }}
```

**Example:**
```liquid
<!-- Input -->
{{ "hello world" | sha1 }}

<!-- Output -->
2aae6c35c94fcfb415dbe95f408b9ce91ee846ed
```

### sha256

Generates SHA256 hash of input text.

```liquid
{{ text | sha256 }}
```

**Example:**
```liquid
<!-- Input -->
{{ "hello world" | sha256 }}

<!-- Output -->
b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
```

## Utility Filters

### ordinal

Converts number to ordinal format (1st, 2nd, 3rd, etc.).

```liquid
{{ number | ordinal }}
```

**Example:**
```liquid
<!-- Input -->
{{ 1 | ordinal }} item
{{ 22 | ordinal }} place
{{ 103 | ordinal }} attempt

<!-- Output -->
1st item
22nd place
103rd attempt
```

### lorem_words

Generates lorem ipsum text with specified number of words.

```liquid
{{ count | lorem_words }}
```

**Arguments:**
- `count` - Number of words to generate

**Example:**
```liquid
<!-- Input -->
{{ 10 | lorem_words }}

<!-- Output -->
Lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod
```

### format_date

Advanced date formatting with custom format strings.

```liquid
{{ date | format_date: "%Y-%m-%d %H:%M:%S" }}
{{ "now" | format_date: "%B %d, %Y" }}
```

**Arguments:**
- `format` - Date format string (uses strftime format)

**Common format codes:**
- `%Y` - 4-digit year (2024)
- `%m` - Month as number (01-12)
- `%d` - Day of month (01-31)
- `%H` - Hour (00-23)
- `%M` - Minute (00-59)
- `%S` - Second (00-59)
- `%B` - Full month name (January)
- `%b` - Abbreviated month (Jan)
- `%A` - Full weekday name (Monday)
- `%a` - Abbreviated weekday (Mon)

**Example:**
```liquid
<!-- Input -->
{{ "2024-01-15T10:30:00Z" | format_date: "%B %d, %Y at %I:%M %p" }}
{{ "now" | format_date: "%A, %Y-%m-%d" }}

<!-- Output -->
January 15, 2024 at 10:30 AM
Monday, 2024-01-15
```

## Filter Chaining

Filters can be chained together for complex transformations:

```liquid
{{ code | dedent | format_lang: "python" | highlight: "def" }}

{{ user_input | strip | truncate: 100 | capitalize }}

{{ data | to_json | indent: 2 }}

{{ filename | basename | slugify | append: ".md" }}
```

## Error Handling

Custom filters handle errors gracefully:

- **Invalid input**: Returns original value or empty string
- **Missing arguments**: Uses sensible defaults
- **Type mismatches**: Attempts conversion or returns original value

## Performance Notes

- **Hash filters** (md5, sha1, sha256) are computationally expensive for large inputs
- **Data transformation filters** (JSON, CSV, YAML) may consume memory for large datasets
- **Text processing filters** are optimized for typical prompt content sizes
- **Code filters** use efficient parsing algorithms

## Integration Examples

### Code Review Prompt
```liquid
# Code Review: {{ filename | basename }}

## File Information
- **Path**: {{ filepath }}
- **Lines**: {{ code | count_lines }}
- **Language**: {{ language | capitalize }}

## Code to Review
{{ code | dedent | format_lang: language }}

## Functions Found
{{ code | extract_functions | bullet_list }}

## Review Checklist
{% assign hash = code | sha256 | truncate: 8 %}
- [ ] Security review (ID: {{ hash }})
- [ ] Performance analysis
- [ ] Style compliance
```

### Data Analysis Prompt
```liquid
# Data Analysis Report

## Dataset Summary
{% assign data = csv_data | from_csv %}
- **Records**: {{ data | size }}
- **Generated**: {{ "now" | format_date: "%Y-%m-%d %H:%M" }}

## Sample Data
{% for row in data limit:3 %}
{{ forloop.index | ordinal }} record: {{ row | to_json }}
{% endfor %}

## Field Analysis
{% assign fields = data[0] | keys %}
Available fields: {{ fields | bullet_list }}
```

### Documentation Generator
```liquid
# API Documentation

## Endpoints
{% for endpoint in api_endpoints %}
### {{ endpoint.method | upcase }} {{ endpoint.path }}

{{ endpoint.description | word_wrap: 80 }}

{% if endpoint.parameters %}
**Parameters:**
{{ endpoint.parameters | to_yaml | indent: 2 }}
{% endif %}

**Example:**
```{{ endpoint.language | default: "bash" }}
{{ endpoint.example | dedent }}
```
{% endfor %}

---
*Generated on {{ "now" | format_date: "%B %d, %Y" }}*
```

## See Also

- [Template Variables](./template-variables.md) - Basic Liquid syntax
- [Advanced Prompts](./advanced-prompts.md) - Using filters in complex templates
- [Testing Guide](./testing-guide.md) - Testing templates with custom filters