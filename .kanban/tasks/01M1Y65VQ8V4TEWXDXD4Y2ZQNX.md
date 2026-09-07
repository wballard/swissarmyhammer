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
- actor: claude-code
  id: 01m1y95n39q13nd7a3z62mnp5v
  text: |-
    ### review — findings
    - evidence: 4 findings — crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1183, builtin/validators/code-hygiene/rules/idioms-swift.md:586, builtin/validators/code-hygiene/rules/idioms-swift.md:597, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1167
    - scope: review sha HEAD~1..HEAD (commit 07109ead7). The engine gave 1 finding over 1 file. No validator matches `*.md`, thus the engine read no line of `idioms-swift.md`, which is the deliverable. The driver measured that file and gave 3 more findings.
    - next: add the ninth tag `SpacingCharacter` to the table and to the probe list, correct the two finding-category types, and name `SWIFT_FORMAT_DIRTY_SOURCE` one time.
  timestamp: 2026-09-07T15:53:12.553973+00:00
- actor: claude-code
  id: 01m1y9684s67gpnjfswaqsen12
  text: 'Correction to the section "Driver check — the rule body, which no validator reads": it says "The two items below" and it holds THREE items. The third item is on the test file, not on the rule body.'
  timestamp: 2026-09-07T15:53:32.057382+00:00
- actor: claude-code
  id: 01m1y977a07002ekr5tmrk6gvs
  text: |-
    ### finish iteration 1 — findings
    - implement: changed — 2 files (idioms-swift.md measurement table, idioms_swift.rs tests)
    - test: green — cargo nextest -p swissarmyhammer-validators swift, 85 run, 85 passed; full crate 853 run, 814 passed, 39 failed, all 39 proved pre-existing by a stash and a re-run on a clean tree; cargo fmt --check clean; cargo clippy --workspace --all-targets -D warnings clean
    - commit: 07109ead7 (the subject was amended from "swiftformat" to "swift format", because the two are different programs)
    - review: findings — crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1183, builtin/validators/code-hygiene/rules/idioms-swift.md:586, builtin/validators/code-hygiene/rules/idioms-swift.md:597, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1167
  timestamp: 2026-09-07T15:54:03.968374+00:00
- actor: claude-code
  id: 01m1yaamznym781xhthncd293r
  text: |-
    All four review findings are closed. What the measurements gave, for the next agent.

    **The ninth tag.** `SpacingCharacter` reproduces exactly as the finding states. Probe `struct A {` then `  let x =<TAB>1` then `}`, with all 43 rules OFF: `P1.swift:2:10: error: [SpacingCharacter] use spaces for spacing`, and NO other tag. The TAB stands between the `=` and the value, so the tool reads it as SPACING and not as indentation, which is what separates `SpacingCharacter` from `Spacing`. The row and the probe are both in.

    **Why `strings` failed, and what does work.** The driver was correct that `strings -a <binary> | grep -cx AddLines` writes 0. The cause: six of the nine tags are 15 bytes or shorter, so Swift keeps each of them inside the instruction stream as a small string and not as data. The three that DO appear — `EndOfLineComment`, `SpacingCharacter`, `TrailingWhitespace` — are each 16 bytes or longer. That is the whole pattern.

    The CASE names, in lowerCamelCase, DO stand in the binary, in the `__swift5_fieldmd` reflection section. A 25-line python walk of the field descriptors of that section writes, with Apple Swift 6.4:

        RuleBasedFindingCategory ['ruleType']
        PrettyPrintFindingCategory ['endOfLineComment', 'trailingComma']
        WhitespaceFindingCategory ['trailingWhitespace', 'indentation', 'spacing', 'spacingCharacter', 'removeLine', 'addLines', 'lineLength']

    A tag is the case name with its first letter in upper case. Those three are EVERY type of the binary whose name ends in `FindingCategory` — `strings` finds no fourth. The third one, `RuleBasedFindingCategory`, carries the name of the RULE that reported, so its tags are the 43 rule names and a configuration key stops each. The other two carry the nine tags, and no key reaches them. The script and its output stand in the rule body, under a new `### Where the nine names come from` heading.

    Two routes were tried and dropped. `swift format --help` and `swift format lint --help` name no category listing. `nm -a` on the binary writes 1162 symbols and 0 hits for `FindingCategory`, because the binary is stripped. There is no `swift-reflection-dump` in this toolchain.

    **The third source in the test.** The test now reads the case set out of the Mach-O reflection metadata of the toolchain binary, through a new `swift_reflection` module in the test file. It first asserts the binary carries exactly `PrettyPrintFindingCategory`, `RuleBasedFindingCategory` and `WhitespaceFindingCategory`, so a release that adds, renames or drops a category type fails by name rather than answering a short list. Then it holds the union of the cases of the first and the third against the two hand-written lists.

    Both halves were proved by breaking them on purpose and re-running:
    - With `RULE_CATEGORY` set to `NoSuchFindingCategory`, the test fails and names the three real types the binary carries. The walk reads the live binary.
    - With `SpacingCharacter` deleted from BOTH the body table and the probe list, the test fails on the new assertion and names the ninth tag. That is the exact hole the finding describes, now closed.

    The module stands under `#[cfg(target_os = "macos")]`, because it reads a Mach-O image. Every CI runner of this repository is `self-hosted` macOS, so nothing is lost. On another image format the enumeration would need an ELF reader.

    **The duplicated literals.** `SWIFT_FORMAT_DIRTY_SOURCE` is defined once, now ahead of `SWIFT_FORMAT_TAG_PROBES`, and the `Spacing` probe references it. Scanning the whole file for the same cause found one more duplicate in the new code: `tempfile::tempdir().expect("stage a probe directory")` stood twice. A `swift_format_probe_directory()` helper now holds it once. `"Probe.swift"` also stands twice, but the second is the named constant `SWIFT_FORMAT_STATUS_PROBE_PATH` and the first is a pre-existing probe of another rule, so they are not one literal repeated.

    **The gate is untouched.** The front matter and the `run:` script of `idioms-swift.md` are byte for byte unchanged. `swiftformat` still runs the gate.
  timestamp: 2026-09-07T16:13:24.853536+00:00
- actor: claude-code
  id: 01m1yabm275sgp6hfrcnr0gyd5
  text: |-
    ### implement — changed
    - evidence: 2 files — `/Users/wballard/github/swissarmyhammer/swissarmyhammer/builtin/validators/code-hygiene/rules/idioms-swift.md` (+88/-4; the first changed line is 577, so the front matter and the `run:` gate script are untouched and `swiftformat` still runs the gate), `/Users/wballard/github/swissarmyhammer/swissarmyhammer/crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` (+442/-19). All four review findings flipped to `- [x]`. `cargo nextest run -p swissarmyhammer-validators swift` — 85 run, 85 passed. Whole crate — 853 run, 814 passed, 39 failed, the same 39 the earlier pass proved pre-existing (no `go`, `revive`, `staticcheck`, `golangci-lint` or `dart` on this machine); no Swift test among them. `cargo fmt --all --check` clean. `cargo clippy -p swissarmyhammer-validators --all-targets -D warnings` clean.
    - next: `/review`
  timestamp: 2026-09-07T16:13:56.679269+00:00
- actor: claude-code
  id: 01m1ybbvzadsemvg69z60xwwye
  text: |-
    ### review — findings
    - evidence: 7 findings — round 2 on 002620a0b (`review sha HEAD~1..HEAD`). Engine: 4 confirmed, 5 refuted, on idioms_swift.rs:1333, :1488, :1499, :1509. Driver: 3 on idioms-swift.md:613, idioms_swift.rs:1158, idioms_swift.rs:1489.
    - All four items of round 1 are addressed. The driver ran the printed python block on Apple Swift 6.4 and it wrote the three recorded lines word for word. The test passes: 1 passed. The gate is not changed; swiftformat still runs it.
    - The new finding is the same class as round 1 item 2: the reason the body gives for `strings` writing 0 is wrong. Measured, `strings -a <binary> | grep -cx addLines` writes 1, and every one of the nine case names stands in the binary as data. The 15-byte limit has no part in it; the tag form differs from the case form by its first letter.
    - next: correct the `strings` reason in the rule body and in the doc comment of `mod swift_reflection`, read the exit status of `xcrun`, and answer the four engine items.
  timestamp: 2026-09-07T16:31:33.354491+00:00
- actor: claude-code
  id: 01m1ybd3rjd232xrgxv6ar7n75
  text: |-
    ### finish iteration 2 — findings
    - implement: changed — 2 files; all 4 findings of round 1 closed; the test now reads a third source, the reflection metadata of the toolchain binary
    - test: green — cargo nextest -p swissarmyhammer-validators swift, 85 run, 85 passed; cargo fmt --all --check clean; cargo clippy -p swissarmyhammer-validators --all-targets -D warnings clean. The driver ran these, not the implementer.
    - commit: 002620a0b
    - review: findings — builtin/validators/code-hygiene/rules/idioms-swift.md:613, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1158, :1333, :1488, :1489, :1499, :1509
    - note: review confirmed all 4 findings of round 1 are truly closed, not only checked. It ran the python block from the body, and it broke the new assertion in both directions.
  timestamp: 2026-09-07T16:32:14.098919+00:00
- actor: claude-code
  id: 01m1ycs7084yng0hexbvfp1a7r
  text: |-
    All seven findings of round 2 are closed. What the measurements gave, for the next agent.

    **The `strings` reason, measured a third time.** The finding is correct that the sentence in the body was wrong. The finding's OWN replacement reason is also not correct, and the body does not carry it. Both forms of each of the nine names were measured on the Apple Swift 6.4 binary:

        addLines             1   AddLines             0
        indentation          5   Indentation          0
        lineLength           2   LineLength           0
        removeLine           1   RemoveLine           0
        spacing              1   Spacing              0
        trailingComma        1   TrailingComma        0
        endOfLineComment     1   EndOfLineComment     1
        spacingCharacter     1   SpacingCharacter     1
        trailingWhitespace   1   TrailingWhitespace   1

    Every CASE name stands in the binary, as the finding states. But the TAG form is NOT absent for every name: `EndOfLineComment`, `SpacingCharacter` and `TrailingWhitespace` each stand once. So "the tag form is never a whole string of the file" is refuted by the same command that refuted the earlier sentence. Those three are exactly the three tag names of 16 bytes or longer, and each of them stands in `__cstring`. The nine case names stand in `__swift5_reflstr`. Both section names came from a walk of the load commands beside the `strings -t d` offsets.

    The 15-byte limit is real, but it is a property of the TAG form only, and it is not the reason to read the reflection metadata. It was measured on its own, with a probe that has no swift-format in it:

        func fifteen() -> String { return "AAAAAAAAAAAAAAA" }
        func sixteen() -> String { return "BBBBBBBBBBBBBBBB" }
        print(fifteen(), sixteen())

        strings -a Probe | grep -cx 'AAAAAAAAAAAAAAA'    # 0
        strings -a Probe | grep -cx 'BBBBBBBBBBBBBBBB'   # 1

    The reason the body now gives is the one the finding names as honest, and it is measured: `strings` writes a run of bytes and never writes the type that owns it. `indentation` stands five times, one of them in `__objc_methname`, and no line says which of the five is a case of `WhitespaceFindingCategory`. The rule body and the doc comment of `mod swift_reflection` both carry that reason now, and they agree.

    The body also said the case names stand in `__swift5_fieldmd`. That section holds the field DESCRIPTORS; each descriptor names its own type and points at a name in `__swift5_reflstr`. The body says that now.

    **The `xcrun` status.** Measured again: `xcrun --find swift-format-does-not-exist` exits 72, writes nothing on stdout, and writes `unable to find utility ... not a developer tool or in PATH` on stderr. The module reads the status now. Proved by setting `SWIFT_FORMAT_BINARY` to a name no toolchain carries and running the test: the message is

        `xcrun --find swift-format-does-not-exist` exited exit status: 72 and named no path; it wrote "xcrun: error: ... unable to find utility ..." on stderr

    instead of the old "No such file or directory (os error 2)".

    **The four `rust/error-handling` items.** `mod swift_reflection` no longer panics anywhere. It answers a `thiserror` enum, `ReflectionFailure`, with one arm for each expected failure: the locator could not start, the locator named no path, the file could not be read, the file is not a Mach-O image, a window runs past the end, a load command states zero bytes, a reflection address falls in no section, a string has no NUL, the image has no `__swift5_fieldmd`, and the category types are not the ones measured. Removing the cause from the whole module, and not only the four lines:

    - Every `.expect()` on a byte window is gone. One `window::<N>` helper replaces `four_bytes`, and the two-, four- and eight-byte readers all go through it, so the four near-identical `.try_into().expect(...)` blocks are one function now.
    - Every raw slice index is gone; `image.get(..)` answers `Truncated`.
    - `assert_eq!` on the Mach-O magic answers `NotMachO`.
    - `categories[*category]` is gone: the map is consumed with `into_iter().filter(..)`, so no key lookup can miss.
    - A load command of zero bytes answered an endless loop before; it answers `EmptyLoadCommand` now.

    **A second instance of the same defect, found by scanning the file.** `swift_format_configuration_with_every_rule_off` also threw away the status and the stderr of its command. A failed `swift format dump-configuration` would have landed on `.expect("`swift format dump-configuration` must write JSON")`, which names the wrong cause, exactly as the xcrun item describes. It reads the status now and puts the stderr in the message.

    **The walk still reads the live binary.** Proved by breaking it in both directions and running the test:
    - `RULE_CATEGORY` set to `NoSuchFindingCategory` fails and names the three real types the binary carries.
    - `SpacingCharacter` deleted from BOTH the body table and the probe list fails on the tag-set assertion and names the ninth tag.

    **Two routes measured and not taken.** `grep -cx` on the tag form was kept out of the body as an argument, because its answer measures the length of the name and not what the tool knows. Section membership was measured but not written into the body as a runnable command, because the `cases.py` walk already in the body reads the same descriptors and answers the owner question directly.

    **The gate is untouched.** The only hunk of `idioms-swift.md` starts at line 613; the front matter and the `run:` script are byte for byte unchanged. `swiftformat` still runs the gate.

    **Test state.** `cargo nextest run -p swissarmyhammer-validators swift` — 85 run, 85 passed. Whole crate — 853 run, 815 passed, 38 failed. The 38 were proved pre-existing by a stash and a re-run: the failing set is IDENTICAL before and after this change, and every one of the 38 is a Go or TypeScript tool-rule test that needs `go`, `revive`, `staticcheck` or `golangci-lint`, none of which this machine holds. No Swift test is among them.
  timestamp: 2026-09-07T16:56:19.208677+00:00
- actor: claude-code
  id: 01m1ycsfgwwxa2jwvz1fd2yttd
  text: |-
    ### implement — changed
    - evidence: 2 files — `/Users/wballard/github/swissarmyhammer/swissarmyhammer/builtin/validators/code-hygiene/rules/idioms-swift.md` (+47/-10, one hunk that opens at line 613, so the front matter and the `run:` gate script are untouched and `swiftformat` still runs the gate), `/Users/wballard/github/swissarmyhammer/swissarmyhammer/crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` (+531/-152). All 7 findings of round 2 flipped to `- [x]`. `cargo nextest run -p swissarmyhammer-validators swift` — 85 run, 85 passed. Whole crate — 853 run, 815 passed, 38 failed, the failing set identical to a stashed re-run and every one of them a Go or TypeScript rule this machine has no tool for. `cargo fmt --all --check` clean. `cargo clippy -p swissarmyhammer-validators --all-targets -D warnings` clean.
    - next: `/review`
  timestamp: 2026-09-07T16:56:27.932628+00:00
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

## Review Findings (2026-09-07 10:44)

> Scope: `review sha HEAD~1..HEAD` — reviewed the diffs only — lines this change added or modified. 1 file(s) reviewed, 21 not reviewed.

> 20 file(s) not reviewed — excluded by an ignore rule:
> - `.kanban/ (from .reviewignore)` — 20 file(s)

> 1 file(s) not reviewed — no validator matched:
> - `builtin/validators/code-hygiene/rules/idioms-swift.md` — no validator matches this file

- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1183` `code-hygiene/magic-numbers` — The string literal `"let alpha = 1+2\n"` is repeated at line 1269 as the value of `SWIFT_FORMAT_DIRTY_SOURCE`. This literal should be named once, so changes are made in one place. Move the definition of `SWIFT_FORMAT_DIRTY_SOURCE` before the `SWIFT_FORMAT_TAG_PROBES` constant array, then reference it at line 1183 instead of the inline literal.

### Driver check — the rule body, which no validator reads

No validator matches `*.md`, thus the engine read no line of `idioms-swift.md`. That file is the deliverable of this task. The three items below come from measurement by the driver. The command and the output stand behind each one. The third of them is on the test file, not on the rule body.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:586` `driver/measured` — The table of tags no configuration can stop names EIGHT tags, and the toolchain writes NINE. `SpacingCharacter` is absent. Measured with all 43 rules OFF, over a file that holds `struct A {` then `  let x =<TAB>1` then `}`: the command `swift format lint --strict --configuration all-off.json P1.swift` wrote `P1.swift:2:10: error: [SpacingCharacter] use spaces for spacing`. The acceptance criterion asks for the FULL list, because the allowlist filter of the future gate is built from this table. A gate built on the eight tags would not know the ninth. Add a `SpacingCharacter` row with its probe and its output, and add the same probe to `SWIFT_FORMAT_TAG_PROBES`.
- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:597` `driver/measured` — The sentence "`PrettyPrintFindingCategory` stands in the binary, and the reflection data beside it lists the same eight cases the eight probes draw" is not correct, and it is the only support the body gives for the list being complete. `PrettyPrintFindingCategory` of `release/6.0` holds TWO cases: `EndOfLineComment` and `TrailingComma`. The other seven tags come from `WhitespaceFindingCategory`, whose seven cases are `TrailingWhitespace`, `Indentation`, `Spacing`, `SpacingCharacter`, `RemoveLine`, `AddLines` and `LineLength`. The command `strings -a <the swift-format binary> | grep -cx <tag>` writes 0 for `AddLines`, `Indentation`, `LineLength`, `RemoveLine`, `Spacing` and `TrailingComma`, thus the reflection data lists no such cases. Name both types, name the whitespace linter beside the pretty-printer at the head of the section, and give the command that enumerates the cases.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1167` `driver/measured` — The test `the_shipped_swift_idioms_rule_body_names_every_tag_swift_format_writes` holds the tag column of the body and `SWIFT_FORMAT_TAG_PROBES` to agreeing with each other. A person wrote both lists, and neither list is read from the tool. Thus a tag the toolchain writes and NEITHER list names fails no test, which is how `SpacingCharacter` passed. Hold the two lists to the case set of `PrettyPrintFindingCategory` and `WhitespaceFindingCategory`, or state in the test why the tool cannot give that set.

## Review Findings (2026-09-07 11:15)

> Scope: `review sha HEAD~1..HEAD` — reviewed the diffs only — lines this change added or modified. 1 file(s) reviewed, 3 not reviewed.

> 2 file(s) not reviewed — excluded by an ignore rule:
> - `.kanban/ (from .reviewignore)` — 2 file(s)

> 1 file(s) not reviewed — no validator matched:
> - `builtin/validators/code-hygiene/rules/idioms-swift.md` — no validator matches this file

- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1333` `rust/error-handling` — Panics on expected failure: .expect() on address lookup will panic if a reflection address does not fall within any section, but malformed binary parsing is an expected failure mode. Return Result from file_offset() and handle missing address as an error instead of panicking.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1488` `rust/error-handling` — Panics on expected failure: External command execution (xcrun) can fail for expected reasons like command not found or permission denied, but this code calls .expect() instead of handling the error properly. Use the ? operator with .context(...) to propagate the error with context instead of panicking.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1499` `rust/error-handling` — Panics on expected failure: assert_eq! comparing type names will panic if the toolchain binary does not carry the expected finding-category types, but toolchain version mismatch or format changes are expected failure modes. Return Result from tags_no_configuration_stops() and handle type name mismatches as an error instead of panicking.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1509` `rust/error-handling` — Panics on expected failure: Direct indexing into categories BTreeMap with categories[*category] will panic if the key is not found, but missing keys are expected failure modes that should be handled explicitly rather than left to panic. Use categories.get(*category)?.iter() or similar, or explicitly return Result instead of panicking on missing keys.

### Driver check — the rule body, which no validator reads

No validator matches `*.md`, thus the engine read no line of `idioms-swift.md`. The two items below come from measurement by the driver. The command and the output stand behind each one.

The four items of the round above are all addressed. The driver measured each one:

1. `SWIFT_FORMAT_DIRTY_SOURCE` now stands ahead of `SWIFT_FORMAT_TAG_PROBES`, and the probe list references the name. `grep -n 'alpha = 1+2'` finds the definition alone, and the byte literal `SWIFT_FORMAT_UNDECODABLE_SOURCE`, which the commit did not touch.
2. The `SpacingCharacter` row stands in the table, and the same probe stands in `SWIFT_FORMAT_TAG_PROBES`.
3. Both types are named, the whitespace linter is named beside the pretty-printer, and the command runs. The driver made `cases.py` from the printed block, with no edit but the removal of the 4-space indent of the markdown, and ran it on Apple Swift 6.4. It wrote the three lines the body records, word for word.
4. The test reads the third source. `cargo nextest run -p swissarmyhammer-validators the_shipped_swift_idioms_rule_body_names_every_tag_swift_format_writes` gives 1 passed. The assertion can fail in both directions: a tag the two lists miss makes `listed` shorter than `owned`, and a release that adds, renames or drops a finding-category type fails the guard at line 1499 first.

The gate is not changed. Both hunks of `idioms-swift.md` start at line 574, and the gate script stands at lines 27 to 31. `swiftformat` still runs the gate.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:613` `driver/measured` — The sentence "Six of the nine names are 15 bytes or shorter, so Swift keeps each of them inside the instruction stream and not as data" is not correct, and it is the reason the body gives for `strings` writing 0. Measured on the Apple Swift 6.4 binary, `strings -a "$(xcrun --find swift-format)" | grep -cx <case>` writes a count of 1 or more for ALL NINE case names: `addLines` 1, `indentation` 5, `lineLength` 2, `removeLine` 1, `spacing` 1, `trailingComma` 1, `endOfLineComment` 1, `spacingCharacter` 1, `trailingWhitespace` 1. Every case name stands in the binary as data, and the 15-byte limit has no part in it. `grep -cx AddLines` writes 0 for one reason only: the binary holds the CASE name `addLines`, and the tool makes the `[AddLines]` tag at output time, so the tag form is never a whole string of the file. Correct the reason. The honest reason to read the reflection metadata is that `strings` cannot say WHICH TYPE owns a name, thus it cannot say which names are the finding categories.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1158` `driver/measured` — The doc comment of `mod swift_reflection` states the same wrong reason: "six of the nine names are 15 bytes or shorter, so Swift keeps them inside the instruction stream rather than as data, and `strings -a <binary> | grep -cx AddLines` writes 0". The measurement above refutes the first half. Correct this comment together with the rule body, so the two agree.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:1489` `driver/measured` — `tags_no_configuration_stops` never reads the exit status of `xcrun`, and it discards the stderr of `xcrun`. Measured: `xcrun --find swift-format-does-not-exist` exits 72, writes nothing to stdout, and writes `xcrun: error: unable to find utility "swift-format-does-not-exist", not a developer tool or in PATH` to stderr. Thus `.output()` gives Ok, the `.expect` of line 1488 does not fire, `path` becomes the empty string, and the panic lands at line 1490 as "read the toolchain's swift-format binary: No such file or directory (os error 2)". That message names the wrong cause, and the message that names the correct cause was thrown away. A machine with no Swift toolchain, which is the different toolchain this module must answer for, gets that message. Read `located.status`, and put the stderr of `xcrun` into the panic message.