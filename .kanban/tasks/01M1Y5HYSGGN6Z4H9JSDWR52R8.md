---
assignees:
- claude-code
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
- 01M1Y5GD85944ACE2XS6SA5XWJ
position_column: todo
position_ordinal: fff680
title: Move missing-docs-swift to the toolchain swift format
---
## What

`builtin/validators/code-hygiene/rules/missing-docs-swift.md` runs `swiftlint`. The Swift toolchain decides the same bullet with its own rule `AllPublicDeclarationsHaveDocumentation`. Move this rule to `swift format lint`, and remove one more Homebrew dependency.

Files to change:
- `builtin/validators/code-hygiene/rules/missing-docs-swift.md`
- `builtin/validators/code-hygiene/rules/missing-docs.md` — it states swiftlint-measured facts about `missing-docs-swift` that this task makes false
- `builtin/validators/code-hygiene/fixtures/missing-docs-swift.fail.swift.tmpl`
- `builtin/validators/code-hygiene/fixtures/missing-docs-swift.pass.swift.tmpl`
- the matching test file under `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/`

Measured on the Swift 6.4 toolchain, over a public struct with no documentation:

```
Dirty.swift:3:1: error: [AllPublicDeclarationsHaveDocumentation] add a documentation comment for 'Probe'
```

`AllPublicDeclarationsHaveDocumentation` is OFF in the default configuration. The written configuration must make it ON.

### The configuration alone is not the gate

`swift format lint` always writes layout findings — `[Indentation]`, `[Spacing]`, `[LineLength]` and `[AddLines]`. No key of the configuration makes them OFF. **Measured over the SHIPPED passing fixture of this rule**, with a configuration that makes ON only `AllPublicDeclarationsHaveDocumentation`:

```
Pass.swift:9:1: error: [Indentation] unindent by 2 spaces
Pass.swift:10:1: error: [Indentation] unindent by 2 spaces
```

Thus the script must read each output line, take the `[<RuleName>]` tag, and KEEP only `AllPublicDeclarationsHaveDocumentation`. Use the same filter and the same status handling the task "Move idioms-swift to the toolchain swift format" writes. Do that task first.

`swift format lint --strict` exits 1 when it reports. The run must capture the status and must not let `set -e` end the run.

Do NOT make ON `ValidateDocumentationComments` or `BeginDocumentationCommentWithOneLineSummary`. Those two decide the bullets of `builtin/validators/swift/rules/doc-parameter-naming.md`, which is a different requirement and a different owner.

## Acceptance Criteria
- [ ] The script names `swift format` and names `swiftlint` nowhere.
- [ ] The script keeps only the `AllPublicDeclarationsHaveDocumentation` tag, and drops every other tag.
- [ ] `doctor.check_command` tests the toolchain. The rule needs Homebrew no longer.
- [ ] The passing fixture gives 0 findings through the filter, at exit 0. Layout findings from the pretty-printer do not reach the report.
- [ ] The failing fixture gives a STATED number of findings, at exit 0.
- [ ] `missing-docs.md` states no swiftlint fact about `missing-docs-swift`.
- [ ] The rule body records the measurement, and states which Swift version gives the rule.

## Tests
- [ ] Update the acceptance tests of this rule for the new tool and the new message shape.
- [ ] Write a test that holds the shipped passing fixture to 0 findings through the filter. A script that drops the filter fails it by name, because the fixture draws two `[Indentation]` lines.
- [ ] Write a test that holds the configuration and the allowlist to naming only `AllPublicDeclarationsHaveDocumentation`.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators missing_docs`. Every test passes.
- [ ] Run `sah doctor`. The rule reports as healthy, and its fixture pair passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift