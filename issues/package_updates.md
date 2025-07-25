# Update Dependencies to Latest Versions

## Overview
Based on analysis of current Cargo.toml files and crates.io versions, several dependencies can be updated to their latest versions for security patches, performance improvements, and new features.

## Current vs Latest Versions

### Major Updates Required
- **tantivy**: `0.22` → `0.24.2` (major version bump, may require API changes)
- **reqwest**: `0.11` → `0.12.22` (major version bump, may require API changes)
- **tabled**: `0.15` → `0.20.0` (major version bump, may require API changes)

### Minor/Patch Updates Available
- **tokio**: `1.x` → `1.46.1` (latest patch)
- **serde**: `1.x` → `1.0.219` (latest patch)
- **clap**: `4.x` → `4.5.41` (latest minor)
- **liquid**: `0.26` → `0.26.11` (latest patch)

### Already Up to Date
- **fastembed**: `5.0.0` ✅
- **tree-sitter**: `0.25.8` ✅
- **duckdb**: `1.3` → `1.3.2` (patch update available)

## Implementation Plan

### Phase 1: Safe Updates (Low Risk)
1. Update patch versions for existing dependencies
   - tokio, serde, liquid, duckdb patches
   - clap minor version

### Phase 2: Major Updates (Requires Testing)
1. **tantivy 0.22 → 0.24.2**
   - Review breaking changes in release notes
   - Update search indexing code
   - Test semantic search functionality

2. **reqwest 0.11 → 0.12.22**
   - Review HTTP client usage
   - Update async API calls
   - Test network operations

3. **tabled 0.15 → 0.20.0**
   - Review table formatting code
   - Update CLI output formatting
   - Test display functionality

## Testing Requirements
- [ ] Run full test suite after each update
- [ ] Test semantic search functionality (tantivy)
- [ ] Test HTTP operations (reqwest)
- [ ] Test CLI table output (tabled)
- [ ] Verify all benchmarks still pass

## Risk Assessment
- **Low Risk**: Patch and minor updates
- **Medium Risk**: Major version updates require code changes
- **High Risk**: tantivy update may affect core search functionality

## Benefits
- Security patches and bug fixes
- Performance improvements
- Access to new features
- Better ecosystem compatibility