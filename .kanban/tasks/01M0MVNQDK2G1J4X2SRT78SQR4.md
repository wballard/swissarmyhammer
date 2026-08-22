---
assignees:
- claude-code
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
position_column: todo
position_ordinal: ffec80
project: swift-validator
title: 'swift: resolve the two conflicts Airbnb''s tool config exposes in our prompt rules'
---
Two SwiftFormat options in Airbnb's config contradict rules we already ship. Decide each one before the swiftformat tool rule enables it. A tool and a prompt rule that disagree produce churn on every review round — the worst failure mode we have.

## Conflict 1 — `noGuardInTests` versus `optionals.md`

Airbnb enables SwiftFormat `noGuardInTests` and `--guard-like-if-statements convert`. That rewrites `guard` statements inside test files into assertions. Their skill states it plainly: "Avoid `guard` statements in unit tests. Use assertions instead of guarding on boolean conditions."

Our `optionals.md` says: "Use `guard let … else { return/throw }` for early exit so the happy path stays unindented." It carves out nothing for tests. It also sanctions IUO for "test fixtures set in `setUp()`", so it already knows tests are different — it just does not apply that thinking here.

They are both right. A `guard` in production code protects the happy path. A `guard` in a test silently skips the test instead of failing it, which is why Airbnb bans it there.

DECIDE: add the test carve-out to `optionals.md` and enable `noGuardInTests`, or leave the rule alone and do not enable it. Recommendation: add the carve-out. A `guard` that returns early in a test turns a failure into a false pass — that is a real defect, not a style preference.

## Conflict 2 — `--property-types inferred` versus `idioms.md`

Airbnb sets `--property-types inferred` and states "Prefer letting the type of a variable or property be inferred from the right-hand-side value rather than writing the type explicitly."

Our `idioms.md` says the opposite for empty collections: "Empty-collection variables use a literal with a type annotation, not a call. DO: `var items: [Int] = []`. DON'T: `var items = [Int]()`." It calls the reverse a validator error and warns against flip-flopping.

UNVERIFIED: I did not confirm that SwiftFormat's `propertyTypes` rule actually rewrites `var items: [Int] = []`. The empty array literal `[]` does not name a type, so `redundantType` should leave it alone. Test this before you decide anything.

DO THIS FIRST: write a probe. Run real SwiftFormat with `--property-types inferred` over `var items: [Int] = []` and over `var ids: Set<String> = []`. Read what comes out.
- If SwiftFormat leaves them alone, there is no conflict. Enable the option and close this half.
- If SwiftFormat rewrites them to `[Int]()` / `Set<String>()`, the option loses. Our rule is the deliberate house style and it is already load-bearing. Do not enable `--property-types inferred`, and record why in the tool rule.

## Acceptance

- Each conflict has a written decision in the tool rule or the prompt rule, with the reason.
- Conflict 2's probe output is recorded in the card comments, not summarized from memory.
- A test proves the two halves agree: the same Swift file gets no finding from the tool AND no finding from the prompt rule. #tool-validators