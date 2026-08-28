---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m12bxspbsh3gr3kmr17r3ha0
  text: |-
    Picked up. Research done.

    Root cause: `ListTasks` (crates/swissarmyhammer-kanban/src/task/list.rs) has no `tag` field at all. The `Verb::List` arm of `execute_task_query_operation` (crates/swissarmyhammer-kanban/src/dispatch.rs) reads only `column`, `filter`, `project`, `page`, `page_size`, `detail`. `KanbanOperation` keeps unknown params in its map and nothing rejects them, so `tag` is accepted and dropped.

    Same defect, same code path, same fix as ^qd9z2xt (`assignee` + `exclude_done`). One change covers both cards.

    Related findings:
    - `search tasks` does NOT advertise a `tag` param (its schema is `query`/`filter`/`top_k` only), so it does not have this defect. Its `filter` already scopes the corpus through the same `parse_filter_expr` path.
    - The existing dispatch tests `dispatch_list_tasks_with_tag_filter` and `dispatch_list_tasks_with_assignee_filter` (crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs) cannot fail: each puts ONE task on the board, so `count == 1` holds whether or not the param filters. Both must get a second, non-matching task.
    - `project` is folded into the filter in `dispatch.rs`, not in `ListTasks::execute`. So `ListTasks { project: Some(..) }` built directly in Rust also silently ignores the field. Folding in `execute` gives one code path for every caller.
    - The existing fold `format!("{filter} && ${project}")` does not parenthesize the caller filter, so a caller filter that uses `||` can bind wrong.

    The `#[operation]` macro derives the op schema from the struct fields, so new `Option<String>` / `Option<bool>` fields become documented params and CLI flags automatically.
  timestamp: 2026-08-27T19:42:36.747863+00:00
- actor: claude-code
  id: 01m12crzq6v4pwh953m41319ze
  text: |-
    Implementation landed. ONE change fixes this card AND ^qd9z2xt — same op, same match arm, same defect. Nothing from ^qd9z2xt was absorbed silently; its acceptance is met and recorded on that card too.

    What changed:
    - `ListTasks` (crates/swissarmyhammer-kanban/src/task/list.rs) gains `tag`, `assignee` and `exclude_done` fields beside the existing `project`, each documented, so the `#[operation]` macro puts them in the op schema and the generated CLI.
    - The filter is now composed in `ListTasks::effective_filter`, not at dispatch. `project`, `tag` and `assignee` each become ONE filter atom AND-ed onto the parsed `filter`. Every caller of the command benefits, not only the dispatch path.
    - Composition is on the `Expr` AST, never on DSL text. Two defects fall out of that choice: `&&` binds tighter than `||` in the DSL, so the old `format!("{filter} && ${project}")` rebound a caller filter that used `||`; and a sugar value carrying a sigil, a space, or an operator character could inject a second atom into the caller's expression. Neither is reachable now.
    - `exclude_done` defaults to `true` when no `column` is named and `false` when one is — the long-standing behaviour, now a value a caller can override.
    - `dispatch::scalar_filter_param` reads the three atom params through `ref_list`, so a scalar, a one-element array and a stringified array all resolve. A multi-valued array is an explicit error pointing at `filter`, never a silently unfiltered listing.
    - `dispatch::bool_param` reads `exclude_done` as a bool OR a case-insensitive `"true"`/`"false"` string, matching the tolerance `get_u64` already carries for pagination. Anything else errors.
    - The kanban MCP tool description (crates/swissarmyhammer-tools/src/mcp/tools/kanban/description.md) gains a "Scoping a listing" section. MCP clients read that file, not the full schema, which is why the card's author had to guess that `tag` existed.

    Tests, RED before GREEN in every case:
    - `dispatch_list_tasks_with_tag_filter` is the card's reproducer: 3 tagged of 20 tasks, asserting total == 3 and the exact ids. It failed returning a 10-task page of the whole board.
    - `dispatch_list_tasks_tag_param_matches_filter_atom` is the card's "Done when": `tag: X` and `filter: "#X"` return identical id lists.
    - Plus intersect-with-filter, unknown-tag-is-empty, multi-value-errors, single-element-array, and the strengthened assignee test.
    - `test_list_tasks_tag_param_does_not_rebind_filter_or` locks the precedence fix.
    - `test_list_tasks_sugar_params_are_documented` (schema.rs) locks declared ⟺ implemented, so the schema example on line 180 cannot go stale again.

    `search tasks` checked, as the card asks. The card's premise is wrong in a useful way: `search tasks` has NO `tag` param at all — its schema is `query`/`filter`/`top_k`, and its `filter` already works through the same `parse_filter_expr` path. So it does not carry this defect. It does now carry a parity gap, because `list tasks` advertises sugar that its sibling does not. Filed as ^pbfrpj3 rather than widened into this card.

    Verification: `cargo nextest run --workspace` — 14252 passed, 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt` applied.

    The MCP tool path is covered, not just the library: `swissarmyhammer-tools/src/mcp/tools/kanban/mod.rs` delegates to `swissarmyhammer_kanban::dispatch::execute_operation`, which is the exact function the dispatch tests drive.
  timestamp: 2026-08-27T19:57:27.654437+00:00
- actor: claude-code
  id: 01m12ctpv5cqcngk9aeqz7e4r1
  text: |-
    ### implement — changed
    - evidence: 5 files — crates/swissarmyhammer-kanban/src/task/list.rs, crates/swissarmyhammer-kanban/src/dispatch.rs, crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs, crates/swissarmyhammer-kanban/src/schema.rs, crates/swissarmyhammer-tools/src/mcp/tools/kanban/description.md. `cargo nextest run --workspace` 14252 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt` applied. 17 new tests, every one seen RED first (10 at dispatch level, 7 at command level).
    - next: /review. ^qd9z2xt is fixed by this same diff and should be reviewed with it — it still sits in `todo` because moving another agent's card was not mine to decide. ^pbfrpj3 filed for the `search tasks` parity gap this change opens up.
  timestamp: 2026-08-27T19:58:24.101047+00:00
- actor: claude-code
  id: 01m12kvwtxhdc77s7j6xvm3xp6
  text: |-
    ## Review Findings (2026-08-27)

    Scope: `review sha HEAD~1..HEAD` (f8fc8394c) — diffs only. 4 files reviewed, 12 excluded by `.reviewignore` (`.kanban/`), 1 unmatched by any validator (`description.md`). 7 validators attempted, 0 failed. 1 finding, confirmed.

    - [ ] `crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs:684` — `completeness/case-sensitivity-coverage`. `bool_param` (dispatch.rs:272-288) accepts a boolean string in any case, for both spellings, through `eq_ignore_ascii_case`. `dispatch_list_tasks_exclude_done_accepts_string_boolean` proves only the `false` variants (`false`, `False`, `FALSE`), so half the case contract is unproven. Cover the `true` variants too.

    A finding names one example of a cause. Check the other `bool_param` call sites in the same file for the same one-sided coverage rather than correcting only line 684.

    Note on the review run: the reviewer sub-agent was killed by its stream watchdog after the card reached `review` but before the engine reported. The engine was then run directly, so this report is the engine's own output, not a reconstruction.
  timestamp: 2026-08-27T22:01:23.037650+00:00
- actor: claude-code
  id: 01m12mfw7332ke9k84vfg719qc
  text: |-
    Review finding fixed: `completeness/case-sensitivity-coverage` on `dispatch_list_tasks_exclude_done_accepts_string_boolean`.

    The cause, not only the line:

    - `bool_param` has exactly ONE call site in the crate — `exclude_done` in the `Verb::List` arm of `execute_task_query_operation` (crates/swissarmyhammer-kanban/src/dispatch.rs). The sweep the finding asks for is therefore complete: no other call site can carry the same one-sided coverage.
    - `dispatch_list_tasks_exclude_done_accepts_string_boolean` now proves BOTH spellings in every case: `false`/`False`/`FALSE` and `true`/`True`/`TRUE`.
    - The `true` half is read on a board state where the value changes the answer, as the card requires. An unscoped listing defaults to `exclude_done: true`, so a `true` assertion there proves nothing. The listing instead names `column: "done"`, where the default is `false`. `exclude_done: "TRUE"` must then return 0, while a dropped param returns 1.
    - The test records both defaults as explicit baselines first, so a reader sees that neither half can pass on the default.

    Every assertion was seen RED before GREEN, through three separate mutations of `bool_param`:

    1. `eq_ignore_ascii_case` -> `==`: FAIL at the `"False"` spelling. Proves the case contract.
    2. Spelling table cut to `[("false", false)]`: FAIL at the `"true"` spelling. Proves the `true` half reaches the parser.
    3. `Ok(None)` returned for a parsed `true` (a dropped param): FAIL with `left: 1, right: 0` on `exclude_done: "true"`. This is the exact weakness ^qd9z2xt named — the `true` assertion cannot pass on the default.

    Production code was restored after each mutation. `git diff` on dispatch.rs is empty.

    Also in this pass:

    - `seed_open_and_done_board` replaces the same seven-line seed block copied into three `exclude_done` tests.
    - `dispatch_list_tasks_non_boolean_exclude_done_errors` now reads three rejected values, `"yes please"`, `"TRU"` and `"falsey"`, so the negative direction of the case contract holds: a case-insensitive match stays a match of the whole word, not a prefix or a stem.
    - `dispatch_list_tasks_sugar_params_match_case_insensitively` is new. The same diff documented `tag`, `project` and `assignee` as case-insensitive, and nothing proved it through the dispatch path. It reads `BUG`, `MyProj` and `ALICE` against a board that holds one matching and one non-matching task. Seen RED by making `scalar_filter_param` drop a single value: FAIL with `left: 2, right: 1`.

    Verification: `cargo nextest run --workspace` — 14253 passed, 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt` applied, no change.
  timestamp: 2026-08-27T22:12:17.763811+00:00
- actor: claude-code
  id: 01m12mg2v71bh651c2dn3ayr7x
  text: |-
    ### implement — changed
    - evidence: 1 file — crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs (+102 / -24). `cargo nextest run --workspace` 14253 passed / 0 failed; `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt` no change. Production code unchanged: `git diff` on dispatch.rs is empty.
    - next: /review. The card stays in `doing`.
  timestamp: 2026-08-27T22:12:24.551596+00:00
- actor: claude-code
  id: 01m12mp6gptk7s53sjhwwvhn9r
  text: |-
    ### finish iteration 2 — clean
    - implement: changed — crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs (+102/-24), tests only. `true` spellings now asserted under `column: "done"`, where the default is `false`, so the assertion changes the answer. Each seen RED through three separate mutations of `bool_param`.
    - test: green — cargo nextest run --workspace, 14253 passed, 0 failed; clippy clean; fmt no change
    - commit: 91fc23cfb
    - review: clean — 0 findings, 7 validators attempted, 1 file reviewed (2 .kanban files excluded)
    - next: none — the card is in done, and ^qd9z2xt closes with it
  timestamp: 2026-08-27T22:15:44.918472+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb380
title: 'kanban list tasks: the tag parameter is ignored and returns the whole board'
---
`list tasks` accepts a `tag` parameter and then does not filter on it.

## What happened

```
list tasks { tag: "objectivity" }
→ count: 10, total: 71
```

71 is the whole board. The result holds tasks with `tags: []`, which cannot carry `objectivity`. The same call with `tag: "tool-validators"` returned 68 the same way.

The filter expression is correct, and gives the answer the `tag` parameter should give:

```
list tasks { filter: "#tool-validators and #READY" }
→ count: 8, total: 8
```

## Why it matters

The failure is silent. The call returns `ok` with a large result, so a caller reads the whole board as the answer to a narrow question. A caller that reads only the first page draws a conclusion from 10 unrelated tasks.

## Work

- Make `list tasks` honor `tag`. Translate it to the filter atom `#<name>`, so one code path answers both.
- If `tag` is to be dropped instead, reject it with an error. Never accept a parameter and ignore it.
- Add a test that gives `tag` a name held by 3 of 20 tasks and asserts a count of 3.
- Check `search tasks` for the same defect. Its `tag` parameter takes the same shape.

## Done when

`list tasks { tag: X }` and `list tasks { filter: "#X" }` return the same tasks. #bug #kanban