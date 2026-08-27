---
assignees:
- claude-code
position_column: todo
position_ordinal: fff680
title: kanban task write corrupts a card whose front matter holds a `---` run
---
A read-write round trip of a task destroys the card when the stored front
matter holds a `---` run. The card loses its title and part of its body.

## What happened

`01M076YBBHE5ZQJCM518491BB0` carries a comment whose text holds a markdown
table. One row of that table is `|---|---|---|---|`. The comment is stored in
the front matter of `.kanban/tasks/01M076YBBHE5ZQJCM518491BB0.md`.

One `update task` on the card rewrote the file as:

```
    | the refusing path | status | stdout | stderr |
    |
title: Untitled
---
|---|---|---|
```

The reader stopped the front matter at the `---` INSIDE the table row. It
then read no `title`, so the writer wrote `title: Untitled`, and the rest of
the table became the first line of the body. The card's real title and the
first part of its description were lost.

The delimiter test must be a WHOLE LINE that equals `---`. It reads a
substring today.

## The blast radius

The sweep on ^4nzhg4s wrote 104 cards and broke 13 of them this way. One
`grep -rl "title: Untitled" .kanban/tasks/*.md` over the board before that
sweep already answered 1 card, so the defect has struck before.

Every op that writes a task carries it: `update task`, `tag task`,
`untag task`, `delete tag` (which rewrites every task that holds the tag),
`rename tag`.

## Work

- Make the front-matter delimiter a whole-line test on both the read and the
  write side.
- Add a test that round-trips a task whose comment text holds `|---|---|`
  and asserts the title, the description, and the comment all survive.
- Add a test that round-trips a task whose DESCRIPTION holds a `---`
  horizontal rule on its own line.
- Sweep the board for the cards this already broke: find each
  `title: Untitled` card and restore it from its changelog.

## Done when

A card whose front matter holds a `---` run survives a write unchanged, and
no card on the board reads `title: Untitled`. #bug #kanban