Get the current issue being worked on. Checks branch name to identify active issue.

## Parameters

- `branch` (optional): Which branch to check (optional, defaults to current)

## Examples

Check current issue on current branch:
```json
{}
```

Check current issue on specific branch:
```json
{
  "branch": "issue/REFACTOR_000123_cleanup-code"
}
```

## Returns

Returns the current issue information including:
- Issue name being worked on
- Branch name
- Issue content (if available)
- Current status