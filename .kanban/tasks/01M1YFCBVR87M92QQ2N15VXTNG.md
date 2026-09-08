---
assignees:
- claude-code
position_column: todo
position_ordinal: fffb80
title: sah doctor lists no row for idioms-swift or disallowed-constructs-swift
---
## What

`sah doctor`, run in a Swift package, writes one `Validator Tool Rule ·` row
for `dead-code-swift`, `function-length-swift`, `magic-numbers-swift` and
`missing-docs-swift`, and NO row at all for `idioms-swift` or
`disallowed-constructs-swift`. The two are not reported as missing, and not
reported as failing. They are absent.

Measured with Apple Swift 6.4, over a probe package that holds one
`Package.swift` and one `Sources/Probe/Probe.swift`:

```
/Users/wballard/github/swissarmyhammer/swissarmyhammer/target/debug/sah doctor \
  | grep -c "Validator Tool Rule"
4
```

**The defect is not new.** The same probe answers 4 with the working tree
stashed back to commit `4fed2803d`, so it predates the move of `idioms-swift`
to `swift format`.

## Why it matters

`sah doctor` is what a person reads to learn if a gate runs. A rule the table
never names reads as a rule that does not exist, and the two rules that go
missing are the two that decide the most Swift bullets.

## What is already known

- Both rules carry a `tool` block, and both match `project_types: [swift]` and
  `files: ["**/*.swift"]`, the same way the four rules that DO appear do.
- The loader parses both: `required_shipped_tool_rule` finds each of them, and
  `every_shipped_idioms_tool_rule_passes_its_fixtures` drives the doctor's own
  `check_tool_rule` path over the `idioms-swift` fixture pair and passes.
- `project_tool_rules` filters only on `rule.tool` being present and on the
  match criteria, and `to_checks` filters nothing, so the drop happens
  somewhere else.
- One hypothesis worth measuring first: the two absent rules are the two
  SLOWEST fixture pairs of the Swift set — 10 s and 15 s in
  `cargo nextest run`, against 4 s to 5 s for each rule that appears. A budget
  or a timeout on one check would split the set exactly this way.

## Acceptance Criteria
- [ ] `sah doctor` in a Swift package writes one row for every shipped Swift
      tool rule, `idioms-swift` and `disallowed-constructs-swift` included.
- [ ] The cause is stated in the task, measured rather than guessed.
- [ ] A test holds the doctor row set to naming every tool rule the loader
      carries for the detected project types.

## Tests
- [ ] Write a test that holds `to_checks` (or the path the CLI really uses) to
      writing one row for each rule `project_tool_rules` answers.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators doctor`. Every test
      passes.
#swift #bug