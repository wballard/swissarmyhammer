# Search and Discovery Guide

SwissArmyHammer provides powerful search capabilities to help you discover and find prompts in your collection. This guide covers search strategies, advanced filtering, and integration workflows.

## Search Strategies Overview

SwissArmyHammer provides three complementary search strategies:

1. **Fuzzy Search**: Fast approximate matching for typos and partial matches
2. **Full-Text Search**: Precise term-based search with boolean operators
3. **Semantic Search**: AI-powered semantic similarity using vector embeddings

Each strategy has different strengths and use cases. You can use them individually or in combination for optimal results.

## Basic Search

### Simple Text Search

The most basic way to search is with a simple text query:

```bash
# Search for prompts containing "code"
swissarmyhammer search code

# Search for multiple terms
swissarmyhammer search "code review"

# Search with partial matches
swissarmyhammer search debug
```

### Search Results Format

```
Found 3 prompts matching "code":

📝 code-review (builtin)
   Review code for best practices and potential issues
   Arguments: code, language (optional)

🔧 debug-helper (user)
   Help debug programming issues and errors
   Arguments: error, context (optional)

📊 analyze-performance (local)
   Analyze code performance and suggest optimizations
   Arguments: code, language, metrics (optional)
```

Each result shows:
- **Icon**: Indicates prompt type (📝 builtin, 🔧 user, 📊 local)
- **Name**: Prompt identifier
- **Source**: Where the prompt is stored
- **Description**: Brief description of the prompt's purpose
- **Arguments**: Required and optional parameters

## Field-Specific Search

### Search in Titles Only

```bash
# Find prompts with "review" in the title
swissarmyhammer search --in title review

# Case-sensitive title search
swissarmyhammer search --in title --case-sensitive "Code Review"
```

### Search in Descriptions

```bash
# Find prompts about debugging in descriptions
swissarmyhammer search --in description debug

# Find prompts mentioning specific technologies
swissarmyhammer search --in description "python javascript"
```

### Search in Content

```bash
# Find prompts that use specific template variables
swissarmyhammer search --in content "{{code}}"

# Find prompts with specific instructions
swissarmyhammer search --in content "best practices"
```

### Search All Fields

```bash
# Search across titles, descriptions, and content (default)
swissarmyhammer search --in all "security"

# Explicit all-field search
swissarmyhammer search "API documentation"
```

## Advanced Search Techniques

### Regular Expression Search

Use regex patterns for powerful pattern matching:

```bash
# Find prompts with "test" followed by any word
swissarmyhammer search --regex "test\s+\w+"

# Find prompts starting with specific words
swissarmyhammer search --regex "^(debug|fix|analyze)"

# Find prompts with email patterns
swissarmyhammer search --regex "\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b"

# Case-sensitive regex
swissarmyhammer search --regex --case-sensitive "^Code"
```

### Search by Source

Filter prompts by their source location:

```bash
# Find only built-in prompts
swissarmyhammer search --source builtin

# Find only user-created prompts
swissarmyhammer search --source user

# Find only local project prompts
swissarmyhammer search --source local

# Combine with text search
swissarmyhammer search review --source user
```

### Search by Arguments

Find prompts based on their argument requirements:

```bash
# Find prompts that accept a "code" argument
swissarmyhammer search --has-arg code

# Find prompts with no arguments (simple prompts)
swissarmyhammer search --no-args

# Find prompts with specific argument combinations
swissarmyhammer search --has-arg code --has-arg language

# Combine with text search
swissarmyhammer search debug --has-arg error
```

## Search Strategies

### Discovery Workflows

#### Finding Prompts for a Task
```bash
# 1. Start broad
swissarmyhammer search "code review"

# 2. Narrow down by context
swissarmyhammer search "code review" --source user

# 3. Check argument requirements
swissarmyhammer search "code review" --has-arg language

# 4. Examine specific matches
swissarmyhammer search --in title "Advanced Code Review"
```

#### Exploring Available Prompts
```bash
# See all available prompts
swissarmyhammer search --limit 50 ""

# Browse by category/topic
swissarmyhammer search documentation
swissarmyhammer search testing
swissarmyhammer search refactoring

# Find simple prompts (no arguments)
swissarmyhammer search --no-args
```

#### Finding Template Examples
```bash
# Find prompts using loops
swissarmyhammer search --in content "{% for"

# Find prompts with conditionals
swissarmyhammer search --in content "{% if"

# Find prompts using specific filters
swissarmyhammer search --in content "| capitalize"
```

### Search Optimization

#### Performance Tips
```bash
# Limit results for faster response
swissarmyhammer search --limit 10 query

# Use specific fields to reduce search scope
swissarmyhammer search --in title query  # faster than all fields

# Use source filtering to narrow search space
swissarmyhammer search --source user query
```

#### Precision vs. Recall
```bash
# High precision (exact matches)
swissarmyhammer search --case-sensitive --regex "^exact pattern$"

# High recall (find everything related)
swissarmyhammer search --in all "broad topic"

# Balanced approach
swissarmyhammer search "specific terms" --limit 20
```

## Integration with Other Commands

### Search and Test Workflow

```bash
# Find debugging prompts
swissarmyhammer search debug

# Test a specific one
swissarmyhammer test debug-helper

# Test with specific arguments
swissarmyhammer test debug-helper --arg error="TypeError: undefined"
```

### Search and Export Workflow

```bash
# Find all review-related prompts
swissarmyhammer search review --limit 20

# Export specific ones found
swissarmyhammer export code-review security-review design-review output.tar.gz

# Or export all matching a pattern
# (manual selection based on search results)
```

### Scripted Search

```bash
#!/bin/bash
# find-and-test.sh

QUERY="$1"
if [ -z "$QUERY" ]; then
    echo "Usage: $0 <search-query>"
    exit 1
fi

echo "Searching for: $QUERY"
PROMPTS=$(swissarmyhammer search --json "$QUERY" | jq -r '.results[].id')

if [ -z "$PROMPTS" ]; then
    echo "No prompts found"
    exit 1
fi

echo "Found prompts:"
echo "$PROMPTS"

echo "Select a prompt to test:"
select PROMPT in $PROMPTS; do
    if [ -n "$PROMPT" ]; then
        swissarmyhammer test "$PROMPT"
        break
    fi
done
```

## JSON Output for Scripting

### Basic JSON Search

```bash
swissarmyhammer search --json "code review"
```

```json
{
  "query": "code review",
  "total_found": 3,
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
  ]
}
```

### Processing JSON Results

```bash
# Extract prompt IDs
swissarmyhammer search --json query | jq -r '.results[].id'

# Get highest scoring result
swissarmyhammer search --json query | jq -r '.results[0].id'

# Filter by score threshold
swissarmyhammer search --json query | jq '.results[] | select(.score > 0.8)'

# Count results by source
swissarmyhammer search --json "" --limit 100 | jq '.results | group_by(.source) | map({source: .[0].source, count: length})'
```

## Search Index Management

### Understanding the Search Index

SwissArmyHammer automatically maintains a search index that includes:
- **Prompt titles** - Weighted heavily in scoring
- **Descriptions** - Medium weight
- **Content text** - Lower weight
- **Argument names** - Considered for relevance
- **File paths** - Used for source filtering

### Index Updates

The search index is automatically updated when:
- Prompts are added to the library
- Existing prompts are modified
- The `serve` command starts (full rebuild)
- File watching detects changes

### Performance Characteristics

- **Index size**: Proportional to prompt collection size
- **Search speed**: Sub-second for collections up to 10,000 prompts
- **Memory usage**: Moderate (index kept in memory)
- **Update speed**: Fast incremental updates

## Troubleshooting Search Issues

### No Results Found

```bash
# Check if prompts exist
swissarmyhammer search --limit 100 ""

# Verify prompt sources
swissarmyhammer search --source builtin
swissarmyhammer search --source user
swissarmyhammer search --source local

# Try broader search
swissarmyhammer search --in all "partial terms"
```

### Too Many Results

```bash
# Use more specific terms
swissarmyhammer search "specific exact phrase"

# Limit by source
swissarmyhammer search broad-term --source user

# Use field-specific search
swissarmyhammer search --in title specific-title

# Limit result count
swissarmyhammer search broad-term --limit 5
```

### Unexpected Results

```bash
# Check what's being matched
swissarmyhammer search --full query

# Use exact matching
swissarmyhammer search --regex "^exact term$"

# Search in specific field
swissarmyhammer search --in description query
```

## Best Practices

### Effective Search Terms

1. **Use specific terms**: "REST API documentation" vs. "API"
2. **Include context**: "Python debugging" vs. "debugging"
3. **Try synonyms**: "review", "analyze", "examine"
4. **Use argument names**: Search for "code", "error", "data" to find relevant prompts

### Search Workflow Patterns

1. **Start broad, narrow down**: Begin with general terms, add filters
2. **Use multiple strategies**: Try both fuzzy and regex search
3. **Check all sources**: Don't assume prompts are only in one location
4. **Combine with testing**: Always test prompts before using

### Organization for Searchability

1. **Clear titles**: Use descriptive, searchable titles
2. **Good descriptions**: Include keywords and use cases
3. **Consistent naming**: Use standard terms across prompts
4. **Tag with arguments**: Use predictable argument names

## Advanced Examples

### Finding Template Patterns

```bash
# Find prompts using custom filters
swissarmyhammer search --in content "format_lang"

# Find prompts with error handling
swissarmyhammer search --in content "default:"

# Find prompts with loops
swissarmyhammer search --in content "{% for"
```

### Building Prompt Collections

```bash
# Find all code-related prompts
swissarmyhammer search --regex "(code|programming|software)" --limit 50

# Find all documentation prompts
swissarmyhammer search --regex "(doc|documentation|readme|guide)" --limit 30

# Find all analysis prompts
swissarmyhammer search --regex "(analy|review|audit|inspect)" --limit 20
```

### Quality Assurance

```bash
# Find prompts without descriptions
swissarmyhammer search --in description "^$" --regex

# Find prompts with no arguments (might need descriptions)
swissarmyhammer search --no-args --limit 50

# Find prompts with many arguments (might be complex)
swissarmyhammer search --json "" --limit 100 | \
  jq '.results[] | select(.arguments | length > 5)'
```

## Semantic Search

### Overview

Semantic search uses AI embeddings to find code and prompts based on meaning rather than exact text matches. This is particularly powerful for finding conceptually similar code even when the exact keywords differ.

### Basic Semantic Search

```bash
# Find code semantically similar to a concept
swissarmyhammer search --semantic "error handling patterns"

# Search for code functionality
swissarmyhammer search --semantic "database connection pooling"

# Find similar algorithms or approaches
swissarmyhammer search --semantic "sorting algorithms implementation"
```

### Language-Specific Semantic Search

```bash
# Find Rust-specific patterns
swissarmyhammer search --semantic "async error handling" --language rust

# Python-specific search
swissarmyhammer search --semantic "decorator patterns" --language python

# JavaScript/TypeScript patterns
swissarmyhammer search --semantic "promise chain handling" --language typescript
```

### Semantic Search with Thresholds

```bash
# High-precision semantic search (only very similar results)
swissarmyhammer search --semantic "REST API client" --threshold 0.8

# Broader semantic search (more results, less similar)
swissarmyhammer search --semantic "authentication" --threshold 0.6

# Maximum recall (find anything remotely related)
swissarmyhammer search --semantic "testing" --threshold 0.4
```

### Code Similarity Detection

```bash
# Find code similar to a specific file
swissarmyhammer search --semantic-file path/to/example.rs

# Find duplicated or similar functions
swissarmyhammer search --semantic "function findUser(id)" --limit 10

# Detect architectural patterns
swissarmyhammer search --semantic "observer pattern implementation"
```

### Multi-Modal Semantic Search

```bash
# Combine text and code structure
swissarmyhammer search --semantic "error handling" --include-structure

# Search including comments and documentation
swissarmyhammer search --semantic "caching strategy" --include-docs

# Focus on specific code constructs
swissarmyhammer search --semantic "async functions" --code-only
```

### Semantic Search Performance

Semantic search characteristics:
- **Latency**: 50-200ms for typical queries
- **Memory**: Scales with result set size and embedding dimensions
- **Accuracy**: High for conceptual matches, lower for exact syntax
- **Best for**: Finding similar algorithms, patterns, and architectural concepts

### When to Use Semantic Search

**Use semantic search when:**
- Looking for conceptually similar code patterns
- Searching across different programming languages
- Finding architectural patterns and design solutions
- Exploring similar algorithms or approaches
- Discovering duplicated or near-duplicate code

**Use traditional search when:**
- Looking for exact variable names or function signatures
- Searching for specific syntax or language constructs
- Finding exact string matches
- Performance is critical (semantic search is slower)

## See Also

- [`search` command](./cli-search.md) - Command reference
- [`test` command](./cli-test.md) - Testing found prompts
- [Prompt Organization](./prompt-organization.md) - Organizing for discoverability