---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0mw4vbk9z5tvdq3qd47x52z
  text: |-
    ### Research — picked up, tool installed

    `brew install swiftformat` -> SwiftFormat 0.62.1 (the newest release; 0.62.1 shipped 2026-07-07).

    **Harness facts the card must satisfy**

    - Rule file: `builtin/validators/code-hygiene/rules/<name>.md`. `name:` in the frontmatter must equal the fixture prefix.
    - Fixtures: the doctor's `find_fixture` takes `<rule>.pass.*` and `<rule>.fail.*` — exactly ONE pair for each rule. So "a pass fixture and a fail fixture per rule group" is one pair whose fail half trips every group and whose pass half trips none.
    - `materialize_fixtures` copies the WHOLE `fixtures/` directory flat into a temp dir and strips `.tmpl`. A `files`-scope rule gets only the fixture under test as `"$@"`.
    - `crates/mirdan/src/builtin_validators.rs` -> `CODE_HYGIENE_FIXTURES` needs the two new `.tmpl` names, or three mirdan roster tests fail.
    - `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/scope_roster.rs` -> `SHIPPED_TOOL_RULE_COUNT` 25 -> 26.
    - `.../shipped/zero_argument.rs` -> `FILES_SCOPE_RULE_COUNT` 14 -> 15. The script MUST open with the three guard lines above every line that runs.
    - `.../shipped/temp_directory.rs` -> a script that calls `mktemp -d` must write `work="$(mktemp -d)"` with `trap 'rm -rf "$work"' EXIT` directly under it, and `doctor.check_command` must name `mktemp`.
    - New shipped test file needs a `mod` line in `.../tests/shipped.rs` and a roster entry.

    **Measured: two card-listed rules do not exist in any released SwiftFormat**

    `swiftformat --rules` on 0.62.1 lists 153 rules. Of the 28 the card names, 26 are present. `preferLazyMap` and `ifExpressions` are NOT:

        error: Unknown rule 'preferLazyMap'. Did you mean 'preferFlatMap'?

    They stand in Airbnb's `airbnb.swiftformat` on master, which tracks SwiftFormat `main`, not a release. Checked the eight newest SwiftFormat releases: neither name appears in any of them.

    `--unknown-rules ignore` does NOT help. Measured on 0.62.1, all three spellings exit 70 with the same error: `--rules a,b`, `--config` file holding `--rules`, and `--disable all --enable a,b`.

    Plan: the script names every card-listed rule and INTERSECTS that list with `swiftformat --rules` at run time. The two unreleased names take effect the day SwiftFormat ships them, with no edit. A shipped test holds the intersection so a typo or a rename fails loudly instead of dropping silently.

    **Measured: the version gate**

    `--min-version 0.99.0` exits 70 with `error: Project specifies SwiftFormat --min-version of 0.99.0.` That is the version gate. `doctor.check_command` carries the same floor so doctor reports the row degraded and the prompt rules keep running, rather than the run breaking mid-review.
  timestamp: 2026-08-22T13:56:42.995732+00:00
- actor: claude-code
  id: 01m0mxrg295fq2tmm3dy3n3x3f
  text: |-
    ### Implementation landed

    The rule is `idioms-swift`, in `code-hygiene`, `scope: files`.

    **Two design answers the card did not anticipate, both forced by measurement**

    1. **The script intersects its roster with `swiftformat --rules`.** It NAMES all 28 of the card's rules, including `preferLazyMap` and `ifExpressions`, and enables the intersection with what the installed swiftformat knows. Naming an unknown rule outright breaks every run at status 70, and `--unknown-rules ignore` does not help — measured over six spellings. The intersection is how the two unreleased names take effect the day SwiftFormat ships them, with no edit. `the_shipped_swift_idioms_tool_rule_names_only_rules_swiftformat_knows` reads the roster off the shipped script and holds every other name to standing in `swiftformat --rules`, so a typo or a rename fails loudly instead of dropping silently.

    2. **The script runs swiftformat once for each file.** One refusing path costs a single swiftformat run EVERY finding it made — measured over four refusing shapes, each reporting 0 findings and leaving the healthy file beside it unjudged. That is the answer `builtin/validators/README.md` refuses. Proved RED->GREEN: with `"$@"` in place of `"$file"` the acceptance test reports `[]` against the expected 37.

    **Measured behaviour recorded in the rule body**

    - The project's own `.swiftformat` cannot turn a rule of this gate off (`--disable`, `--enable` and `--rules` all lose to the command line), and its `--exclude` list still holds — which is the generated-code carve-out, reached the same way `function-length-swift` reaches it. So the script names no `--config`.
    - Five rules read the Swift language version. The run passes NO `--swift-version`; a project states its version in `.swift-version`. Passing the Airbnb pin would tell a Swift 5 project to write `values.count(where:)`, a finding its own toolchain cannot satisfy.
    - `testSuiteAccessControl` and `validateTestCases` read the SAME declaration, and in a run holding both only the first reports it. Measured over three shapes, three ways each. Both are enabled; the test pins the order.
    - `redundantMemberwiseInit` reports every line of the range it would delete, so one initializer is five findings. Collapsing consecutive same-rule lines was measured and REFUSED: `typeSugar` reports rows 13, 14 and 15 of the fail fixture for three distinct properties, and a collapse would report one.

    **`supersedes` is empty, on purpose**

    The key names a WHOLE prompt rule and the engine skips that rule whole. This gate decides six bullets across `swift/rules/idioms.md` and `swift/rules/value-semantics.md`, and neither rule is only those bullets — naming either one would take its other bullets out of every review the moment swiftformat is installed. Card ^qs32yvp owns the prompt half. `stuttering-name-go` and `unused-dependencies-rust` are the precedent for an empty key.

    **Version floor: SwiftFormat 0.62.1**, stated twice — `doctor.check_command` ends in `printf '' | swiftformat --lint --quiet --min-version 0.62.1` (exit 0 at the floor, exit 70 below it), and the run states `--min-version` too, reading that one error line apart from a declined path and exiting 1 for it. No install commands, `fix_hint: brew install swiftformat` — the same shape `function-length-swift`, `missing-docs-swift` and `dead-code-swift` each state, and for the same measured reason: Homebrew installs the current version only, so no Homebrew command can pin one.

    **Fixtures**: one pair, `idioms-swift.fail.swift.tmpl` / `.pass.swift.tmpl`. The doctor's `find_fixture` takes exactly one `<rule>.fail.*` and one `<rule>.pass.*`, so "a pair per rule group" is one pair whose fail half holds a declaration for every group under a `// MARK:` heading. Measured: 37 findings carrying 25 of the 26 enabled rules, and 0 findings with 0 bytes of stderr for the pass half.

    **Blast radius closed**: `SHIPPED_TOOL_RULE_COUNT` 25->26, `FILES_SCOPE_RULE_COUNT` 14->15, `TEMP_DIRECTORY_RULE_COUNT` 22->23, `CODE_HYGIENE_IDIOMS_TOOL_RULES` added to the builtin roster test, `CODE_HYGIENE_FIXTURES` in mirdan. The `zero_argument` doc measurement was stale before this change (it read "16 rules, 6 on the first line, 10 under `set -e`" over a roster of 14); re-measured and corrected to 15 rules, 4 and 11.

    `code-hygiene/VALIDATOR.md` claimed `stuttering-name-go` was "the only TOOL rule of this set" to supersede nothing and "one of two tree-wide". Both are now false, so both were corrected, and the set gained a section for the new rule. `doc/src/concepts/validators.md` carried the same claim and was corrected.

    ### implement — changed
    - evidence: `cargo nextest run -p swissarmyhammer-validators -p mirdan` -> 1330 passed, 0 failed. `cargo clippy --all-targets -- -D warnings` -> clean. RED proven for `the_shipped_swift_idioms_tool_rule_reports_a_file_beside_one_it_declined` by reverting the per-file loop.
    - next: /review
  timestamp: 2026-08-22T14:24:55.369476+00:00
- actor: claude-code
  id: 01m0mybdbwa7m64xfq8wk77st0
  text: |-
    ### test — green
    - evidence: `cargo nextest run --workspace` — 14169 tests run: 14169 passed, 0 skipped; `cargo fmt --check` clean; `cargo clippy --workspace --all-targets -- -D warnings` clean
    - note: fixed a `cargo fmt` violation in the untracked file `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` (whitespace-only, unrelated idioms-swift work in progress). Re-verified `cargo nextest run --package swissarmyhammer-validators` (826/826 passed) and the full workspace suite again after the fix.
    - next: none
  timestamp: 2026-08-22T14:35:15.196589+00:00
- actor: claude-code
  id: 01m0myd3w5s6krf4beekahshcd
  text: |-
    ### commit — changed
    - evidence: 06e7a2fce feat(validators): add idioms-swift tool rule to code-hygiene
    - next: none
  timestamp: 2026-08-22T14:36:11.013215+00:00
- actor: claude-code
  id: 01m0mypv1gac5n9hqjwdmzxd4j
  text: |
    ### review — clean
    - evidence: `review sha HEAD~1..HEAD` (commit 06e7a2fce) — 0 findings, 0 confirmed, 0 refuted; 7 validators attempted, 0 failed, 0 skipped. 8 files reviewed. Not reviewed: 14 `.kanban/` files (.reviewignore) and the 2 `idioms-swift` fixture templates (validator fixtures). No prior Review Findings section on this task, so no open items.
    - next: task moved to `done`. The prompt half of the gate stays with card ^qs32yvp.
  timestamp: 2026-08-22T14:41:29.648028+00:00
- actor: claude-code
  id: 01m0myqsfqjc5z0ftzqvse0tha
  text: |-
    ### finish iteration 1 — clean
    - implement: changed — 13 files; installed swiftformat 0.62.1; new `idioms-swift` tool rule in code-hygiene, fixture pair, 5 shipped acceptance tests driving real swiftformat
    - test: green — cargo nextest run --workspace, 14169 passed, 0 failed, 0 skipped; cargo fmt --check clean; cargo clippy --workspace --all-targets -- -D warnings clean
    - commit: 06e7a2fce
    - review: clean — review sha HEAD~1..HEAD, 0 findings, 7 validators attempted, 0 failed, 0 skipped
    - result: done in one iteration

    Three decisions were forced by measurement, not assumption, and all three survived review:
    1. `supersedes` left empty — the gate covers 6 bullets across 2 prompt rule files, and `supersedes` names a whole rule. Handed to ^qs32yvp with a written blocker note.
    2. The script names all 28 rules but enables the intersection with what the installed swiftformat knows. `preferLazyMap` and `ifExpressions` exist only on SwiftFormat `main` (Airbnb tracks `main`, we track releases), and an unknown rule name aborts the entire run.
    3. swiftformat gets one path per run — a single bad path was measured to zero out every finding in a batched run, across four failure modes.
  timestamp: 2026-08-22T14:42:00.823850+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffab80
project: swift-validator
title: 'swift: add a swiftformat tool rule to code-hygiene (the missing tool half)'
---
No validator runs `swiftformat` today. `builtin/_partials/project-types/swift.md:40` mentions it only as a parenthetical. Build the tool rule that makes SwiftFormat a real validator.

## What to build

Add a `swiftformat` tool rule under `builtin/validators/code-hygiene/rules/`, next to the four Swift tool rules that already exist (`function-length-swift.md`, `magic-numbers-swift.md`, `missing-docs-swift.md`, `dead-code-swift.md`). Follow their shape exactly — they are the working pattern.

Run SwiftFormat in lint mode over the changed `.swift` files. Report each violation as a finding. Do NOT rewrite files; review is read-only.

## Rules to enable

These decide, deterministically, things our prompt rules currently ask an LLM to judge:

| Enables | Supersedes (prompt rule) |
|---|---|
| `typeSugar` + `--short-optionals always` | `idioms.md` shorthand type sugar |
| `void` | `idioms.md` return `Void` not `()` |
| `redundantMemberwiseInit` | `idioms.md` synthesized memberwise init |
| `preferForLoop` | `idioms.md` `for` over `forEach` |
| `hoistPatternLet` + `--pattern-let inline` | `idioms.md` per-case `let` binding |
| `preferFinalClasses` | `value-semantics.md` mark classes `final` |

These are pure gain — we have no rule for any of them:

- Performance: `preferCountWhere`, `isEmpty`, `preferContains`, `preferFirstWhere`, `preferMinOverSorted`, `preferFlatMap`, `preferLazyMap`
- Swift Testing: `swiftTestingTestCaseNames`, `redundantSwiftTestingSuite`, `testSuiteAccessControl`, `validateTestCases`, `noForceUnwrapInTests`, `noForceTryInTests`
- SwiftUI: `environmentEntry`, `redundantEmptyView`, `redundantViewBuilder`, `redundantSwiftUIGroup`
- Other: `redundantEquatable`, `genericExtensions`, `opaqueGenericParameters`, `ifExpressions`, `conditionalAssignment`

Take the full list from Airbnb's config: https://raw.githubusercontent.com/airbnb/swift/master/Sources/AirbnbSwiftFormatTool/airbnb.swiftformat

## Do NOT enable

Formatting-only rules (`indent`, `wrap*`, `sortImports`, `blankLines*`, `spaceAround*`, `trailingSpace`, `braces`, `semicolons`). Those change whitespace. They belong to the format step, not to review. Findings about whitespace are noise in a diff review.

Also hold back `--property-types inferred` and `noGuardInTests` — both conflict with existing prompt rules. See the conflict card.

## Version gate

Airbnb pins `--swift-version 6.3`. Several rules named above are recent SwiftFormat additions (`preferFinalClasses`, `redundantMemberwiseInit`, `noGuardInTests`, `redundantEquatable`). Record a minimum SwiftFormat version. Make `doctor` report it and offer the install, the same way the existing Swift tool rules handle `swiftlint` and `periphery`.

## Acceptance

- A pass fixture and a fail fixture per rule group, in `builtin/validators/code-hygiene/fixtures/`, following the `*-swift.pass.swift.tmpl` / `*-swift.fail.swift.tmpl` convention.
- A shipped acceptance test that drives REAL `swiftformat`, in `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/`. Model it on `function_length_swift.rs` and `dead_code_swift.rs`.
- `mirdan`'s fixture roster at `crates/mirdan/src/builtin_validators.rs` names each new `.tmpl`. Update it.
- Doctor reports a missing or too-old `swiftformat` and can install it. #tool-validators