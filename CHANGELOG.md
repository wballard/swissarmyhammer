# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Updated major dependencies with breaking change handling:
  - **tantivy**: `0.22` → `0.24.2` - Updated document retrieval API calls with explicit type annotations
  - **tabled**: `0.15` → `0.20.0` - Updated row selection API from `Rows::single()` to `Rows::one()`
  - **reqwest**: `0.11` → `0.12.22` - No breaking changes (transitive dependency)

- Updated minor/patch dependencies:
  - **tokio**: `1` → `1.46`
  - **serde**: `1` → `1.0.219`
  - **liquid**: `0.26` → `0.26.11`
  - **clap**: `4` → `4.5.41`
  - **duckdb**: `1.3` → `1.3.2`

### Fixed
- Fixed clippy warnings for uninlined format args in semantic indexer tests
- Enhanced test robustness for missing fastembed models in CI environments
- Improved file watcher cleanup with better async task termination
- Removed unused variable warnings identified by updated clippy

### Security
- Updated all dependencies to latest versions with security patches

### Performance
- Access to latest performance improvements in all updated crates
- Better integration with current Rust ecosystem