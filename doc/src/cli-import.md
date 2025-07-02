# import - Import Prompts from Various Sources

The `import` command allows you to import prompts from archives, URLs, Git repositories, and other sources into your local prompt collection.

## Synopsis

```bash
swissarmyhammer import [OPTIONS] <SOURCE>
```

## Description

Import prompts from various sources including local files, URLs, Git repositories, and other SwissArmyHammer installations. Supports conflict resolution, validation, and backup creation.

## Arguments

- `SOURCE` - Import source (file path, URL, Git repository, or directory)

## Options

### Conflict Resolution
- `--strategy STRATEGY` - Conflict resolution strategy
  - `ask` - Prompt for each conflict (default)
  - `overwrite` - Overwrite existing prompts
  - `skip` - Skip conflicting prompts
  - `rename` - Rename conflicting prompts
  - `abort` - Abort on first conflict

### Source Options
- `--git-branch BRANCH` - Git branch to import from (for Git sources)
- `--git-path PATH` - Path within Git repository to import
- `--extract-to DIR` - Extract archives to specific directory first

### Validation and Safety
- `--dry-run` - Show what would be imported without making changes
- `--no-backup` - Skip creating backup before import
- `--no-validation` - Skip prompt validation during import
- `--force` - Force import even with validation errors

### Output Control
- `--quiet, -q` - Suppress progress output
- `--verbose, -v` - Show detailed import progress
- `--json` - Output results in JSON format

## Examples

### File Import
```bash
# Import from local archive
swissarmyhammer import prompts-backup.tar.gz

# Import from directory
swissarmyhammer import ./shared-prompts/

# Import with specific conflict strategy
swissarmyhammer import --strategy overwrite team-prompts.zip
```

### URL Import
```bash
# Import from URL
swissarmyhammer import https://example.com/prompts.tar.gz

# Import from GitHub release
swissarmyhammer import https://github.com/team/prompts/releases/download/v1.0/prompts.tar.gz
```

### Git Repository Import
```bash
# Import from Git repository
swissarmyhammer import https://github.com/team/prompt-library.git

# Import specific branch
swissarmyhammer import --git-branch develop https://github.com/team/prompts.git

# Import specific path within repository
swissarmyhammer import --git-path prompts/ https://github.com/team/project.git
```

### Advanced Import
```bash
# Dry run to preview import
swissarmyhammer import --dry-run team-prompts.tar.gz

# Import with validation and backup
swissarmyhammer import --verbose --strategy ask prompts.tar.gz

# Force import with errors
swissarmyhammer import --force --no-validation broken-prompts.tar.gz
```

## Import Sources

### Supported Formats
- **tar.gz** - Compressed tar archives
- **zip** - ZIP archives  
- **directory** - Local directories
- **Git repositories** - Remote Git repositories
- **URLs** - HTTP/HTTPS downloads

### Auto-Detection
SwissArmyHammer automatically detects source types:

```bash
# These are all detected automatically
swissarmyhammer import prompts.tar.gz           # Archive
swissarmyhammer import ./prompts/               # Directory
swissarmyhammer import https://example.com/prompts.tar.gz  # URL
swissarmyhammer import git@github.com:team/prompts.git     # Git SSH
swissarmyhammer import https://github.com/team/prompts.git # Git HTTPS
```

## Conflict Resolution

### Interactive Mode (ask)
```bash
$ swissarmyhammer import team-prompts.tar.gz
Found 3 prompts to import, 1 conflict detected.

Conflict: 'code-review' already exists
Existing: User prompt from ~/.swissarmyhammer/prompts/review/code.md
Incoming: Team prompt "Advanced Code Review Helper"

? How should this conflict be resolved?
  > View differences
    Overwrite existing
    Skip this prompt  
    Rename to 'code-review-team'
    Rename to 'code-review-2'
    Abort import
```

### Automatic Strategies
```bash
# Overwrite all conflicts
swissarmyhammer import --strategy overwrite prompts.tar.gz

# Skip all conflicts
swissarmyhammer import --strategy skip prompts.tar.gz

# Rename all conflicts
swissarmyhammer import --strategy rename prompts.tar.gz
```

## Validation and Safety

### Pre-Import Validation
```bash
$ swissarmyhammer import --dry-run prompts.tar.gz
Import Analysis:
✓ Archive format: tar.gz (valid)
✓ Manifest present and valid
✓ Checksums verified

Prompts to import:
  ✓ code-review (valid template)
  ✓ debug-helper (valid template)  
  ⚠ broken-prompt (warning: unused argument 'old_arg')
  ✗ invalid-prompt (error: invalid YAML front matter)

Conflicts:
  ! code-review - conflicts with existing user prompt

Summary: 2 valid, 1 warning, 1 error, 1 conflict
Would import 3 prompts (excluding invalid)
```

### Backup Creation
```bash
# Automatic backup before import
$ swissarmyhammer import prompts.tar.gz
Creating backup: ~/.swissarmyhammer/backups/pre-import-20240115-103000.tar.gz
Importing 3 prompts...
✓ Import completed successfully

# Restore from backup if needed
swissarmyhammer import ~/.swissarmyhammer/backups/pre-import-20240115-103000.tar.gz --strategy overwrite
```

## Import Structure

### Expected Directory Structure
```
prompts-archive.tar.gz/
├── manifest.json              # Optional metadata
├── prompts/                   # Prompt files
│   ├── review/
│   │   └── code.md
│   └── debug/
│       └── error.md
└── README.md                  # Optional documentation
```

### Flexible Structure Support
SwissArmyHammer handles various directory structures:

```bash
# All these structures are supported:
./prompts/*.md                 # Flat structure
./category/*/prompt.md         # Nested categories
./src/prompts/**/*.md         # Deep nesting
./random-structure/**/*.md     # Any structure with .md files
```

## Git Integration

### Authentication
```bash
# SSH key authentication (recommended)
swissarmyhammer import git@github.com:team/prompts.git

# HTTPS with credentials
export GIT_USERNAME=user
export GIT_PASSWORD=token
swissarmyhammer import https://github.com/team/prompts.git

# GitHub CLI integration
gh auth login
swissarmyhammer import https://github.com/private/prompts.git
```

### Branch and Path Selection
```bash
# Import from specific branch
swissarmyhammer import --git-branch feature/new-prompts \
  https://github.com/team/prompts.git

# Import only specific directory
swissarmyhammer import --git-path specialized-prompts/ \
  https://github.com/team/monorepo.git

# Combine branch and path
swissarmyhammer import \
  --git-branch develop \
  --git-path prompts/production/ \
  https://github.com/team/project.git
```

## Output and Logging

### Progress Output
```bash
$ swissarmyhammer import --verbose prompts.tar.gz
Downloading: prompts.tar.gz (1.2MB) [████████████████████] 100%
Extracting archive...
Validating 5 prompts...
  ✓ code-review (valid)
  ✓ debug-helper (valid)
  ✓ api-docs (valid)
  ⚠ legacy-prompt (deprecated syntax)
  ✗ broken-template (invalid liquid syntax)

Importing valid prompts:
  → code-review (new)
  → debug-helper (conflict: renamed to debug-helper-imported)
  → api-docs (new)
  → legacy-prompt (new, with warnings)

Skipped: 1 invalid prompt
Imported: 4 prompts
Conflicts resolved: 1
```

### JSON Output
```bash
$ swissarmyhammer import --json prompts.tar.gz
{
  "success": true,
  "imported": 4,
  "skipped": 1,
  "conflicts": 1,
  "backup_created": "~/.swissarmyhammer/backups/pre-import-20240115-103000.tar.gz",
  "prompts": [
    {
      "id": "code-review",
      "action": "imported",
      "source_path": "prompts/review/code.md",
      "target_path": "~/.swissarmyhammer/prompts/review/code.md"
    },
    {
      "id": "debug-helper", 
      "action": "renamed",
      "original_id": "debug-helper",
      "new_id": "debug-helper-imported",
      "reason": "conflict_resolution"
    }
  ],
  "errors": [
    {
      "prompt": "broken-template",
      "error": "Invalid Liquid syntax at line 15",
      "action": "skipped"
    }
  ]
}
```

## Integration Examples

### Team Onboarding
```bash
#!/bin/bash
# onboard-developer.sh

echo "Setting up SwissArmyHammer with team prompts..."

# Import base team prompts
swissarmyhammer import \
  --strategy ask \
  https://github.com/company/team-prompts.git

# Import project-specific prompts  
swissarmyhammer import \
  --strategy skip \
  https://github.com/company/project-prompts/releases/latest/download/prompts.tar.gz

echo "✓ Prompt setup complete"
echo "Use 'swissarmyhammer doctor' to verify installation"
```

### Continuous Integration
```yaml
# .github/workflows/sync-prompts.yml
name: Sync Team Prompts
on:
  schedule:
    - cron: '0 9 * * MON'  # Weekly sync

jobs:
  sync:
    runs-on: ubuntu-latest
    steps:
      - name: Install SwissArmyHammer
        run: cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli
      
      - name: Import latest team prompts
        run: |
          swissarmyhammer import \
            --strategy overwrite \
            --no-backup \
            https://github.com/company/team-prompts.git
      
      - name: Validate imported prompts
        run: swissarmyhammer doctor
```

### Migration Script
```bash
#!/bin/bash
# migrate-from-old-system.sh

OLD_PROMPTS="/old-system/prompts"
BACKUP_DIR="/backup/prompts-$(date +%Y%m%d)"

echo "Migrating prompts from old system..."

# Create backup
mkdir -p "$BACKUP_DIR"
cp -r "$OLD_PROMPTS" "$BACKUP_DIR/"

# Convert and import
for file in "$OLD_PROMPTS"/*.txt; do
  # Convert old format to SwissArmyHammer format
  ./convert-prompt.py "$file" > "converted/$(basename "$file" .txt).md"
done

# Import converted prompts
swissarmyhammer import converted/ --strategy ask

echo "Migration complete. Backup at: $BACKUP_DIR"
```

## See Also

- [`export`](./cli-export.md) - Export prompts for sharing
- [Sharing Guide](./sharing-guide.md) - Collaboration workflows
- [`doctor`](./cli-doctor.md) - Validate installation after import