# Memoranda Create Tool

Create a new memo with the given title and content. Returns the created memo with its unique ID.

## Parameters

- `title` (string): Title of the memo
- `content` (string): Markdown content of the memo

## Returns

The created memo with metadata including unique ULID identifier.

## Examples

Create a memo with title and content:
```json
{
  "title": "Meeting Notes",
  "content": "# Team Meeting\n\nDiscussed project roadmap..."
}
```

## Implementation Notes

- Both title and content can be empty - storage layer supports this
- Uses ULID for unique identifiers
- Content is stored as markdown format
- Automatic timestamp generation for created_at and updated_at