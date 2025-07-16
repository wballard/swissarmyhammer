
# Goal

Use State Diagrams to define workflow processes in markdown.

## Technical

- We're going to be using Mermaid State diagrams https://mermaid.js.org/syntax/stateDiagram.html
- We're going to use the rust mermaid-parser from <git@github.com>:wballard/mermaid_parser.git
- We're going to use Claude Code command line SDK https://docs.anthropic.com/en/docs/claude-code/sdk

## Use Cases

### File Based

As a user, I want to be able to create a new workflow in any `.swissarmyhammer/workflows` directory.

Each workflow will be a `.mermaid` file containing a state diagram.

### Start

As a user, I expect swissarmyhammer to start workflows at the mermaid start.

### State Actions

As a user, I want to use the state description feature of mermaid to specify what to send to Claude when
we are at that step.

`claude --dangerously-skip-permissions --print --output-format stream-json "<description goes here">`

### State Transitions

As a user, I want to chain together multiple State Actions with mermaid transitions.

### State Choice

As a user, I want to use the mermaid choice feature to control which State Action to take next.

As a user, I expect swissarmyhammer to evaluate the output from Claude agaist a string constant or regex for the choice.

### Fork

As a user, I want to use mermaid state diagrams forks to run actions in parallel.

### Concurrency

As a user, I want to use mermaid state diagram concurrency to run multiple workflows in parallel.

### End

As a user, when we reach the end state, I expect swissarmyhammer to note that it is finished an exit.

### Output

As a user, I expect swissarmyhammer to create a unique run_id for each run.

As a user, I expect swissarmyhammer to take the streaming json output https://docs.anthropic.com/en/docs/claude-code/sdk#streaming-json-output from Claude and append it to a `$PWD/.swissarmyhammer/workflows/runs/<run_id>.jsonl runlog.

As a user, I expect the json messages will be written whole when there are parallel Claudes.

As a user, I expect swissarmyhammer to write a json message into the runlog for each:

- state enter
- state exit
- choice taken
- fork
- join

As a user, I expect swissarmyhammer to take the streaming json output from Claude and print a beautiful message to stdout for me.

### Delegation

As a user, if I use the name of a workflow for a state description, I want to invoke that workflow instead of Claude.

### 'list`

As a user, I expect `swissarmyhammer list` to show workflows as well as prompts:

- color coded as is with source
- adding additional emoji icons for 'prompt' and 'workflow'

### `validate`

As a user, I expect `swissarmyhammer validate` to check that my workflows are valid:

- validate the mermaid parsing

### `flow`

As a user, I expect swissarmyhammer to have a `flow` command that will invoke a named workflow.

The name to invoke will be the relative path without the .mermaid suffix.

This will start at the start, and exit at end, streaming all output from claude through to the console.

### `--resume`

As a user, I expect `swissarmyhammer flow` to have a --resume switch that I can use the <session_id> to recover from a crash.