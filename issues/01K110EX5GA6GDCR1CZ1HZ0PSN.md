```
2025-07-25T14:26:06.784341Z  INFO swissarmyhammer::semantic::parser: TreeSitter parse success: ./swissarmyhammer/src/workflow/visualization.rs | 0 chunks | 24.92ms | 0 chunks/sec | 700352 bytes/sec
2025-07-25T14:26:06.784346Z  WARN swissarmyhammer::semantic::indexer: No chunks extracted from file: ./swissarmyhammer/src/workflow/visualization.rs
2025-07-25T14:26:06.810589Z  INFO swissarmyhammer::semantic::parser: TreeSitter parse success: ./swissarmyhammer-cli/src/memo.rs | 0 chunks | 26.22ms | 0 chunks/sec | 599474 bytes/sec
2025-07-25T14:26:06.810593Z  WARN swissarmyhammer::semantic::indexer: No chunks extracted from file: ./swissarmyhammer-cli/src/memo.rs
```

So I don't believe there should be no chunks of parsed code in visualization.rs.

We need tests to make sure when we are running tree sitter, we are getting actual chunks with actual text.

## Proposed Solution

I will implement a Test Driven Development approach to fix the TreeSitter chunk extraction issue:

1. **Analyze Current Implementation**: Examine the TreeSitter parser and indexer code to understand how chunks are supposed to be extracted
2. **Examine Sample Files**: Look at the actual content of visualization.rs and memo.rs to understand what should be parsed  
3. **Write Failing Test**: Create a test that verifies TreeSitter actually extracts meaningful chunks from Rust source code
4. **Identify Root Cause**: Debug why the parser reports success but extracts 0 chunks
5. **Fix Implementation**: Modify the TreeSitter parsing logic to properly extract chunks
6. **Verify Success**: Ensure the test passes and chunk extraction works correctly

The issue appears to be that while TreeSitter parsing succeeds, the chunk extraction logic is not working properly, resulting in no semantic chunks being created for indexing despite the files containing valid Rust code.