# SwissArmyHammer Codebase Architecture and Structure Patterns

## Workspace Organization

**Multi-Crate Workspace Pattern**
- Uses Cargo workspace with two main crates: `swissarmyhammer` (library) and `swissarmyhammer-cli` (binary)
- Shared dependencies defined at workspace level in root `Cargo.toml`
- Common configuration (linting, profiles) shared across workspace

**Layered Architecture**
- **Library Core** (`swissarmyhammer/`): Business logic, storage, templates, workflows
- **CLI Layer** (`swissarmyhammer-cli/`): Command-line interface, argument parsing, user interaction
- **Documentation** (`doc/`): mdBook-based documentation with examples
- **Testing** (`tests/`): Integration tests at workspace level

## Module Organization Patterns

**Domain-Driven Structure**
- Core modules organized by domain: `prompts`, `workflow`, `issues`, `memoranda`
- Cross-cutting concerns: `error`, `config`, `security`, `validation`
- Infrastructure modules: `storage`, `mcp`, `file_loader`, `search`

**Common Utilities Pattern**
- `common/` module with shared utilities: `error_context`, `file_types`, `validation_builders`
- `test_utils.rs` for testing infrastructure
- `prelude.rs` for convenient imports

**Hierarchical Module Structure**
- Complex features use nested modules: `workflow/`, `mcp/`, `issues/`
- Test modules co-located with functionality: `workflow/actions_tests/`
- Clear separation between public API and internal implementation

## File Organization Conventions

**Configuration First**
- `Cargo.toml` files define dependencies, features, and metadata
- `clippy.toml` for code quality rules
- `build.rs` for build-time code generation

**Documentation Integration**
- Comprehensive README files at multiple levels
- Inline documentation with `//!` and `///` comments
- `doc/` directory with structured documentation using mdBook

**Resource Management**
- `builtin/` directory for embedded resources
- `examples/` for usage demonstrations
- `benches/` for performance benchmarks

This architectural pattern supports maintainability through clear separation of concerns, modularity through workspace organization, and extensibility through plugin systems and trait abstractions.