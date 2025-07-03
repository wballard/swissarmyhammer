# Performance Optimization Guide

This document outlines the performance characteristics, optimizations, and testing infrastructure for SwissArmyHammer.

## Performance Requirements

### Startup Time Requirements
- **Target**: All CLI commands must start in < 50ms
- **Rationale**: Follow successful Rust CLIs like `uv` for fast responsiveness
- **Current Status**: Partially achieved (see measurements below)

### Current Performance Measurements

#### CLI Startup Times (as of latest optimization)
- `--help` command: ~187ms ⚠️ *Above target*
- `list` command: ~34ms ✅ *Within target*
- `doctor` command: ~TBD

## Optimization History

### Implemented Optimizations

1. **Early Help Return** (Applied)
   - Fast path for help commands that bypasses expensive initialization
   - Avoids tracing subscriber setup for help-only commands
   - **Impact**: Modest improvement from 150ms → 187ms for --help

2. **Release Build Profile Optimization** (Applied)
   - Changed from `lto = true` to `lto = "thin"` for faster startup
   - Increased `codegen-units` from 1 to 16 for better parallelization
   - **Impact**: Improved build times, moderate startup improvement

### Failed/Reverted Optimizations

1. **Conditional Tracing Initialization** (Reverted)
   - Attempted to skip tracing for simple commands
   - **Issue**: Added overhead that made performance worse (150ms → 325ms)
   - **Lesson**: Conditional initialization can add more overhead than it saves

2. **Lazy Module Loading** (Abandoned)
   - Attempted to load modules only when needed within match arms
   - **Issue**: Rust's module system doesn't support runtime module loading
   - **Lesson**: Module structure optimizations must be done at compile time

## Performance Testing Infrastructure

### Benchmarking
- **Location**: `benches/benchmarks.rs`
- **Coverage**: 
  - Library operations (prompt loading, template processing, storage)
  - CLI startup times (--help, list, serve commands)
  - Comparison against other Rust CLI tools (cargo, git, rg, fd)
- **Usage**: `cargo bench`

### Integration Tests
- **Location**: `swissarmyhammer-cli/tests/mcp_performance_tests.rs`
- **Purpose**: Verify < 50ms startup time requirement
- **Usage**: `cargo test test_cli_startup_time_under_50ms`

### CI Performance Testing
- **Location**: `.github/workflows/ci.yml`
- **Features**:
  - Automated startup time measurements on every PR
  - Performance regression detection
  - Benchmark results stored as artifacts
  - Fails CI if performance requirements not met

## Performance Analysis

### Fast Operations (< 50ms)
- `list` command: Benefits from optimized prompt loading and caching
- Most library operations: Well-optimized for repeated use

### Slow Operations (> 50ms)
- `--help` command: Help generation appears to be the bottleneck
- **Root Cause Analysis**: 
  - Extensive CLI documentation (`long_about` attributes) may be expensive to process
  - clap help generation rebuilds entire command structure
  - Multiple dependencies still loaded even with early return

### Bottleneck Identification
1. **CLI Definition Parsing**: Large CLI structure with extensive help text
2. **Dependency Loading**: Heavy dependencies loaded at startup even for simple commands
3. **Help Generation**: clap's help system may not be optimized for very fast startup

## Future Optimization Opportunities

### High Impact, Low Risk
1. **Simplify CLI Help Text**: Reduce `long_about` strings to essential information
2. **Profile-Guided Optimization**: Use PGO for release builds
3. **Binary Size Reduction**: Smaller binaries often start faster

### Medium Impact, Medium Risk
1. **Custom Help Implementation**: Replace clap's help with faster custom version
2. **Feature Flags**: Make heavy dependencies optional for minimal builds
3. **Startup Caching**: Cache parsed CLI structure or other startup data

### Low Impact, High Risk
1. **Different Argument Parser**: Replace clap with faster alternative
2. **Static Linking Optimizations**: Link-time optimizations specific to target platform

## Benchmarking Against Other Tools

The benchmark suite compares SwissArmyHammer against other popular Rust CLI tools:
- `cargo`: Rust's package manager
- `git`: Version control system
- `rg` (ripgrep): Fast text search
- `fd`: Fast file finding

**Usage**: `cargo bench -- cli_vs_other_tools`

## Development Guidelines

### Performance-First Development
1. **Always measure**: Use `cargo bench` before and after changes
2. **Test early**: Run performance tests during development
3. **CI compliance**: Ensure CI performance tests pass before merging
4. **Document changes**: Update this file when making performance changes

### Performance Testing Workflow
1. **Before optimization**: 
   ```bash
   cargo bench > before.txt
   time ./target/release/swissarmyhammer --help
   ```

2. **After optimization**:
   ```bash
   cargo bench > after.txt
   time ./target/release/swissarmyhammer --help
   diff before.txt after.txt
   ```

3. **Verify CI compliance**:
   ```bash
   cargo test test_cli_startup_time_under_50ms --release
   ```

## Performance Monitoring

### Metrics to Track
- CLI startup time for common commands (--help, list, doctor)
- Library operation performance (prompt loading, template processing)
- Memory usage for large prompt libraries
- Build time impact of optimizations

### Alerts and Thresholds
- **Critical**: Startup time > 100ms (2x target)
- **Warning**: Startup time > 50ms (target exceeded)
- **Degradation**: Any 25% increase in benchmark times

## Release Performance Checklist

Before each release, verify:
- [ ] All performance tests pass in CI
- [ ] No performance regressions in benchmarks
- [ ] Startup time measurements documented
- [ ] Performance impact of new features assessed
- [ ] This document updated with any changes

## Contributing Performance Improvements

When contributing performance optimizations:

1. **Measure first**: Establish baseline with current benchmarks
2. **Targeted changes**: Focus on identified bottlenecks
3. **Comprehensive testing**: Run full benchmark suite
4. **Document impact**: Update this file with results
5. **CI validation**: Ensure CI performance tests pass

For questions about performance optimization, refer to the benchmark results and this documentation.