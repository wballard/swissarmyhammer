---
assignees:
- claude-code
position_column: todo
position_ordinal: ffee80
title: 'review engine: the reviewed-file tally does not reconcile'
---
Found while reviewing commit `06e7a2fce` (task ^3fx5bny). The review engine's own file accounting does not add up.

## What was measured

`review sha HEAD~1..HEAD` over commit `06e7a2fce`:
- The commit touches 27 files.
- The report says 14 `.kanban/` files were excluded by `.reviewignore`, plus 2 `idioms-swift` fixture templates excluded as validator fixtures. That leaves 11 eligible.
- The report then says 8 reviewed and 16 not reviewed — 24 total.

24 is not 27, and 8 is not 11. Three files are unaccounted for:
- `builtin/validators/code-hygiene/VALIDATOR.md`
- `builtin/validators/code-hygiene/rules/idioms-swift.md`
- `doc/src/concepts/validators.md`

All three are markdown.

## Why this is probably a reporting fault, not an analysis gap

The same report says `failed: 0` and `skipped: 0`. No validator errored, and none was dropped. So the likely fault is in how the tally counts files, not in what the engine actually reviewed.

That is a guess. Prove it. Do not close this card on the guess.

## Why it matters

A tally that does not close cannot be used as evidence of coverage. Today a reviewer has to notice the arithmetic by hand. If the engine really did skip three markdown files, every markdown change in every review has been unreviewed and nothing said so.

## Work

1. Reproduce. Run `review sha` over a commit with a mix of code, markdown, ignored paths, and fixtures. Record the real numbers.
2. Find where the counts are produced and where the excluded, reviewed, and not-reviewed sets are built. They are probably three separate walks that disagree.
3. Decide the invariant: `total == excluded + reviewed + not_reviewed`, with every file in exactly one set.
4. Make the invariant a test, not a comment.

## Acceptance

- A test asserts the tally closes for a mixed commit — code, markdown, `.reviewignore` paths, and fixtures.
- The report names WHY each not-reviewed file was not reviewed. "Not reviewed" with no reason is what made this hard to diagnose.
- If markdown was genuinely never reviewed, that is a separate defect. Say so plainly and raise its own card. #review-tool