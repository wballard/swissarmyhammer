# Changelog

All notable changes to SwissArmyHammer will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive documentation with mdBook
- GitHub Pages deployment for documentation
- Enhanced error messages with context
- Validation for prompt arguments
- Support for YAML anchors in prompts
- Performance benchmarks

### Changed
- **BREAKING**: The `validate` command no longer accepts custom workflow directories via `--workflow-dir`. Workflows are now only loaded from standard locations (builtin, user `~/.swissarmyhammer/workflows`, and local `./.swissarmyhammer/workflows`)
- Improved template rendering performance
- Better error handling in MCP server
- Enhanced file watching efficiency
- Validation error paths for workflows now include source location (e.g., `workflow:builtin:example` instead of `workflow:example`)

### Fixed
- Memory leak in file watcher
- Prompt loading on Windows paths
- Template escaping for special characters

## [0.2.0] - 2024-03-01

### Added
- MCP (Model Context Protocol) server implementation
- File watching for automatic prompt reloading
- Doctor command for system health checks
- Liquid template engine integration
- Support for prompt arguments and validation
- Recursive directory scanning
- YAML front matter parsing

### Changed
- Migrated from simple templates to Liquid engine
- Improved prompt discovery algorithm
- Enhanced CLI output formatting
- Better error messages and diagnostics

### Fixed
- Cross-platform path handling
- Unicode support in prompts
- Memory usage optimization

### Security
- Added input sanitization for templates
- Implemented secure file access controls

## [0.1.0] - 2024-01-15

### Added
- Initial release
- Basic prompt management functionality
- CLI interface with subcommands
- List command to show available prompts
- Serve command for MCP integration
- Simple template substitution
- Configuration file support
- Basic documentation

### Changed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Security
- N/A (initial release)

## Version History

### Versioning Policy

SwissArmyHammer follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for backwards-compatible functionality additions
- **PATCH** version for backwards-compatible bug fixes

### Pre-1.0 Versions

During the 0.x series:
- Minor version bumps may include breaking changes
- The API is considered unstable
- Features may be experimental

### Migration Guides

#### 0.1.x to 0.2.x

**Breaking Changes:**

1. **Template Engine Change**
   - Old: Simple `{variable}` substitution
   - New: Liquid templates with `{{variable}}`
   - Migration: Update all prompts to use double braces

2. **Configuration Format**
   - Old: JSON configuration
   - New: TOML configuration
   - Migration: Convert config.json to config.toml

3. **Prompt Metadata**
   - Old: Optional metadata
   - New: Required YAML front matter
   - Migration: Add minimal front matter to all prompts

**Example Migration:**

Old prompt (0.1.x):
```markdown
# Code Review

Review this {language} code:
{code}
```

New prompt (0.2.x):
```markdown
---
name: code-review
title: Code Review
arguments:
  - name: language
    required: true
  - name: code
    required: true
---

# Code Review

Review this {{language}} code:
{{code}}
```

### Release Schedule

- **Patch releases**: As needed for bug fixes
- **Minor releases**: Monthly with new features
- **Major releases**: When breaking changes are necessary

### Support Policy

- Latest version: Full support
- Previous minor version: Security fixes only
- Older versions: No support

## Contributing to Changelog

When contributing, please:

1. Add entries under "Unreleased"
2. Use the appropriate section
3. Reference issue/PR numbers
4. Keep descriptions concise
5. Sort entries by importance

Example entry:
```markdown
### Fixed
- Fix memory leak in file watcher (#123)
```

## Links

- [GitHub Releases](https://github.com/swissarmyhammer/swissarmyhammer/releases)
- [Release Process](./release-process.md)
- [Contributing Guide](./contributing.md)

[Unreleased]: https://github.com/swissarmyhammer/swissarmyhammer/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/swissarmyhammer/swissarmyhammer/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/swissarmyhammer/swissarmyhammer/releases/tag/v0.1.0