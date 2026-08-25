---
comments:
- actor: claude-code
  id: 01m0x45sdwsg2zry6jd5m4cnze
  text: |-
    Research results.

    Code locations:
    - crates/swissarmyhammer-tools/src/mcp/tools/ralph/state.rs — file storage, `find_active_ralph` (the newest-file fallback), `clear_all_ralph`.
    - crates/swissarmyhammer-tools/src/mcp/tools/ralph/execute/mod.rs — the four operations. `check ralph` falls back to `find_active_ralph` on a session miss. `clear ralph` with no explicit id and no own file calls `clear_all_ralph`.
    - ARCHITECTURE.md (Ralph section) and ralph/description.md document the old fallback and must change with it.

    Why the ids never match: `set ralph` keys the file by `ToolContext.session_id`, a ULID that the MCP server process mints at start (tool_registry.rs). The Stop hook runs `sah tool ralph ralph check --` in a fresh CLI process and pipes the harness session id on stdin. The two processes are both children of the same harness process, so the ids can never be equal. The old fallback bridged this gap with "newest file wins", which is the reported fault.

    Selected design — ownership by process, not by newest file:
    - `set ralph` records `owner_pid` (the setter process id) in the frontmatter.
    - `check ralph` blocks on an exact session-id match, as before. On a miss it accepts only an instruction whose owner process (1) is alive, (2) is not the checking process itself, and (3) is a proper ancestor of the checking process, or has a parent that is a proper ancestor of the checking process (system pids 0 and 1 excluded). In the real deployment the setter (MCP server) and the checker (hook CLI) are sibling children of one harness process, so this matches only the owner session, with no race. A live peer session never matches. A dead owner never blocks.
    - `clear ralph` with an explicit id removes only that file. With no id it removes the caller's own file; when the caller has none, it removes only files whose owner process is no longer alive (was: remove everything).
    - `get ralph` keeps its read-only newest-file fallback for display.

    Dependency: `sysinfo` (already a workspace dependency) for pid liveness and parent lookup.

    Tests to change: the three old cross-session fallback tests and the clear-all tests pin the faulty behavior and will be rewritten to the new contract, plus the four tests the card requires.
  timestamp: 2026-08-25T18:50:57.852157+00:00
- actor: claude-code
  id: 01m0x57v8wanr0rt2jpa341wp2
  text: |-
    Implementation complete. Summary of the correction:

    - New module `crates/swissarmyhammer-tools/src/mcp/tools/ralph/ownership.rs`: decides whether an instruction belongs to the current session's process tree. The owner matches only when its process (1) is alive, (2) is not the checking process itself, and (3) is a proper ancestor of the checking process, or has a parent that is such an ancestor. System pids 0 and 1 never match, so an orphaned owner cannot capture every session. Uses `sysinfo` (already a workspace dependency).
    - `state.rs`: `RalphState` gets `owner_pid` (written and parsed in the frontmatter). `find_newest_ralph_matching` is the filtered scan; `find_active_ralph` delegates to it. `clear_all_ralph` is removed. New `clear_dead_ralph` removes only files whose owner process is not alive (dead pid, no pid, or unparseable file); it takes the liveness check as a function, so tests can inject it.
    - `execute/mod.rs`: `set ralph` records the writing process as `owner_pid`. `check ralph` blocks on an exact session match, and on a miss it falls back ONLY to an instruction owned by this session's process tree. A live peer's instruction and a dead session's instruction never block. `clear ralph` with no id removes the caller's own file; when there is none, it removes only the files of ended sessions.
    - Docs updated: ralph/description.md, ARCHITECTURE.md ralph section, CheckRalph/ClearRalph operation descriptions, a stale doc comment in apps/swissarmyhammer-cli/src/main.rs.

    TDD notes: ownership predicate and state functions were driven red-green (stubs first, watched 3 and 8 assertion failures, then implemented). The 4 tests from the card are in execute/mod.rs: test_check_ralph_does_not_block_for_another_sessions_instruction, test_check_ralph_after_own_clear_allows_despite_peer_instruction, test_check_ralph_ignores_instruction_of_ended_session (+ the ownerless variant), test_clear_ralph_explicit_keeps_other_sessions_instruction. The three old tests that pinned the faulty newest-file fallback were rewritten to the ownership contract, with the owner set to the parent pid of the test process to model the harness topology.

    Known limit, recorded on purpose: when a deployment spawns the MCP server through a persistent shell wrapper, the owner's parent is the wrapper, not the harness, and the fallback does not fire. The session then stops normally. This degrades to "no loop", never to "wrongly held open".

    Verification: cargo test -p swissarmyhammer-tools --lib → 1335 passed, 0 failed. cargo test -p swissarmyhammer-cli --test tool_output_format → 3 passed. cargo clippy (lib+tests) → no warnings. cargo fmt --check → clean.
  timestamp: 2026-08-25T19:09:33.852266+00:00
- actor: claude-code
  id: 01m0x582k8hz2hkekmkyyhb3d5
  text: |-
    ### implement — changed
    - evidence: 8 files — crates/swissarmyhammer-tools/src/mcp/tools/ralph/ownership.rs (new), crates/swissarmyhammer-tools/src/mcp/tools/ralph/state.rs, crates/swissarmyhammer-tools/src/mcp/tools/ralph/execute/mod.rs, crates/swissarmyhammer-tools/src/mcp/tools/ralph/mod.rs, crates/swissarmyhammer-tools/src/mcp/tools/ralph/description.md, crates/swissarmyhammer-tools/Cargo.toml, apps/swissarmyhammer-cli/src/main.rs, ARCHITECTURE.md. Tests: 1335 lib tests pass, 90 ralph tests, clippy and fmt clean.
    - next: /review
  timestamp: 2026-08-25T19:09:41.352241+00:00
position_column: doing
position_ordinal: '8280'
title: 'ralph: one session''s Stop hook instruction blocks every session in the repository'
---
## What

The `ralph` Stop hook stops the wrong session. One session's instruction
blocks EVERY session in the same repository from stopping.

The `ralph` tool documents the cause: *"the hook runs in a separate process
whose session id never matches the setter's, so a miss on the named session
falls back to the newest active instruction in `.ralph/`."* That fallback does
not look at who owns the instruction. Thus a session that set no instruction,
or that cleared its own, is held open by the instruction of a different
session.

## What was seen

Two sessions worked in the same repository,
`/Users/wballard/github/swissarmyhammer/FoundationModelsMultitool`, on
2026-08-25.

- Session A, id `01M0RD52ANHT8MRP9DRB3JN0H5`, called `clear ralph` two times.
  Each call answered `{"cleared": true}`.
- Session B, id `01M0WW6AJJMN5V6RY8KVX130FY`, kept an active instruction:
  *"Finish all ready kanban tasks in phase-3 until the scope is clear"*. The
  original text has a tag marker in front of `phase-3`. The marker is removed
  here, because the description parser reads a marker in this text and makes a
  tag from it.
- `.ralph/` held exactly one file, `01M0WW6AJJMN5V6RY8KVX130FY.md`. Session A
  had no file of its own.
- The Stop hook of session A then fired 50 times, from iteration 1 to the cap
  of 50. Each time it gave session B's instruction text.

Session A had handed its work to session B and had nothing to do. It could not
stop. Each wake made a new request to the model.

A second form of the same fault was seen earlier on the same day in the same
repository. Two instructions stayed in `.ralph/` from sessions that had ended.
They blocked a new session in the same way.

## Why the available workaround is bad

`clear ralph` without a `session_id` removes EVERY active instruction. So the
only way a blocked session can release itself is to break the loop of each
other session that runs at that moment.

In the case above, session B was in the middle of a task that gated 17 other
tasks. To stop itself, session A would have had to end session B's loop.
Session A did not do this, and it stayed in the loop to the cap.

## Suggested correction

Each instruction must have an owner, and the hook must fire only for that
owner.

- Write the owning session into the instruction file, and make the hook match
  on it. Do not fall back to a different owner.
- If a fallback is necessary, limit it to an instruction whose owning session
  is no longer alive. This keeps the correction for the stale-file case and
  does not capture a live peer.
- Give `clear ralph` a way to remove only the instructions of sessions that
  have ended.

## Acceptance Criteria

- [x] A session that holds no instruction of its own stops, although another
      session holds an active instruction in the same repository.
- [x] A session that calls `clear ralph` for itself stops, although another
      session holds an active instruction in the same repository.
- [x] An instruction of a session that has ended does not block a new session.
- [x] `clear ralph` for one session does not remove the instruction of a
      different session.

## Tests

- [x] A test sets an instruction for session X, then runs the Stop hook check
      for session Y, and asserts the answer is not `block`.
- [x] A test sets an instruction for session X, clears it for session X, and
      asserts the Stop hook check for session X is not `block`.
- [x] A test writes an instruction file for a session id that is not alive,
      runs the Stop hook check for a live session, and asserts the answer is
      not `block`.
- [x] A test sets instructions for two sessions, calls `clear ralph` with the
      id of the first, and asserts the file of the second stays.
#ralph