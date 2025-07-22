# Implement Memoranda CLI Integration

## Overview
Add memoranda commands to the SwissArmyHammer CLI, providing a complete command-line interface for memo management operations.

## Tasks

### 1. Add Memoranda Subcommand Structure
- Update `swissarmyhammer-cli/src/cli.rs` to include memoranda commands
- Add `MemoCommand` enum with all operations:
  - `Create { title: String, content: Option<String> }`
  - `Update { id: String, content: Option<String> }`
  - `Get { id: String }`
  - `Delete { id: String }`
  - `List`
  - `Search { query: String }`
  - `Context` (get all context)

### 2. CLI Handler Implementation
- Create `swissarmyhammer-cli/src/memo.rs` for memo CLI operations
- Implement handler functions for each memo command
- Handle stdin input for content when not provided as argument
- Format output for terminal display

### 3. CLI Error Handling
- Add memoranda error handling to `swissarmyhammer-cli/src/error.rs`
- Proper exit codes for different error conditions
- User-friendly error messages

### 4. Command Completion
- Update `swissarmyhammer-cli/src/completions.rs` for memo commands
- Add memo ID completion where applicable
- Search term completion if feasible

### 5. Integration with Main CLI
- Update `swissarmyhammer-cli/src/main.rs` to route memo commands
- Ensure proper initialization of memo storage
- Handle configuration and storage directory setup

## Command Examples
```bash
# Create memo
swissarmyhammer memo create "Meeting Notes" 
swissarmyhammer memo create "Task List" --content "1. Review code\n2. Write tests"

# List memos
swissarmyhammer memo list

# Search memos  
swissarmyhammer memo search "meeting"

# Get specific memo
swissarmyhammer memo get 01GX5Q2D1NPRZ3KXFW2H8V3A1Y

# Update memo
swissarmyhammer memo update 01GX5Q2D1NPRZ3KXFW2H8V3A1Y --content "Updated content"

# Delete memo
swissarmyhammer memo delete 01GX5Q2D1NPRZ3KXFW2H8V3A1Y

# Get all context (for AI)
swissarmyhammer memo context
```

## Implementation Notes
- Follow existing CLI patterns from issues and other commands
- Support both interactive and non-interactive modes
- Proper terminal output formatting with colors/highlighting
- stdin support for large content input

## Acceptance Criteria
- [ ] All memo commands implemented and functional
- [ ] Proper CLI argument parsing and validation
- [ ] Error handling with appropriate exit codes
- [ ] Command completion working
- [ ] Terminal output properly formatted
- [ ] Integration tests for CLI commands