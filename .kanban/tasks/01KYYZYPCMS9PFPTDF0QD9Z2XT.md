---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m12csdwdys87a2gwzfdtxg4t
  text: |-
    Fixed by the same change as ^j6k3qpz — recording it here rather than absorbing this card's scope in silence.

    `tag`, `assignee` and `exclude_done` are all dropped by the SAME `Verb::List` arm of `execute_task_query_operation`, so one edit answers both cards. This card's acceptance, item by item:

    1. "A `list tasks` call with `assignee` filters the page. Test must fail first, with at least two tasks on the board and only one assigned."
       Done. `ListTasks` now has an `assignee` field that becomes the `@<assignee>` filter atom, AND-ed onto `filter` — this card's Option 1. `dispatch_list_tasks_with_assignee_filter` now adds a second, unassigned task and asserts `total == 1` plus the returned title. It was RED before the fix, returning both tasks.

    2. "`exclude_done` either filters or is removed from `schema.rs`."
       It filters. `ListTasks.exclude_done` defaults to `true` when no `column` is named and to `false` when one is — the long-standing behaviour, now overridable. `exclude_done: false` widens an unscoped listing to the whole board; `exclude_done: true` with `column: "done"` returns empty rather than quietly ignoring one of the two params. The schema example on schema.rs line 180 is now truthful, and `test_list_tasks_sugar_params_are_documented` fails if either param ever leaves the schema again.

    3. "`dispatch_list_tasks_with_assignee_filter` is strengthened so it can fail: a second, unassigned task on the board."
       Done, as above. The identical weakness in `dispatch_list_tasks_with_tag_filter` was fixed too — it now seeds 3 tagged of 20 tasks.

    Beyond the card: `exclude_done` is read by a new `dispatch::bool_param` that accepts a real boolean or a case-insensitive `"true"`/`"false"` string, because MCP transports that stringify every argument would otherwise lose the param to a silent type mismatch — the same tolerance `get_u64` already carries for pagination. Any other value is an explicit error.

    Verification: `cargo nextest run --workspace` — 14252 passed, 0 failed. Clippy clean.

    Read ^j6k3qpz's comments for the full change description. This card needs no separate implementation; it should go through review with ^j6k3qpz's diff.
  timestamp: 2026-08-27T19:57:42.157010+00:00
- actor: claude-code
  id: 01m12mpncdrkh7r1e8dpzdh4jj
  text: |-
    Closed by the diff for ^j6k3qpz. Same op, same `Verb::List` match arm, same defect class: `tag`, `assignee`, and `exclude_done` were all read from the operation map and dropped.

    Commits: `f8fc8394c` (the fix) and `91fc23cfb` (the review finding on its test coverage).

    Your three acceptance items, each checked against the code rather than taken on the implementer's word:

    1. **A `list tasks` call with `assignee` filters the page, proved by a test that can fail.** `dispatch_list_tasks_with_assignee_filter` (`crates/swissarmyhammer-kanban/src/dispatch/tests/tasks.rs`) now seeds two tasks — "Worker task", assigned to `worker`, and "Nobody's task", unassigned — so the count distinguishes a working filter from a dropped one. That is exactly the strengthening item 3 asked for, so items 1 and 3 are answered by the same change.
    2. **`exclude_done` filters; it was not removed from the schema.** `dispatch.rs:831` reads it through `bool_param`, and `ListTasks::excludes_done` (`task/list.rs:167`) resolves it: an explicit value decides, otherwise the default follows `column` — an unscoped listing hides finished work, a listing naming a column returns that column whichever one it is. The schema example at `schema.rs:180` is now true.
    3. **`dispatch_list_tasks_with_assignee_filter` is strengthened.** Confirmed above.

    You offered two options. Option 1 was taken — `assignee` folds into the filter as the `@` atom, the way `project` folds as `$` — with one correction to how you framed it: the folding happens on the `Expr` AST inside `ListTasks::effective_filter`, not by appending text to the DSL. Text appending carried two defects of its own. `&&` binds tighter than `||`, so `format!("{filter} && ${project}")` rebound a caller filter that used `||`; and a value carrying a sigil or a space could inject a second atom into the caller's expression. Building the node avoids both.

    One thing you flagged that turned out to be wrong, usefully: `search tasks` does not carry this defect. Its schema is `query`/`filter`/`top_k` — it has no `tag` param at all. But `list tasks` now advertises sugar its sibling lacks, which is a new parity gap, filed as ^pbfrpj3.

    Verification: `cargo nextest run --workspace` 14253 passed, 0 failed; clippy and fmt clean; the review engine returned 0 findings over the final commit.
  timestamp: 2026-08-27T22:16:00.141671+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb480
title: list tasks silently ignores the documented `assignee` and `exclude_done` params
---
Found while auditing `dispatch.rs` for silent-drop params on ^n36mc1q.

`crates/swissarmyhammer-kanban/src/schema.rs` advertises this example:

```
{"op": "list tasks", "assignee": "alice", "exclude_done": true}
```

The `list tasks` arm of `execute_task_query_operation`
(crates/swissarmyhammer-kanban/src/dispatch.rs) reads only `column`, `filter`,
`project`, `page`, `page_size`, and `detail`. It never reads `assignee`, and
`exclude_done` appears nowhere else in the workspace. Both params are dropped
and the caller gets an `ok` with an unfiltered page.

## Evidence

- `exclude_done` matches exactly one line in `crates/` — the schema example
  itself. No handler exists.
- The existing test `dispatch_list_tasks_with_assignee_filter`
  (dispatch.rs) does NOT prove the filter works: the board holds one task, so
  `count == 1` passes whether or not `assignee` filters anything. Add a second,
  unassigned task and the test would still pass while the filter does nothing.

## Options

1. Fold `assignee` into the filter DSL the way `project` is folded
   (`@<assignee>`), and add `exclude_done` as a filter atom or drop it from the
   example.
2. Drop both keys from the schema example so the docs match the code.

Option 1 matches what the example promises. Option 2 is honest but removes a
capability an agent reading the schema believes it has.

## Acceptance

- A `list tasks` call with `assignee` filters the page. Test must fail first,
  with at least two tasks on the board and only one assigned.
- `exclude_done` either filters or is removed from `schema.rs`.
- `dispatch_list_tasks_with_assignee_filter` is strengthened so it can fail: a
  second, unassigned task on the board. #bug #kanban