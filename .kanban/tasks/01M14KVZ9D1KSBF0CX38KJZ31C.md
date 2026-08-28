---
assignees:
- claude-code
position_column: todo
position_ordinal: fffa80
title: 'kanban CLI: a failed command prints nothing and exits 1'
---
The `kanban` CLI writes every error to the log file, not to stderr. A command
that fails prints NOTHING and exits 1. The caller sees an empty screen and no
reason.

## What happens

Measured with a binary built at `158012bcc`, on a fresh board:

```
kanban task update --id <id> --depends_on '["<unknown-id>"]'
```

The command exits 1. stdout is empty. stderr is empty. `--debug` changes
nothing.

The error text goes to `.kanban/mcp.<pid>.log`:

```
ERROR Error: task not found: ["01M14KF6NR0TPV1NRXT4BKKYY1"]
```

## Why it happens

`apps/kanban-cli/src/main.rs` reports every failure with the `error!` macro
(`handle_kanban_command`, `execute_kanban_operation`, `run_serve`).
`apps/kanban-cli/src/logging.rs` sends tracing to `.kanban/mcp.<pid>.log` when a
`.kanban/` directory is present. So the CLI path and the MCP `serve` path share
one sink, and the CLI half of it is invisible.

Found while working ^18kd3j9. A silent exit 1 made a wrong `--depends_on` value
look like a hang or a no-op, and the reason was only in the log.

## Work

- Print the error text of a failed CLI command to stderr, in every case.
- Keep the file log for `serve`, where stderr belongs to the MCP transport.
- Add a test that runs the binary with a bad ref and asserts the message is on
  stderr.

## Done when

A failed `kanban` command states its reason on stderr, and the test holds it
there. #bug #kanban #cli