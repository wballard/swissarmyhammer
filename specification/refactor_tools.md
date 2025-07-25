The tools processing is getting to be way too large of a source file.

Let's start with tool_handlers.rs -- seems like lots of duplication with mcp.rs and I cannot find where these tools are used.


## Organization

We need to separate into a module for each related group of tools.

- ./mcp/tools/memoranda
- ./mcp/tools/issues
- ./mcp/tools/semantic

and get the code that implements each of these tools into those folders to improve the organization

## Individual tool modules

Each tools should be an individual folder like

./tools/issues/issue_create

Each tool Description needs to be a markdown files brought in by a build macro much like built in prompts.

./tools/issues/issue_create/description.md

## Missing tools

These tools appear to be missing

semantic/semantic_index
semantic/semantic_query

## Tool Registry

A big match statement on strings is hack. Use a tool service registry -- a toolbox.

`list_prompts` and `get_prompt` have the prompt library -- this is a good pattern, so I know you know how to do a good job.

## MCP Root

There is a ./mcp module folder and a mcp.rs -- super confusing -- get all the mcp in ./mcp

This really needs to be in ./mcp/mod.rs, subdivided into modules rather than all crammed into ./mcp.rs

## CLI

Cli commands need to invoke the exact same tools as the mcp server, turning command line arguments into parameters as needed, formatting the results for CLI output.

## TDD

Make sure to TDD -- ensuring tests that invoke each existing MCP before moving anything. Double check if a test exists before.

## Duplicate Code

Once each of the tools is isolated in a module, use it to then scan the entire rest of the source base to eliminate duplication.

Refactor toward the tools