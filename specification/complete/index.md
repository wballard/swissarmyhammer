# swissarmyhammer

`swissarmyhammer` is an MCP console server implemented in Rust with the [MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)

## Technology

- [MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code/sdk), invoked by CLI

## Concepts

It provides the ability to create prompts and tools with just markdown + yaml front matter.

These tools are special in that they invoke an LLM model to execute, instead of being just code.

In effect you are making tools for the LLM with LLM.

## Guidelines

Search the web for other MCP servers in Rust, in particular focusin on ones with many GitHub stars.
Think deeply about what makes them popular and take that into account we building the plan.

Create a great rust CLI, seeking inspirations from other popular rust based CLI programs such as `uv`.
Go above and beyond and exceed user expectations.

## Requirements

### CLI

As a user, I will use swissarmyhammer as a command line executable.

### MCP

As a user, I will add swissarmyhammer as a local stdio MCP server as documented at https://docs.anthropic.com/en/docs/claude-code/mcp.

As a user, if I run swissarmyhammer stand alone in the shell, it will give me help information on how to add it to Claude Code.

### Prompts in .swissarmyhammer

As a user, I will create prompts as simple markdown `.md` files in directories using my own editor.

These will go in directories named `.swissarmyhammer`.

These will have a stacking natures where:

- swissarmyhammer has internal prompts in is source tree `var/prompts/`, these get compiled in as resources
- ~/.swissarmyhammer provides user specific prompts
- $PWD/.swissarmyhammer in the CWD where `claude` is invoked stacks on more prompts

The prompt file name stem becomes the prompt name.

Prompts will over-write if they have the same relative path, allowing user override.

### Autoloading Prompts

As a user, when I edit a prompt, I want it to show up immediately as an available prompt without restarting.

### Expose Prompts to MCP

As a user, I want all my available prompts to show up via MCP to a consuming MCP client.

- Use the MCP listChanged capability

### Prompt Front Matter

As a user, I want to configure my prompts with YAML front matter.
This includes name, title, description, and arguments.

As a user, I want my arguments to NOT be required by default.

### Doctor

As a user, I want to be able to say `swissarmyhammer doctor` and get a diagnosis of any problem or error I need to correct.
