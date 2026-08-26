---
position_column: todo
position_ordinal: ffe880
title: 'review: a Markdown file reports attempted 0, which is indistinguishable from a clean pass'
---
## What

`review file` on a Markdown file reports `attempted 0` and `findings 0`. The
counts of a file that got no coverage are the same as the counts of a file that
passed. An orchestrator that reads `findings == 0` as a pass will close a task
that no validator ever looked at.

Reproduction, run in `FoundationModelsRouter` on a README of about 150 lines:

    review file README.md

    {
      "markdown": "## Review Findings (2026-08-26 17:05)\n\n> Scope: `review file README.md` — reviewed the whole of each named file. 0 file(s) reviewed, 0 not reviewed.\n\nNothing in scope to review.\n",
      "counts": { "findings": 0, "confirmed": 0, "refuted": 0,
                  "attempted": 0, "failed": 0, "skipped": 0, "skipped_files": [] }
    }

Note `skipped: 0` and `skipped_files: []`. The file is not counted as skipped.
It is not counted at all. The report says "0 file(s) reviewed, 0 not reviewed",
which does not add up to the one file that was named.

## Why it matters

The `finish` pipeline closes a task when a review returns zero findings. A
Markdown-only task therefore cannot pass the gate honestly, and it can pass the
gate dishonestly. Both outcomes are wrong:

- Read the counts strictly and no documentation task can ever close, because
  zero coverage must never count as a pass.
- Read the counts loosely and every documentation task closes unreviewed.

This blocked a real task. `^xpbhgac` in the FoundationModelsRouter board changed
only `README.md`, and there was no way to tell a correct README from an
unreviewed one.

## What good looks like

Either outcome is acceptable. The requirement is that the result must not be
mistakable for a pass.

1. A Markdown file that is out of scope by design is reported as **skipped**,
   with the file in `skipped_files` and a reason. `attempted 0` with a non-empty
   `skipped_files` is then honestly distinguishable from a clean pass.
2. Or Markdown gets real validators — dead links, stale symbol names, headings.

Whichever is chosen, a caller must be able to tell "nothing looked at this" from
"this is clean" using the counts alone, without parsing the prose.

## Acceptance Criteria

- [ ] `review file <some.md>` returns a result a caller can distinguish from a
      clean pass by the counts alone.
- [ ] The named file appears in `skipped_files` with a reason, or is genuinely
      reviewed.
- [ ] The prose line reconciles: naming 1 file never reports "0 reviewed, 0 not
      reviewed".

## Tests

- [ ] A test calls `review file` on a Markdown fixture and asserts the result is
      not equal to the clean-pass shape.
- [ ] A test asserts reviewed plus not-reviewed equals the number of files named.

## Reported by

FoundationModelsRouter session, 2026-08-26. Related: the sibling-repository
scope limit, filed separately. #review