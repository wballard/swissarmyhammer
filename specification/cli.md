The cli should just 'call the MCP tools' passing in CLI parameters to the MCP tools.

Instead the cli is calling into the implementations of the MCP tools, creating duplicate logic.

A good example is issue.rs merge_issue, which has terrible duplication of the MergeIssueTool

Create a plan to go through each cli command module, and each command case to eliminate this horrific blunder.