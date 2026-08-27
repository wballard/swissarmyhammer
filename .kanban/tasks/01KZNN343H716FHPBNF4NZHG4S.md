---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m11yssd0bp7sdd4a6b0md3s6
  text: |-
    Research. The join happens in dispatch, not in the commands.

    `Verb::Tag` and `Verb::Untag` read their ref with `req(op, "tag")`, which is
    `Operation::get_string` — a scalar read. A stringified array
    (`"[\"tool-validators\", \"objectivity\"]"`, the shape an MCP client sends when
    the wire schema gives no array type-hint) came through as ONE string, and
    `tag_parser::normalize_slug` collapsed every run of `[`, `"`, `,` and space
    into a single `-`. That is exactly `tool-validators-objectivity`.

    `add task` and `update task` never had the defect: they route their `tags`
    param through `dispatch::ref_list`, which accepts an array, a scalar, and a
    stringified array. The two tagging ops simply did not use it.

    A real JSON array on `tag task` did not join — it failed with
    `missing required field: tag`, because `get_string` answers `None` for an
    array. Both halves of the card's report are the same missing call.
  timestamp: 2026-08-27T15:53:13.888343+00:00
- actor: claude-code
  id: 01m11yt7ekdqndf8cw7406jxmj
  text: |-
    BLOCKER on the sweep item. The board's own `delete tag` corrupts cards, so
    the sweep cannot finish. The whole sweep was reverted; the task tree is now
    byte-identical to HEAD.

    What I ran, and what it cost:

    1. Deleted the 66 tag entities whose name does not round-trip as a slug, plus
       the joined `tool-validators-objectivity`. `delete tag` rewrote 104 cards.
    2. Two defects in that write path damaged them:
       - `remove_tag` strips a `#word` out of a markdown HEADING, which
         `parse_tags` never counted as a tag. Three cards lost heading text —
         `### Notes on offender #2 (...)` became `### Notes on offender (...)`.
         Filed as ^kt3gfhq.
       - A read-write round trip destroys a card whose front matter holds a `---`
         run: the reader stops the front matter inside a markdown table row, so
         the card loses its title (`title: Untitled`) and part of its body.
         13 cards broke this way. Filed as ^7rh0bvj.
    3. Reverted `.kanban/tasks` to HEAD, then restored the 47 tag entities whose
       slug a task body actually carries, so state and history stay consistent.
       `git diff HEAD -- .kanban/tasks` is empty.

    What stands: 38 fragment tag entities are deleted — every one no task body
    carries. 30 fragment names remain, all of them tags a prose reference in some
    card creates (`1,` `2.` `4)` `8).` `init-doctor):` and the like).

    Why they cannot go yet: deleting one rewrites the cards that hold the prose
    `#1`, `#2` … and both defects above fire on that rewrite. The correction each
    of those cards needs — put the prose reference in inline code, `` `#4` ``, so
    the parser stops reading it as a tag — is itself a write, and it hits the same
    two defects.

    The sweep resumes once ^7rh0bvj and ^kt3gfhq land.
  timestamp: 2026-08-27T15:53:28.275350+00:00
- actor: claude-code
  id: 01m11z3c3fsm9h1b4rpbxbnnbg
  text: |-
    ### implement — stuck
    - code: `tag task` and `untag task` now read `tags`/`tag` through the same
      `dispatch::ref_list` shape normalizer `add task`/`update task` use, so an
      array, a scalar, and a stringified array each mean one tag per element.
      `TagTask`/`UntagTask` carry `tags: Vec<String>`; `apply_one_tag_ref` became
      `apply_tag_refs_to_task`, which resolves every ref before it edits the body
      (one bad ref rejects the whole call) and errors on an empty list.
    - tests: 7 new dispatch tests in
      `crates/swissarmyhammer-kanban/src/dispatch/tests/tags.rs`, including the
      board invariant that no tag name holds a comma, a quotation mark, a bracket,
      or a colon. 5 of the 7 were RED before the change; the stringified-array test
      reproduced `["tool-validators-objectivity"]` exactly.
    - verification: `cargo nextest run --workspace` — 14224 passed, 0 failed,
      0 skipped. `cargo fmt --check` clean, `cargo clippy --workspace --all-targets
      -- -D warnings` clean.
    - gap: the sweep item is NOT done. See the blocker comment above. 38 of the 68
      fragment tag entities are deleted; 30 remain, blocked on ^7rh0bvj and
      ^kt3gfhq. The task tree is byte-identical to HEAD.
    - next: land ^7rh0bvj and ^kt3gfhq, then finish the sweep.
  timestamp: 2026-08-27T15:58:27.951175+00:00
position_column: doing
position_ordinal: '8380'
title: 'kanban tag task: an array of tags becomes one hyphen-joined tag'
---
`tag task` does not split an array. It joins the array into one tag name.

## What happened

```
tag task { task: 01KZKCPTPD45XYMC34KN8PTDXB, tag: ["tool-validators", "objectivity"] }
```

The call reported `ok: true`. It created **one** tag named `tool-validators-objectivity` and applied that tag to the task. It did not apply `tool-validators`. It did not apply `objectivity`.

The tool contract says the opposite. It says the `tags` field takes a single tag, a JSON array, or a stringified JSON array, and that a bad reference is an error and never a silent no-op.

## A second defect in the same call

`tag task` does not accept `tags` at all:

```
tag task { task: <id>, tags: ["tool-validators", "objectivity"] }
→ MCP error -32603: tag task: parse error: missing required field: tag
```

Only `add task` and `update task` accept `tags`. The documentation gives `tag` as a one-element alias for `tags`, so the two fields must both work on `tag task`, and both must split an array.

## The blast radius

Run `list tags`. The board holds 218 tags. Many are wreckage from this defect class:

`1,`  `2,`  `3,`  `fi`  `f`  `4):`  `2:`  `BLOCK"`  `BLOCKED"`  `init-doctor):`  `[serial(cwd)]);`  `15012](https://github.com/ggml-org/llama.cpp/issues/15012))`

These are fragments of prose and of code, not tags. Each one is a silent write of a tag the caller did not ask for.

This is the same defect class as ^4t0ke4q, where `depends_on` silently drops a stringified array.

## Work

- Make `tag task` accept `tags` and `tag`, and make both split an array into one tag for each element.
- Make an unresolvable reference an error, not a silent write of a joined name.
- Add a test that applies two tags in one call and asserts two tags on the task.
- Add a test that asserts no tag name holds a comma, a quotation mark, a bracket, or a colon.
- Sweep the 218 tags on this board. Delete every fragment tag and correct the tasks that carry one.

## Done when

One call applies two separate tags, and the sweep leaves no fragment tag on the board. #bug #kanban