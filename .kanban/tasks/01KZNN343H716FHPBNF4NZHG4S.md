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
- actor: claude-code
  id: 01m11z98mqtnqf1s8abcpcf6y9
  text: |-
    ### commit — changed
    - evidence: 7af361e16 fix(kanban): route tag task/untag task through ref_list dispatch
    - included: the 7 code/doc files for the fix, this card's own move and comments, the 38 fragment tag-entity deletions (moved to .kanban/tags/.trash), and the two follow-on defect cards filed during the sweep (^7rh0bvj, ^kt3gfhq)
    - excluded: an unrelated new task (01M11WWMDXV903CEQV2C78RS70, a "project operation removal" card) left untracked — not part of this card's work
    - next: land ^7rh0bvj and ^kt3gfhq, then resume the fragment-tag sweep
  timestamp: 2026-08-27T16:01:41.015141+00:00
- actor: claude-code
  id: 01m11za0q3apjs53emq666y920
  text: |-
    ### finish iteration 1 — stuck
    - implement: changed — 7 files; `tag task`/`untag task` now read `tags`/`tag` through `dispatch::ref_list`; 7 new tests, 5 RED first
    - test: green — cargo nextest run --workspace, 14224 passed, 0 failed, 0 skipped; fmt and clippy clean (run inside the implement step)
    - commit: 7af361e16
    - review: not run — the step before it reported stuck
    - blocker: the sweep item cannot finish. `delete tag` rewrites cards, and that write path holds two defects that damaged 16 cards. The whole sweep was reverted. Blocked on ^7rh0bvj and ^kt3gfhq.
    - next: land ^7rh0bvj and ^kt3gfhq, then finish the sweep and take this card through review
  timestamp: 2026-08-27T16:02:05.667496+00:00
- actor: claude-code
  id: 01m12av0z6mhj44w1yrm489g08
  text: |-
    Sweep done. The board holds 134 tags, down from 181. Every remaining name is a
    valid slug, so no fragment name is left.

    How the sweep was made safe. `delete tag` normalizes the stored name to a slug
    and then runs `remove_tag` over EVERY card. So a fragment is only safe to
    delete once no body carries its slug — then the `new_body != body` guard skips
    every write and the delete touches the entity alone. The order was therefore:
    correct the prose first, delete the entities second.

    1. Prose corrections — 48 cards, 129 markers. Every prose reference to a
       numbered item (`#1` … `#8`, `#10`, `#29`, `#64`, `#1b`) was put in inline
       code, so the parser stops reading it as a tag and the sentence still says
       what it said. Measured, not assumed: no marker stood on a line that holds
       only tag markers, so not one of the 129 was a label anyone applied on
       purpose. The rewrite was proved byte-faithful before it ran — reassembling
       all 3063 card bodies from the same line walk returned every one unchanged —
       and each corrected body differs from its original by exactly two backticks
       for each wrapped marker and by nothing else.

    2. Entity deletions — 47 fragment entities. 44 were bare-number debris, and
       the slug collisions were severe: 10 separate entities all normalized to `2`,
       7 to `1`, 7 to `3`, 7 to `4`.

    3. The two hyphen-joined tags this card is about were repaired, not just
       deleted. `bug-code-context-indexer-lsp-live-leader` was the ONLY tag on its
       card, and the card's create record carries no such marker — the joined name
       was written later by the defect. The five tags the caller asked for
       (`bug`, `code-context`, `indexer`, `lsp-live`, `leader`) are now applied
       separately. `tool-validators-objectivity` already had its two real tags
       beside it, so only the joined marker went.

    4. `init-doctor):` needed special handling and is worth writing down. Its slug
       is `init-doctor`, which is also a REAL tag carried by 20 cards. Deleting it
       the ordinary way would have stripped all 20 real markers, and `update tag`
       is no escape — it renames bodies using the OLD slug, and its duplicate-name
       check rejects the rename anyway. The way through: rename the REAL tag aside
       to `init-doctor-sweep-tmp` (in-place marker rewrite), delete the fragment
       (now carried by nobody, so zero card writes), rename back. The 20 cards
       hash byte-identical before and after.

    Kept on purpose, because they are labels someone applied, not prose or code
    debris: `te`, `power`, `new-tag`, `sample`, `junk-and-things`. Each sits in a
    deliberate tag-marker position. `te` is meaningless as a label, but it stands
    next to `#field-special-case` in the trailing marker run of its card, so it
    fails the "prose or code debris" test this card sets.

    Verification. All 3310 tracked cards were compared against HEAD: 0 titles
    changed, 0 newly `Untitled`, 0 files missing. The `Untitled` count is 1 both
    before and after — `01M11YRV7M56YWPCJNG7RH0BVJ` (^7rh0bvj) is already
    `Untitled` in HEAD, damaged by the very defect it documents, and this sweep
    neither caused nor cured it. Across the 48 correction cards the diff is 102
    lines rewritten and identical once backticks are stripped.
  timestamp: 2026-08-27T19:23:37.318359+00:00
- actor: claude-code
  id: 01m12b3k3jaf8e1n3hdggb9jw6
  text: |-
    ### implement — changed
    - evidence: board sweep only; no Rust source changed. Tags 181 → 134; 47 fragment entities deleted; 48 cards corrected (129 prose markers put in inline code); 2 hyphen-joined tags replaced by the tags the caller asked for. `cargo nextest run --workspace` — 14236 passed, 0 failed, 0 skipped. `cargo fmt --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean. All 3310 tracked cards compared against HEAD: 0 titles changed, 0 newly `Untitled`, 0 files missing.
    - discovered: the SAME defect class is still live on the CLI surface. `kanban task add|update|tag --tags '["a","b"]'` writes one tag `a-b` and reports `ok: true`. `dispatch::ref_list` is correct, so the CLI does not reach it for this param. Reproduced on a fresh board with a binary built at `c157c8a72`. Filed as ^18kd3j9. One such call fired against the live board during this sweep and created the tag `code-context-indexer`; the card body and the stray entity were both reverted, and the card is byte-identical to HEAD again.
    - note: two `kanban serve` processes (PIDs 65082, 14821) ran on this board for the whole sweep. No damage is attributable to them — every batch was diffed, and the only changed files are the intended ones.
    - next: `/review`.
  timestamp: 2026-08-27T19:28:18.034526+00:00
- actor: claude-code
  id: 01m12bpc5g7xdr6r9n1cgheejv
  text: |-
    ### review — clean
    - evidence: two engine passes, 0 findings in both.

    1. `review sha HEAD~1..HEAD` (754bf2608) — **0 files reviewed, 317 excluded**. Every
       file in that commit is board data under `.kanban/`, which `.reviewignore`
       excludes. This pass looked at NO code. Read alone it is not evidence about
       any source file.

    2. `review sha 7af361e16` — **8 files reviewed**, 14 validator/file pairs
       attempted, 0 failed, 0 skipped, 0 findings. This is the code half of the
       card, and it is the pass that carries the verdict.

    Why the second pass was run. The finish comment of iteration 1 records
    `review: not run — the step before it reported stuck`. The Rust change for this
    card had therefore never reached the review engine. Moving the card on pass 1
    alone would have read an empty scan as a clean bill of health for code no
    validator ever opened.

    Board state checked directly, not taken from the sweep note: 134 tag entities;
    zero names that are not a clean slug; zero names holding a comma, a quotation
    mark, a bracket, or a colon. This matches the card's "Done when".

    Work items, each met: `tag task`/`untag task` read `tags` and `tag` through
    `dispatch::ref_list`; an unresolvable ref rejects the whole call; a test applies
    two tags in one call; a test asserts the tag-name character invariant; the sweep
    leaves no fragment tag.

    Out of scope, correctly filed elsewhere: the same defect class is still live on
    the CLI surface (`kanban task add|update|tag --tags '["a","b"]'` writes one tag
    `a-b`), tracked as ^18kd3j9. It is not a finding against this card.

    - next: none. Card moves to done.
  timestamp: 2026-08-27T19:38:33.520643+00:00
- actor: claude-code
  id: 01m12bqgxtnat9bk8q048kbnrr
  text: |-
    ### finish iteration 2 — clean
    - implement: changed — board data only; 48 cards corrected (129 prose markers wrapped in inline code), 47 fragment tag entities deleted (181 → 134 tags), both hyphen-joined tags repaired
    - test: green — cargo nextest run --workspace, 14236 passed, 0 failed, 0 skipped; clippy and fmt clean
    - commit: 754bf2608
    - review: clean — 0 findings. The HEAD~1..HEAD pass scanned 0 files (all 317 excluded by .reviewignore), so a second pass ran over 7af361e16, the code half, which had never reached the engine: 8 files, 14 validator pairs, 0 findings.
    - next: none — the card is in done. Two follow-ons stand open: ^18kd3j9 (the same defect class is still live on the CLI) and the recovery of ^7rh0bvj's own card file.
  timestamp: 2026-08-27T19:39:11.162852+00:00
- actor: claude-code
  id: 01m14dkqyxzkgq34qd5c637sc6
  text: |
    Correction to the sweep verification above, made while working ^jp77n3a.

    The sweep comment says: "`01M11YRV7M56YWPCJNG7RH0BVJ` (^7rh0bvj) is already
    `Untitled` in HEAD, damaged by the very defect it documents". That is wrong.
    The card is whole. Nothing was lost.

    The measurement was a `grep "^title: Untitled$"`, which cannot tell front
    matter from body prose. The one match is line 267 of that card, in the BODY,
    inside a fenced code block. The block quotes the corrupt output the card
    reports. The card's CREATE record already carried the line.

    Line 251, in the front matter, carries the real title:

        title: kanban task write corrupts a card whose front matter holds a `---` run

    Proof: I replayed all 17 records of the card's `.jsonl` from an empty document
    with the `patch` utility. The result is byte-identical to the file on disk,
    `sha256 351db22cb...`, 16725 bytes. All four committed versions carry the same
    title.

    A front-matter-aware sweep of all 3314 cards finds ZERO live cards without a
    title. The only 2 `Untitled` cards are the two empty trashed tasks, which never
    had a title.

    The sweep this card records is therefore correct and complete. Only its
    `Untitled` reading was wrong. Measure with a front-matter-aware check, never a
    bare grep.
  timestamp: 2026-08-28T14:50:33.309875+00:00
depends_on:
- 01M11YRV7M56YWPCJNG7RH0BVJ
- 01M11YS6AG9C75AP9RDKT3GFHQ
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb280
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