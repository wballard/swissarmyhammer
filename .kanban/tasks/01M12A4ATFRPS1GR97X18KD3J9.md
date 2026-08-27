---
assignees:
- claude-code
position_column: todo
position_ordinal: fff680
title: 'kanban CLI: --tags joins an array into one hyphen-joined tag'
---
The `kanban` CLI does not split a `--tags` array. It joins the whole argument
into one tag name. This is the same defect class as ^4nzhg4s, on the CLI
surface instead of the MCP dispatch surface.

## What happens

Measured on a fresh board, with a binary built at `c157c8a72`:

```
kanban task add    --title T1 --description hello --tags '["red","blue"]'
kanban task update --id <id> --tags '["red","blue"]'
kanban task tag    --id <id> --tags '["beta","gamma"]'
```

Each call reports `ok: true`. Each writes ONE tag:

```
hello #red-blue
hi #red-blue
hello #alpha #beta-gamma
```

The caller asked for two tags and got one. A scalar (`--tags 'alpha'`) is
correct, so only the array shape is affected.

## Where it is NOT

`dispatch::ref_list` is correct. It parses a stringified JSON array back into
its elements, and `tag_refs` / `req_tag_refs` route `tags` and `tag` through
it. ^4nzhg4s fixed that path and its tests pass. So the CLI either does not
reach `ref_list` for this param, or it mangles the value before dispatch.
Start at the `kanban-cli` argument layer for `--tags`.

## Blast radius

All three writing ops take `--tags`: `task add`, `task update`, `task tag`.
`untag` takes the same param and is very likely affected too. Every joined
name also mints a tag entity nobody asked for, so each call leaves debris on
the board — the same debris ^4nzhg4s had to sweep.

Found while sweeping the board for ^4nzhg4s: a CLI `--tags` call made during
that sweep created the tag `code-context-indexer` on the live board. The card
body and the stray entity were both reverted.

## Work

- Route the CLI `--tags` param through the same shape normalizer the dispatch
  layer uses, so an array, a scalar, and a stringified array each mean one tag
  for each element.
- Check `--assignees`, `--depends_on`, and `--attachments` on the CLI for the
  same gap; they share the forgiving-list contract.
- Add a CLI-level test that passes two tags in one call and asserts two tags on
  the task. A dispatch-level test does not cover this — the defect lives above
  dispatch.

## Done when

One CLI call applies two separate tags, and no CLI call mints a hyphen-joined
name the caller did not ask for. #bug #kanban #cli