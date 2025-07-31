# Integrate Shell Action with Dispatch System

Refer to ./specification/shell.md

## Overview

Integrate the shell action parser with the main action dispatch system so that shell actions can be recognized and instantiated when parsing workflow descriptions.

## Objective

Update the `parse_action_from_description` function to include shell action parsing, enabling workflows to use shell actions through the standard action parsing mechanism.

## Tasks

### 1. Update parse_action_from_description Function

In `/Users/wballard/github/swissarmyhammer/swissarmyhammer/src/workflow/actions.rs`, modify the `parse_action_from_description` function to include shell action parsing.

Add shell action parsing to the dispatch sequence:

```rust
pub fn parse_action_from_description(description: &str) -> ActionResult<Option<Box<dyn Action>>> {
    let parser = ActionParser::new()?;
    let description = description.trim();
    
    // Parse different action patterns using the robust parser
    if let Some(prompt_action) = parser.parse_prompt_action(description)? {
        return Ok(Some(Box::new(prompt_action)));
    }
    
    // ADD THIS: Shell action parsing
    if let Some(shell_action) = parser.parse_shell_action(description)? {
        return Ok(Some(Box::new(shell_action)));
    }
    
    if let Some(wait_action) = parser.parse_wait_action(description)? {
        return Ok(Some(Box::new(wait_action)));
    }
    
    // ... rest of existing parsers
}
```

### 2. Update Module Exports

In `/Users/wballard/github/swissarmyhammer/swissarmyhammer/src/workflow/mod.rs`, add `ShellAction` to the public exports:

```rust
pub use actions::{
    parse_action_from_description, parse_action_from_description_with_context, Action, ActionError,
    ActionResult, LogAction, LogLevel, PromptAction, SetVariableAction, ShellAction, // <- ADD THIS
    SubWorkflowAction, WaitAction,
};
```

### 3. Import Shell Action in Parser Module

Ensure the `ShellAction` type is available in the action_parser module by adding it to the imports at the top of `action_parser.rs`:

```rust
use crate::workflow::actions::{
    AbortAction, ActionError, ActionResult, LogAction, LogLevel, PromptAction, SetVariableAction,
    ShellAction, // <- ADD THIS  
    SubWorkflowAction, WaitAction,
};
```

### 4. Verify Integration Order

The order of parser calls in `parse_action_from_description` is important. Shell actions should be parsed:
- After prompt actions (to avoid conflicts with "Execute" keyword)
- Before other actions that might have generic patterns
- Follow the established ordering pattern

### 5. Add Basic Integration Test

Write a simple integration test to verify the dispatch system works:

```rust
#[test]
fn test_parse_shell_action_integration() {
    let action = parse_action_from_description("Shell \"echo hello\"")
        .unwrap()
        .unwrap();
    assert_eq!(action.action_type(), "shell");
    assert_eq!(action.description(), "Execute shell command: echo hello");
}
```

## Success Criteria

- [ ] Shell action parsing integrated into dispatch function
- [ ] Module exports updated to include ShellAction
- [ ] Imports updated in parser module
- [ ] Integration test passes
- [ ] No breaking changes to existing functionality
- [ ] All existing tests continue to pass

## Testing

Verify that:
1. Shell actions can be parsed through the main dispatch function
2. All existing action types still work correctly
3. Parser order doesn't create conflicts
4. Invalid shell syntax still returns None appropriately

Run the full test suite to ensure no regressions:
```bash
cargo test
```

## Edge Cases to Consider

- Ensure shell action parsing doesn't interfere with other action types
- Verify that partial matches don't create false positives
- Test that malformed shell syntax fails gracefully

## Next Steps

After completing this step, proceed to implementing the actual shell command execution logic.