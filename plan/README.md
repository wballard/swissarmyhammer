# SwissArmyHammer Implementation Plan

This directory contains the step-by-step implementation plan for swissarmyhammer, an MCP (Model Context Protocol) server that enables creating LLM prompts through markdown files with YAML front matter.

## Overview

SwissArmyHammer allows users to:
- Create prompts as simple markdown files
- Configure prompts with YAML front matter
- Stack prompts from multiple locations (built-in → user → local)
- Auto-reload prompts when files change
- Expose prompts via MCP to AI assistants like Claude

## Key Design Decisions

- **Performance First**: Following successful Rust CLIs like `uv` and `ripgrep`
- **User Experience**: Clear errors, helpful messages, beautiful output
- **MCP Focus**: Specifically targeting the underutilized prompts primitive
- **Flexibility**: Stacking system allows customization at multiple levels
- **Developer Friendly**: Auto-reload, doctor command, great docs

## Success Metrics

- Startup time < 50ms
- Zero configuration required to start
- Works seamlessly with Claude Desktop
- Comprehensive built-in prompt library
- Active community contributions

## Next Steps

Each step file contains detailed requirements and implementation notes. Start with step_0001.md and proceed sequentially, as each step builds on the previous ones.