---
comments:
- actor: claude-code
  id: 01m1024y6zzvj677jy4jk2rr96
  text: |-
    Picked up. Research on the cause.

    The file is NOT dropped by the ignore filter or the fixture split. It is dropped by validator matching.

    `scope_review` in `crates/swissarmyhammer-validators/src/review/scope.rs` runs `match_validators_and_files`. That function skips a file when `loader.matching_rulesets(&ctx)` is empty:

        let rulesets = loader.matching_rulesets(&ctx);
        if rulesets.is_empty() {
            continue;
        }

    No shipped validator declares a `*.md` match, so a Markdown file resolves, is announced as a `FileScoped` progress event, and then leaves the work-list with no record at all. It becomes no `ValidatorWork` file and no `ExcludedFile`.

    The engine already knows this is a hole. `WorkList::resolved_files` documents it word for word:

        /// Reviewed plus excluded need not reach it — a resolved file no validator matched is neither.

    That is exactly the reported symptom. The scope line reads "0 file(s) reviewed, 0 not reviewed" because `render_scope` prints `counts.skipped_files.len()` as the not-reviewed number, and `not_reviewed_paths` is built from the over-cap skips plus `excluded` only.

    Plan: give the drop a name. Add a third `ExclusionKind` for "no validator matches this file", record one `ExcludedFile` for every resolved file that paired with no validator, and render it in its own block. Then the path lands in `skipped_files`, the scope line reconciles, and the counts alone tell a caller nothing looked at the file.

    One trap to avoid: `render_full_exclusion` claims "this is a clean review" when the exclusions cover the whole resolved set. That claim is true for an ignore rule and for a fixture, and false for a no-validator file — which is the one reading this task forbids. The new kind must be kept out of that claim.
  timestamp: 2026-08-26T22:13:16.127305+00:00
- actor: claude-code
  id: 01m103bbmm7pdppjv6yda9ta8c
  text: |-
    Implementation landed. Option 1 from the card: the file is reported as not reviewed, with a reason.

    What changed:

    - `scope/excluded.rs` — a third `ExclusionKind::NoMatchingValidator`, its reason constant `"no validator matches this file"`, and the constructor `ExcludedFile::no_matching_validator`. Also `ExclusionKind::is_deliberate()`, which separates the two exclusions someone ASKED for from this one, which is a coverage gap.
    - `scope.rs` — `unmatched_exclusions` builds one `ExcludedFile` for every resolved file `match_validators_and_files` paired with nothing, and `scope_review` chains those onto the excluded list. Each path also logs at DEBUG, the way the ignore filter does.
    - `synthesize.rs` — the fixture renderer is generalized to `render_named_exclusions(kind, cause)` and called twice, so the new block reuses it instead of copying it. `render_full_exclusion` now counts only deliberate exclusions, so an unmatched file never carries the "this is a clean review" claim.
    - `review_op/response.rs` — the `skipped_files` docs now name the third reason and state that a caller closing on `findings == 0` must read this list too.

    Result for the reported case. The counts now carry `skipped_files: ["README.md"]` where a clean pass carries `[]`, and the report reads:

        > Scope: `review file README.md` — reviewed the whole of each named file. 0 file(s) reviewed, 1 not reviewed.

        > 1 file(s) not reviewed — no validator matched:
        > - `README.md` — no validator matches this file

    Two traps found and handled:

    1. `render_full_exclusion` would otherwise have fired on a scope whose only file was unmatched, printing "Every file in scope was excluded ... this is a clean review". That is the exact misreading this card forbids, so `is_deliberate` keeps the new kind out of that count.
    2. A first attempt at the scope-stage test used an UNTRACKED `README.md` under `Scope::Working` and failed for the wrong reason. `resolve_working` filters the untracked half through `is_code_file`, so an untracked non-code file never reaches the pairing stage at all. That is a narrower gate and a different question. The test now commits the Markdown file and then changes it, so it arrives as a tracked change. The test comment records this so the next agent does not repeat it.

    Discovery worth keeping: `review working` on a repository now names every tracked changed file no validator matches — docs, manifests, configuration. That is the honest report, and no in-repository caller gates on `skipped_files` being empty (checked: only tests and the wire type reference it).

    Each test was watched to fail first. The two `drive.rs` tests reproduced the card's output byte for byte before the fix. The `scope.rs` test was proved by temporarily removing the wiring, watching it fail, and restoring.
  timestamp: 2026-08-26T22:34:15.060653+00:00
- actor: claude-code
  id: 01m103bm596rert420s5yeqn1y
  text: |-
    ### implement — changed
    - evidence: 6 files — crates/swissarmyhammer-validators/src/review/scope/excluded.rs, crates/swissarmyhammer-validators/src/review/scope.rs, crates/swissarmyhammer-validators/src/review/scope/tests.rs, crates/swissarmyhammer-validators/src/review/synthesize.rs, crates/swissarmyhammer-validators/src/review/drive.rs, crates/swissarmyhammer-tools/src/mcp/tools/review/review_op/response.rs. `cargo nextest run --workspace`: 14217 run, 14166 passed, 51 failed — the same 51 that fail on a stashed clean tree (shipped Go/Swift/TypeScript tool-rule tests plus one environment-dependent precondition), so zero regressions. `cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - next: /review
  timestamp: 2026-08-26T22:34:23.785040+00:00
- actor: claude-code
  id: 01m1047r5y5wx6njkcx2ewa8x8
  text: |-
    ### implement — changed
    - evidence: 6 files — review/scope.rs, review/scope/excluded.rs, review/scope/tests.rs, review/synthesize.rs, review/drive.rs (swissarmyhammer-validators), mcp/tools/review/review_op/response.rs (swissarmyhammer-tools). Added `ExclusionKind::NoMatchingValidator`, recorded at the stage that drops an unpaired file, so the path reaches `skipped_files` with the reason "no validator matches this file". Added `ExclusionKind::is_deliberate()` to keep the new kind out of the "this is a clean review" claim in `render_full_exclusion`.
    - next: test
  timestamp: 2026-08-26T22:49:45.406008+00:00
- actor: claude-code
  id: 01m10481x3cky0azfmb6fp9rzf
  text: |-
    ### test — red
    - evidence: `cargo nextest run --workspace --no-fail-fast`. With the change: 14217 run, 14166 passed, 51 failed. Baseline (`git stash -u`, same command, `git stash pop`, verified with `git status` and `git stash list`): 14214 run, 14163 passed, 51 failed. The 51 failing test names are IDENTICAL in both runs — zero failures come from this task. Pre-existing causes: 13 need `swiftformat`, 35 need the Go toolchain (golangci-lint/staticcheck/revive), 2 need the npm package `eslint-plugin-sonarjs`, 1 is the precondition mismatch in `review::drive::tests::review_file_excludes_a_shipped_builtin_fixture_under_a_shadowing_user_set` (left: Some(Project), right: Some(User)). One extra first-pass failure was proved a flake: `swissarmyhammer-diagnostics::leader_follower_request_ipc follower_multi_step_rename_gets_real_leader_edits_under_one_lock` passed 5/5 alone and on the full-suite rerun; that crate is not in the diff. `cargo fmt --all --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - next: commit the checkpoint, then review. The 51 are missing local toolchains and one older defect. They are outside this card's scope, so implement is not re-run on them.
  timestamp: 2026-08-26T22:49:55.363326+00:00
position_column: doing
position_ordinal: '8380'
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

- [x] `review file <some.md>` returns a result a caller can distinguish from a
      clean pass by the counts alone.
- [x] The named file appears in `skipped_files` with a reason, or is genuinely
      reviewed.
- [x] The prose line reconciles: naming 1 file never reports "0 reviewed, 0 not
      reviewed".

## Tests

- [x] A test calls `review file` on a Markdown fixture and asserts the result is
      not equal to the clean-pass shape.
- [x] A test asserts reviewed plus not-reviewed equals the number of files named.

## Reported by

FoundationModelsRouter session, 2026-08-26. Related: the sibling-repository
scope limit, filed separately. #review