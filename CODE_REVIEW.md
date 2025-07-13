# Code Review for issue/000127

## Todo Items

### Critical Issues

- [ ] Fix typo in CODING_STANDARDS.md: "resolutuion" should be "resolution"

### Design Issues

- [ ] Document the timeout change in workflow/actions.rs from 30 seconds to 1 hour (3600 seconds) - this seems like a significant change that needs justification and should be configurable
- [ ] Remove the unused `_workflow_dirs` parameter from `run_validate_command` function - keeping it for "compatibility" while ignoring it is confusing
- [ ] Consider a better format for workflow validation error paths than "workflow:name" - perhaps include the source location (builtin/user/local)

### Consistency Issues

- [ ] The new test file `test_doc_examples.rs` uses `walkdir::WalkDir` directly, which contradicts the fix in this issue that removed custom directory walking in favor of resolvers
- [ ] Consider if test_doc_examples.rs should use a similar resolver pattern for consistency

### Documentation

- [ ] Add more comprehensive documentation to CODING_STANDARDS.md beyond the single line about file loading
- [ ] Document the behavior change in validate command - it no longer accepts custom workflow directories
- [ ] Update any CLI help text or documentation that might reference the old workflow_dirs parameter

### Test Coverage

- [ ] Add integration tests that verify the validate command only loads workflows from standard locations (builtin, user, local)
- [ ] Test that workflows outside standard locations are NOT validated (negative test case)
- [ ] Verify that validate and flow list commands show the same workflows

### API Compatibility

- [ ] Consider if removing the workflow_dirs parameter from validate_all_with_options is a breaking change for any consumers of this library
- [ ] If it is, consider deprecation instead of removal