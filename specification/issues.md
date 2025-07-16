# Overview

Track issues directly in the repository, storing them in as files in git, with an MCP server.

## Rules

Issues are individual markdown files stored in the root of a repository in `./issues`.

Each Issue describes work to be done in markdown.

When issues are completed, they are moved to `./issues/complete`.

Each issue file name starts with a 6 digit increasing integer, like <nnnnnn>_<more naming>.md.

When a new issue is created, it need to start with the next highest number so that issues are sequential.

## Use Cases

This will be an MCP Tool capability to swissarmyhammer.
Think deeply about creating great descriptions of the tool capability to allow Claude to call it reliably.
Research best practices for making MCP tools.

### Create

As a User, I want to be able to create an issue from the swissarmyhammer command line.

As an LLM, I want to be able to create an issue using an MCP tool.

As an LLM, when a user reports and issue, I want to record it as an issue for later.

### Complete

As a User, I want to be able to mark an issue complete from the swissarmyhammer command line.

As an LLM, I want to be able to mark an issue complete using an MCP tool.

### Update

As an LLM, I want to be able to record additional context in an issue using an MCP tool.

### Current

As an LLM, I want to be able to ask what is the current issue at the head of the line to work using an MCP tool.
