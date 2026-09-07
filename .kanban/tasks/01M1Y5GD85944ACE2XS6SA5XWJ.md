---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m1ydxkqe4nhzfynnk044mhss
  text: |-
    Research done. Measurements made on this machine, Apple Swift 6.4.

    New facts the card's table did not hold, each measured:

    1. A `--configuration` whose `rules` table names only some rules turns EVERY
       unnamed rule OFF. `AlwaysUseLowerCamelCase` is ON by default and reports
       `let Bad_One = 1`; under a `rules` table naming only
       `AlwaysUseLiteralForEmptyCollectionInit` it is silent. So one small JSON
       file states the whole rule set, and the other configuration keys
       (`lineLength`, `indentation`) keep their defaults.
    2. The nine unstoppable tags still come through that configuration:
       `[Indentation]` and `[LineLength]` each reported under it. The tag filter
       stays the gate.
    3. A DIRECTORY in a shared work list costs the WHOLE run: `swift format lint
       --strict Dirty.swift Adir.swift` exits 64, writes ZERO findings, and
       `Dirty.swift` is judged in none of it. That is the measured reason to keep
       one run for each file, and it is stronger than the reason the body records.
    4. The run's `--configuration` wins over a project `.swift-format`: a project
       file holding `"AlwaysUseLiteralForEmptyCollectionInit": false` did not stop
       the finding.
    5. `AlwaysUseLiteralForEmptyCollectionInit` reports `[Int]()` and
       `[String: Int]()`. It stays SILENT for `Set<String>()`.
    6. `NoVoidReturnOnFunctionSignature` reports `-> ()` AND `-> Void`, and stays
       silent for a closure parameter `(Int) -> ()`. That last shape belongs to
       `ReturnVoidInsteadOfEmptyTuple`, which the allowlist does not name.
    7. `printf '' | swift format lint --strict -` exits 0, so it is a doctor probe.
       `swift format --version` writes `main` and names no release, so the floor is
       read off the SUBCOMMAND standing at all.

    Blast radius the card did not list, found by reading the callers:

    - `tests/shipped/swift_guidelines_partial.rs` reads the shipped `idioms-swift`
      script for `swiftformat --lint` and five options. Those options go away with
      this move, so that half of
      `swift_partial_agrees_with_the_shipped_swift_tool_validators` loses its
      source and the test would fail. Task ^ywgq61n owns the partial rewrite; this
      task makes the minimum edit that keeps the test honest and green.
    - `tests/shipped/swift_judgment_rules.rs` uses `SWIFT_VERSION_SUPPORT`, which
      this task deletes.
    - `idioms.md` still states the empty-collection bullet and the omit-the-clause
      bullet, and the gate now decides both. That two-owner state is temporary and
      task ^2hd90th owns it, so no prompt bullet moves here.
    - `preferFinalClasses` has no successor rule in the toolchain, so the
      `value-semantics.md` `final` bullet has no owner until ^2hd90th returns it.
      Its row leaves the superseded-bullet table here.
  timestamp: 2026-09-07T17:16:11.886218+00:00
- actor: claude-code
  id: 01m1yfx1047wrp2taeh1c17ge8
  text: |-
    Implementation landed. What the gate is now:

    The script writes ONE list of seven rule tags, and that list does two jobs: a
    `sed` turns it into the `{"rules": {...}}` configuration the run passes with
    `--configuration`, and a `paste` turns it into the alternation the output
    filter reads. One list, so the two cannot drift.

    Per file: `[ ! -e "$file" ]` first, then
    `swift format lint --strict --configuration ... "$file" > /dev/null 2>err`
    with `|| status=$?`. A stderr line holding `: error: [` is a tagged finding;
    every other line is trouble, and a file that wrote one is declined whole with
    a `sah-diagnostic:` line. The findings pass through
    `grep -E ": error: \[($allowed)\] "` and a `sed` that writes
    `path:line: RuleName: reason`.

    TDD evidence, both directions measured:

    - RED, whole set: with the rule file and the two fixtures stashed back to the
      swiftformat version, `cargo nextest run -p swissarmyhammer-validators
      idioms_swift` answered 6 passed, 9 FAILED.
    - RED, the filter alone: with the allowlist alternation replaced by
      `[A-Za-z]*` in the shipped script, the same run answered 11 passed, 4
      FAILED, and
      `the_shipped_swift_idioms_tool_rule_drops_every_tag_outside_its_allowlist`
      failed with its own message — "the `[AddLines]` probe holds a layout defect
      and no idiom ... the run reported [\"AddLines\", \"AddLines\"]".
    - GREEN: 15 of 15 pass, and `cargo nextest run -p swissarmyhammer-validators
      swift` answers 86 of 86.

    Measured numbers now written into the body and held by a test:

    - The failing fixture: 11 findings, exit 0, carrying all 7 allowlist rules.
    - The passing fixture: 0 findings through the filter, while the raw
      `swift format lint --strict` over the same bytes writes 34 `[Indentation]`
      lines. The doctor's own fixture pair therefore measures the filter.
    - Each of the nine unstoppable tags: 0 findings through the shipped script.

    Two things this task did NOT do, both by the card's own order:

    - No prompt bullet moved. `idioms.md` still states the empty-collection bullet
      and the omit-the-clause half of the `Void` bullet, and the gate now decides
      both. The body names that two-owner state and names the task that ends it.
    - `UseWhereClausesInForLoops` stays OFF, and
      `the_shipped_swift_idioms_tool_rule_leaves_the_where_clause_rule_off` holds
      the shipped script to naming it NOWHERE — a configuration key alone would
      turn it on with no allowlist entry.

    `preferFinalClasses` left the superseded-bullet table, because the toolchain
    carries no rule like it. The `value-semantics.md` `final` bullet therefore has
    no owner until task ^2hd90th states it again. The body says so in as many
    words.
  timestamp: 2026-09-07T17:50:49.860398+00:00
- actor: claude-code
  id: 01m1yfxe5z161nce1fya740dmd
  text: |-
    One test item of the card is answered in part, and the gap is pre-existing.

    "Run `sah doctor`. The rule reports as healthy, and its fixture pair passes."

    The fixture pair passes. `every_shipped_idioms_tool_rule_passes_its_fixtures`
    drives the doctor's OWN `check_tool_rule` path over the shipped pair, and it
    passes.

    `sah doctor` writes no ROW for this rule. Measured over a probe Swift package,
    the table names `dead-code-swift`, `function-length-swift`,
    `magic-numbers-swift` and `missing-docs-swift`, and neither `idioms-swift` nor
    `disallowed-constructs-swift`. The same probe answers the same 4 rows with the
    working tree stashed back to commit `4fed2803d`, so the defect predates this
    change and is not its work. Task ^15vxtng records it, with what is already
    known and the hypothesis to measure first.

    Two more notes for the tasks that follow.

    - ^ywgq61n owns the partial. This task made ONE minimum edit to
      `swift_guidelines_partial.rs` to keep it honest: the half of
      `swift_partial_agrees_with_the_shipped_swift_tool_validators` that read five
      swiftformat options off the shipped script had no source left, so it now
      reads the LINT COMMAND HEAD off the shipped script and holds the partial to
      naming it. The partial already writes `swift format lint`, so no partial
      edit was needed. The empty-collection half is untouched.
    - ^37yt9xy owns the document sweep.
      `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs` still
      carries a doc comment saying this script intersects its roster with
      `swiftformat --rules`. That sentence is now false, and the card of ^37yt9xy
      lists that file, so it was left there.
  timestamp: 2026-09-07T17:51:03.359919+00:00
- actor: claude-code
  id: 01m1yfxq7etkx42z8m4bs6kvp0
  text: |-
    ### implement — changed
    - evidence: 8 files — builtin/validators/code-hygiene/rules/idioms-swift.md, builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl, builtin/validators/code-hygiene/fixtures/idioms-swift.pass.swift.tmpl, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_judgment_rules.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_guidelines_partial.rs, crates/swissarmyhammer-validators/src/builtin/mod.rs. `cargo nextest run -p swissarmyhammer-validators swift` — 86 run, 86 passed. `cargo nextest run -p swissarmyhammer-validators` — 854 run, 816 passed, 38 failed; the same 38 names fail at HEAD with the change stashed, so every one is pre-existing (missing go, revive, staticcheck, golangci-lint and eslint-plugin-sonarjs). `cargo clippy -p swissarmyhammer-validators --tests` — 0 warnings. `cargo fmt --check` — clean.
    - next: /review
  timestamp: 2026-09-07T17:51:12.622142+00:00
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
position_column: doing
position_ordinal: '8380'
title: Move idioms-swift to the toolchain swift format
---
## What

`builtin/validators/code-hygiene/rules/idioms-swift.md` runs `swiftformat`, the tool of Nick Lockwood. Only Homebrew installs that tool. The new policy is the Swift toolchain and its own `swift format`. Change this rule to run `swift format lint`.

The task "Measure the Swift toolchain rules the gate needs" is done. Its measurement table is in the body of the same file. **Read that table first, and build the script from it. Do not measure again.**

Files to change:
- `builtin/validators/code-hygiene/rules/idioms-swift.md` — the frontmatter script, the doctor block and the body
- `builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl`
- `builtin/validators/code-hygiene/fixtures/idioms-swift.pass.swift.tmpl`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped.rs` — the `.swift-version` machinery `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT`, which exists only for the swiftformat version gate
- `crates/swissarmyhammer-validators/src/builtin/mod.rs`

### What the measurement decided, and the script must obey

| the fact | what the script must do |
|---|---|
| `swift format lint` writes NINE tags that no configuration can stop | Read the tag off each output line. KEEP only a tag of the gate allowlist. The configuration alone is not the gate. |
| Every finding and every error goes to **stderr**. stdout is always 0 bytes. | Read stderr, not stdout. |
| A path that holds no file exits **0** and writes nothing | The script must test the path itself, with `[ ! -e "$file" ]`, and write its own `sah-diagnostic:` line. The tool gives no error for it. |
| A file with findings exits **1**. A clean file exits 0. A directory exits 64. | Capture the status with `|| status=$?`. Read 1 as "findings, continue". `set -e` must not end the run. |
| The version floor is **Swift 6.0**, from the `swift format` subcommand | State that floor. Do not state 0.62.1, which measures swiftformat. |
| `swift format` reads **no** `.swift-version` file | Delete the section about `.swift-version` and the five version-gated rules. |
| The exemption directive is `// swift-format-ignore` | Rewrite the section "The directive an author writes". The bare form silences the pretty-printer tags as well; the named form does not. |

Keep the one-run-for-each-file shape, thus a refusing path costs only its own file.

### The allowlist

| the rule | what it decides |
|---|---|
| `AlwaysUseLiteralForEmptyCollectionInit` | the empty-collection bullet of `idioms.md`, in the correct direction. OFF by default; the configuration must make it ON |
| `NoVoidReturnOnFunctionSignature` | the `Void` return clause, BOTH halves |
| `ReplaceForEachWithForLoop` | the `forEach` half of the loop bullet |
| `UseLetInEveryBoundCaseVariable` | the pattern-let bullet |
| `UseShorthandTypeNames` | the shorthand type sugar bullet |
| `DontRepeatTypeInStaticProperties` | the type-name bullet. **Row 3 of the table proves it DOES report**, for `public static let redColor = Color()`. The member's own type must be the enclosing type. It is silent for `= 1`, which infers `Int`. Read row 3, and state in the body which shapes it misses |
| `UseSynthesizedInitializer` | the redundant memberwise initializer, the **internal** half only. Row 4 proves it is silent for a `public` initializer, and that answer is correct, because Swift synthesizes an internal initializer and never a public one |

**`UseWhereClausesInForLoops` stays OFF.** Row 5 of the table records that it contradicts `immutability.md`, and it states two answers and chooses neither. A person must choose. Do not make this rule ON in this task. The task "Rebalance the Swift prompt rules for the toolchain gate" carries that decision.

Do NOT make ON `NeverForceUnwrap`, `NeverUseForceTry` or `NeverUseImplicitlyUnwrappedOptionals`. `disallowed-constructs-swift` owns those three bullets, and swiftlint keeps the test-target split and the `ignored_literal_argument_functions` option that `swift format` does not have. One requirement takes one owner.

### What the gate loses

The gate loses the 22 rules the body lists — the performance group of 7, the Swift Testing group of 6, the SwiftUI group of 4, and 5 others — PLUS `preferFinalClasses` and `noGuardInTests`, which each own a prompt bullet. The task "Rebalance the Swift prompt rules for the toolchain gate" gives those bullets an owner. Do not delete a bullet here.

Remove the `--rules` intersection with `swiftformat --rules`, the `--min-version` test and the four command-line options. The toolchain gives a fixed set of 43 rules, and each rule is only ON or OFF.

## Acceptance Criteria
- [ ] The script names `swift format` and names `swiftformat` nowhere.
- [ ] The script holds an allowlist of rule tags, and it drops every output line whose tag is not in the allowlist.
- [ ] A Swift file that holds only layout defects gives 0 findings THROUGH the filter.
- [ ] A file that reports does not end the run. Over two files that both report, the run gives the findings of both, at exit 0.
- [ ] A path that holds no file writes one `sah-diagnostic:` line and costs only its own file.
- [ ] `doctor.check_command` tests the toolchain and the Swift 6.0 floor. `doctor.fix_hint` names the toolchain, because Homebrew does not install it.
- [ ] The failing fixture gives a STATED number of findings that carry a STATED number of rules.
- [ ] The passing fixture gives 0 findings and exit 0.
- [ ] The section about the exemption directive names `// swift-format-ignore` and names `swiftformat:disable` nowhere.
- [ ] The body holds no row that measures swiftformat, and nothing about `.swift-version`.
- [ ] `UseWhereClausesInForLoops` is OFF.

## Tests
- [ ] Update every test in `tests/shipped/idioms_swift.rs`. Delete the tests that hold the swiftformat roster and the 0.62.1 floor. Keep the measurement tests the previous task wrote.
- [ ] Write a test that holds a badly formatted probe file to 0 findings through the filter. A script that drops the filter fails it by name.
- [ ] Write a test that holds the allowlist to naming only rules that stand in `swift format dump-configuration`.
- [ ] Write a test that holds two reporting files in one run to giving the findings of both.
- [ ] Write a test that holds `UseWhereClausesInForLoops` to being OFF, and name row 5 as the reason.
- [ ] Delete or rewrite `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT` in `tests/shipped.rs`.
- [ ] Update the bullet-ownership tests in `crates/swissarmyhammer-validators/src/builtin/mod.rs`.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes.
- [ ] Run `sah doctor`. The rule reports as healthy, and its fixture pair passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift