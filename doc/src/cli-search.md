# search - Search and Discover Prompts

The `search` command provides powerful functionality to find prompts in your collection using various search strategies and filters.

## Synopsis

```bash
swissarmyhammer search [OPTIONS] [QUERY]
```

## Description

Search through your prompt collection using fuzzy matching, regular expressions, or exact text matching. The search can target specific fields and provides relevance-ranked results.

## Arguments

- `QUERY` - Search term or pattern (optional if using filters)

## Options

### Search Strategy
- `--case-sensitive, -c` - Enable case-sensitive matching
- `--regex, -r` - Use regular expressions instead of fuzzy matching
- `--fuzzy` - Use fuzzy string matching (default for simple queries)
- `--semantic` - Use AI-powered semantic search with embeddings
- `--hybrid` - Combine fuzzy, full-text, and semantic search results
- `--full, -f` - Show full prompt content in results

### Field Targeting
- `--in FIELD` - Search in specific field (title, description, content, all)
  - `title` - Search only in prompt titles
  - `description` - Search only in prompt descriptions
  - `content` - Search only in prompt content/body
  - `all` - Search in all fields (default)

### Filtering
- `--source SOURCE` - Filter by prompt source (builtin, user, local)
- `--has-arg ARG` - Show prompts that have a specific argument
- `--no-args` - Show prompts with no arguments
- `--language LANG` - Filter by programming language (for semantic search)

### Semantic Search Options
- `--threshold FLOAT` - Similarity threshold for semantic search (0.0-1.0)
- `--model MODEL` - Embedding model to use for semantic search
- `--include-structure` - Include code structure in semantic analysis
- `--include-docs` - Include documentation and comments in search
- `--code-only` - Search only code content, exclude comments

### Output Control
- `--limit, -l N` - Limit results to N prompts (default: 20)
- `--json` - Output results in JSON format

## Examples

### Basic Search
```bash
# Find prompts containing "code"
swissarmyhammer search code

# Case-sensitive search
swissarmyhammer search --case-sensitive "Code Review"
```

### Field-Specific Search
```bash
# Search only in titles
swissarmyhammer search --in title "review"

# Search only in descriptions
swissarmyhammer search --in description "debugging"

# Search in content/body
swissarmyhammer search --in content "TODO"
```

### Regular Expression Search
```bash
# Find prompts with "test" followed by any word
swissarmyhammer search --regex "test\s+\w+"

# Find prompts starting with specific pattern
swissarmyhammer search --regex "^(debug|fix|analyze)"
```

### Advanced Filtering
```bash
# Find built-in prompts only
swissarmyhammer search --source builtin

# Find prompts with "code" argument
swissarmyhammer search --has-arg code

# Find prompts without any arguments
swissarmyhammer search --no-args

# Combine filters
swissarmyhammer search review --source user --has-arg language
```

### Output Options
```bash
# Show full content of matching prompts
swissarmyhammer search code --full

# Limit to 5 results
swissarmyhammer search --limit 5 test

# Get JSON output for scripting
swissarmyhammer search --json "data analysis"
```

### Semantic Search Examples
```bash
# Basic semantic search
swissarmyhammer search --semantic "error handling patterns"

# Language-specific semantic search
swissarmyhammer search --semantic "async functions" --language rust

# High-precision semantic search
swissarmyhammer search --semantic "database connection" --threshold 0.8

# Hybrid search combining all strategies
swissarmyhammer search --hybrid "authentication middleware"

# Semantic search with specific model
swissarmyhammer search --semantic "testing patterns" --model all-mpnet-base-v2

# Code-only semantic search
swissarmyhammer search --semantic "sorting algorithm" --code-only
```

## Output Format

### Default Output
```
Found 3 prompts matching "code":

📝 code-review (builtin)
   Review code for best practices and potential issues
   Arguments: code, language (optional)

🔧 debug-code (user)
   Help debug programming issues and errors
   Arguments: error, context (optional)

📊 analyze-performance (local)
   Analyze code performance and suggest optimizations
   Arguments: code, language, metrics (optional)
```

### JSON Output
```json
{
  "query": "code",
  "results": [
    {
      "id": "code-review",
      "title": "Code Review Helper",
      "description": "Review code for best practices and potential issues",
      "source": "builtin",
      "path": "/builtin/review/code.md",
      "arguments": [
        {"name": "code", "required": true},
        {"name": "language", "required": false, "default": "auto-detect"}
      ],
      "score": 0.95
    }
  ],
  "total_found": 3
}
```

## Search Scoring

Results are ranked by relevance using these factors:

1. **Exact matches** score higher than partial matches
2. **Title matches** score higher than description or content matches
3. **Multiple field matches** increase the overall score
4. **Argument name matches** are considered for relevance

## Performance

- Search is optimized with an in-memory index
- Fuzzy matching uses efficient algorithms
- Results are cached for repeated queries
- Large prompt collections are handled efficiently

## Integration with Other Commands

Search integrates well with other SwissArmyHammer commands:

```bash
# Find and test a prompt
PROMPT=$(swissarmyhammer search --json code | jq -r '.results[0].id')
swissarmyhammer test "$PROMPT"

# Export search results
swissarmyhammer search debug --limit 5 | \
  grep -o '\w\+-\w\+' | \
  xargs swissarmyhammer export
```

## See Also

- [`test`](./cli-test.md) - Test prompts found through search
- [`export`](./cli-export.md) - Export specific prompts
- [Search Guide](./search-guide.md) - Advanced search strategies
- [Search Architecture](./search-architecture.md) - Technical architecture details
- [Index Management](./index-management.md) - Managing search indices