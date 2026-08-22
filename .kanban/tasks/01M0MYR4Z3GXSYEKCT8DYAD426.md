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
- actor: claude-code
  id: 01m0n8yzznr064h5qch4vdk5v2
  text: |
    ### reproduction 4 — markdown dropped from a diff-scoped review

    `{"op": "review sha", "sha": "HEAD~1..HEAD"}` over commit `0182f3bf` on ^qs32yvp.

    The commit holds 11 files: 4 `.kanban/`, 4 markdown, 3 Rust.

    The engine answered "3 file(s) reviewed, 4 not reviewed" with `counts.findings: 0` and `counts.attempted: 7`. The 4 not reviewed are named in `skipped_files`, and all 4 are `.kanban/`, excluded by `.reviewignore`.

    The 4 markdown files are in NEITHER count:

    - `builtin/validators/code-hygiene/VALIDATOR.md`
    - `builtin/validators/code-hygiene/rules/idioms-swift.md`
    - `builtin/validators/swift/VALIDATOR.md`
    - `builtin/validators/swift/rules/idioms.md`

    3 reviewed + 4 skipped = 7, and 11 - 7 = 4 files that no line of the report accounts for. `skipped_files` names none of them, so the report carries no signal that anything was dropped.

    A hand review of those 4 files against real swiftformat 0.62.1 found 3 findings, so `findings: 0` was not a clean result. Same shape as reproductions 1-3: markdown is silently absent, and the tally hides the absence.
  timestamp: 2026-08-22T17:40:42.613157+00:00
position_column: todo
position_ordinal: ffee80
title: 'review engine: markdown files are silently dropped from diff-scoped review'
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

## Reproduction 3 — commit `f79727e2e`, task ^qs32yvp (2026-08-22)

The cleanest reproduction so far. The split is exactly along file type, with no fixture exclusion to confuse it.

`review sha HEAD~1..HEAD` over commit `f79727e2e`:

- The commit touches 24 files.
- 10 are `.kanban/` — the report names all 10 and the `.reviewignore` rule that excluded them.
- 14 are eligible: 9 `.md` and 5 `.rs`.
- The report says "5 file(s) reviewed, 10 not reviewed".

5 reviewed + 10 excluded = 15, not 24. Nine files are unaccounted for, and they are the nine markdown files — every one of them:

```
builtin/validators/code-hygiene/VALIDATOR.md
builtin/validators/code-hygiene/rules/disallowed-constructs-swift.md
builtin/validators/code-hygiene/rules/idioms-swift.md
builtin/validators/swift/VALIDATOR.md
builtin/validators/swift/rules/concurrency.md
builtin/validators/swift/rules/error-handling.md
builtin/validators/swift/rules/idioms.md
builtin/validators/swift/rules/optionals.md
builtin/validators/swift/rules/value-semantics.md
```

The 5 reviewed are precisely the 5 `.rs` files. `counts` reported `attempted: 7, failed: 0, skipped: 0, findings: 0`, and `skipped_files` listed only the 10 `.kanban/` paths.

### This reproduction settles the open question

Reproduction 1 left it open whether the fault was in the tally or in the analysis. This one answers it: the reviewed set is `.rs` only. Markdown is not merely miscounted, it is **not reviewed at all**, and the report claims a clean result without saying so.

Per the acceptance criterion above — "If markdown was genuinely never reviewed, that is a separate defect. Say so plainly and raise its own card" — that condition is now met and the separate card is owed.

### What this cost

Commit `f79727e2e` is a documentation commit: its whole substance is the 9 markdown files. The engine returned `findings: 0` over a commit whose reviewable content it never opened. A hand review of the same 9 files found four defects, including a real Swift coverage hole (`preferForLoop` silent on the `forEach` + `if` shape whose prompt bullet the commit deleted). A `findings: 0` that means "nothing was read" is worse than an error, because it reads as a pass.