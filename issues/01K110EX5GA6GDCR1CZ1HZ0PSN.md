TreeSitter parser logs show successful parsing but 0 chunks extracted from visualization.rs and memo.rs files.

The parser reports success but extracts no semantic chunks for indexing, which suggests the chunk extraction logic has an issue.

We need tests to verify TreeSitter extracts actual chunks with meaningful text content.

## Proposed Solution

I will implement a Test Driven Development approach to fix the TreeSitter chunk extraction issue:

1. **Analyze Current Implementation**: Examine the TreeSitter parser and indexer code to understand how chunks are supposed to be extracted
2. **Examine Sample Files**: Look at the actual content of visualization.rs and memo.rs to understand what should be parsed
3. **Write Failing Test**: Create a test that verifies TreeSitter actually extracts meaningful chunks from Rust source code
4. **Identify Root Cause**: Debug why the parser reports success but extracts 0 chunks
5. **Fix Implementation**: Modify the TreeSitter parsing logic to properly extract chunks
6. **Verify Success**: Ensure the test passes and chunk extraction works correctly

The issue appears to be that while TreeSitter parsing succeeds, the chunk extraction logic is not working properly, resulting in no semantic chunks being created for indexing despite the files containing valid Rust code.