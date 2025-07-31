# Issue Show

Display details of a specific issue by name.

## Parameters

- `name` (required): Name of the issue to show
- `raw` (optional): Show raw content only without formatting (default: false)

## Examples

Show issue with formatted display:
```json
{
  "name": "FEATURE_000123_user-auth"
}
```

Show raw issue content only:
```json
{
  "name": "FEATURE_000123_user-auth",
  "raw": true
}
```

## Returns

Returns the issue details including status, creation date, file path, and content. When `raw` is true, returns only the raw markdown content.