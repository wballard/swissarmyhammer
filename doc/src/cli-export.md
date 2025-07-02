# export - Export Prompts for Sharing

The `export` command allows you to package and share prompts in various formats for backup, distribution, or collaboration.

## Synopsis

```bash
swissarmyhammer export [OPTIONS] [PROMPTS...] <OUTPUT>
```

## Description

Export prompts to archives, directories, or other formats for sharing, backup, or deployment. Supports exporting individual prompts, categories, or entire collections.

## Arguments

- `PROMPTS...` - Specific prompt IDs to export (optional if using `--all` or `--category`)
- `OUTPUT` - Output path (file for archives, directory for directory format)

## Options

### Selection
- `--all, -a` - Export all prompts
- `--category, -c CATEGORY` - Export prompts from specific category
- `--source SOURCE` - Export prompts from specific source (builtin, user, local)

### Format
- `--format, -f FORMAT` - Output format (tar.gz, zip, directory)
  - `tar.gz` - Compressed tar archive (default)
  - `zip` - ZIP archive
  - `directory` - Plain directory structure

### Content Control
- `--include-metadata` - Include metadata files (manifest, checksums)
- `--no-validation` - Skip prompt validation before export
- `--exclude-builtin` - Exclude built-in prompts from export

### Output Control
- `--dry-run` - Show what would be exported without creating files
- `--quiet, -q` - Suppress progress output
- `--verbose, -v` - Show detailed export progress

## Examples

### Basic Export
```bash
# Export specific prompts to tar.gz
swissarmyhammer export code-review debug-helper exported-prompts.tar.gz

# Export all prompts
swissarmyhammer export --all my-prompts.tar.gz

# Export to directory
swissarmyhammer export --all --format directory ./prompt-backup/
```

### Category and Source Export
```bash
# Export all review-related prompts
swissarmyhammer export --category review review-prompts.zip

# Export only user-created prompts
swissarmyhammer export --source user --format tar.gz user-prompts.tar.gz

# Export local project prompts
swissarmyhammer export --source local project-prompts.tar.gz
```

### Advanced Options
```bash
# Dry run to see what would be exported
swissarmyhammer export --all --dry-run backup.tar.gz

# Export with metadata and validation
swissarmyhammer export --all --include-metadata --verbose complete-backup.tar.gz

# Export excluding built-ins
swissarmyhammer export --all --exclude-builtin custom-prompts.tar.gz
```

## Export Formats

### tar.gz (Default)
- **Benefits**: Widely supported, good compression, preserves permissions
- **Use case**: Sharing collections, backup, CI/CD pipelines
- **File structure**: Flat or hierarchical based on source organization

### zip
- **Benefits**: Native support on Windows, good tool ecosystem
- **Use case**: Sharing with Windows users, web distribution
- **File structure**: Same as tar.gz but in ZIP format

### directory
- **Benefits**: Easy to browse, no extraction needed, version control friendly
- **Use case**: Local backup, development, manual inspection
- **File structure**: Mirrors the original prompt directory organization

## Output Structure

### Archive Formats (tar.gz, zip)
```
exported-prompts.tar.gz/
├── manifest.json                 # Export metadata
├── checksums.sha256             # File integrity checks
├── prompts/
│   ├── review/
│   │   ├── code.md
│   │   └── security.md
│   ├── debug/
│   │   └── error.md
│   └── docs/
│       └── api.md
└── README.md                    # Export information
```

### Directory Format
```
prompt-backup/
├── .swissarmyhammer/
│   ├── manifest.json           # Export metadata
│   └── checksums.sha256        # File integrity
├── prompts/
│   ├── review/
│   │   ├── code.md
│   │   └── security.md
│   └── debug/
│       └── error.md
└── README.md                   # Export information
```

## Metadata Files

### manifest.json
```json
{
  "exported_at": "2024-01-15T10:30:00Z",
  "swissarmyhammer_version": "0.1.0",
  "export_options": {
    "format": "tar.gz",
    "include_builtin": false,
    "sources": ["user", "local"]
  },
  "prompts": [
    {
      "id": "code-review",
      "title": "Code Review Helper",
      "source": "user",
      "path": "prompts/review/code.md",
      "checksum": "sha256:abc123...",
      "arguments": ["code", "language"]
    }
  ],
  "statistics": {
    "total_prompts": 15,
    "total_size": "45.2KB",
    "categories": ["review", "debug", "docs"]
  }
}
```

### checksums.sha256
```
abc123... prompts/review/code.md
def456... prompts/debug/error.md
ghi789... prompts/docs/api.md
```

## Export Validation

Before export, prompts are validated for:

- **Syntax**: Valid YAML front matter and Markdown structure
- **Templates**: Liquid template syntax correctness
- **Arguments**: Argument definitions match template usage
- **References**: No broken internal references or includes

Skip validation with `--no-validation` for faster exports or when dealing with known issues.

## Performance and Size

### Compression Ratios
- **tar.gz**: 70-80% size reduction for typical prompt collections
- **zip**: 65-75% size reduction, faster compression
- **directory**: No compression, fastest export

### Performance Tips
```bash
# For large collections, use parallel processing
export RAYON_NUM_THREADS=4
swissarmyhammer export --all large-collection.tar.gz

# For frequent exports, skip validation
swissarmyhammer export --all --no-validation quick-backup.tar.gz
```

## Integration Examples

### Backup Script
```bash
#!/bin/bash
# backup-prompts.sh

DATE=$(date +%Y%m%d)
BACKUP_DIR="$HOME/prompt-backups"
mkdir -p "$BACKUP_DIR"

echo "Creating daily prompt backup..."
swissarmyhammer export --all \
  --include-metadata \
  --format tar.gz \
  "$BACKUP_DIR/prompts-$DATE.tar.gz"

# Keep only last 30 days
find "$BACKUP_DIR" -name "prompts-*.tar.gz" -mtime +30 -delete

echo "Backup complete: $BACKUP_DIR/prompts-$DATE.tar.gz"
```

### CI/CD Pipeline
```yaml
# .github/workflows/export-prompts.yml
name: Export Prompts
on:
  push:
    tags: ['v*']

jobs:
  export:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install SwissArmyHammer
        run: cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli
      - name: Export prompts
        run: |
          swissarmyhammer export --all \
            --exclude-builtin \
            --format tar.gz \
            prompts-${GITHUB_REF#refs/tags/}.tar.gz
      - name: Upload release asset
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./prompts-*.tar.gz
          asset_name: prompts-${{ github.ref }}.tar.gz
          asset_content_type: application/gzip
```

### Team Sharing
```bash
# Export team prompts for sharing
swissarmyhammer export \
  --category "team" \
  --include-metadata \
  team-prompts-$(date +%Y%m%d).tar.gz

# Upload to shared location
scp team-prompts-*.tar.gz team@shared-server:/shared/prompts/
```

## See Also

- [`import`](./cli-import.md) - Import exported prompts
- [Sharing Guide](./sharing-guide.md) - Collaboration workflows
- [`search`](./cli-search.md) - Find prompts to export