---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m1y6vs4563bkjq2tgd18at8q
  text: |-
    Picked up. Measurement pass is complete. Toolchain: Apple Swift version 6.4 (swiftlang-6.4.0.33.1 clang-2100.3.33.1), Target arm64-apple-macosx27.0.0. `swift format --version` writes `main`, so the Swift version is the only identifier a table can carry.

    Row 1 — eight pretty-printer tags, each measured with all 43 rules OFF: AddLines, EndOfLineComment, Indentation, LineLength, RemoveLine, Spacing, TrailingComma, TrailingWhitespace. One probe file for each tag, so a release that stops writing a tag fails by name. The shipped fixture `missing-docs-swift.pass.swift.tmpl` draws 2 `[Indentation]` findings, because it is written with 4-space indentation and the default is 2.

    Row 2 — a path that holds no file exits 0 and writes NOTHING. That is the dangerous shape. All findings and all errors go to STDERR; stdout is empty. A refusing path beside a dirty file does NOT throw away the dirty file's findings (unlike swiftformat). A directory path exits 64.

    Row 3 — a reporting shape EXISTS: `public static let redColor = Color()` in `public struct Color` reports. The task's probe `= 1` is silent because the inferred type is Int, not Color. The annotated-plus-initializer form `redColor: Color = Color()` is silent.

    Row 4 — internal init reported, public init silent. Measured also that an INTERNAL init inside a `public struct` reports, and that `swift format` reports the default-value form that swiftformat's `redundantMemberwiseInit` misses.

    Row 5 — the conflict reproduces, and `swift format --in-place` WRITES the first DON'T of `immutability.md` character for character. Recorded, not resolved.

    Row 6 — `ReplaceForEachWithForLoop` reports the filter chain and `UseWhereClausesInForLoops` is silent. New fact: `ReplaceForEachWithForLoop` writes NO fix — `--in-place` leaves the file unchanged.

    Row 7 — `// swiftformat:disable:next` is dead against `swift format`. `// swift-format-ignore` works above a declaration, and it also silences the pretty-printer tags. `// swift-format-ignore: <RuleName>` works for a rule but NOT for a pretty-printer tag. A trailing comment on the same line does nothing.

    Row 8 — the floor is Swift 6.0, set by the `swift format` SUBCOMMAND and not by either rule. `swift format` does NOT read `.swift-version`. The CI runner reports the same Swift 6.4.
  timestamp: 2026-09-07T15:12:51.845616+00:00
- actor: claude-code
  id: 01m1y80y2gkz89my7jjc2p2a1d
  text: |-
    Implementation landed. Two notes for the next agent.

    **Row 2 and the four `swiftformat` rows.** The card states "Those rows become false, thus this table replaces them." The card ALSO states that `swiftformat` must still run the gate when this task is done. Those two together mean the four `swiftformat` refusing-path rows are still TRUE for the tool that runs the gate today, and deleting them would leave `the_shipped_swift_idioms_tool_rule_reports_a_file_beside_one_it_declined` holding a behaviour the body no longer explains. So the body KEEPS the four rows and states, in bold, that the new table replaces them the day the gate moves to `swift format` and not before. If the intent was a deletion now, a reviewer must say so.

    **A kanban side effect this session did not ask for.** The task `01M0N5YKDR6M02FG2BRMQP3JZE` — "swift: decide whether redundantVoidReturnType joins the idioms-swift roster" — moved to `.kanban/tasks/.archive/` during this session. No `archive task` call was made from here. The content is preserved at `.kanban/tasks/.archive/01M0N5YKDR6M02FG2BRMQP3JZE.01M1Y6APNEYH48SPPXXFC83AMS.*` and the task shows in `list archived`. The commit step must decide whether to carry that deletion.

    **Pre-existing test failures, unrelated.** `cargo nextest run -p swissarmyhammer-validators` (the whole crate) fails 18 Go tests — `magic_numbers_go`, `stuttering_name_go`, the Go rows of `missing_docs` and `function_length`. This machine has no `go`, `revive`, `staticcheck` or `golangci-lint`. Confirmed pre-existing: the same tests fail with this change stashed.

    ### implement — changed
    - evidence: 2 files — `/Users/wballard/github/swissarmyhammer/swissarmyhammer/builtin/validators/code-hygiene/rules/idioms-swift.md` (+346 lines, appended after line 550; the front matter and the gate script are byte for byte unchanged), `/Users/wballard/github/swissarmyhammer/swissarmyhammer/crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` (+465 lines, two new acceptance tests). `cargo nextest run -p swissarmyhammer-validators swift` — 85 tests run, 85 passed. `cargo fmt --check` clean, `cargo clippy --all-targets` clean.
    - next: `/review`
  timestamp: 2026-09-07T15:33:09.328855+00:00
position_column: doing
position_ordinal: '8380'
title: Measure the Swift toolchain rules the gate needs
---
## What

The move of the Swift gates to the toolchain's `swift format` needs facts that no person has measured yet. Three other tasks wait for different answers from this one. Measure each row, write the probe output behind it, and put the table into the body of `builtin/validators/code-hygiene/rules/idioms-swift.md`. Write no gate script in this task.

Make the probe files in a temporary directory. Use the Swift toolchain of the machine, and write its version at the top of the table.

### Row 1 — the layout leak, and the filter it forces

`swift format lint` reports `[Indentation]`, `[Spacing]`, `[LineLength]` and `[AddLines]`. Those four are NOT members of the 43-rule set, and no key of the configuration makes them OFF. They come from the pretty-printer, which always runs. Measured with a configuration that makes all 43 rules OFF, a file with only bad indentation gave 7 findings.

Thus the configuration is only a second gate. The gate must read each output line, take the `[<RuleName>]` tag, and KEEP only a tag that stands in an allowlist the script states.

Measure and record: the full list of tags the pretty-printer writes that no configuration can stop. Run the shipped fixture `builtin/validators/code-hygiene/fixtures/missing-docs-swift.pass.swift.tmpl` and record which tags it draws.

### Row 2 — the exit statuses

Measure `swift format lint --strict` for each of these, and record the status and the output:
- a file with findings
- a file with no finding
- a path that holds no file
- a file with no read permission
- a file whose bytes are not UTF-8
- a file the parser cannot read

The current rule body holds the same four refusing-path rows for `swiftformat`. Those rows become false, thus this table replaces them.

### Row 3 — `DontRepeatTypeInStaticProperties`

Measured: it reports NOTHING for `public static let redColor = 1` in `public struct Color`, which is the DON'T of `idioms.md` word for word. It was also silent for `colorRed`, for a typed `Color` property, for an enum and for an extension.

Find a shape it DOES report, or record that you found none. This row decides if `idioms.md` keeps the type-name bullet.

### Row 4 — `UseSynthesizedInitializer`

Measured: it reports an `internal` explicit memberwise initializer, and it is silent for a `public` one. `idioms.md` states that bullet no longer, because the swiftformat gate took it. Record both halves. This row decides which half a prompt bullet must take back.

### Row 5 — `UseWhereClausesInForLoops` against `immutability.md`

`builtin/validators/swift/rules/immutability.md` names this shape as a DON'T, and its fix is `map` or `filter`, NOT a `where` clause:

```swift
for user in users { if user.isActive { names.append(user.name) } }
```

Measured, the toolchain rule reports that line and asks for a `where` clause. An author who obeys the finding writes the first DON'T of `immutability.md`, word for word. This is the same class of conflict the repository already refused for `--property-types inferred`.

Measure it again and record it. Then state the two answers a person must choose between:
1. Keep `UseWhereClausesInForLoops` OFF, and let `idioms.md` and `immutability.md` keep the whole question.
2. Rewrite the carve-out of `immutability.md` so the two rules cannot disagree.

**Do not choose. Record the evidence and stop.** A tool and a prompt rule that disagree make churn on every review round.

### Row 6 — which rule reports a filter chain

Measured: `UseWhereClausesInForLoops` is silent for `things.filter { $0 > 2 }.forEach { thing in print(thing) }`, and `ReplaceForEachWithForLoop` reports it and asks for a for-in loop. The `where` bullet of `idioms.md` names that line as a DON'T.

Record which rule reports the line, and the shape its fix lands on. The section of the rule body about `preferForLoop` shows the shape this row must take.

### Row 7 — the directive an author writes

The shipped documents tell an author to write `// swiftformat:disable:next <rule>`. `swift format` does not read that comment. Measure its own directive, `// swift-format-ignore` and `// swift-format-ignore: <RuleName>`, in the line form and in the file form. Record which form works.

### Row 8 — the Swift version

Measure which Swift version first gives `AlwaysUseLiteralForEmptyCollectionInit` and `UseWhereClausesInForLoops`. That version is the new floor. Also measure if `swift format` reads a `.swift-version` file. Read `swift --version` on the self-hosted macOS CI runner and record it, because the gate must not turn CI red.

## Acceptance Criteria
- [x] The body of `idioms-swift.md` holds a table with all eight rows, and the probe output stands behind each row.
- [x] The table names the Swift toolchain version of every measurement.
- [x] Row 1 gives the full list of tags no configuration can stop, thus the allowlist filter can be written.
- [x] Row 5 states both answers and chooses neither.
- [x] Row 8 states the version floor, and states the Swift version of the CI runner.
- [x] No gate script changes in this task. `swiftformat` still runs the gate when this task is done.

## Tests
- [x] Write a test that reads the table out of the rule body and holds each measured tag of row 1 to appearing in a real `swift format lint` run over a badly formatted probe file. A toolchain release that stops writing a tag fails the test by name.
- [x] Write a test that holds row 2 to the measured statuses.
- [x] Run `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes, and the gate still runs swiftformat.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift