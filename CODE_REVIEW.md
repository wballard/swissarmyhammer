# Code Review TODO List

## Issue #000145 - Partial Template Validation

### 1. [x] Skip Variable Validation for Partial Templates
**Location**: `swissarmyhammer-cli/src/validate.rs:208-218`
**Status**: COMPLETE - The fix correctly skips variable validation for partial templates
**Implementation**:
- Partials are identified in prompts.rs:1215-1220 using two methods:
  - Explicit: Files with `{% partial %}` marker
  - Implicit: Files detected by `is_likely_partial` heuristics
- Validation detects partials by checking for description "Partial template for reuse in other prompts"
- Skips both field validation and variable usage validation for partials
- Test added at lines 1949-1996 to verify the fix works correctly

### 2. [ ] Consider More Robust Partial Detection
**Location**: `swissarmyhammer-cli/src/validate.rs:172-176`
**Issue**: Partial detection relies on a specific description string which is fragile
**Action**:
- Consider adding a dedicated `is_partial` field to the Prompt struct
- Or use a more robust detection method (e.g., checking for {% partial %} marker in content)
- This would make the code less brittle and more maintainable

### 3. [ ] Add Integration Test for Partial Validation
**Issue**: While there's a unit test, there's no integration test for the full validation flow
**Action**:
- Add an integration test that validates actual .liquid partial files
- Test both explicit partials (with {% partial %} marker) and implicit partials
- Ensure the validation command output is correct for partials

## Minor Issues

### 4. [ ] Fix Cargo.toml Warning About Duplicate Binary Targets
**Location**: `swissarmyhammer-cli/Cargo.toml`
**Issue**: Warning appears in test output: "file src/main.rs found to be present in multiple build targets"
**Action**:
- Review the [[bin]] sections in Cargo.toml
- Ensure both `sah` and `swissarmyhammer` binaries are correctly configured
- Consider if they should share the same source file or have separate entry points

## Business Logic Migration from CLI to Library (Issue #000143)

### 1. [ ] Create Validation Module in Library
**Location**: `swissarmyhammer-cli/src/validate.rs:113-1107`
**Issue**: Entire validation framework exists in CLI but should be in library
**Action**:
- Create `swissarmyhammer/src/validation/` module
- Move `ContentValidator` trait to library
- Move all validator implementations (EncodingValidator, LineEndingValidator, YamlTypoValidator, etc.)
- Move Liquid syntax validation logic
- Move variable usage validation
- Move workflow validation logic
- Create proper module structure with `mod.rs`, `validators.rs`, `prompt_validator.rs`, `workflow_validator.rs`

### 2. [ ] Move Template Rendering with Environment Variables to Library
**Location**: `swissarmyhammer-cli/src/test.rs`
**Issue**: Environment variable support for template rendering is in CLI
**Action**:
- Add `render_with_env()` method to library's `Template` or `TemplateEngine`
- Move interactive argument collection logic as a utility
- Support environment variable interpolation in templates

### 3. [ ] Move Workflow Execution Utilities to Library
**Location**: `swissarmyhammer-cli/src/flow.rs`
**Issue**: Common workflow execution patterns are implemented in CLI
**Action**:
- Move variable parsing utilities to library
- Move interactive variable collection to library
- Move execution timeout handling to library
- Move test mode execution support to library
- Add methods: `parse_variables()`, `collect_variables_interactive()`

## Code Quality Issues

### 4. [ ] Fix Code Duplication Between CLI and Library
**Issue**: `PromptSource` enum and conversion logic is duplicated
**Action**:
- Remove duplicate `PromptSource` enum from CLI
- Use library's `PromptSource` directly in CLI
- Simplify conversion logic between CLI and library types

### 5. [ ] Add Integration Tests
**Issue**: Missing integration tests between CLI and library
**Action**:
- Create integration tests that verify CLI correctly uses library functionality
- Test end-to-end scenarios for filtering, searching, and validation

### 6. [ ] Enhance Documentation
**Issue**: Documentation could be more comprehensive
**Action**:
- Add module-level documentation for new library modules
- Add usage examples in documentation
- Document the separation of concerns between CLI and library

### 7. [ ] Fix Remaining Test Failures
**Issue**: Some tests may be failing due to refactoring
**Action**:
- Run `cargo test` and fix any failing tests
- Update tests that depend on moved functionality
- Ensure all tests pass in both CLI and library

## Architecture Improvements

### 8. [ ] Consider Creating a Facade Pattern for CLI
**Issue**: CLI modules directly use multiple library modules
**Action**:
- Consider creating a high-level API in the library that simplifies CLI usage
- Reduce coupling between CLI and library internals

### 9. [ ] Review Error Handling Consistency
**Issue**: Error handling patterns may differ between CLI and library
**Action**:
- Ensure consistent error types and handling
- Propagate library errors properly to CLI
- Provide helpful error messages to users

## New Code Quality Issues (2025-07-14)

### 10. [ ] Extract Common Argument Validation Logic
**Location**: `test.rs:342-350`, `prompts.rs:298-376`
**Issue**: Argument validation logic is duplicated across multiple render methods
**Action**:
- Create a shared trait or module for argument validation
- Consolidate validation logic from `render_prompt`, `render_prompt_with_env`, and Prompt methods
- Reduce code duplication in argument processing

### 11. [ ] Fix Performance Issues with Regular Expressions
**Location**: `template.rs:217-224`
**Issue**: Regular expressions are compiled on every call to `extract_template_variables`
**Action**:
- Use `lazy_static` or `once_cell` for compiled regular expressions
- Cache regex patterns that are used repeatedly
- Profile template parsing performance after optimization

### 12. [ ] Split Large Validation Module
**Location**: `validate.rs` (1945 lines)
**Issue**: The validate.rs file is too large and handles multiple concerns
**Action**:
- Split into separate modules: prompt_validator.rs, workflow_validator.rs, content_validator.rs
- Create clear module boundaries and interfaces
- Improve code organization and maintainability

### 13. [ ] Add Missing Tests for New Environment Variable Features
**Location**: `test.rs`, `prompts.rs`, `template.rs`
**Issue**: New `render_with_env` methods lack test coverage
**Action**:
- Add tests for `render_prompt_with_env` in test.rs
- Add tests for `render_with_partials_and_env` in prompts.rs
- Add tests for environment variable edge cases and error handling
- Test environment variable precedence and override behavior

### 14. [ ] Replace Hard-coded Values with Configuration
**Location**: Multiple files
**Issue**: Magic numbers and strings are hard-coded throughout
**Action**:
- Extract hard-coded emojis (test.rs:131) to constants
- Make validation thresholds configurable (validate.rs:321)
- Create named constants for line limits and complexity thresholds
- Move source location strings to enums or constants

### 15. [ ] Improve Error Context and Messages
**Location**: Throughout codebase
**Issue**: Error messages lack context about operations and files
**Action**:
- Add file paths to validation error messages
- Include operation context in error types
- Make validation suggestions more actionable
- Provide better error recovery hints

### 16. [ ] Add Input Validation for File Paths
**Location**: `test.rs:298`
**Issue**: User-provided file paths are used without validation
**Action**:
- Validate file paths before use
- Check for path traversal attempts
- Ensure paths are within expected directories
- Add proper error handling for invalid paths

### 17. [ ] Create Abstraction for Partial Resolution
**Location**: `template.rs:99-181`
**Issue**: Partial resolution logic could be more modular
**Action**:
- Extract partial resolution into a `PartialResolver` trait
- Allow custom partial resolution strategies
- Improve testability of partial resolution logic

### 18. [ ] Optimize Directory Scanning Performance
**Location**: `prompts.rs:1004`
**Issue**: WalkDir usage without filtering could be slow
**Action**:
- Add file extension filtering to WalkDir
- Implement parallel directory scanning for large trees
- Add progress reporting for long operations
- Consider caching directory scan results

### 19. [ ] Consolidate Metadata Parsing Logic
**Location**: `prompts.rs:1025-1219`
**Issue**: Duplicate metadata parsing in `load_file_with_base` and `load_from_string`
**Action**:
- Extract metadata parsing into a shared function
- Reduce code duplication in YAML parsing
- Improve error handling for malformed metadata

### 20. [ ] Implement Builder Pattern for Validation Configuration
**Location**: `validation/mod.rs`
**Issue**: Complex validation pipelines are hard to configure
**Action**:
- Create a ValidationConfigBuilder
- Allow fluent configuration of validators
- Simplify validation setup for common use cases
- Add preset configurations for typical scenarios

## MCP Server Shutdown Issues (Issue #000144)

### 21. [ ] Implement Proper Server Shutdown on Client Disconnect
**Location**: `swissarmyhammer-cli/src/main.rs:178-196`
**Issue**: Server only has a TODO comment but no actual implementation for exiting when Claude disconnects
**Action**:
- Research rmcp crate API for detecting transport closure
- Implement detection of EOF on stdin (client disconnect)
- Add graceful shutdown when stdio transport closes
- Ensure server exits cleanly without requiring Ctrl+C

### 22. [ ] Add Timeout for Server Shutdown
**Location**: `swissarmyhammer-cli/src/main.rs:186`
**Issue**: No timeout protection if shutdown hangs
**Action**:
- Add a configurable timeout for graceful shutdown
- Force exit if shutdown takes too long
- Log appropriate messages during shutdown process

### 23. [ ] Create Integration Tests for Server Lifecycle
**Location**: New test file needed
**Issue**: No tests verify server properly starts and stops
**Action**:
- Create tests that verify server exits when client disconnects
- Test Ctrl+C shutdown path
- Test error scenarios during startup and shutdown
- Verify proper cleanup of resources

### 24. [ ] Research Alternative Shutdown Detection Methods
**Location**: `swissarmyhammer-cli/src/main.rs`
**Issue**: rmcp may not provide direct transport closure detection
**Action**:
- Investigate spawning a separate task to monitor stdin for EOF
- Research if rmcp provides callbacks or events for transport status
- Consider implementing a heartbeat mechanism as fallback
- Document findings and chosen approach

### 25. [ ] Remove TODO Comment After Implementation
**Location**: `swissarmyhammer-cli/src/main.rs:183-185`
**Issue**: TODO comment was added but no actual implementation was done
**Action**:
- After implementing proper shutdown detection, remove the TODO comment
- Replace with actual working code that handles client disconnection
- Ensure the issue is fully resolved, not just documentedwarning: /Users/wballard/github/swissarmyhammer/swissarmyhammer-cli/Cargo.toml: file `/Users/wballard/github/swissarmyhammer/swissarmyhammer-cli/src/main.rs` found to be present in multiple build targets:
  * `bin` target `sah`
  * `bin` target `swissarmyhammer`
   Compiling swissarmyhammer v0.1.0 (/Users/wballard/github/swissarmyhammer/swissarmyhammer)
   Compiling swissarmyhammer-cli v0.1.0 (/Users/wballard/github/swissarmyhammer/swissarmyhammer-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.01s
     Running unittests src/lib.rs (target/debug/deps/swissarmyhammer-c3030e01d2c52822)

running 401 tests
test file_loader::tests::test_file_source_display ... ok
test file_loader::tests::test_file_entry_creation ... ok
test error::tests::test_error_context ... ok
test file_loader::tests::test_file_entry_name_from_path ... ok
test file_loader::tests::test_file_source_equality ... ok
test error::tests::test_error_chain_display ... ok
test file_loader::tests::test_file_entry_nested_name ... ok
test file_loader::tests::test_virtual_file_system_new ... ok
test file_watcher::tests::test_file_watcher_config_default ... ok
test directory_utils::tests::test_find_swissarmyhammer_dirs_upward ... ok
test file_loader::tests::test_virtual_file_system_precedence ... ok
test file_loader::tests::test_virtual_file_system_add_builtin ... ok
test file_loader::tests::test_virtual_file_system_get_source ... ok
test directory_utils::tests::test_walk_files_with_extensions ... ok
test file_loader::tests::test_virtual_file_system_load_directory ... ok
test file_watcher::tests::test_is_prompt_file ... ok
test file_loader::tests::test_virtual_file_system_list ... ok
test fs_utils::tests::test_mock_filesystem_read_write ... ok
test file_watcher::tests::test_file_watcher_creation ... ok
test fs_utils::tests::test_json_serialization ... ok
test fs_utils::tests::test_yaml_serialization ... ok
test mcp::tests::test_mcp_server_exposes_prompt_capabilities ... ok
test mcp::tests::test_mcp_server_exposes_workflow_tools_capability ... ok
test mcp::tests::test_mcp_server_does_not_expose_partial_templates ... ok
test mcp::tests::test_mcp_server_creation ... ok
test plugins::tests::test_duplicate_plugin_registration ... ok
test plugins::tests::test_filter_application ... ok
test plugins::tests::test_plugin_registration ... ok
test plugins::tests::test_plugin_registry_creation ... ok
test prompt_filter::tests::test_combined_filters ... ok
test prompt_filter::tests::test_filter_by_arguments ... ok
test prompt_filter::tests::test_filter_by_category ... ok
test prompt_filter::tests::test_filter_by_search_term ... ok
test prompt_filter::tests::test_filter_by_source ... ok
test mcp::tests::test_mcp_server_list_prompts ... ok
test mcp::tests::test_mcp_server_uses_same_directory_discovery ... ok
test prompt_resolver::tests::test_get_prompt_directories ... ok
test mcp::tests::test_mcp_server_file_watching_integration ... ok
test prompt_resolver::tests::test_user_prompt_overrides_builtin_source_tracking ... ignored
test prompts::tests::test_extension_stripping ... ok
test prompts::tests::test_partial_template_without_description ... ok
test prompts::tests::test_prompt_creation ... ok
test prompt_resolver::tests::test_prompt_resolver_loads_local_prompts ... ok
test prompt_resolver::tests::test_prompt_resolver_loads_user_prompts ... ok
test prompt_resolver::tests::test_debug_error_prompt_is_correctly_tracked_as_builtin ... ok
test mcp::tests::test_mcp_server_uses_same_prompt_paths_as_cli ... ok
test prompts::tests::test_prompt_loader_loads_only_valid_prompts ... ok
test search::tests::test_default_search_engine ... ok
test mcp::tests::test_mcp_server_graceful_error_for_missing_prompt ... ok
test mcp::tests::test_mcp_server_get_prompt ... ok
test prompts::tests::test_prompt_render ... ok
test search::tests::test_commit ... ok
test search::tests::test_full_text_search ... ok
test search::tests::test_full_text_search_category ... ok
test search::tests::test_full_text_search_complex_query ... ok
test search::tests::test_full_text_search_description ... ok
test search::tests::test_fuzzy_search ... ok
test file_watcher::tests::test_file_watcher_start_stop ... ok
test search::tests::test_full_text_search_empty_query ... ok
test search::tests::test_fuzzy_search_category_match ... ok
test search::tests::test_fuzzy_search_description_match ... ok
test search::tests::test_fuzzy_search_empty_query ... ok
test search::tests::test_fuzzy_search_empty_prompts ... ok
test search::tests::test_fuzzy_search_no_match ... ok
test search::tests::test_fuzzy_search_sorting ... ok
test search::tests::test_fuzzy_search_tag_match ... ok
test search::tests::test_hybrid_search_empty_index ... ok
test search::tests::test_full_text_search_no_match ... ok
test search::tests::test_full_text_search_tags ... ok
test search::tests::test_hybrid_search_score_combination ... ok
test search::tests::test_index_multiple_prompts ... ok
test search::tests::test_full_text_search_template ... ok
test search::tests::test_hybrid_search ... ok
test search::tests::test_score_weighting_in_fuzzy_search ... ok
test search::tests::test_multiple_tag_scoring ... ok
test search::tests::test_search_engine_creation ... ok
test search::tests::test_search_result_clone ... ok
test search::tests::test_search_result_creation ... ok
test search_advanced::tests::test_excerpt_generation ... ok
test search_advanced::tests::test_excerpt_generation_with_highlight ... ok
test search::tests::test_hybrid_search_combines_results ... ok
test search_advanced::tests::test_search_with_filter ... ok
test search_advanced::tests::test_regex_search ... ok
test search_advanced::tests::test_search_with_limit ... ok
test search_advanced::tests::test_simple_search_case_insensitive ... ok
test search_advanced::tests::test_simple_search_case_sensitive ... ok
test search::tests::test_index_prompt_minimal_fields ... ok
test search::tests::test_index_prompt_with_all_fields ... ok
test search::tests::test_index_single_prompt ... ok
test security::tests::test_validate_workflow_complexity_exceeds_limits ... ok
test security::tests::test_validate_workflow_complexity_within_limits ... ok
test search::tests::test_prompt_not_found_in_search_results ... ok
test storage::tests::test_filesystem_storage_creation ... ok
test security::tests::test_validate_path_security_parent_dir ... ok
test security::tests::test_validate_path_security_safe_path ... ok
test storage::tests::test_filesystem_storage_get_nonexistent ... ok
test storage::tests::test_filesystem_storage_clone_box ... ok
test security::tests::test_validate_path_security_absolute_path ... ok
test security::tests::test_calculate_path_depth ... ok
test storage::tests::test_filesystem_storage_exists_and_count ... ok
test storage::tests::test_filesystem_storage_invalid_yaml_file ... ok
test storage::tests::test_filesystem_storage_remove_nonexistent ... ok
test storage::tests::test_filesystem_storage_non_yaml_files ... ok
test storage::tests::test_filesystem_storage_nonexistent_directory ... ok
test storage::tests::test_memory_storage ... ok
test storage::tests::test_memory_storage_clone_box ... ok
test storage::tests::test_memory_storage_count ... ok
test storage::tests::test_memory_storage_default ... ok
test storage::tests::test_memory_storage_exists ... ok
test storage::tests::test_memory_storage_get_nonexistent ... ok
test storage::tests::test_memory_storage_remove_nonexistent ... ok
test storage::tests::test_filesystem_storage_list ... ok
test storage::tests::test_filesystem_storage_remove ... ok
test storage::tests::test_prompt_storage_memory ... ok
test storage::tests::test_prompt_storage_new_with_backend ... ok
test search::tests::test_search_engine_with_directory ... ok
test storage::tests::test_search ... ok
test storage::tests::test_search_by_category ... ok
test storage::tests::test_search_by_description ... ok
test storage::tests::test_search_by_tags ... ok
test storage::tests::test_search_case_insensitive ... ok
test storage::tests::test_search_empty_query ... ok
test storage::tests::test_search_no_matches ... ok
test storage::tests::test_storage_backend_exists_error_handling ... ok
test storage::tests::test_prompt_path_generation ... ok
test storage::tests::test_filesystem_storage_reload_cache ... ok
test storage::tests::test_filesystem_storage_search ... ok
test storage::tests::test_filesystem_storage_store_and_get ... ok
test storage::tests::test_prompt_storage_file_system ... ok
test search::tests::test_search_engine_with_directory_nonexistent_path ... ok
test template::tests::test_empty_template ... ok
test template::tests::test_default_value ... ok
test template::tests::test_boolean_value ... ok
test template::tests::test_extract_template_variables_long_names ... ok
test template::tests::test_extract_template_variables_duplicates ... ok
test template::tests::test_extract_template_variables_no_recursive_parsing ... ok
test template::tests::test_extract_template_variables ... ok
test template::tests::test_extract_template_variables_for_loops ... ok
test template::tests::test_extract_template_variables_unicode ... ok
test template::tests::test_extract_template_variables_with_conditionals ... ok
test template::tests::test_missing_argument_no_validation ... ok
test template::tests::test_liquid_default_filter_with_missing_variable ... ok
test template::tests::test_liquid_default_filter_multiple_variables ... ok
test template::tests::test_liquid_default_filter_with_provided_variable ... ok
test template::tests::test_extract_template_variables_whitespace_variations ... ok
test template::tests::test_multiple_occurrences ... ok
test test_utils::tests::test_concurrent_access ... ok
test test_utils::tests::test_create_simple_test_prompt ... ok
test template::tests::test_no_placeholders ... ok
test test_utils::tests::test_create_test_prompt_library ... ok
test test_utils::tests::test_create_test_prompts ... ok
test test_utils::tests::test_guard_restores_home ... ok
test test_utils::tests::test_setup_test_home ... ok
test test_utils::tests::test_create_temp_prompt_dir ... ok
test validation::tests::test_encoding_validator ... ok
test validation::tests::test_line_ending_validator ... ok
test validation::tests::test_validation_manager ... ok
test validation::tests::test_validation_result_add_error ... ok
test validation::tests::test_validation_result_creation ... ok
test validation::tests::test_yaml_typo_validator ... ok
test workflow::action_parser::tests::test_parse_log_action ... ok
test workflow::action_parser::tests::test_parse_prompt_action ... ok
test workflow::action_parser::tests::test_parse_set_variable_action ... ok
test test_utils::tests::test_test_file_system ... ok
test workflow::action_parser::tests::test_parse_sub_workflow_action ... ok
test workflow::action_parser::tests::test_parse_wait_action ... ok
test workflow::actions::tests::test_log_action_execution ... ok
test workflow::actions::tests::test_parse_log_action ... ok
test workflow::action_parser::tests::test_variable_substitution ... ok
test workflow::actions::tests::test_parse_set_variable_action ... ok
test workflow::actions::tests::test_parse_prompt_action ... ok
test workflow::actions::tests::test_parse_sub_workflow_action ... ok
test workflow::actions::tests::test_parse_wait_action ... ok
test workflow::actions::tests::test_prompt_action_builder_methods ... ok
test workflow::actions::tests::test_prompt_action_with_max_retries_builder ... ok
test workflow::actions::tests::test_prompt_action_with_quiet ... ok
test workflow::actions::tests::test_prompt_action_with_rate_limit_retry ... ok
test workflow::actions::tests::test_sub_workflow_circular_dependency_detection ... ok
test workflow::actions::tests::test_set_variable_action_execution ... ok
test template::tests::test_numeric_value ... ok
test template::tests::test_render_with_env ... ok
test workflow::actions::tests::test_variable_substitution ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_empty ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_log ... ok
test workflow::actions::tests::test_sub_workflow_variable_substitution ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_no_match ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_prompt ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_set_variable ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_sub_workflow ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_wait ... ok
test workflow::actions_tests::action_parsing_tests::test_parse_action_from_description_whitespace ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_empty_string ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_arrays ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_invalid_json ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_nested_objects ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_whitespace ... ok
test template::tests::test_render_with_env_args_override ... ok
test template::tests::test_special_characters ... ok
test template::tests::test_required_argument_validation ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_log_actions ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_action_error_isolation ... ok
test template::tests::test_simple_template ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_set_variable_actions ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_sub_workflow_actions ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_with_syntax_highlighting ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_source_code ... ok
test workflow::actions_tests::error_handling_tests::test_action_error_display ... ok
test workflow::actions_tests::error_handling_tests::test_action_error_from_io_error ... ok
test workflow::actions_tests::error_handling_tests::test_action_error_from_json_error ... ok
test workflow::actions_tests::integration_tests::test_action_context_key_constants ... ok
test workflow::actions_tests::integration_tests::test_action_error_propagation ... ok
test workflow::actions_tests::integration_tests::test_action_execution_context_preservation ... ok
test workflow::actions_tests::integration_tests::test_multiple_actions_sequence ... ok
test workflow::actions_tests::resource_cleanup_tests::test_action_cleanup_on_failure ... ok
test workflow::actions_tests::resource_cleanup_tests::test_action_cleanup_with_multiple_errors ... ok
test workflow::actions_tests::resource_cleanup_tests::test_action_context_cleanup_on_panic ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_mixed_content ... ok
test workflow::actions_tests::claude_output_formatting_tests::test_format_claude_output_as_yaml_multiline_string ... ok
test workflow::actions_tests::resource_cleanup_tests::test_log_action_file_handle_cleanup ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_mixed_actions ... ok
test workflow::actions_tests::resource_cleanup_tests::test_sub_workflow_cleanup_on_circular_dependency ... ok
test workflow::actions_tests::resource_cleanup_tests::test_wait_action_cancellation_cleanup ... ok
test workflow::cache::tests::test_cache_manager_combined_operations ... ok
test workflow::cache::tests::test_cache_stats_hit_rate ... ok
test workflow::cache::tests::test_cel_cache_compilation_timing ... ok
test workflow::cache::tests::test_cel_program_cache ... ok
test workflow::actions_tests::integration_tests::test_action_timeout_behavior ... ok
test workflow::cache::tests::test_workflow_cache_basic_operations ... ok
test workflow::cache::tests::test_workflow_cache_eviction ... ok
test workflow::definition::tests::test_workflow_validation_missing_initial_state ... ok
test workflow::definition::tests::test_workflow_validation_no_terminal_state ... ok
test workflow::definition::tests::test_workflow_validation_success ... ok
test workflow::error_utils::tests::test_command_succeeded ... ok
test workflow::error_utils::tests::test_extract_stdout_stderr ... ok
test workflow::error_utils::tests::test_handle_claude_command_error_failure ... ok
test workflow::error_utils::tests::test_handle_claude_command_error_rate_limit ... ok
test workflow::error_utils::tests::test_handle_claude_command_error_success ... ok
test workflow::error_utils::tests::test_handle_command_error_failure ... ok
test workflow::error_utils::tests::test_handle_command_error_success ... ok
test workflow::error_utils::tests::test_handle_command_error_with_mapper ... ok
test workflow::error_utils::tests::test_is_rate_limit_error ... ok
test workflow::error_utils::tests::test_time_until_next_hour ... ok
test workflow::actions_tests::resource_cleanup_tests::test_concurrent_action_resource_cleanup ... ok
test workflow::executor::tests::test_cel_expression_complex_json_handling ... ok
test workflow::executor::tests::test_cel_expression_caching_behavior ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_wait_actions ... ok
test workflow::executor::tests::test_cel_expression_length_limits ... ok
test workflow::executor::tests::test_cel_expression_nesting_limits ... ok
test workflow::executor::tests::test_cel_expression_security_validation ... ok
test workflow::executor::tests::test_cel_expression_suspicious_quotes ... ok
test workflow::executor::tests::test_cel_expression_invalid_syntax ... ok
test workflow::executor::tests::test_choice_state_determinism_validation ... ok
test workflow::executor::tests::test_cel_expression_evaluation ... ok
test workflow::executor::tests::test_choice_state_never_condition_validation ... ok
test workflow::executor::tests::test_choice_state_no_matching_conditions ... ok
test workflow::executor::tests::test_choice_state_execution ... ok
test workflow::executor::tests::test_choice_state_no_transitions ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_action_context_consistency ... ok
test workflow::executor::tests::test_custom_condition_without_expression ... ok
test workflow::executor::tests::test_cel_expression_with_variables ... ok
test workflow::executor::tests::test_choice_state_with_cel_conditions ... ok
test workflow::actions_tests::resource_cleanup_tests::test_semaphore_cleanup_in_rate_limited_actions ... ok
test workflow::executor::tests::test_evaluate_transitions_always_condition ... ok
test workflow::executor::tests::test_execution_history_limit ... ok
test workflow::cache::tests::test_transition_cache_with_ttl ... ok
test workflow::executor::tests::test_fork_join_context_merging ... ok
test workflow::executor::tests::test_fork_join_parallel_execution ... ok
test workflow::executor::tests::test_manual_intervention_recovery ... ok
test workflow::executor::tests::test_max_transition_limit ... ok
test workflow::executor::tests::test_never_condition ... ok
test workflow::executor::tests::test_on_failure_condition_with_context ... ok
test workflow::executor::tests::test_on_success_condition_with_context ... ok
test workflow::executor::tests::test_resume_completed_workflow_fails ... ok
test workflow::actions_tests::resource_cleanup_tests::test_prompt_action_cleanup_on_timeout ... ok
test workflow::actions_tests::concurrent_action_tests::test_concurrent_prompt_action_rate_limiting ... ok
test workflow::executor::tests::test_compensation_rollback ... ok
test workflow::executor::tests::test_fallback_state_on_error ... ok
test workflow::executor::tests::test_error_context_capture ... ok
test workflow::executor::tests::test_start_workflow ... ok
test workflow::executor::tests::test_error_handler_state ... ok
test workflow::executor::tests::test_say_hello_workflow ... ok
test workflow::executor::tests::test_transition_order_evaluation ... ok
test workflow::graph::tests::test_detect_cycle ... ok
test workflow::graph::tests::test_find_reachable_states ... ok
test workflow::graph_tests::tests::test_adjacency_list_with_multiple_transitions ... ok
test workflow::executor::tests::test_transition_to_invalid_state ... ok
test workflow::graph_tests::tests::test_analyzer_with_isolated_states ... ok
test workflow::graph_tests::tests::test_build_adjacency_list_basic ... ok
test workflow::graph_tests::tests::test_build_adjacency_list_empty ... ok
test workflow::graph_tests::tests::test_build_adjacency_list_complex ... ok
test workflow::executor::tests::test_workflow_execution_to_completion ... ok
test workflow::graph_tests::tests::test_detect_all_cycles_multiple_cycles ... ok
test workflow::graph_tests::tests::test_detect_all_cycles_no_cycles ... ok
test workflow::graph_tests::tests::test_detect_all_cycles_single_cycle ... ok
test workflow::graph_tests::tests::test_detect_cycle_from_no_cycle ... ok
test workflow::graph_tests::tests::test_detect_cycle_from_simple_cycle ... ok
test workflow::graph_tests::tests::test_detect_cycle_from_with_cycle ... ok
test workflow::graph_tests::tests::test_find_paths_multiple_paths ... ok
test workflow::graph_tests::tests::test_find_paths_no_path ... ok
test workflow::graph_tests::tests::test_find_paths_same_state ... ok
test workflow::graph_tests::tests::test_find_paths_single_path ... ok
test workflow::graph_tests::tests::test_find_paths_with_cycle ... ok
test workflow::graph_tests::tests::test_find_reachable_states_basic ... ok
test workflow::graph_tests::tests::test_find_reachable_states_empty ... ok
test workflow::graph_tests::tests::test_find_reachable_states_with_cycles ... ok
test workflow::graph_tests::tests::test_find_unreachable_states ... ok
test workflow::graph_tests::tests::test_find_unreachable_states_none ... ok
test workflow::graph_tests::tests::test_graph_analyzer_creation ... ok
test workflow::graph_tests::tests::test_graph_error_display ... ok
test workflow::graph_tests::tests::test_topological_sort_acyclic ... ok
test workflow::graph_tests::tests::test_topological_sort_empty_workflow ... ok
test workflow::graph_tests::tests::test_topological_sort_complex_acyclic ... ok
test workflow::graph_tests::tests::test_topological_sort_single_state ... ok
test workflow::graph_tests::tests::test_topological_sort_with_cycle ... ok
test workflow::metrics::tests::test_memory_metrics ... ok
test workflow::metrics::tests::test_workflow_metrics_new ... ok
test workflow::metrics::tests::test_record_state_execution ... ok
test workflow::metrics::tests::test_start_run ... ok
test workflow::parser::tests::test_extract_actions_without_bold_markers ... ok
test workflow::parser::tests::test_no_initial_state_error ... ok
test workflow::parser::tests::test_parse_transition_condition ... ok
test workflow::parser::tests::test_parse_simple_state_diagram ... ok
test workflow::parser::tests::test_parse_fork_join_diagram ... ok
test workflow::parser::tests::test_parse_state_diagram_with_actions ... ok
test workflow::parser::tests::test_parse_nested_fork_join_diagram ... ok
test workflow::parser::tests::test_unreachable_states_validation ... ok
test workflow::run::tests::test_workflow_run_completion ... ok
test workflow::run::tests::test_workflow_run_creation ... ok
test workflow::run::tests::test_workflow_run_id_creation ... ok
test workflow::run::tests::test_workflow_run_id_parse_invalid ... ok
test workflow::run::tests::test_workflow_run_id_parse_and_to_string ... ok
test workflow::run::tests::test_workflow_run_id_parse_valid_ulid ... ok
test workflow::state::tests::test_state_creation ... ok
test workflow::state::tests::test_state_id_creation ... ok
test workflow::run::tests::test_workflow_run_transition ... ok
test workflow::parser::tests::test_parse_wrong_diagram_type ... ok
test workflow::state::tests::test_state_id_try_new_empty_error ... ok
test workflow::state::tests::test_state_id_try_new_success ... ok
test workflow::state::tests::test_state_serialization ... ok
test workflow::storage::tests::test_cleanup_old_runs ... ok
test workflow::state::tests::test_state_id_new_panics_on_empty - should panic ... ok
test workflow::storage::tests::test_combined_workflow_storage ... ok
test workflow::storage::tests::test_memory_workflow_run_storage ... ok
test workflow::storage::tests::test_memory_workflow_storage ... ok
test workflow::storage::tests::test_parse_hello_world_workflow ... ok
test workflow::storage::tests::test_workflow_directories ... ok
test workflow::storage::tests::test_workflow_resolver_precedence ... ignored, Test depends on dirs::home_dir() behavior which varies by platform
test workflow::storage::tests::test_workflow_resolver_user_workflows ... ignored, Test depends on dirs::home_dir() behavior which varies by platform
test workflow::test_liquid_rendering::tests::test_action_parsing_with_array_template_value ... ok
test workflow::storage::tests::test_compressed_storage_integration ... ok
test workflow::storage::tests::test_compressed_workflow_storage ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_default_values ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_complex_object_template_value ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_empty_template_vars ... ok
test workflow::storage::tests::test_builtin_workflows_loaded ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_invalid_liquid_syntax ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_missing_template_var ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_liquid_templates ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_null_template_value ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_nested_liquid_errors ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_special_characters_in_template ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_without_templates ... ok
test workflow::test_liquid_rendering::tests::test_action_parsing_with_undefined_filter ... ok
test workflow::transition::tests::test_transition_creation ... ok
test workflow::transition_key::tests::test_transition_key_creation ... ok
test workflow::transition_key::tests::test_transition_key_display ... ok
test workflow::transition_key::tests::test_transition_key_equality ... ok
test workflow::transition_key::tests::test_transition_key_from_refs ... ok
test workflow::test_liquid_rendering::tests::test_set_variable_action_with_template ... ok
test workflow::test_liquid_rendering::tests::test_prompt_action_with_template_in_arguments ... ok
test workflow::visualization_tests::tests::test_color_scheme_default ... ok
test workflow::visualization_tests::tests::test_constants_are_reasonable ... ok
test workflow::storage::tests::test_workflow_resolver_local_workflows ... ok
test workflow::visualization_tests::tests::test_execution_step_creation ... ok
test workflow::visualization_tests::tests::test_execution_visualizer_creation ... ok
test workflow::visualization_tests::tests::test_execution_step_serialization ... ok
test workflow::visualization_tests::tests::test_execution_visualizer_default ... ok
test workflow::visualization_tests::tests::test_execution_visualizer_minimal ... ok
test workflow::visualization_tests::tests::test_execution_trace_serialization ... ok
test workflow::visualization_tests::tests::test_export_trace_json ... ok
test workflow::visualization_tests::tests::test_generate_execution_report ... ok
test workflow::visualization_tests::tests::test_generate_execution_report_with_errors ... ok
test workflow::visualization_tests::tests::test_generate_html_basic ... ok
test workflow::visualization_tests::tests::test_generate_mermaid_empty_execution ... ok
test workflow::visualization_tests::tests::test_generate_html_xss_prevention ... ok
test workflow::visualization_tests::tests::test_generate_mermaid_with_execution ... ok
test workflow::visualization_tests::tests::test_generate_mermaid_with_timing ... ok
test workflow::visualization_tests::tests::test_generate_mermaid_without_timing ... ok
test workflow::visualization_tests::tests::test_generate_html_dos_protection ... ok
test workflow::visualization_tests::tests::test_generate_trace_basic ... ok
test workflow::visualization_tests::tests::test_generate_trace_empty_workflow ... ok
test workflow::visualization_tests::tests::test_generate_trace_with_metrics ... ok
test workflow::visualization_tests::tests::test_visualization_format_display ... ok
test workflow::visualization_tests::tests::test_mermaid_state_and_transition_formatting ... ok
test workflow::visualization_tests::tests::test_html_escape_integration ... ok
test workflow::visualization_tests::tests::test_visualization_options_default ... ok
test workflow::visualization_tests::tests::test_mermaid_unexecuted_states ... ok
test workflow::executor::tests::test_skip_failed_state ... ok
test workflow::executor::tests::test_retry_with_exponential_backoff ... ok
test workflow::executor::tests::test_dead_letter_state ... ok

test result: ok. 398 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.80s

     Running tests/integration_tests.rs (target/debug/deps/integration_tests-1828f3e76d1f565f)

running 17 tests
test test_custom_filters ... ignored, Custom filters not yet implemented
test test_library_creation ... ok
test test_library_search ... ok
test test_missing_required_argument ... ok
test test_liquid_file_extension_loading ... ok
test test_mcp_server ... ok
test test_prompt_loader ... ok
test test_partial_rendering_without_variables ... ok
test test_partial_rendering_with_md_extension ... ok
test test_md_liquid_extension ... ok
test test_partial_rendering ... ok
test test_prompt_creation_and_rendering ... ok
test test_example_usage ... ok
test test_library_with_directory ... ok
test test_prompt_with_arguments ... ok
test test_template_engine ... ok
test test_search_engine ... ok

test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/plugin_tests.rs (target/debug/deps/plugin_tests-7e8e1d0b6cefd733)

running 10 tests
test test_template_engine_with_multiple_custom_filters ... ignored
test test_template_engine_with_plugins ... ignored
test test_plugin_initialization_and_cleanup ... ok
test test_plugin_registration_and_basic_usage ... ok
test test_multiple_plugin_registration ... ok
test test_custom_filter_with_non_string_input ... ok
test test_duplicate_plugin_registration_fails ... ok
test test_template_engine_plugin_registry_access ... ok
test test_empty_plugin_registry ... ok
test test_template_engine_without_plugins ... ok

test result: ok. 8 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/test_home_integration.rs (target/debug/deps/test_home_integration-815fe5f3003f4577)

running 3 tests
test test_home_directory_override_works ... ok
test test_prompt_resolver_with_test_home ... ok
test test_prompt_loading_with_test_home ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/test_partials_issue.rs (target/debug/deps/test_partials_issue-5c3051c37d02db97)

running 2 tests
test test_partials_with_liquid_extension ... ok
test test_partials_without_extension ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/lib.rs (target/debug/deps/swissarmyhammer_cli-85aa4922ca760927)

running 48 tests
test cli::tests::test_cli_invalid_subcommand ... ok
test cli::tests::test_cli_doctor_subcommand ... ok
test cli::tests::test_cli_no_subcommand ... ok
test cli::tests::test_cli_quiet_flag ... ok
test cli::tests::test_cli_prompt_list_subcommand ... ok
test cli::tests::test_cli_help_works ... ok
test cli::tests::test_cli_flow_test_subcommand ... ok
test cli::tests::test_cli_flow_test_subcommand_with_options ... ok
test cli::tests::test_cli_serve_subcommand ... ok
test cli::tests::test_cli_search_subcommand_basic ... ok
test cli::tests::test_cli_search_subcommand_with_fields ... ok
test cli::tests::test_cli_serve_with_verbose ... ok
test cli::tests::test_cli_test_subcommand_with_file ... ok
test cli::tests::test_cli_test_subcommand_with_arguments ... ok
test cli::tests::test_cli_test_subcommand_with_all_flags ... ok
test cli::tests::test_cli_search_subcommand_with_flags ... ok
test cli::tests::test_cli_test_subcommand_with_prompt_name ... ok
test cli::tests::test_cli_verbose_flag ... ok
test cli::tests::test_cli_validate_command ... ok
test cli::tests::test_cli_validate_command_with_options ... ok
test cli::tests::test_cli_version_works ... ok
test cli::tests::test_cli_test_subcommand_with_set_variables ... ok
test validate::tests::test_validate_workflow_circular_dependency ... ok
test validate::tests::test_validate_command_includes_workflows ... ok
test validate::tests::test_validate_workflow_circular_dependency_single_warning ... ok
test validate::tests::test_validate_workflow_empty_name ... ok
test validate::tests::test_validate_workflow_invalid_name ... ok
test validate::tests::test_validate_workflow_complex_edge_cases ... ok
test validate::tests::test_validate_workflow_empty_file ... ok
test validate::tests::test_validate_workflow_missing_terminal_state ... ok
test validate::tests::test_validate_workflow_path_traversal_attempts ... ok
test validate::tests::test_validate_workflow_nested_conditions ... ok
test validate::tests::test_validate_workflow_self_loop ... ok
test validate::tests::test_validate_workflow_malformed_mermaid ... ok
test validate::tests::test_validate_workflow_syntax_invalid ... ok
test validate::tests::test_validate_workflow_syntax_valid ... ok
test validate::tests::test_validate_workflow_undefined_variables ... ok
test validate::tests::test_validation_result_add_error ... ok
test validate::tests::test_validation_result_add_warning ... ok
test validate::tests::test_validation_result_creation ... ok
test validate::tests::test_validator_creation ... ok
test validate::tests::test_validate_all_workflows_uses_standard_locations ... ok
test validate::tests::test_validate_workflow_unreachable_states ... ok
test validate::tests::test_validate_workflow_with_actions ... ok
test validate::tests::test_validate_all_workflows_integration ... ok
test validate::tests::test_validate_command_loads_same_workflows_as_flow_list ... ok
test validate::tests::test_validate_only_loads_from_standard_locations ... ok
test validate::tests::test_validate_all_handles_partial_templates ... ok

test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s

     Running unittests src/main.rs (target/debug/deps/sah-61a159672064b9c3)

running 98 tests
test cli::tests::test_cli_invalid_subcommand ... ok
test cli::tests::test_cli_quiet_flag ... ok
test cli::tests::test_cli_doctor_subcommand ... ok
test cli::tests::test_cli_no_subcommand ... ok
test cli::tests::test_cli_prompt_list_subcommand ... ok
test cli::tests::test_cli_help_works ... ok
test cli::tests::test_cli_flow_test_subcommand ... ok
test cli::tests::test_cli_flow_test_subcommand_with_options ... ok
test cli::tests::test_cli_serve_subcommand ... ok
test cli::tests::test_cli_serve_with_verbose ... ok
test cli::tests::test_cli_test_subcommand_with_all_flags ... ok
test cli::tests::test_cli_search_subcommand_basic ... ok
test cli::tests::test_cli_test_subcommand_with_arguments ... ok
test cli::tests::test_cli_test_subcommand_with_file ... ok
test cli::tests::test_cli_search_subcommand_with_flags ... ok
test cli::tests::test_cli_search_subcommand_with_fields ... ok
test cli::tests::test_cli_test_subcommand_with_prompt_name ... ok
test cli::tests::test_cli_test_subcommand_with_set_variables ... ok
test cli::tests::test_cli_validate_command ... ok
test cli::tests::test_cli_verbose_flag ... ok
test cli::tests::test_cli_version_works ... ok
test cli::tests::test_cli_validate_command_with_options ... ok
test completions::tests::test_completion_includes_flags ... ok
test completions::tests::test_print_completion_bash ... ok
test completions::tests::test_completion_includes_subcommands ... ok
test doctor::checks::tests::test_claude_not_in_path ... ok
test doctor::tests::test_check_status_exit_codes ... ok
test doctor::tests::test_doctor_creation ... ok
test doctor::tests::test_exit_code_conversion ... ok
test completions::tests::test_print_completion_fish ... ok
test completions::tests::test_print_completion_zsh ... ok
test doctor::checks::tests::test_claude_path_detection ... ok
test flow::tests::test_execute_workflow_test_mode_empty_workflow ... ok
test flow::tests::test_execute_workflow_test_mode_no_transitions ... ok
test flow::tests::test_execute_workflow_test_mode_with_conditions ... ok
test flow::tests::test_execute_workflow_test_mode_simple_workflow ... ok
test flow::tests::test_execute_workflow_test_mode_with_variables ... ok
test flow::tests::test_parse_duration ... ok
test flow::tests::test_workflow_run_id_helpers ... ok
test flow::tests::test_workflow_run_id_parse_error ... ok
test flow::tests::test_parse_set_variables ... ok
test flow::tests::test_set_variables_in_context ... ok
test list::tests::test_color_coding_when_terminal ... ok
test doctor::tests::test_workflow_diagnostics_in_run_diagnostics ... ok
test doctor::tests::test_run_diagnostics ... ok
test completions::tests::test_generate_completions_to_directory ... ok
test list::tests::test_prompt_info_creation ... ok
test list::tests::test_title_extraction_logic ... ok
test list::tests::test_builtin_prompts_should_be_identified_correctly ... ok
code.md | Code.md
  Partial template for reuse in other prompts

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/code | Code Review
  Review code for quality, bugs, and improvements

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test list::tests::test_list_command_source_filter ... ok
are_issues_complete | are_issues_complete
  Check if the plan is complete.

are_reviews_done | are_reviews_done
  Check if all the code review items are complete.

are_tests_passing | are_tests_passing
  Check if all tests are passing.

code.md | Code.md
  Partial template for reuse in other prompts

coding_standards | Coding Standards
  Partial template for reuse in other prompts

commit | Commit
  Commit your work to git.

coverage | coverage
  Improve the test coverage.

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

do_code_review | Do Code Review
  Code up the code review

do_next_issue | do_next_issue
  Debug and correct issues

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

document | document
  Create documentation for the project

documentation | Documentation
  Partial template for reuse in other prompts

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

lint | lint
  Iterate to correct all lint reported errors and warnings in the code base.

merge | merge
  Merge your work into the main branch.

plan | plan
  Generate a step by step development plan from a specification.

principals | Principals
  Partial template for reuse in other prompts

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/branch | review code
  Improved the current code changes

review/code | Code Review
  Review code for quality, bugs, and improvements

review/comprehensive | review code comprehensively
  Improved the all the code

review/documentation | review documentation
  Improved the documentation for the project

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test | test
  Iterate to correct test failures in the codebase.

todo | Todo
  Partial template for reuse in other prompts

test list::tests::test_list_command_with_no_prompts ... ok
test search::tests::test_generate_excerpt ... ok
test search::tests::test_generate_excerpt_with_long_text ... ok
test search::tests::test_search_result_creation ... ok
example | Example Prompt
  An example prompt for testing

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test list::tests::test_list_command_with_search ... ok
test list::tests::test_list_command_json_format ... ok
are_issues_complete | are_issues_complete
  Check if the plan is complete.

are_reviews_done | are_reviews_done
  Check if all the code review items are complete.

are_tests_passing | are_tests_passing
  Check if all tests are passing.

code.md | Code.md
  Partial template for reuse in other prompts

coding_standards | Coding Standards
  Partial template for reuse in other prompts

commit | Commit
  Commit your work to git.

coverage | coverage
  Improve the test coverage.

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

do_code_review | Do Code Review
  Code up the code review

do_next_issue | do_next_issue
  Debug and correct issues

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

document | document
  Create documentation for the project

documentation | Documentation
  Partial template for reuse in other prompts

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

lint | lint
  Iterate to correct all lint reported errors and warnings in the code base.

merge | merge
  Merge your work into the main branch.

plan | plan
  Generate a step by step development plan from a specification.

principals | Principals
  Partial template for reuse in other prompts

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/branch | review code
  Improved the current code changes

review/code | Code Review
  Review code for quality, bugs, and improvements

review/comprehensive | review code comprehensively
  Improved the all the code

review/documentation | review documentation
  Improved the documentation for the project

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test | test
  Iterate to correct test failures in the codebase.

todo | Todo
  Partial template for reuse in other prompts

test prompt::tests::test_run_prompt_command_list ... ok
test signal_handler::tests::test_signal_handler_does_not_block ... ok
test signal_handler::tests::test_signal_handler_setup ... ok
test list::tests::test_list_command_yaml_format ... ok
test prompt::tests::test_run_prompt_command_test_with_invalid_prompt ... ok
test test::tests::test_parse_arguments ... ok
test test::tests::test_parse_arguments_invalid_format ... ok
test test::tests::test_parse_arguments_with_set_variables ... ok
test test::tests::test_runner_creation ... ok
test prompt::tests::test_run_prompt_command_search ... ok
test test::tests::test_get_prompt_validation ... ok
test validate::tests::test_validate_all_workflows_uses_standard_locations ... ok
test validate::tests::test_validate_all_workflows_integration ... ok
test validate::tests::test_validate_command_includes_workflows ... ok
test signal_handler::tests::test_ctrl_c_signal_setup ... ok
test signal_handler::tests::test_unix_terminate_signal_setup ... ok
test validate::tests::test_validate_workflow_circular_dependency_single_warning ... ok
test validate::tests::test_validate_workflow_circular_dependency ... ok
test validate::tests::test_validate_command_loads_same_workflows_as_flow_list ... ok
test validate::tests::test_validate_workflow_empty_name ... ok
test validate::tests::test_validate_workflow_invalid_name ... ok
test validate::tests::test_validate_workflow_complex_edge_cases ... ok
test validate::tests::test_validate_workflow_empty_file ... ok
test validate::tests::test_validate_only_loads_from_standard_locations ... ok
test validate::tests::test_validate_workflow_path_traversal_attempts ... ok
test validate::tests::test_validate_workflow_missing_terminal_state ... ok
test validate::tests::test_validate_workflow_self_loop ... ok
test validate::tests::test_validate_workflow_nested_conditions ... ok
test validate::tests::test_validate_workflow_syntax_invalid ... ok
test validate::tests::test_validate_workflow_malformed_mermaid ... ok
test validate::tests::test_validate_workflow_syntax_valid ... ok
test validate::tests::test_validate_workflow_undefined_variables ... ok
test validate::tests::test_validation_result_add_error ... ok
test validate::tests::test_validation_result_add_warning ... ok
test validate::tests::test_validation_result_creation ... ok
test validate::tests::test_validator_creation ... ok
test validate::tests::test_validate_workflow_unreachable_states ... ok
test validate::tests::test_validate_workflow_with_actions ... ok
test signal_handler::tests::test_multiple_signal_handler_setup ... ok
test flow::tests::test_execute_workflow_test_mode_timeout ... ok
test signal_handler::tests::test_signal_handler_behavior ... ok
test validate::tests::test_validate_all_handles_partial_templates ... ok

test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s

     Running unittests src/main.rs (target/debug/deps/swissarmyhammer-ab756d608b5f5de1)

running 98 tests
test cli::tests::test_cli_invalid_subcommand ... ok
test cli::tests::test_cli_no_subcommand ... ok
test cli::tests::test_cli_quiet_flag ... ok
test cli::tests::test_cli_doctor_subcommand ... ok
test cli::tests::test_cli_flow_test_subcommand ... ok
test cli::tests::test_cli_prompt_list_subcommand ... ok
test cli::tests::test_cli_help_works ... ok
test cli::tests::test_cli_flow_test_subcommand_with_options ... ok
test cli::tests::test_cli_search_subcommand_basic ... ok
test cli::tests::test_cli_serve_with_verbose ... ok
test cli::tests::test_cli_search_subcommand_with_flags ... ok
test cli::tests::test_cli_search_subcommand_with_fields ... ok
test cli::tests::test_cli_test_subcommand_with_arguments ... ok
test cli::tests::test_cli_serve_subcommand ... ok
test cli::tests::test_cli_test_subcommand_with_file ... ok
test cli::tests::test_cli_test_subcommand_with_prompt_name ... ok
test cli::tests::test_cli_validate_command ... ok
test cli::tests::test_cli_test_subcommand_with_set_variables ... ok
test cli::tests::test_cli_verbose_flag ... ok
test cli::tests::test_cli_version_works ... ok
test cli::tests::test_cli_validate_command_with_options ... ok
test cli::tests::test_cli_test_subcommand_with_all_flags ... ok
test completions::tests::test_completion_includes_subcommands ... ok
test completions::tests::test_print_completion_bash ... ok
test doctor::tests::test_check_status_exit_codes ... ok
test doctor::tests::test_doctor_creation ... ok
test doctor::tests::test_exit_code_conversion ... ok
test doctor::checks::tests::test_claude_not_in_path ... ok
test completions::tests::test_print_completion_fish ... ok
test completions::tests::test_completion_includes_flags ... ok
test completions::tests::test_print_completion_zsh ... ok
test flow::tests::test_execute_workflow_test_mode_empty_workflow ... ok
test flow::tests::test_execute_workflow_test_mode_no_transitions ... ok
test flow::tests::test_execute_workflow_test_mode_simple_workflow ... ok
test flow::tests::test_execute_workflow_test_mode_with_conditions ... ok
test flow::tests::test_execute_workflow_test_mode_with_variables ... ok
test flow::tests::test_parse_duration ... ok
test flow::tests::test_workflow_run_id_helpers ... ok
test flow::tests::test_set_variables_in_context ... ok
test flow::tests::test_parse_set_variables ... ok
test flow::tests::test_workflow_run_id_parse_error ... ok
test list::tests::test_color_coding_when_terminal ... ok
test completions::tests::test_generate_completions_to_directory ... ok
test list::tests::test_builtin_prompts_should_be_identified_correctly ... ok
test doctor::tests::test_workflow_diagnostics_in_run_diagnostics ... ok
test list::tests::test_list_command_json_format ... ok
test list::tests::test_prompt_info_creation ... ok
test list::tests::test_title_extraction_logic ... ok
test doctor::tests::test_run_diagnostics ... ok
code.md | Code.md
  Partial template for reuse in other prompts

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/code | Code Review
  Review code for quality, bugs, and improvements

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test list::tests::test_list_command_source_filter ... ok
are_issues_complete | are_issues_complete
  Check if the plan is complete.

are_reviews_done | are_reviews_done
  Check if all the code review items are complete.

are_tests_passing | are_tests_passing
  Check if all tests are passing.

code.md | Code.md
  Partial template for reuse in other prompts

coding_standards | Coding Standards
  Partial template for reuse in other prompts

commit | Commit
  Commit your work to git.

coverage | coverage
  Improve the test coverage.

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

do_code_review | Do Code Review
  Code up the code review

do_next_issue | do_next_issue
  Debug and correct issues

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

document | document
  Create documentation for the project

documentation | Documentation
  Partial template for reuse in other prompts

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

lint | lint
  Iterate to correct all lint reported errors and warnings in the code base.

merge | merge
  Merge your work into the main branch.

plan | plan
  Generate a step by step development plan from a specification.

principals | Principals
  Partial template for reuse in other prompts

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/branch | review code
  Improved the current code changes

review/code | Code Review
  Review code for quality, bugs, and improvements

review/comprehensive | review code comprehensively
  Improved the all the code

review/documentation | review documentation
  Improved the documentation for the project

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test | test
  Iterate to correct test failures in the codebase.

todo | Todo
  Partial template for reuse in other prompts

test list::tests::test_list_command_with_no_prompts ... ok
test search::tests::test_generate_excerpt ... ok
test search::tests::test_generate_excerpt_with_long_text ... ok
test search::tests::test_search_result_creation ... ok
are_issues_complete | are_issues_complete
  Check if the plan is complete.

are_reviews_done | are_reviews_done
  Check if all the code review items are complete.

are_tests_passing | are_tests_passing
  Check if all tests are passing.

code.md | Code.md
  Partial template for reuse in other prompts

coding_standards | Coding Standards
  Partial template for reuse in other prompts

commit | Commit
  Commit your work to git.

coverage | coverage
  Improve the test coverage.

debug/error | Debug Error Messages
  Analyze error messages and provide debugging guidance with potential solutions

debug/logs | Analyze Log Files
  Analyze log files to identify issues and patterns

do_code_review | Do Code Review
  Code up the code review

do_next_issue | do_next_issue
  Debug and correct issues

docs/comments | Generate Code Comments
  Add comprehensive comments and documentation to code

docs/readme | Generate README Documentation
  Create comprehensive README documentation for a project

document | document
  Create documentation for the project

documentation | Documentation
  Partial template for reuse in other prompts

empty.md | Empty.md
  Partial template for reuse in other prompts

example | Example Prompt
  An example prompt for testing

generate/property | Generate Property-Based Tests
  Create property-based tests to find edge cases automatically

generate/unit | Generate Unit Tests
  Create comprehensive unit tests for code with good coverage

help | Help Assistant
  A prompt for providing helpful assistance and guidance to users

lint | lint
  Iterate to correct all lint reported errors and warnings in the code base.

merge | merge
  Merge your work into the main branch.

plan | plan
  Generate a step by step development plan from a specification.

principals | Principals
  Partial template for reuse in other prompts

prompts/create | Create New Prompt
  Help create effective prompts for swissarmyhammer

prompts/improve | Improve Existing Prompt
  Analyze and enhance existing prompts for better effectiveness

review/_review_format | Review/ Review Format
  Partial template for reuse in other prompts

review/accessibility | Accessibility Review
  Review code for accessibility compliance and best practices

review/branch | review code
  Improved the current code changes

review/code | Code Review
  Review code for quality, bugs, and improvements

review/comprehensive | review code comprehensively
  Improved the all the code

review/documentation | review documentation
  Improved the documentation for the project

review/patterns | Pattern Code Review
  Perform a comprehensive review of the code to improve pattern use.

review/security | Security Code Review
  Perform a comprehensive security review of code to identify vulnerabilities

review_format.md | Review Format.md
  Partial template for reuse in other prompts

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test | test
  Iterate to correct test failures in the codebase.

todo | Todo
  Partial template for reuse in other prompts

test prompt::tests::test_run_prompt_command_list ... ok
test list::tests::test_list_command_yaml_format ... ok
test prompt::tests::test_run_prompt_command_test_with_invalid_prompt ... ok
test signal_handler::tests::test_signal_handler_does_not_block ... ok
test signal_handler::tests::test_signal_handler_setup ... ok
example | Example Prompt
  An example prompt for testing

say-hello | Say Hello
  A simple greeting prompt that can be customized with name and language

test list::tests::test_list_command_with_search ... ok
test test::tests::test_get_prompt_validation ... ok
test test::tests::test_parse_arguments ... ok
test test::tests::test_parse_arguments_invalid_format ... ok
test test::tests::test_parse_arguments_with_set_variables ... ok
test test::tests::test_runner_creation ... ok
test prompt::tests::test_run_prompt_command_search ... ok
test validate::tests::test_validate_all_workflows_integration ... ok
test validate::tests::test_validate_all_workflows_uses_standard_locations ... ok
test signal_handler::tests::test_ctrl_c_signal_setup ... ok
test validate::tests::test_validate_command_includes_workflows ... ok
test signal_handler::tests::test_unix_terminate_signal_setup ... ok
test validate::tests::test_validate_workflow_circular_dependency ... ok
test validate::tests::test_validate_workflow_circular_dependency_single_warning ... ok
test validate::tests::test_validate_workflow_complex_edge_cases ... ok
test validate::tests::test_validate_only_loads_from_standard_locations ... ok
test validate::tests::test_validate_workflow_empty_name ... ok
test validate::tests::test_validate_workflow_invalid_name ... ok
test validate::tests::test_validate_workflow_empty_file ... ok
test validate::tests::test_validate_workflow_missing_terminal_state ... ok
test validate::tests::test_validate_command_loads_same_workflows_as_flow_list ... ok
test validate::tests::test_validate_workflow_path_traversal_attempts ... ok
test validate::tests::test_validate_workflow_nested_conditions ... ok
test validate::tests::test_validate_workflow_malformed_mermaid ... ok
test validate::tests::test_validate_workflow_self_loop ... ok
test validate::tests::test_validate_workflow_syntax_invalid ... ok
test validate::tests::test_validate_workflow_syntax_valid ... ok
test validate::tests::test_validate_workflow_undefined_variables ... ok
test validate::tests::test_validation_result_add_error ... ok
test validate::tests::test_validation_result_add_warning ... ok
test validate::tests::test_validate_workflow_unreachable_states ... ok
test validate::tests::test_validation_result_creation ... ok
test validate::tests::test_validate_workflow_with_actions ... ok
test validate::tests::test_validator_creation ... ok
test signal_handler::tests::test_multiple_signal_handler_setup ... ok
test doctor::checks::tests::test_claude_path_detection ... ok
test flow::tests::test_execute_workflow_test_mode_timeout ... ok
test signal_handler::tests::test_signal_handler_behavior ... ok
test validate::tests::test_validate_all_handles_partial_templates ... ok

test result: ok. 98 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.26s

     Running tests/binary_aliases_test.rs (target/debug/deps/binary_aliases_test-a77f81fca7e782e4)

running 4 tests
test test_swissarmyhammer_binary_exists ... ok
test test_both_binaries_same_version ... ok
test test_sah_binary_exists ... ok
test test_both_binaries_have_same_commands ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

     Running tests/cli_integration_test.rs (target/debug/deps/cli_integration_test-687b76f01a413156)

running 45 tests
test test_doctor_command ... ignored, doctor command may fail in CI due to environment differences
test test_error_exit_codes ... ok
test test_flow_test_interactive_mode ... ignored, interactive mode requires user input
test test_flow_test_custom_workflow_dir ... ok
test test_flow_test_help ... ok
test test_completion_command ... ok
test test_concurrent_commands ... ok
test test_flow_test_empty_set_value ... ok
test test_flow_test_coverage_complete ... ok
test test_flow_test_nonexistent_workflow ... ok
test test_flow_test_invalid_set_format ... ok
test test_flow_test_quiet_mode ... ok
test test_prompt_help ... ok
test test_prompt_subcommand_list ... ok
test test_flow_test_simple_workflow ... ok
test test_prompt_subcommand_search ... ok
test test_prompt_subcommand_validate ... ok
test test_prompt_subcommand_test ... ok
test test_root_help_includes_validate ... ok
test test_flow_test_special_chars_in_set ... ok
test test_quiet_flag ... ok
test test_flow_test_with_set_variables ... ok
test test_flow_test_with_timeout ... ok
test test_prompt_list_formats ... ok
test test_root_validate_help ... ok
test test_root_validate_invalid_format ... ok
test test_root_validate_invalid_yaml ... ok
test test_concurrent_flow_test ... ok
test test_root_validate_empty_workflow_dirs ... ok
test test_root_validate_command ... ok
test test_root_validate_error_exit_codes ... ok
test test_root_validate_json_format ... ok
test test_root_validate_malformed_workflow ... ok
test test_root_validate_stress_many_files ... ignored, stress test - only run manually
test test_root_validate_missing_fields ... ok
test test_root_validate_mixed_valid_invalid_prompts ... ok
test test_root_validate_quiet ... ok
test test_root_validate_mixed_valid_invalid_workflows ... ok
test test_root_validate_absolute_relative_paths ... ok
test test_root_validate_special_chars_in_paths ... ok
test test_root_validate_nonexistent_workflow_dir ... ok
test test_verbose_flag ... ok
test test_root_validate_with_multiple_workflow_dirs ... ok
test test_root_validate_undefined_variables ... ok
test test_root_validate_with_workflow_dirs ... ok

test result: ok. 42 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 2.86s

     Running tests/mcp_e2e_tests.rs (target/debug/deps/mcp_e2e_tests-a8a60125807e919d)

running 9 tests
test tests::test_e2e_get_prompt_with_args ... ok
test tests::test_e2e_get_prompt ... ok
test tests::test_e2e_prompt_not_found ... ok
test tests::test_e2e_server_startup ... ok
test tests::test_e2e_error_recovery ... ok
test tests::test_e2e_missing_required_args ... ok
test tests::test_e2e_concurrent_requests ... ok
test tests::test_e2e_template_edge_cases ... ok
test tests::test_e2e_file_watching ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s

     Running tests/mcp_integration_test.rs (target/debug/deps/mcp_integration_test-f5c0ab3f1d188957)

running 3 tests
test test_mcp_server_basic_functionality ... ok
test test_mcp_server_builtin_prompts ... ok
test test_mcp_server_prompt_loading ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.52s

     Running tests/mcp_mock_integration_tests.rs (target/debug/deps/mcp_mock_integration_tests-d63fdec28cf40c4f)

running 12 tests
test tests::test_performance_with_many_prompts ... ok
test tests::test_get_prompt_simple ... ok
test tests::test_get_prompt_with_args ... ok
test tests::test_real_time_updates ... ok
test tests::test_list_prompts ... ok
test tests::test_get_prompt_with_optional_args ... ok
test tests::test_argument_validation ... ok
test tests::test_concurrent_access ... ok
test tests::test_prompt_metadata ... ok
test tests::test_get_prompt_missing_required_args ... ok
test tests::test_prompt_not_found ... ok
test tests::test_template_validation ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/mcp_notification_simple_test.rs (target/debug/deps/mcp_notification_simple_test-9d0e57998c35e323)

running 1 test
test test_mcp_notification_simple ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.03s

     Running tests/mcp_partial_e2e_test.rs (target/debug/deps/mcp_partial_e2e_test-defaefc3321736bc)

running 1 test
test test_mcp_server_partial_rendering ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s

     Running tests/mcp_performance_tests.rs (target/debug/deps/mcp_performance_tests-38467d4a5b78dd8f)

running 12 tests
test tests::test_cli_startup_time_under_50ms ... ignored
test tests::test_doctor_command_startup_time_under_50ms ... ignored
test tests::test_list_command_startup_time_under_50ms ... ignored
test tests::test_load_performance_large ... ignored
test tests::test_load_performance_small ... ok
test tests::test_load_performance_medium ... ok
test tests::test_search_performance ... ok
test tests::test_get_prompt_performance ... ok
test tests::test_render_prompt_performance ... ok
test tests::test_memory_usage_with_large_library ... ok
test tests::test_list_prompts_performance ... ok
test tests::test_concurrent_access_performance ... ok

test result: ok. 8 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 0.96s

     Running tests/test_builtin_validation.rs (target/debug/deps/test_builtin_validation-ad3c65ca36a482c3)

running 1 test
test test_builtin_prompts_validate_directly ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/test_doc_examples.rs (target/debug/deps/test_doc_examples-3180b42e054d1bdd)

running 4 tests
test test_doc_examples_directory_structure ... ok
test test_example_prompts_have_required_fields ... ok
test test_all_doc_example_prompts_are_valid ... ok
test test_doc_markdown_includes_valid_paths ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/test_example_actions_workflow.rs (target/debug/deps/test_example_actions_workflow-df46bac1cd86b9d9)

running 9 tests
test test_example_actions_workflow_loads ... ok
test test_branch_decision_condition1 ... ok
test test_branch_decision_default ... ok
test test_failure_branch_execution ... ok
test test_success_branch_execution ... ok
test test_full_workflow_with_branching ... ok
test test_branch_decision_condition2 ... ok
test test_debug_cel_expressions ... ok
test test_all_branches_are_reachable ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.39s

     Running tests/test_set_variables.rs (target/debug/deps/test_set_variables-46e9a56c3070e2f0)

running 14 tests
test test_invalid_set_variable_format ... ok
test test_set_and_var_together ... ok
test test_workflow_with_complex_liquid_templates ... ok
test test_workflow_with_empty_set_value ... ok
test test_workflow_with_conflicting_set_and_var_names ... ok
test test_full_workflow_execution_with_liquid_templates ... ok
test test_prompt_test_with_empty_set_value ... ok
test test_prompt_test_with_set_overriding_arg ... ok
test test_workflow_with_set_variables ... ok
test test_workflow_with_missing_template_variables ... ok
test test_workflow_with_liquid_injection_attempts ... ok
test test_workflow_with_equals_sign_in_set_value ... ok
test test_workflow_with_malformed_liquid_templates ... ok
test test_workflow_with_special_chars_in_set_values ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/test_sub_workflow_integration.rs (target/debug/deps/test_sub_workflow_integration-e47c1afac368d2a3)

running 10 tests
test test_sub_workflow_parallel_execution ... ok
test test_deeply_nested_sub_workflows ... ok
test test_sub_workflow_circular_dependency_detection_integration ... ok
test test_sub_workflow_timeout_propagation ... ok
test test_sub_workflow_with_memory_storage ... ok
test test_sub_workflow_timeout_behavior ... ok
test test_sub_workflow_timeout_cancellation ... ok
test test_sub_workflow_deep_nesting_limit ... ok
test test_sub_workflow_context_isolation ... ok
test test_sub_workflow_in_process_execution ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.97s

     Running tests/test_utils.rs (target/debug/deps/test_utils-99056d821d6a9bef)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests swissarmyhammer

running 35 tests
test swissarmyhammer/src/file_loader.rs - file_loader::VirtualFileSystem (line 181) - compile ... ok
test swissarmyhammer/src/lib.rs - (line 15) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::render_with_partials (line 390) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::render_with_partials_and_env (line 450) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::ArgumentSpec (line 171) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::new (line 240) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::add_argument (line 508) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary (line 616) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt (line 71) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::add_directory (line 695) - compile ... ok
test swissarmyhammer/src/lib.rs - prompts (line 50) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::render (line 279) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::render_with_env (line 339) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::with_category (line 563) ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::with_description (line 539) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::render_prompt (line 781) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::Prompt::with_tags (line 588) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::render_prompt_with_env (line 819) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::add (line 928) ... ok
test swissarmyhammer/src/test_utils.rs - test_utils (line 24) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::get (line 726) ... ok
test swissarmyhammer/src/test_utils.rs - test_utils::Prompt (line 24) - compile ... ok
test swissarmyhammer/src/test_utils.rs - test_utils::ProcessGuard (line 57) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::list (line 751) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::list_filtered (line 893) ... ok
test swissarmyhammer/src/test_utils.rs - test_utils::PromptLibrary (line 24) - compile ... ok
test swissarmyhammer/src/test_utils.rs - test_utils::create_test_home_guard (line 234) - compile ... ok
test swissarmyhammer/src/test_utils.rs - test_utils::setup_test_home (line 144) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::new (line 645) ... ok
test swissarmyhammer/src/workflow/executor/validation.rs - workflow::executor::validation (line 55) - compile ... ok
test swissarmyhammer/src/workflow/executor/validation.rs - workflow::executor::validation::WorkflowExecutor::evaluate_cel_expression (line 461) - compile ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::remove (line 953) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::search (line 860) ... ok
test swissarmyhammer/src/prompts.rs - prompts::PromptLibrary::with_storage (line 668) ... ok
test swissarmyhammer/src/workflow/error_utils.rs - workflow::error_utils::handle_command_error (line 24) ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s

   Doc-tests swissarmyhammer_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

