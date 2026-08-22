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
- actor: claude-code
  id: 01m0n4szjjqan6ans3xr9nsf0s
  text: |-
    ### CORRECTIONS — this card's body has three wrong facts. Measurement beats the card.

    I wrote this card before either tool rule existed. Three claims in it are now known wrong. Trust the measurements below, not the card body.

    **1. "Twelve bullets" is wrong. The real count is seven, across three rules.**
    Measured against swiftformat 0.62.1 and swiftlint 0.65.0:
    - `idioms.md` — 5 bullets
    - `value-semantics.md` — 1 bullet
    - `optionals.md` — 1 bullet

    "Seven across three" is now stated consistently in the `idioms-swift` rule body, `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs`, and `crates/swissarmyhammer-validators/src/builtin/mod.rs`. My earlier comment on this card said "six across two" — that was also wrong. Use seven across three.

    **2. The `unused_optional_binding` row in the card's table is bogus.** It maps to a bullet in `optionals.md` that does not exist. That file has four bullets and none of them is it. Do not go looking for it.

    **3. Six enabled swiftformat rules never fire.** The roster names 29; swiftformat 0.62.1 knows 27; the failing fixture yields 42 findings carrying only **21 distinct rules**. The six silent ones are `preferCountWhere`, `opaqueGenericParameters`, `environmentEntry`, `genericExtensions`, `conditionalAssignment`, `validateTestCases`.

    A prompt bullet whose only tool coverage is one of those six is **NOT covered**. Do not delete it. Verify each bullet against a rule that actually fires, not against a rule that is merely enabled.

    The same holds for `preferLazyMap` and `ifExpressions`, which exist only on SwiftFormat `main` and in no release.

    **4. `noGuardInTests` is now enabled** (decided in ^t78sqr4) and `optionals.md` already gained its test carve-out. That work is done — do not redo it. Note the tool is silent on the shorthand `guard let value else`, so the prompt half still carries that case. Do not delete the bullet as covered.

    ### Read this before deleting anything

    There is evidence the review engine does not review markdown on a diff-scoped run — see ^dyad426, two reproductions so far, and in the second one the unreviewed markdown WAS the whole change. Every file you touch on this card is markdown.

    So you cannot rely on `/review` to catch a mistake here. Prove each deletion yourself: for every bullet you remove, run the real tool over a file containing that defect and show it reports. Put the command and its output in a task comment. A bullet deleted without that evidence is a coverage hole that nothing downstream will catch.
  timestamp: 2026-08-22T16:28:04.050695+00:00
- actor: claude-code
  id: 01m0n558spvb1tmq8q59aexjph
  text: |-
    ### Measurement — every bullet probed against the real tool before any deletion

    The card comment asks for evidence, not reasoning. Each probe below is one file holding ONE defect, judged by the SHIPPED script of the tool rule, extracted from the `run:` block of the rule's own frontmatter. swiftformat 0.62.1, swiftlint 0.65.0.

    The swiftformat probes stand beside a `.swift-version` holding `6.3`, so no clean answer is one the version gate bought. The swiftlint probes stand under `Sources/`, so the three force rules apply.

    Command shape:

        sh  idioms-swift.sh <file>                 # the swiftformat gate
        bash disallowed-constructs-swift.sh <file> # the swiftlint gate, which uses bash arrays

    #### `idioms-swift` — REPORTS

        === TypeSugar.swift
        {"line":2,"message":"typeSugar: Prefer shorthand syntax for Arrays, Dictionaries and Optionals."}
        === MemberwiseInit.swift
        {"line":4..8,"message":"redundantMemberwiseInit: Remove explicit internal memberwise initializers that are redundant."}   (5 lines, one for each line of the span)
        === ForEachLoop.swift
        {"line":3,"message":"preferForLoop: Convert functional forEach calls to for loops."}
        === PatternLet.swift
        {"line":7,"message":"hoistPatternLet: Reposition let or var bindings within pattern."}
        === FinalClass.swift
        {"line":1,"message":"preferFinalClasses: Prefer defining final classes. To suppress this rule, add \"Base\" to the class name, add a doc comment mentioning \"base class\" or \"subclass\", make the class open, or use a // swiftformat:disable:next preferFinalClasses directive."}

    Each bullet was also probed in its DO form, and each DO form drew NOTHING: `case .at(let x, let y)`, `public final class Worker`, and a `public init` memberwise initializer (which is the exception `idioms.md` states, and which the tool's own message reads as "explicit INTERNAL memberwise initializers").

    `typeSugar` was probed over all six forms the bullet names, one for each line:

        line 2 Array<Int>            -> reported
        line 3 Dictionary<String,Int>-> reported
        line 4 Optional<String>      -> reported
        line 5 [Int]                 -> silent
        line 6 [String: Int]         -> silent
        line 7 String?               -> silent

    #### `disallowed-constructs-swift` — REPORTS

        === Sources/ForceUnwrap.swift
        {"line":5,"message":"force_unwrapping: Force unwrapping should be avoided"}
        === Sources/ImplicitlyUnwrapped.swift
        {"line":4,"message":"implicitly_unwrapped_optional: Implicitly unwrapped optionals should be avoided when possible"}
        === Sources/ForceTry.swift
        {"line":5,"message":"force_try: Force tries should be avoided"}
        === Sources/ForceCast.swift
        {"line":7,"message":"force_cast: Force casts should be avoided"}
        === Sources/UncheckedSendable.swift
        {"line":3,"message":"no_unchecked_sendable: Instead of @unchecked Sendable, write a plain Sendable conformance or a @preconcurrency import. If the type really must be @unchecked Sendable, write // swiftlint:disable:next no_unchecked_sendable above it with the synchronization invariant that makes the type thread-safe"}

    #### THREE bullets the measurement REFUSES to call covered

    **1. `idioms.md` return `Void`, not `()` — HALF covered, so the bullet is SPLIT, not deleted.**

    The bullet states two requirements: write `Void` rather than `()`, and omit the clause entirely when it is `Void`. SwiftFormat's `void` rule decides the first and NOT the second. Measured over one file holding three declarations:

        line 2  public static func run() -> ()             -> void reported
        line 3  public static func handler(_ b: (Int) -> ())-> void reported
        line 4  public static func typed() -> Void {}       -> SILENT

        $ swiftformat --rules void --quiet VoidParen.swift   # what it rewrites to
        public static func run() -> Void {}
        public static func handler(_ body: (Int) -> Void) {}
        public static func typed() -> Void {}

    `void` rewrites `()` INTO `-> Void` and stops there. The bullet's own DO is `func f() {}`. SwiftFormat states that in a separate rule, `redundantVoidReturnType`, which this roster does not name. So the `()` half comes out and the omit-the-clause half stays. A new card carries the `redundantVoidReturnType` question.

    **2. `optionals.md` never `guard` in a test — NOT covered, left whole.** Confirms the earlier comment. Measured over a Swift Testing suite holding `guard let source else { return }`, the shorthand: the gate reported NOTHING.

    **3. `casing.md` — NOT covered by either gate, left whole.** Measured over one file holding `MAX_RETRY_COUNT`, `kMaximumRetries`, `strName` and `bIsValid`: the swiftformat gate reported nothing and the swiftlint gate reported nothing. No roster of either gate names `identifier_name` or any casing regex. `test_swift_casing_accepts_both_acronym_spellings` is therefore untouched.

    #### The deletion set

    Ten bullets, plus one half. `idioms.md` 4 + the `()` half, `value-semantics.md` 1, `optionals.md` 2, `error-handling.md` 2, `concurrency.md` 1. No rule file loses every bullet, so no file is deleted.

    The card body's count of twelve was twelve across six rule files. The correction comment's "seven across three" counts the swiftformat gate ALONE; the swiftlint gate decides five more, which its own body states as "FIVE bullets spread across three prompt rules". Twelve minus the one `casing.md` bullet no tool decides, minus the half of the `void` bullet no tool decides, is what this card removes. No sibling card owns the swiftlint prompt half, and the acceptance criterion "No rule file states a requirement a tool also states" reaches it, so this card carries both gates.
  timestamp: 2026-08-22T16:34:13.942341+00:00
- actor: claude-code
  id: 01m0n60kmkczc0cve5fsseda7q
  text: |-
    ### The token saving, measured — 350 tokens for each validator task

    The card asks for this as evidence for the `implement-rules-preload` prompt-cap work, so the number had to be the one a review run actually PAYS rather than a byte count.

    **What a run pays.** `render_suffix` in `crates/swissarmyhammer-validators/src/review/fleet/render.rs` builds the validator suffix. It emits the `VALIDATOR.md` body as a `## Guidance` block through `render_validator_guidance`, then each non-tool rule body under `## Rules`. That one function is the body behind both prompt shapes — the warm `render_validator_suffix` and the degraded `render_fleet_prompt` — and the run prime carries no validator text, so none of it sits in a cached prefix. The suffix goes out once for each validator task, so a manifest body and a rule body cost the SAME per prompt. The measurement below therefore counts both.

    Counted over the markdown under each front matter, with `tiktoken` `o200k_base`, against `HEAD`:

    | rule | before | after | saved |
    |---|---|---|---|
    | `VALIDATOR.md` | 174 | 401 | **-227** |
    | `idioms.md` | 510 | 325 | 185 |
    | `value-semantics.md` | 280 | 242 | 38 |
    | `optionals.md` | 430 | 308 | 122 |
    | `error-handling.md` | 352 | 197 | 155 |
    | `concurrency.md` | 380 | 303 | 77 |
    | `casing.md` | 741 | 741 | 0 |
    | **TOTAL** | **2867** | **2517** | **350** |

    The rule bodies alone give up **577** tokens. `VALIDATOR.md` takes 227 of them back, because the card asks it to name the two tool rules and state the standing order. That is the honest net: **350 tokens, 12.2% of the set, for every validator task the run plans.**

    A first draft of `VALIDATOR.md` cost 531 tokens more and left a net saving of 46. It was cut to the sentence that decides ("if a tool can decide it, the tool owns it"), the two rule names, the `supersedes` reason, and the split rule. The per-bullet measurements live in the two tool rule bodies, which the review prompt does NOT carry — they are read by a person, not by the model on every batch.

    ### Two facts the reading turned up that the prompt-cap work should have

    1. **`casing.md` is 741 tokens — 29% of the set after this change, and the single largest rule.** It is untouched here because measurement showed neither gate names `identifier_name` or any casing regex. The card was right that the length is the price of judgment; the price is now measured. A tool that decides Swift identifier casing would take about a quarter of this set's prompt cost with it.
    2. **A long manifest body costs more than its own tokens.** `validator_suffix_framing_bytes` runs the same `render_suffix`, and `prompt_framing` takes the MAX over validators — so a fat `VALIDATOR.md` on ONE validator shrinks the file-payload budget for EVERY batch in the run. That is a reason to keep a manifest body short that a byte count does not show.

    ### A gap in the preload itself

    `dump validators` does NOT render the manifest body, and it DOES include tool rules — the review prompt does the opposite on both counts (`crates/swissarmyhammer-tools/src/mcp/tools/review/validators.rs`, `render_rules_markdown`). An agent preloading the dump therefore reads a different rule set than the reviewer is held to: it misses the `## Guidance` block and reads tool-rule bodies no reviewer sees. Filed as ^aw6m2cb.
  timestamp: 2026-08-22T16:49:09.779034+00:00
- actor: claude-code
  id: 01m0n643m0x1dzkh1qe1p598vb
  text: |-
    ### Implementation landed

    Written test-first. The coverage-guard test was added BEFORE any bullet came out, and it failed for the right reason — the tool half of each row passed, and the prompt half failed because the bullets were still there:

        `typeSugar` decides `shorthand type sugar`, so `idioms.md` must not state it as well; one requirement takes one owner
        `force_unwrapping` decides `No force unwrap (`!`) in non-test code.`, so `optionals.md` must not state it as well

    Both went green after the deletion.

    #### The two tests, and what each holds

    `the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` and
    `the_shipped_swift_disallowed_constructs_tool_rule_owns_each_bullet_it_took`.
    Each walks a table of `SupersededSwiftBullet` rows and holds every row to both
    halves of the claim, through the shared
    `verify_superseded_swift_bullet` in `tests/shipped.rs`:

    1. the tool rule REPORTS a probe holding that one defect — so a rule that goes silent in a later release fails by name rather than leaving a hole;
    2. the prompt rule body no longer STATES it — so a bullet written back in fails too.

    The swiftformat probes stage a `.swift-version`, so no clean answer is one the version gate bought. The swiftlint probes stand at the repository root, which no test target holds, so the three force rules apply.

    #### The rows, one for each removed bullet

    | test | prompt rule | tool rule | probe |
    |---|---|---|---|
    | `..._idioms_..._owns_each_bullet_it_took` | `idioms.md` | `typeSugar` | `SWIFT_IDIOMS_LONG_TYPE` |
    | the same | `idioms.md` | `void` | `SWIFT_IDIOMS_PAREN_RETURN` |
    | the same | `idioms.md` | `redundantMemberwiseInit` | `SWIFT_IDIOMS_REDUNDANT_INIT` |
    | the same | `idioms.md` | `preferForLoop` | `SWIFT_IDIOMS_FOR_EACH` |
    | the same | `idioms.md` | `hoistPatternLet` | `SWIFT_IDIOMS_HOISTED_LET` |
    | the same | `value-semantics.md` | `preferFinalClasses` | `SWIFT_IDIOMS_OPEN_CLASS` |
    | `..._disallowed_constructs_..._owns_each_bullet_it_took` | `optionals.md` | `force_unwrapping` | `SWIFT_DISALLOWED_FORCE_UNWRAP` |
    | the same | `optionals.md` | `implicitly_unwrapped_optional` | `SWIFT_DISALLOWED_IMPLICITLY_UNWRAPPED` |
    | the same | `error-handling.md` | `force_try` | `SWIFT_DISALLOWED_FORCE_TRY` |
    | the same | `error-handling.md` | `force_cast` | `SWIFT_DISALLOWED_FORCE_CAST` |
    | the same | `concurrency.md` | `no_unchecked_sendable` | `SWIFT_UNCHECKED_SENDABLE_PLAIN` |

    The last row reuses the probe the directive test already drives, so one pair of constants carries both answers: the conformance reports, and the same bytes behind the directive do not.

    #### What was NOT deleted, and why

    - `optionals.md` never `guard` in a test — `noGuardInTests` is silent on the shorthand. Left whole.
    - `idioms.md` return `Void` — SPLIT, not deleted. The `()` half is the tool's; the omit-the-clause half stays as its own bullet. Follow-up ^mqp3jze carries the `redundantVoidReturnType` question.
    - `casing.md` — no roster of either gate names a rule that reads a Swift identifier, so `test_swift_casing_accepts_both_acronym_spellings` is untouched, and so is the "BOTH accepted" bullet.
    - No rule file lost every bullet, so no file was deleted. Remaining counts: `idioms.md` 3, `value-semantics.md` 4, `optionals.md` 3, `error-handling.md` 3, `concurrency.md` 6.

    #### Everything the deletion made stale

    Every place that stated a bullet, a count, or a cross-reference to one was found by search and rewritten, not just the rule files:

    - `builtin/validators/swift/VALIDATOR.md` — the sentence "there are no engine probes" was false. Rewritten around the standing order, naming both tool rules, the empty `supersedes` and its reason, and the split rule.
    - both tool rule bodies — the mapping tables now read "the bullet it took", each names its coverage-guard test, and `idioms-swift` gained a section measuring which half of the `void` bullet it decides.
    - `idioms-swift.md` no longer claims `optionals.md` "sanctions an implicitly unwrapped optional for a fixture set in `setUp()`" — that bullet is gone, and the sanction is now stated in the `disallowed-constructs-swift` carve-out table.
    - `code-hygiene/VALIDATOR.md` — its swift-idioms section said SIX bullets across TWO rules and a roster of 28. Both were already wrong before this card; measured against the shipped script and `SWIFT_IDIOMS_ROSTER_SIZE`, it is SEVEN across THREE and 29. Corrected in the same paragraphs this card rewrote.
    - Rust doc comments stating the counts: `crates/swissarmyhammer-validators/src/builtin/mod.rs` (both tool-rule groups) and `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs` (both roster constants).
    - `disallowed_constructs_swift.rs` — the test-target rationale cited two bullets that no longer exist by that name.

    `swift_prompt_rule_body` and the Swift prompt-rule name constants moved from `idioms_swift.rs` up to `tests/shipped.rs`, so both gate test files share one reader of the shipped bodies rather than each keeping a copy.

    Verified: `cargo fmt --all -- --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo nextest run -p swissarmyhammer-validators` 838/838 pass.

    #### One note for whoever commits

    `.kanban/tasks/01M0MVNQDK2G1J4X2SRT78SQR4.*` and `.kanban/tasks/01M0MYR4Z3GXSYEKCT8DYAD426.*` were already dirty when this card started — they carry comments written at 16:27, before this session. They are not this card's work.
  timestamp: 2026-08-22T16:51:04.448142+00:00
- actor: claude-code
  id: 01m0n64mh28wh3pzgf2d8412by
  text: |-
    ### implement — changed

    - evidence: 14 files. Prompt rules `builtin/validators/swift/rules/{idioms,value-semantics,optionals,error-handling,concurrency}.md` and `builtin/validators/swift/VALIDATOR.md`; tool rule bodies `builtin/validators/code-hygiene/rules/{idioms-swift,disallowed-constructs-swift}.md` and `builtin/validators/code-hygiene/VALIDATOR.md`; Rust `crates/swissarmyhammer-validators/src/builtin/mod.rs`, `src/review/tool_rules/tests.rs`, `src/review/tool_rules/tests/shipped.rs`, `src/review/tool_rules/tests/shipped/idioms_swift.rs`, `src/review/tool_rules/tests/shipped/disallowed_constructs_swift.rs`.
    - 10 bullets deleted plus one half, each with a probe run against the real tool BEFORE deletion and a passing coverage-guard test row after. 3 bullets measured NOT covered and left standing: `optionals.md` never `guard` in a test, the omit-the-clause half of `idioms.md` return `Void`, and every bullet of `casing.md`.
    - token saving: 350 for each validator task, 12.2% of the `swift` set. Rule bodies alone give up 577; `VALIDATOR.md` takes 227 back for the standing order the card asked it to state.
    - tests: `cargo nextest run --workspace --no-fail-fast` — 14181 run, 14181 passed, 0 skipped. `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - new cards: ^mqp3jze (does `redundantVoidReturnType` join the roster), ^aw6m2cb (`dump validators` drops the `VALIDATOR.md` guidance the review prompt carries).
    - next: `/review`. Note ^dyad426 — the review engine may not read markdown on a diff-scoped run, and every rule file this card touched is markdown. The two coverage-guard tests are the gate that does not depend on that.
  timestamp: 2026-08-22T16:51:21.762071+00:00
- actor: claude-code
  id: 01m0n6d3xac5fpr265z0wdzt7e
  text: |-
    ### test — green
    - evidence: `cargo nextest run --workspace` — 14181 tests run, 14181 passed, 0 failed, 0 skipped (baseline 14179 + 2 new coverage-guard tests). `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - confirmation 1: `test_swift_casing_accepts_both_acronym_spellings` exists at `crates/swissarmyhammer-validators/src/builtin/mod.rs` and passes (verified individually: PASS). `git diff HEAD -- builtin/validators/swift/rules/casing.md` is empty — casing.md is unchanged, matching the claim that neither tool gate covers casing.
    - confirmation 2: both new tests exist and pass (verified individually): `the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` at `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`, and `the_shipped_swift_disallowed_constructs_tool_rule_owns_each_bullet_it_took` at `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/disallowed_constructs_swift.rs`.
    - next: ready for review.
  timestamp: 2026-08-22T16:55:59.658527+00:00
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
- 01M0MVM2VZ71SBQ95754S2N0RX
- 01M0MVNQDK2G1J4X2SRT78SQR4
position_column: doing
position_ordinal: '8280'
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