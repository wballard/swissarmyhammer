---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0n4s728v9y20tackbcrxjac
  text: |-
    ### Second independent occurrence — this is reproducible, not a one-off

    Commit `c097c1548`, reviewed in task ^t78sqr4. Same fault, same shape as the `06e7a2fce` case that opened this card.

    Measured: 15 files changed. The engine counted **3 reviewed** and **8 excluded** (6 `.kanban/`, 2 validator fixtures) = 11. Four files unaccounted for.

    The four missing files are **all markdown**, and in this commit they were the files that carried the entire deliverable — the two decisions this card existed to make lived in `builtin/validators/swift/rules/optionals.md`, `builtin/validators/swift/rules/idioms.md`, `builtin/validators/swift/rules/error-handling.md`, and `builtin/validators/code-hygiene/rules/idioms-swift.md`.

    This raises the severity. In the first occurrence the unaccounted files were incidental docs. Here the unreviewed markdown WAS the change. The reviewer only caught the claims because they re-measured all four files by hand against real swiftformat, outside the engine. That is not a gate; that is one reviewer being thorough.

    Two occurrences, both with every unaccounted file being markdown, makes "markdown is silently dropped from diff-scoped review" the leading hypothesis — not the "counting quirk" this card originally guessed. Test that hypothesis FIRST.

    If markdown is genuinely never reviewed on `review sha` / `review working`, then every prompt-rule change, every VALIDATOR.md change, and every doc change we have shipped has gone through an empty gate while reporting clean. That is a much bigger defect than a tally that does not add up, and it deserves its own card and its own priority.
  timestamp: 2026-08-22T16:27:38.952428+00:00
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