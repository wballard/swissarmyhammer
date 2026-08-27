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
position_column: todo
position_ordinal: dc80
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