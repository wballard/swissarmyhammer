Create a new issue with auto-assigned number. Issues are markdown files stored in ./issues directory for tracking work items.

## Parameters

- `content` (required): Markdown content of the issue
- `name` (optional): Name of the issue (will be used in filename)
  - When provided, creates files like `000123_name.md`
  - When omitted, creates files like `000123.md`

## Examples

Create a named issue:
```json
{
  "name": "feature_name",
  "content": "# Implement new feature\n\nDetails..."
}
```

Create a nameless issue:
```json
{
  "content": "# Quick fix needed\n\nDetails..."
}
```

## Returns

Returns the created issue name and confirmation message.