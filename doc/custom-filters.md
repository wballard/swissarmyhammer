# Custom Liquid Filters

SwissArmyHammer extends the Liquid template engine with custom filters designed specifically for prompt engineering and development workflows.

## Code and Development Filters

### format_lang

Formats code with language-specific syntax highlighting.

```liquid
{{ code | format_lang: "python" }}
```

Supported languages: python, rust, javascript, typescript, go, java, c, cpp, and more.

### extract_functions

Extracts function names from code.

```liquid
Functions found: {{ code | extract_functions | join: ", " }}
```

### basename

Gets the base name from a file path.

```liquid
{{ "/path/to/file.txt" | basename }}
# Output: file.txt
```

### dirname

Gets the directory name from a file path.

```liquid
{{ "/path/to/file.txt" | dirname }}
# Output: /path/to
```

### count_lines

Counts the number of lines in text.

```liquid
This code has {{ code | count_lines }} lines
```

### count_tokens

Estimates the number of tokens (useful for LLM context windows).

```liquid
Token count: {{ text | count_tokens }}
```

### dedent

Removes common leading indentation from all lines.

```liquid
{{ code | dedent }}
```

## Text Processing Filters

### extract_urls

Finds all URLs in text.

```liquid
{% assign urls = text | extract_urls %}
Found {{ urls | size }} URLs:
{% for url in urls %}
- {{ url }}
{% endfor %}
```

### extract_emails

Finds all email addresses in text.

```liquid
{% assign emails = text | extract_emails %}
Contact: {{ emails | first }}
```

### slugify

Creates URL-friendly slugs from text.

```liquid
{{ "Hello World!" | slugify }}
# Output: hello-world
```

### word_wrap

Wraps text at word boundaries.

```liquid
{{ long_text | word_wrap: 80 }}
```

### indent

Adds indentation to each line.

```liquid
{{ text | indent: 4 }}
```

With custom indentation string:

```liquid
{{ text | indent: 2, "> " }}
```

### bullet_list

Converts lines to bullet points.

```liquid
{{ items | bullet_list }}
```

With custom bullet:

```liquid
{{ items | bullet_list: "* " }}
```

## Data Transformation Filters

### to_json

Converts data to JSON format.

```liquid
{{ data | to_json }}
```

Pretty printed:

```liquid
{{ data | to_json: true }}
```

### from_json

Parses JSON string into data.

```liquid
{% assign data = json_string | from_json %}
Name: {{ data.name }}
```

### from_csv

Parses CSV data.

```liquid
{% assign rows = csv_data | from_csv %}
{% for row in rows %}
  {{ row[0] }}: {{ row[1] }}
{% endfor %}
```

### from_yaml

Parses YAML data.

```liquid
{% assign config = yaml_string | from_yaml %}
Version: {{ config.version }}
```

### to_csv

Converts array data to CSV format.

```liquid
{{ data | to_csv }}
```

### keys

Gets all keys from a hash/object.

```liquid
{% assign all_keys = data | keys %}
Properties: {{ all_keys | join: ", " }}
```

### values

Gets all values from a hash/object.

```liquid
{% assign all_values = data | values %}
```

## Utility Filters

### format_date

Formats dates using strftime syntax.

```liquid
{{ date | format_date: "%Y-%m-%d" }}
{{ date | format_date: "%B %d, %Y" }}
```

### lorem

Generates Lorem Ipsum placeholder text.

```liquid
{{ 50 | lorem }}
# Generates 50 words of Lorem Ipsum
```

### ordinal

Converts numbers to ordinal form.

```liquid
{{ 1 | ordinal }} place  # 1st place
{{ 2 | ordinal }} place  # 2nd place
{{ 3 | ordinal }} place  # 3rd place
{{ 21 | ordinal }} birthday  # 21st birthday
```

### highlight

Highlights occurrences of a term in text.

```liquid
{{ text | highlight: "important" }}
```

With custom markers:

```liquid
{{ text | highlight: "important", "<mark>", "</mark>" }}
```

### sample

Random sampling from arrays.

```liquid
{% assign selection = items | sample: 3 %}
Random selection: {{ selection | join: ", " }}
```

## Combining Filters

Filters can be chained together for powerful transformations:

```liquid
# Extract and format function names
{{ code | extract_functions | sort | join: "\n" | bullet_list }}

# Process file paths
{{ file_list | split: "\n" | map: "basename" | uniq | sort }}

# Clean and format text
{{ user_input | strip | downcase | slugify }}

# Parse and transform data
{% assign data = json_response | from_json %}
{{ data.items | map: "name" | join: ", " }}
```

## Error Handling

All custom filters handle errors gracefully:

```liquid
# Invalid JSON returns empty object
{% assign data = invalid_json | from_json %}

# Non-existent language falls back to plain text
{{ code | format_lang: "unknown" }}

# Invalid numbers return 0
{{ "abc" | count_tokens }}
```

## Performance Tips

1. **Cache Results**: For expensive operations, assign to variables:
   ```liquid
   {% assign functions = code | extract_functions %}
   Found {{ functions | size }} functions: {{ functions | join: ", " }}
   ```

2. **Order Matters**: Place filtering operations efficiently:
   ```liquid
   # Good: filter first, then process
   {{ lines | compact | count_lines }}
   
   # Less efficient: process then filter
   {{ lines | count_lines | compact }}
   ```

3. **Use Built-ins**: Prefer built-in Liquid filters when available:
   ```liquid
   # Use built-in 'size' instead of custom 'count_lines' for arrays
   {{ array | size }}
   ```

## Creating Custom Filters

While SwissArmyHammer provides many filters, you can extend it further by implementing the `CustomFilter` trait:

```rust
use swissarmyhammer::template::CustomFilter;

struct MyFilter;

impl CustomFilter for MyFilter {
    fn name(&self) -> &str {
        "my_filter"
    }
    
    fn filter(&self, input: &str, args: &[&str]) -> Result<String, Box<dyn Error>> {
        // Your filter logic here
        Ok(input.to_uppercase())
    }
}
```

Then register it with the template engine:

```rust
let mut engine = TemplateEngine::new();
engine.register_filter(Box::new(MyFilter));
```