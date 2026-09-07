---
assignees:
- claude-code
position_column: todo
position_ordinal: fff880
title: 'search tasks: no tag/assignee/project sugar, unlike list tasks'
---
Found while fixing ^j6k3qpz (`list tasks` dropped its `tag` param).

`list tasks` now honours `tag`, `assignee`, `project` and `exclude_done` as
sugar for the `#`, `@` and `$` filter atoms. `search tasks` takes only
`query`, `filter` and `top_k`.

## Why it matters

The two ops are siblings: `search tasks` scopes its corpus through the same
`parse_filter_expr` path `list tasks` uses. An agent that learns `tag` from
`list tasks` will send it to `search tasks` and get a silently unscoped
corpus back, because dispatch keeps unknown params in the map and nothing
rejects them.

This is the same defect class as ^j6k3qpz, one op over.

## Work

- Give `SearchTasks` the same `tag` / `assignee` / `project` fields, folded
  into the filter the same way. `ListTasks::effective_filter`
  (crates/swissarmyhammer-kanban/src/task/list.rs) is the shape to share —
  lift it beside `parse_filter_expr` in `task/shared.rs` so both ops use one
  code path rather than two copies.
- Reuse `dispatch::scalar_filter_param` for the `Verb::Search` arm.
- Test each param against a corpus where the unfiltered answer differs from
  the scoped one. A one-task board cannot fail.

## Also in scope

`filter` itself is read with `op.get_string` on both ops, so
`{"filter": ["#bug"]}` is dropped without a word. Route it through the same
shape-tolerant reader, or reject a non-string.

## Done when

`search tasks { tag: X }` scopes the corpus exactly as
`search tasks { filter: "#X" }` does. #bug #kanban