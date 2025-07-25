```
2025-07-25T14:26:06.784341Z  INFO swissarmyhammer::semantic::parser: TreeSitter parse success: ./swissarmyhammer/src/workflow/visualization.rs | 0 chunks | 24.92ms | 0 chunks/sec | 700352 bytes/sec
2025-07-25T14:26:06.784346Z  WARN swissarmyhammer::semantic::indexer: No chunks extracted from file: ./swissarmyhammer/src/workflow/visualization.rs
2025-07-25T14:26:06.810589Z  INFO swissarmyhammer::semantic::parser: TreeSitter parse success: ./swissarmyhammer-cli/src/memo.rs | 0 chunks | 26.22ms | 0 chunks/sec | 599474 bytes/sec
2025-07-25T14:26:06.810593Z  WARN swissarmyhammer::semantic::indexer: No chunks extracted from file: ./swissarmyhammer-cli/src/memo.rs
```

So I don't believe there should be no chunks of parsed code in visualization.rs.

We need tests to make sure when we are running tree sitter, we are getting actual chunks with actual text.