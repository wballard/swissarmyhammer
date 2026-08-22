---
assignees:
- claude-code
position_column: todo
position_ordinal: fff080
project: swift-validator
title: 'swift: decide whether redundantVoidReturnType joins the idioms-swift roster'
---
`idioms.md` used to state one bullet holding two requirements: write `Void` rather than `()`, and omit the return clause entirely when it is `Void`. Measured on swiftformat 0.62.1 while working ^qs32yvp, SwiftFormat's `void` rule decides the FIRST and not the second.

    line 2  public static func run() -> ()               -> void reported
    line 3  public static func handler(_ b: (Int) -> ())  -> void reported
    line 4  public static func typed() -> Void {}         -> SILENT

    $ swiftformat --rules void --quiet VoidParen.swift
    public static func run() -> Void {}
    public static func handler(_ body: (Int) -> Void) {}
    public static func typed() -> Void {}

`void` rewrites `()` INTO `-> Void` and stops. SwiftFormat removes the clause under a SEPARATE rule, `redundantVoidReturnType`, which the `idioms-swift` roster does not name.

So ^qs32yvp split the bullet: the `()` half came out of `idioms.md` and `idioms-swift` owns it; the omit-the-clause half stays in `idioms.md` as its own bullet.

## What to decide

Does `redundantVoidReturnType` join the roster?

- Airbnb's `airbnb.swiftformat` was the source of the roster. Read whether it names the rule. The roster is Airbnb's list narrowed to the rules that decide an IDIOM, so a rule Airbnb does not name needs its own argument.
- Probe it. Run `swiftformat --lint --rules redundantVoidReturnType` over `func typed() -> Void {}` and over a closure type `(Int) -> Void`, and read what it reports. A rule that reports a closure's `-> Void` return type would fight the `void` rule, which asks for exactly that spelling — measure before deciding.
- Check it against every other rule of the roster for the same kind of contradiction `--property-types inferred` had with the empty-collection bullet.

If it goes in: the roster count moves from 29, `SWIFT_IDIOMS_ROSTER_SIZE` moves with it, the omit-the-clause bullet comes out of `idioms.md`, and it joins `SWIFT_IDIOMS_SUPERSEDED_BULLETS` in `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` with a probe of its own — which is what makes the deletion safe.

If it stays out: record the measurement in the `idioms-swift` body beside the `void` half section, so the next reader does not ask again.

## Acceptance

- The decision is written in the `idioms-swift` rule body with the probe output behind it.
- If the rule goes in, `the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` carries a row for it, and `idioms.md` no longer states the bullet.

#tool-validators