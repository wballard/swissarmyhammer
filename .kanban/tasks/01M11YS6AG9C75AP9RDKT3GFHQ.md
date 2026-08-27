---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m129121z7tdc5z9nysqndcf4
  text: |-
    Research. The production fix this card asks for was already in the tree. What was missing is the contract that holds it there.

    `markdown_lines` flags a fence line, a line inside a fenced block, and a heading as not tag-bearing, and BOTH writers already honor that flag. That landed on 2026-07-31 in `f1ad4bc61` ("stop remove_tag from mutating bystander task bodies") and `4a7bbf7e0`, with the single-walker refactor `d312017bb` the day before. Every one of them is an ancestor of HEAD, so the code was in place weeks before the sweep that filed this card.

    Measured, not assumed:

    - The three cards this card names are intact and byte-clean against HEAD. `01KQM6VWQTK6KCQMQNKS0BX5V3` still reads `### Notes on offender #2 (perspective-tab-bar)`; `01KT57BGTASD8W45HE708FM01R` still reads `### ⚠️ #1 TRAP …`; `01KT57DNAKPKRHSXJ1KH7NQSQJ` still reads `## RESOLVED — absorbed by card #4 …`.
    - A test built from those exact three headings passes against unchanged production code.

    So the reported heading loss does not reproduce on current source. The most probable single cause of the damage the sweep saw is ^7rh0bvj, whose write side WAS genuinely unguarded and destroyed 16 cards: a card mangled by that defect loses body text, which reads as lost heading text. ^7rh0bvj is fixed and done.

    What this card adds is the contract, stated as a rule rather than as three examples: a line the reader skips is a line neither writer may edit. `test_writers_copy_every_line_the_reader_skips` builds a body out of every ordered triple of 21 line shapes under both `\n` and `\r\n` endings, and asserts each skipped line survives in place through `remove_tag` and `rename_tag`. `test_writers_leave_a_heading_only_marker_untouched` pins the three real headings.

    The card's three work items are met: the writers honor the flag, `append_tag` and `rename_tag` are checked against it, and the heading-only test exists.
  timestamp: 2026-08-27T18:51:57.887551+00:00
- actor: claude-code
  id: 01m1298nh0b3fk9z4hqg61eg2a
  text: |-
    ### review — clean
    - evidence: `review sha HEAD~1..HEAD` (c157c8a72) — findings 0, confirmed 0, refuted 0, attempted 7, failed 0, skipped 0. 1 file reviewed: `crates/swissarmyhammer-kanban/src/tag_parser.rs`. 4 `.kanban/` files excluded by `.reviewignore`.
    - next: task moved to `done`.
  timestamp: 2026-08-27T18:56:07.200769+00:00
- actor: claude-code
  id: 01m1299cs8pxf0rwgjkcxk68dt
  text: |-
    ### finish iteration 1 — clean
    - implement: changed — crates/swissarmyhammer-kanban/src/tag_parser.rs, tests only; the production behavior already held (f1ad4bc61 / 4a7bbf7e0, 2026-07-31). The first implement agent died on a session rate limit; its work survived in the tree and was verified, not restarted.
    - test: green — cargo nextest run --workspace, 14236 passed, 0 failed, 0 skipped; clippy and fmt clean
    - commit: c157c8a72
    - review: clean — 0 findings, 7 validators attempted
    - next: none — the card is in done. ^4nzhg4s is now unblocked on both of its dependencies.
  timestamp: 2026-08-27T18:56:31.016339+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb180
title: kanban delete tag strips a `#word` out of a markdown heading
---
`tag_parser::parse_tags` does not count a `#word` inside a markdown heading
as a tag. `remove_tag` removes it anyway, so `delete tag` edits heading text
it never read as a tag.

## What happened

`.kanban/tasks/01KQM6VWQTK6KCQMQNKS0BX5V3.md` holds the heading

```
### Notes on offender #2 (perspective-tab-bar)
```

`delete tag` on a tag whose name normalizes to `2` rewrote it as

```
### Notes on offender (perspective-tab-bar)
```

The card lost the number the heading names. `parse_tags` never reported `2`
for that card, so the removal answered a tag the reader does not see.

Two more cards lost heading text the same way:

- `01KT57BGTASD8W45HE708FM01R` — `### ⚠️ #1 TRAP — must set ...` lost `#1`.
- `01KT57DNAKPKRHSXJ1KH7NQSQJ` — `## RESOLVED — absorbed by card #4 ...`
  lost `#4`.

## The rule the writers must obey

`tag_parser` documents one markdown walk behind the reader and both writers:
`markdown_lines` flags a fence line, a line inside a fenced block, and a
heading as NOT tag-bearing. `parse_tags` honors the flag. `remove_tag` does
not honor it for a heading.

## Work

- Make `remove_tag` skip a line `markdown_lines` marks not tag-bearing.
- Check `append_tag` and `rename_tag` against the same flag.
- Add a test: a body whose only `#bug` stands in a heading is unchanged by
  `remove_tag`, and `parse_tags` reports no tag for it either.

## Done when

A `#word` in a heading survives `delete tag`, and the reader and the writers
agree on which lines carry tags. #bug #kanban