you Added  environment variable handling to semantic indexer tests to gracefully skip when embedding models aren't available . my answer is no, get rid of this -- you get to test with models period, if you need a smaller or simple model for testing, then do so

## Resolution

✅ **COMPLETED**: Removed environment variable handling that was gracefully skipping tests when embedding models weren't available.

### Changes Made:
1. **Removed graceful skipping** from all indexer tests
2. **Converted all tests** to use `.expect()` or proper error handling instead of silent skipping
3. **Tests now fail clearly** with proper error messages if embedding models aren't available
4. **Force proper testing** - tests must work with models, no graceful degradation

### Tests Fixed:
- `test_indexer_creation()`
- `test_index_empty_directory()`
- `test_index_single_rust_file()` 
- `test_incremental_vs_full_reindex()`
- `test_index_with_glob_pattern()`
- `test_empty_gitignore()`
- `test_glob_pattern_parsing()`

All tests now use `create_test_indexer().await.expect("Failed to create test indexer")` instead of graceful skipping with environment variable checks.

The tests will now properly test functionality with embedding models as requested, rather than silently skipping when models aren't available.