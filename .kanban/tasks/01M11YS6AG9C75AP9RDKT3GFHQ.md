---
assignees:
- claude-code
position_column: todo
position_ordinal: fff780
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