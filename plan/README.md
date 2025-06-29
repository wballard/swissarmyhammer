# SwissArmyHammer Implementation Plan

This directory contains the step-by-step implementation plan for swissarmyhammer, an MCP (Model Context Protocol) server that enables creating LLM prompts through markdown files with YAML front matter.

## Overview

SwissArmyHammer allows users to:
- Create prompts as simple markdown files
- Configure prompts with YAML front matter
- Stack prompts from multiple locations (built-in → user → local)
- Auto-reload prompts when files change
- Expose prompts via MCP to AI assistants like Claude

## Implementation Steps

### Foundation (Steps 1-3)
1. **step_0001.md** - Initialize Rust project with basic structure
2. **step_0002.md** - Build CLI framework with excellent UX
3. **step_0003.md** - Implement MCP protocol basics

### Core Features (Steps 4-9)
4. **step_0004.md** - Implement prompt discovery from directories
5. **step_0005.md** - Parse YAML front matter and markdown
6. **step_0006.md** - Implement stacking/override mechanism
7. **step_0007.md** - Add file watching and auto-reload
8. **step_0008.md** - Expose prompts via MCP protocol
9. **step_0009.md** - Process prompt templates with arguments

### Polish & Distribution (Steps 10-14)
10. **step_0010.md** - Create doctor command for diagnostics
11. **step_0011.md** - Add comprehensive testing and polish
12. **step_0012.md** - Build library of built-in prompts
13. **step_0013.md** - Set up packaging and distribution
14. **step_0014.md** - Create documentation and community

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