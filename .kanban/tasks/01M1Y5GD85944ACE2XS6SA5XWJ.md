---
assignees:
- claude-code
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
position_column: todo
position_ordinal: fff580
title: Move idioms-swift to the toolchain swift format
---
## What

`builtin/validators/code-hygiene/rules/idioms-swift.md` runs `swiftformat`, the tool of Nick Lockwood. Only Homebrew installs that tool. The new policy is the Swift toolchain and its own `swift format`. Change this rule to run `swift format lint`.

Do the task "Measure the Swift toolchain rules the gate needs" first. This task writes the script that its table describes. Do not measure again here; read the table.

Files to change:
- `builtin/validators/code-hygiene/rules/idioms-swift.md` — the frontmatter script, the doctor block and the body
- `builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl`
- `builtin/validators/code-hygiene/fixtures/idioms-swift.pass.swift.tmpl`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped.rs` — the `.swift-version` machinery `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT`, which was built for the five version-gated swiftformat rules
- `crates/swissarmyhammer-validators/src/builtin/mod.rs`

### The script reads the output, and the configuration alone is not the gate

`swift format lint` always writes layout findings — `[Indentation]`, `[Spacing]`, `[LineLength]` and `[AddLines]`. Those tags are NOT members of the 43-rule set, and NO key of the configuration makes them OFF. The pretty-printer always runs. Measured: with all 43 rules OFF, a file with only bad indentation gave 7 findings.

Thus the script must do BOTH of these:
- write a temporary `.swift-format` configuration that makes ON only the rules of the allowlist, and
- read each output line, take the `[<RuleName>]` tag, and KEEP only a line whose tag stands in the allowlist the script states.

Review is read-only, thus a layout finding must never reach the report.

### The script must swallow the status

Measured: `swift format lint --strict` exits 1 when it reports, and exits 0 when it does not. The script opens with `set -e`, thus the first file that reports would end the whole run. `builtin/validators/README.md` refuses that answer: "A nonzero exit fails the WHOLE run, so one unjudged path throws away every finding the run did make."

The run must capture the status with `|| status=$?`, must read 1 as "findings, continue", and must read a status above 1 as a file that declined. A declined file writes one line that opens with `sah-diagnostic:` and names the path and the reason. Keep the one-run-for-each-file shape, thus a refusing path costs only its own file.

### The allowlist

Read the measurement table for each row. These rules are candidates:

| the rule | what it decides |
|---|---|
| `AlwaysUseLiteralForEmptyCollectionInit` | the empty-collection bullet of `idioms.md`, in the correct direction |
| `NoVoidReturnOnFunctionSignature` | the `Void` return clause, BOTH halves |
| `ReplaceForEachWithForLoop` | the `forEach` half of the loop bullet |
| `UseLetInEveryBoundCaseVariable` | the pattern-let bullet |
| `UseShorthandTypeNames` | the shorthand type sugar bullet |

`AlwaysUseLiteralForEmptyCollectionInit` is OFF in the default configuration. The written configuration must make it ON.

**`DontRepeatTypeInStaticProperties` is NOT in the table.** Measured, it reports nothing for the shape `idioms.md` names. Row 3 of the measurement task holds the evidence. That bullet stays in the prompt.

**`UseSynthesizedInitializer` decides only the internal half.** Measured, it is silent for a `public` memberwise initializer. Row 4 holds the evidence.

**`UseWhereClausesInForLoops` waits for a decision.** It contradicts `immutability.md`. Row 5 of the measurement task states the two answers and chooses neither. Do NOT make this rule ON until a person chooses.

Do NOT make ON `NeverForceUnwrap`, `NeverUseForceTry` or `NeverUseImplicitlyUnwrappedOptionals`. `disallowed-constructs-swift` owns those three bullets, and swiftlint keeps the test-target split and the `ignored_literal_argument_functions` option that `swift format` does not have. One requirement takes one owner.

### The directive an author writes

The section "The directive an author writes" tells an author to write `// swiftformat:disable:next <rule>`. `swift format` does not read that comment. Rewrite the section for the directive row 7 of the measurement task proves, which is `// swift-format-ignore` or `// swift-format-ignore: <RuleName>`.

### What the gate loses

The gate loses the 22 rules the body lists — the performance group of 7, the Swift Testing group of 6, the SwiftUI group of 4, and 5 others — PLUS `preferFinalClasses` and `noGuardInTests`, which each own a prompt bullet. The task "Rebalance the Swift prompt rules for the toolchain gate" gives those bullets an owner. Do not delete a bullet here.

Remove the `--rules` intersection with `swiftformat --rules`, the `--min-version` test, and the four command-line options. The toolchain gives a fixed set of 43 rules, and each rule is only ON or OFF.

## Acceptance Criteria
- [ ] The script names `swift format` and names `swiftformat` nowhere.
- [ ] The script holds an allowlist of rule tags, and it drops every output line whose tag is not in the allowlist.
- [ ] A Swift file that holds only layout defects gives 0 findings THROUGH the filter.
- [ ] A file that reports does not end the run. Measured over two files, where the first reports and the second reports, the run gives the findings of both at exit 0.
- [ ] `doctor.check_command` tests the toolchain. `doctor.fix_hint` names the toolchain, because Homebrew does not install it.
- [ ] The failing fixture gives a STATED number of findings that carry a STATED number of rules, in the shape the body already uses: "42 findings, exit 0, carrying 21 of the 27 enabled rules".
- [ ] The passing fixture gives 0 findings and exit 0.
- [ ] A path that holds no file costs only its own file, and writes one `sah-diagnostic:` line.
- [ ] The section about the exemption directive names `// swift-format-ignore` and names `swiftformat:disable` nowhere.
- [ ] The rule body holds no row that measures swiftformat.

## Tests
- [ ] Update every test in `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`. Delete the tests that hold the swiftformat roster and the 0.62.1 floor.
- [ ] Write a test that holds a badly formatted probe file to 0 findings through the filter. A script that drops the filter fails it by name.
- [ ] Write a test that holds the allowlist to naming only rules that stand in `swift format dump-configuration`.
- [ ] Write a test that holds two reporting files in one run to giving the findings of both.
- [ ] Delete or rewrite `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT` in `tests/shipped.rs`, which exist only for the swiftformat version gate.
- [ ] Update the bullet-ownership tests in `crates/swissarmyhammer-validators/src/builtin/mod.rs`.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes.
- [ ] Run `sah doctor`. The rule reports as healthy, and its fixture pair passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift