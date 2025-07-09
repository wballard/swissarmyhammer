# Overview

Track issues directly in the repository, storying them in git, with an MCP server.

## Rules

Issues are individual markdown files stored in the root of a repository in `./issues`.

Each Issue Describes work to be done in markdown.

When issues are completed, they are moved to `./issues/complete`.

Each issue file name starts with a 6 digit increasing integer, like <nnnnnn>_<more naming>.md.

When a new issue is created, it need to start with the next highest number so that issues are sequential.

## Use Cases

### Create

As a User, I want to be able to create an issue from the swissarmyhammer command line.

As an LLM, I want to be able to create an issue using an MCP tool.

### Complete

As a User, I want to be able to mark an issue complete from the swissarmyhammer command line.

As an LLM, I want to be able to mark an issue complete using an MCP tool.

### Update

As an LLM, I want to be able to record additional context in an issue using an MCP tool.

### Next Up

As an LLM, I want to be able to ask what is the next issue to work.
