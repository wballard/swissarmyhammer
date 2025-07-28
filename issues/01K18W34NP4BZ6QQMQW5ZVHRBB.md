tool_handlers.rs is a terrible idea, you have divided up the tools neatly, and then they call into one grab bag module

the actual implementation of the tools needs to be in the ./swissarmyhammer/src/mcp/tools modules, organized

think deeply about making smart organization and not having big grab bag modules
