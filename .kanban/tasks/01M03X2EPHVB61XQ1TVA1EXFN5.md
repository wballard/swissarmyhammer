---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m133qbfjsde81df5whhcr1gg
  text: |-
    Research done. Cause found, and one card premise corrected.

    **Correction to the card**: the directory is NOT untracked. All 11 files are committed to git (`git ls-files` shows them). It entered the tree in `f634e733f` (2026-04-02, "test(coverage): add tests across 55 coverage-gap cards") and moved to its present path in `375d20b16` (2026-05-15, the crates/ refactor). A broad `git add` committed a test artifact. Because it is tracked, removal is recoverable from history.

    **Contents (no unique data)**: `boards/board.yaml` = "Untitled Board", the three default columns (To Do / Doing / Done), one actor `filtering-proxy`, and `undo_stack.yaml` whose five entries are all bootstrap — create board, create column todo/doing/done, create actor filtering-proxy. Zero tasks. Nothing worth keeping.

    **The writer**: `start_mcp_server(mode, library, working_dir)` documents `working_dir` as "uses current_dir if None". Cargo runs a test binary with CWD set to its own package root, so `None` resolves to `crates/swissarmyhammer-mcp-proxy`. Two call sites in this crate's integration tests pass `None`:
    - `tests/integration/handler_tests.rs` — the shared upstream in `run_upstream_forever`
    - `tests/integration/end_to_end.rs` — its own separate upstream

    The proxy then connects to that upstream as an MCP client named `filtering-proxy` (`src/proxy.rs`, `Implementation::new("filtering-proxy", ...)` inside `get_peer`). The upstream registers the connecting client as a kanban actor, which bootstraps the board. That is exactly the actor recorded on disk, so the artifact matches the mechanism.

    `src/server.rs` already passes `Some(temp.path().to_path_buf())` with a comment explaining why. The integration tests never followed.

    **Sweep for stray dot directories of the same shape**: only two `.kanban` directories exist in the tree — the repository-root board and this one. Classifying every dot directory under `crates/` and `apps/`: the `.sah`, `.code-context`, `.llama-sessions` directories are all gitignored (`**/.sah/` etc.) and untracked. Three `.shell` directories carry one tracked file each, which is the self-ignoring `.shell/.gitignore` — that belongs to ^2kqcswb and is out of scope here. `crates/acp-conformance/.swissarmyhammer` has 106 tracked files and is a real conformance fixture set, not a stray. So this board directory is the only stray of this shape.
  timestamp: 2026-08-28T02:38:31.410745+00:00
- actor: claude-code
  id: 01m134c24afz0m0jjr5gvy9854
  text: |-
    Implementation landed. The cause is fixed and proved, not guessed.

    **First hypothesis was wrong, and I checked instead of claiming it.** I first wrote the regression test around a proxy `list_tools` call, on the theory that the client handshake registered the actor and bootstrapped the board. That test PASSED with the bug still in place, and a full run of the crate's 39 tests with `working_dir: None` left no board behind. Merely starting the upstream and connecting to it does not write a board, so I did not have a real RED and did not treat the hypothesis as confirmed.

    **What actually writes it.** The board is created when the `kanban` tool is CALLED, against the upstream server's working directory. The original `handler_tests.rs` called `all.tools[0]` — at the time, the first tool was the kanban tool. Today `tools[0]` resolves to a different tool, so the defect survives only by luck of tool ordering while `working_dir: None` remains. Rewriting the test to call the `kanban` tool by name instead of by list position reproduced it exactly:

        the upstream MCP server wrote a kanban board into the crate directory at
        /Users/.../crates/swissarmyhammer-mcp-proxy/.kanban

    That is a verified RED. Fixing the working directory turned it GREEN. I ran both directions myself.

    **The fix.**
    - `tests/integration/upstream.rs` (new): the single shared upstream. It starts `start_mcp_server` with `Some(work_dir.path().to_path_buf())` from a `tempfile::TempDir`, held alive for the process. The two duplicated helper sets that lived in `handler_tests.rs` moved here, so there is now exactly one place that can start an upstream.
    - `tests/integration/handler_tests.rs`: 112 lines of duplicated helpers deleted, now uses the shared module.
    - `tests/integration/end_to_end.rs`: was starting a SECOND upstream with `None` in the same test binary. It now uses the shared isolated upstream.
    - `tests/integration/workspace_isolation.rs` (new): the regression test. It calls the kanban tool by name through the proxy and asserts no board appears in the crate directory, with a precondition assert so a pre-existing directory reports as a precondition failure rather than a false pass.

    **Removal.** All 11 tracked files removed with `git rm -r`. Contents were confirmed empty of unique data before deletion (recorded in the previous comment), and the files remain recoverable from history.

    **Defense in depth.** `.gitignore` gained `/crates/*/.kanban/` and `/apps/*/.kanban/`. These are anchored so the tracked root board is untouched — verified both ways: the root board is still not ignored and still has 7309 tracked files, while a probe board under the crate is ignored by line 97. This blocks the specific failure that happened here, a broad `git add` committing a test artifact. It does not mask the regression test, which asserts the directory does not EXIST rather than that it is untracked.

    **Sweep result.** After a full `cargo nextest run --workspace`, no `.kanban` directory exists anywhere under `crates/` or `apps/`.

    **Verification.**
    - `cargo nextest run --workspace`: 14255 tests run, 14255 passed, 0 skipped.
    - `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
    - `cargo fmt --all`: applied.
    - Checked no orphaned nextest process was running before the workspace run, per the known collision hazard.

    **Out of scope, left alone.** `.sah` and `.shell` still sit in this crate directory. Both are covered by existing `**/.sah/` and `**/.shell/` ignores, and `.shell` belongs to ^2kqcswb.
  timestamp: 2026-08-28T02:49:49.962113+00:00
- actor: claude-code
  id: 01m134cahk2gqff9ja4merd83h
  text: |-
    ### implement — changed
    - evidence: 6 files. New: crates/swissarmyhammer-mcp-proxy/tests/integration/upstream.rs, crates/swissarmyhammer-mcp-proxy/tests/integration/workspace_isolation.rs. Modified: crates/swissarmyhammer-mcp-proxy/tests/integration/handler_tests.rs, crates/swissarmyhammer-mcp-proxy/tests/integration/end_to_end.rs, crates/swissarmyhammer-mcp-proxy/tests/integration/mod.rs, .gitignore. Deleted: all 11 tracked files of crates/swissarmyhammer-mcp-proxy/.kanban. Tests: cargo nextest run --workspace = 14255 run, 14255 passed, 0 skipped. cargo clippy --workspace --all-targets -- -D warnings = exit 0. RED verified before the fix, GREEN after.
    - next: /review
  timestamp: 2026-08-28T02:49:58.579355+00:00
position_column: doing
position_ordinal: '8380'
title: Stray .kanban board directory sits under crates/swissarmyhammer-mcp-proxy
---
`crates/swissarmyhammer-mcp-proxy/.kanban` is an untracked board directory dated 5 July. It predates the current work by weeks, and no `.gitignore` entry covers it.

A stray board directory is a hazard: the live application opens a board directory it finds, and it then writes to it. A board opened by mistake reads as empty.

## What to do

- Find what wrote it. An agent or a test that ran with the wrong current directory is the usual cause.
- Remove the directory, or make the cause write to the repository root board.
- Sweep the tree for other stray dot directories of the same shape.

## Found by

The implementer of ^s1qh4tv, twice, while it checked its own working tree for probe files. It is out of scope for that card.

#bug #kanban