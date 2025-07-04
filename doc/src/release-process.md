# Release Process

This guide documents the release process for SwissArmyHammer, including versioning, testing, building, and publishing.

## Overview

SwissArmyHammer follows a structured release process:
1. **Version Planning** - Determine version number and scope
2. **Pre-release Testing** - Comprehensive testing
3. **Release Preparation** - Update version, changelog
4. **Building** - Create release artifacts
5. **Publishing** - Release to crates.io and GitHub
6. **Post-release** - Announcements and documentation

## Versioning

### Semantic Versioning

We follow [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH

1.0.0
│ │ └── Patch: Bug fixes, no API changes
│ └──── Minor: New features, backward compatible
└────── Major: Breaking changes
```

### Version Guidelines

#### Patch Release (0.0.X)
- Bug fixes
- Documentation improvements
- Performance improvements (no API change)
- Security patches

#### Minor Release (0.X.0)
- New features
- New commands
- New configuration options
- Deprecations (with warnings)

#### Major Release (X.0.0)
- Breaking API changes
- Removal of deprecated features
- Major architectural changes
- Incompatible configuration changes

## Release Checklist

### Pre-release Checklist

```markdown
## Pre-release Checklist

- [ ] All tests passing on main branch
- [ ] No outstanding security issues
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated
- [ ] Release branch created
- [ ] Release PR approved
```

### Release Build Checklist

```markdown
## Build Checklist

- [ ] Clean build on all platforms
- [ ] All features compile
- [ ] Binary size acceptable
- [ ] Performance benchmarks acceptable
- [ ] Security audit passing
```

## Release Preparation

### 1. Create Release Branch

```bash
# Create release branch from main
git checkout main
git pull origin main
git checkout -b release/v1.2.3

# Or for release candidates
git checkout -b release/v1.2.3-rc1
```

### 2. Update Version Numbers

Update version in multiple files:

```bash
# Cargo.toml
[package]
name = "swissarmyhammer"
version = "1.2.3"  # Update this

# Update lock file
cargo update -p swissarmyhammer
```

### 3. Update Changelog

Edit `CHANGELOG.md`:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.3] - 2024-03-15

### Added
- New `validate` command for prompt validation
- Support for YAML anchors in prompts
- Performance monitoring dashboard

### Changed
- Improved error messages for template rendering
- Updated minimum Rust version to 1.70

### Fixed
- Fixed file watcher memory leak (#123)
- Corrected prompt loading on Windows (#124)

### Security
- Updated dependencies to patch CVE-2024-XXXXX

[1.2.3]: https://github.com/wballard/swissarmyhammer/compare/v1.2.2...v1.2.3
```

### 4. Update Documentation

```bash
# Update version in documentation
find doc -name "*.md" -exec sed -i 's/0\.1\.0/1.2.3/g' {} \;

# Rebuild documentation
cd doc
mdbook build

# Update README if needed
vim README.md
```

### 5. Run Pre-release Tests

```bash
# Full test suite
cargo test --all-features

# Test on different platforms
cargo test --target x86_64-pc-windows-gnu
cargo test --target x86_64-apple-darwin

# Integration tests
cargo test --test '*' -- --test-threads=1

# Benchmarks
cargo bench

# Security audit
cargo audit

# Check for unused dependencies
cargo machete
```

## Building Release Artifacts

### Local Build Script

Create `scripts/build-release.sh`:

```bash
#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "Building SwissArmyHammer v$VERSION"

# Clean previous builds
cargo clean
rm -rf target/release-artifacts
mkdir -p target/release-artifacts

# Build for multiple platforms
PLATFORMS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

for platform in "${PLATFORMS[@]}"; do
    echo "Building for $platform..."
    
    if [[ "$platform" == *"windows"* ]]; then
        ext=".exe"
    else
        ext=""
    fi
    
    # Build
    cross build --release --target "$platform"
    
    # Package
    cp "target/$platform/release/swissarmyhammer$ext" \
       "target/release-artifacts/swissarmyhammer-$VERSION-$platform$ext"
    
    # Create tarball/zip
    if [[ "$platform" == *"windows"* ]]; then
        cd target/release-artifacts
        zip "swissarmyhammer-$VERSION-$platform.zip" \
            "swissarmyhammer-$VERSION-$platform.exe"
        rm "swissarmyhammer-$VERSION-$platform.exe"
        cd ../..
    else
        cd target/release-artifacts
        tar -czf "swissarmyhammer-$VERSION-$platform.tar.gz" \
            "swissarmyhammer-$VERSION-$platform"
        rm "swissarmyhammer-$VERSION-$platform"
        cd ../..
    fi
done

# Generate checksums
cd target/release-artifacts
shasum -a 256 * > checksums.sha256
cd ../..

echo "Release artifacts built in target/release-artifacts/"
```

### GitHub Actions Release

`.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  create-release:
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - uses: actions/checkout@v3
      
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: true
          prerelease: false
          body_path: RELEASE_NOTES.md

  build-release:
    needs: create-release
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: swissarmyhammer
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            artifact: swissarmyhammer
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: swissarmyhammer
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: swissarmyhammer
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: swissarmyhammer.exe
    
    runs-on: ${{ matrix.os }}
    
    steps:
      - uses: actions/checkout@v3
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Package (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czf swissarmyhammer-${{ github.ref_name }}-${{ matrix.target }}.tar.gz swissarmyhammer
          mv *.tar.gz ../../../
      
      - name: Package (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          7z a swissarmyhammer-${{ github.ref_name }}-${{ matrix.target }}.zip swissarmyhammer.exe
          mv *.zip ../../../
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./swissarmyhammer-${{ github.ref_name }}-${{ matrix.target }}.${{ matrix.os == 'windows-latest' && 'zip' || 'tar.gz' }}
          asset_name: swissarmyhammer-${{ github.ref_name }}-${{ matrix.target }}.${{ matrix.os == 'windows-latest' && 'zip' || 'tar.gz' }}
          asset_content_type: ${{ matrix.os == 'windows-latest' && 'application/zip' || 'application/gzip' }}
```

## Publishing

### 1. Publish to crates.io

```bash
# Dry run first
cargo publish --dry-run

# Verify package contents
cargo package --list

# Publish
cargo publish

# Note: You need to be logged in
cargo login <token>
```

### 2. Create GitHub Release

```bash
# Push release branch
git push origin release/v1.2.3

# Create and merge PR
gh pr create --title "Release v1.2.3" \
  --body "Release version 1.2.3. See CHANGELOG.md for details."

# After PR merged, create tag
git checkout main
git pull origin main
git tag -a v1.2.3 -m "Release version 1.2.3"
git push origin v1.2.3
```

### 3. Update GitHub Release

After CI builds artifacts:

```bash
# Edit release notes
gh release edit v1.2.3 --notes-file RELEASE_NOTES.md

# Publish release (remove draft status)
gh release edit v1.2.3 --draft=false
```

## Post-release

### 1. Update Documentation

```bash
# Update stable docs
git checkout gh-pages
cp -r doc/book/* .
git add .
git commit -m "Update documentation for v1.2.3"
git push origin gh-pages
```

### 2. Announcements

Create announcement template:

```markdown
# SwissArmyHammer v1.2.3 Released!

We're excited to announce the release of SwissArmyHammer v1.2.3!

## Highlights

- 🚀 New validate command for prompt validation
- 📊 Performance monitoring dashboard
- 🐛 Fixed file watcher memory leak
- 🔒 Security updates

## Installation

```bash
# Install with cargo
cargo install swissarmyhammer

# Or download binaries
https://github.com/wballard/swissarmyhammer/releases/tag/v1.2.3
```

## What's Changed

[Full changelog](https://github.com/wballard/swissarmyhammer/blob/main/CHANGELOG.md)

## Thank You

Thanks to all contributors who made this release possible!
```

Post to:
- GitHub Discussions
- Discord/Slack channels
- Twitter/Social media
- Dev.to/Medium article

### 3. Update Homebrew Formula

If maintaining Homebrew formula:

```ruby
class Swissarmyhammer < Formula
  desc "MCP server for prompt management"
  homepage "https://github.com/wballard/swissarmyhammer"
  version "1.2.3"
  
  if OS.mac? && Hardware::CPU.intel?
    url "https://github.com/wballard/swissarmyhammer/releases/download/v1.2.3/swissarmyhammer-v1.2.3-x86_64-apple-darwin.tar.gz"
    sha256 "HASH_HERE"
  elsif OS.mac? && Hardware::CPU.arm?
    url "https://github.com/wballard/swissarmyhammer/releases/download/v1.2.3/swissarmyhammer-v1.2.3-aarch64-apple-darwin.tar.gz"
    sha256 "HASH_HERE"
  elsif OS.linux?
    url "https://github.com/wballard/swissarmyhammer/releases/download/v1.2.3/swissarmyhammer-v1.2.3-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "HASH_HERE"
  end

  def install
    bin.install "swissarmyhammer"
  end
end
```

### 4. Monitor Release

```bash
# Check crates.io
open https://crates.io/crates/swissarmyhammer

# Monitor GitHub issues
gh issue list --label "v1.2.3"

# Check download stats
gh api repos/wballard/swissarmyhammer/releases/tags/v1.2.3
```

## Hotfix Process

For critical fixes:

```bash
# Create hotfix branch from tag
git checkout -b hotfix/v1.2.4 v1.2.3

# Make fixes
# ...

# Update version to 1.2.4
vim Cargo.toml

# Fast-track release
cargo test
cargo publish
git tag -a v1.2.4 -m "Hotfix: Critical bug in prompt loading"
git push origin v1.2.4
```

## Release Automation

### Release Script

`scripts/prepare-release.sh`:

```bash
#!/bin/bash
set -e

VERSION=$1
TYPE=${2:-patch} # patch, minor, major

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> [patch|minor|major]"
    exit 1
fi

echo "Preparing release v$VERSION ($TYPE)"

# Update version
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Update lock file
cargo update -p swissarmyhammer

# Run tests
echo "Running tests..."
cargo test --all-features

# Update changelog
echo "Updating CHANGELOG.md..."
# Auto-generate from commits
git log --pretty=format:"- %s (%h)" v$(cargo pkgid | cut -d# -f2)..HEAD >> CHANGELOG_NEW.md

# Build documentation
echo "Building documentation..."
cd doc && mdbook build && cd ..

# Create release notes
echo "# Release v$VERSION" > RELEASE_NOTES.md
echo "" >> RELEASE_NOTES.md
cat CHANGELOG_NEW.md >> RELEASE_NOTES.md

echo "Release preparation complete!"
echo "Next steps:"
echo "1. Review and edit CHANGELOG.md and RELEASE_NOTES.md"
echo "2. Commit changes"
echo "3. Create PR for release/v$VERSION"
echo "4. After merge, tag and push"
```

## Rollback Procedure

If issues are found after release:

1. **Yank from crates.io** (if critical):
   ```bash
   cargo yank --vers 1.2.3
   ```

2. **Update GitHub Release**:
   ```bash
   gh release edit v1.2.3 --prerelease
   ```

3. **Communicate**:
   - Post in announcements
   - Create issue for tracking
   - Prepare hotfix

4. **Fix and Re-release**:
   ```bash
   # Create fix
   git checkout -b fix/critical-issue
   # ... make fixes ...
   
   # New version
   cargo release patch
   ```

## Best Practices

1. **Test Thoroughly**
   - Run full test suite
   - Test on all platforms
   - Manual smoke tests

2. **Document Changes**
   - Update CHANGELOG.md
   - Write clear release notes
   - Update migration guides

3. **Communicate Clearly**
   - Announce deprecations early
   - Provide migration paths
   - Respond to feedback quickly

4. **Automate When Possible**
   - Use CI for builds
   - Automate version updates
   - Script repetitive tasks

## Next Steps

- Review [Contributing](./contributing.md) for development workflow
- See [Testing](./testing.md) for test requirements
- Check [Development](./development.md) for build setup
- Read [Changelog](./changelog.md) for version history