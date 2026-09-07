---
assignees:
- claude-code
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
- 01M1Y5GD85944ACE2XS6SA5XWJ
position_column: todo
position_ordinal: fff880
title: Rewrite the Swift project partial to state one formatter
---
## What

`builtin/_partials/project-types/swift.md` gives an author THREE formatting tools and a menu to choose between them: Apple swift-format, SwiftFormat of Nick Lockwood, and SwiftLint, plus a fourth path through the Airbnb SwiftPM plugin. A menu is not a policy. The policy is now the Swift toolchain and its own `swift format`.

Files to change:
- `builtin/_partials/project-types/swift.md` — the section with the heading **Formatting and linting**
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_guidelines_partial.rs`

Delete from that section:
- the three-tool table, and the sentence that opens "Use the tool whose config file the repo already holds"
- the whole Airbnb plugin part, with its `swift package --allow-writing-to-package-directory format` commands and the caution about `--property-types inferred`. The Airbnb plugin is not a toolchain tool: it wraps SwiftFormat and SwiftLint and gives them as binary products.
- the SwiftFormat command with its four options `--pattern-let inline`, `--short-optionals always`, `--single-line-for-each convert` and `--guard-like-if-statements convert`
- the paragraph that opens "Write the project Swift version in `.swift-version`". That file is a SwiftFormat mechanism. Row 8 of the measurement task states if `swift format` reads it. Delete the paragraph if it does not.

Write instead:
- ONE formatter: `swift format`, which the Swift toolchain gives. No install step.
- `swift format -i -r Sources Tests` to write the files. `swift format lint -s -r Sources Tests` to check them. Keep `-s`, because `swift format lint` without it writes warnings and exits 0.
- The tool reads `.swift-format`, a JSON file, from the directory of the file or from a parent. Obey a `.swift-format` the project already holds. Do not write one as a side effect of formatting.
- `swift format dump-configuration` writes the default configuration, which is the way to make a first `.swift-format`.
- The exemption directive an author writes, which row 7 of the measurement task proves. It is `// swift-format-ignore`, and it is NOT `// swiftformat:disable:next`.
- Name the two tools the review gates still need, and say Homebrew installs them: `swiftlint` for the disallowed constructs, the function length and the magic numbers; `periphery` for the dead code. The toolchain has no rule for any of those.

Keep every other section of the partial as it is: ULID, testing, common commands and file locations.

Name a section by its heading text. Do not name a line number, because a line number moves.

## Acceptance Criteria
- [ ] The partial names `swiftformat` nowhere, and names the Airbnb plugin nowhere.
- [ ] The partial gives exactly one formatting tool and no choice between tools.
- [ ] The partial names `swiftlint` and `periphery` only as the tools the review gates need, never as a formatter.
- [ ] The partial names the same exemption directive the `idioms-swift` rule body names.
- [ ] Every command in the partial runs and gives the stated result on the Swift toolchain of the measurement table.

## Tests
- [ ] Update `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_guidelines_partial.rs`.
- [ ] Write a test that holds the partial to naming `swiftformat` nowhere and `--property-types` nowhere.
- [ ] Write a test that holds the partial and the `idioms-swift` gate to naming the SAME tool and the SAME exemption directive. The two files moved together before, and they must move together now.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators swift_guidelines_partial`. Every test passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift