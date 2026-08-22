---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0mxtk5psdvz8h537w4m2ddq
  text: |-
    ### blocker discovered while building ^3fx5bny — read before you start

    The `supersedes` key does NOT have the granularity this card assumes.

    `supersedes` names a WHOLE prompt rule, and the engine then skips that rule entirely. But the new `idioms-swift` tool gate covers **six bullets spread across two rule files** (`swift/rules/idioms.md` and `swift/rules/value-semantics.md`). Naming either file in `supersedes` would drop that file's OTHER bullets from every review — bullets no tool covers.

    So ^3fx5bny shipped with `supersedes` empty, deliberately. The prompt half is this card's job, and it cannot be done by adding a `supersedes` key.

    Two ways forward. Decide which, and say why:

    1. **Delete the superseded bullets from the prompt rule files by hand.** The rule file keeps its remaining bullets; the tool owns the deleted ones. Simple, and it works today. The cost is that the mapping lives only in a commit message — nothing enforces that the tool still covers what the prompt stopped saying.

    2. **Give `supersedes` bullet-level granularity.** A larger change to the validator engine, and out of scope for this card. If you pick this, make it its own card and block this one on it.

    Recommendation: option 1 for this card, plus a coverage-guard test that asserts each deleted bullet has a named tool-rule test covering it. That keeps the mapping honest without an engine change. The acceptance criterion already on this card — "Each removed bullet has a passing tool-rule test that covers the same defect. Name the test in the commit message." — becomes that test.

    Also note: two rules in the ^3fx5bny roster, `preferLazyMap` and `ifExpressions`, exist only on SwiftFormat `main`, not in any release (Airbnb tracks `main`). The tool rule enables the intersection with what the installed swiftformat actually knows. So the prompt bullets those two would supersede are NOT yet covered by a tool. Do not delete them.
  timestamp: 2026-08-22T14:26:04.086567+00:00
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
- 01M0MVM2VZ71SBQ95754S2N0RX
- 01M0MVNQDK2G1J4X2SRT78SQR4
position_column: todo
position_ordinal: ffea80
project: swift-validator
title: 'swift: supersede the prompt-rule bullets the new tool rules now decide'
---
Once the swiftformat and swiftlint tool rules exist, twelve bullets in `builtin/validators/swift/rules/` are decided by a tool. Remove them from the prompt. This is the `objectivity-over-judgment` standing order: a tool rule supersedes its prompt equivalent.

Do NOT start this card until both tool cards are done and green. Removing prompt text before the tool exists loses coverage.

## Bullets to remove, and what replaces each

`idioms.md`
- shorthand type sugar (`[Int]`, `String?`) → SwiftFormat `typeSugar`
- return `Void`, not `()` → SwiftFormat `void`
- no memberwise init identical to the synthesized one → SwiftFormat `redundantMemberwiseInit`
- `for` loop over `forEach` → SwiftFormat `preferForLoop`
- bind each case variable with its own `let` → SwiftFormat `hoistPatternLet`

`value-semantics.md`
- mark classes `final` → SwiftFormat `preferFinalClasses`

`optionals.md`
- no implicitly unwrapped optionals → SwiftLint `implicitly_unwrapped_optional`
- no force unwrap in non-test code → SwiftLint `force_unwrapping`

`error-handling.md`
- no `try!` in non-test code → SwiftLint `force_try`
- no `as!` in non-test code → SwiftLint `force_cast`

`concurrency.md`
- `@unchecked Sendable` needs a documented invariant → SwiftLint custom rule `no_unchecked_sendable` plus its annotation escape hatch

`casing.md`
- no `SCREAMING_SNAKE_CASE`, no `k`-prefix, no Hungarian notation → SwiftLint `identifier_name` or a new custom regex rule

## Why casing is the important one

`casing.md` is our longest Swift rule. Most of its words are anti-churn guards — "never flag one toward the other", "is a validator error", "renaming between LoRA-style and LORA-style across review rounds is always a validator error". We pay that prose because an LLM decides the rule and will otherwise generate churn.

Airbnb pays none of that prose on any rule a tool decides. Their whole acronym rule is one sentence. The length of our rule is the price of judgment. Measure the token saving and record it — it is evidence for the `implement-rules-preload` prompt-cap work.

Keep the acronym-flexibility bullet. That one is a deliberate policy choice and no tool encodes it.

## Blast radius

- `crates/swissarmyhammer-validators/src/builtin/mod.rs:733` — the test `test_swift_casing_accepts_both_acronym_spellings` asserts the `casing` rule body contains "BOTH accepted". If you touch that bullet, that test moves with it.
- `builtin/validators/swift/VALIDATOR.md` says "Each rule is an in-file idiom judgment read from the diff — there are no engine probes." That sentence becomes false. Rewrite it to name the tool rules that carry the deterministic half, and to state the rule: if a tool can decide it, the tool owns it.
- A rule file that loses every bullet is deleted, not left empty.

## Acceptance

- Each removed bullet has a passing tool-rule test that covers the same defect. Name the test in the commit message.
- No rule file states a requirement that a tool also states. One owner per rule.
- VALIDATOR.md describes what the bundle actually is. #tool-validators