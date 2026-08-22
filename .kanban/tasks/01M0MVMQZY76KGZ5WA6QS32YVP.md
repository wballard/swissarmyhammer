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
- actor: claude-code
  id: 01m0n6eyhcw46xvg0f0y7h08jh
  text: |-
    ### commit — changed
    - evidence: f79727e2e docs(validators): remove Swift prompt rules now owned by tool rules
    - next: proceed to review
  timestamp: 2026-08-22T16:56:59.692180+00:00
- actor: claude-code
  id: 01m0n6xnna61cbzry7aa5td1f9
  text: |-
    ### review — findings
    - evidence: 5 findings. Engine `review sha HEAD~1..HEAD` returned `findings: 0` but reviewed only the 5 `.rs` files of 24; all 9 `.md` files were silently dropped, so its clean result is not evidence. Hand review of the 9 markdown files against real swiftformat 0.62.1 / swiftlint 0.65.0 found a genuine coverage hole: `preferForLoop` is SILENT on the `forEach` + `if` shape the deleted `idioms.md` bullet named (shipped run block omits `--single-line-for-each convert`, default is `ignore`), and never reads the `where`-clause half at all. The guard test's probe stages the multi-line form — the one shape that fires — so it passes without proving the claim. Findings at `builtin/validators/swift/rules/idioms.md` (deleted forEach bullet, x2), `builtin/validators/swift/VALIDATOR.md`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:675`, `builtin/validators/code-hygiene/rules/idioms-swift.md:99`.
    - verified clean by hand: all 10 other deleted bullets FIRE with literal diagnostics; all 3 deliberately-kept bullets confirmed SILENT (correct sole owners); VALIDATOR.md rewrite accurate on all three of its claims. `verify_superseded_swift_bullet` is a real fires-check, not an enabled-check.
    - next: restore the `forEach` bullet whole with the missed-shape note, or add `--single-line-for-each convert` and re-measure; then fix the guard probe and the two overstated ownership claims. Task stays in `review`.
    - also: logged reproduction 3 on ^dyad426 — it settles the open question there, markdown is not miscounted, it is not reviewed at all.
  timestamp: 2026-08-22T17:05:02.122419+00:00
- actor: claude-code
  id: 01m0n6z3hxb36gjbp5nhhyz350
  text: |-
    ### finish iteration 1 — findings
    - implement: changed — 14 files; 10 bullets + half of an 11th deleted, each probed before removal; 3 candidates refused on measurement
    - test: green — cargo nextest run --workspace, 14181 passed, 0 failed, 0 skipped; fmt clean; clippy clean
    - commit: f79727e2e
    - review: findings — 5 open, ALL from hand review; the engine found 0 and its 0 is not evidence

    **The engine could not review this commit.** 24 files changed (10 `.kanban/`, 9 `.md`, 5 `.rs`). The engine reported 5 reviewed and 10 not reviewed = 15, not 24, with `skipped_files` naming only the `.kanban/` paths. All 9 markdown files were dropped with no line in the tally — and markdown is the entire substance of this commit. Logged as reproduction 3 on ^dyad426.

    **One real coverage hole, and it is the important finding.** The deleted `idioms.md` bullet —

    > Prefer a `for` loop (with a `where` clause when filtering) over `forEach` + `if` when you need control flow

    — names a shape no tool reports. Measured under the shipped run block on swiftformat 0.62.1:

    ```swift
    things.forEach { if $0 > 2 { print($0) } }   // exit=0, ZERO findings
    ```

    `preferForLoop` has `--single-line-for-each`, which defaults to `ignore`, and the shipped script never passes it. The `where`-clause half has no owner at all — `preferForLoop` converts `forEach` to `for` and never suggests `where`.

    **The guard test passed for the wrong reason.** `SWIFT_IDIOMS_FOR_EACH` stages the multi-line `values.forEach { value in print(value) }`, which DOES fire. So the row proved ownership of a shape the bullet did not name, while the shape it did name goes unreported. The mechanism is sound — `verify_superseded_swift_bullet` really does check the tool REPORTED, not merely that it is enabled — but a probe body chosen wrong defeats a correct mechanism.

    **This commit's own new VALIDATOR.md text condemns the deletion:** "A bullet stating ONE requirement the tool reads only partly stays here whole, and says which part the tool misses." The `void` bullet in the same commit was handled exactly that way. `forEach` was not.

    **Everything else holds.** The other 10 deleted bullets all fire with literal diagnostics. All three deliberately-kept claims are confirmed silent, so each kept bullet is the correct sole owner. The casing verdict is stronger than claimed: `identifier_name` is not in the roster, and even enabled it would fire only on `MAX_RETRY_COUNT` as a charset check — it would not cover the bullet.
  timestamp: 2026-08-22T17:05:49.117351+00:00
- actor: claude-code
  id: 01m0n7ar9xe8gm6z6bc89wzm80
  text: |-
    ### Picked up for iteration 2 — measurement first, then the decision

    Every number below was taken with the SHIPPED `run:` block, extracted verbatim out of the front matter of `builtin/validators/code-hygiene/rules/idioms-swift.md`, on swiftformat 0.62.1, beside a `.swift-version` holding `6.3`.

    #### The hole reproduces exactly as the finding states

        === SingleLineIf      things.forEach { if $0 > 2 { print($0) } }        exit=0, ZERO findings
        === MultiLineIf       things.forEach { thing in if thing > 2 { … } }    preferForLoop, line 3
        === FilterChain       things.filter { $0 > 2 }.forEach { thing in … }   exit=0, ZERO findings
        === SingleLinePlain   things.forEach { print($0) }                      exit=0, ZERO findings
        === MultiLinePlain    values.forEach { value in print(value) }          preferForLoop, line 3

    The last row is the body `SWIFT_IDIOMS_FOR_EACH` stages today. It is the one row that reports without the option, and it holds no `if` at all — so the guard row proved ownership of a shape the deleted bullet never named.

    #### The option closes the first half, and only the first half

    The same five probes under the same script plus `--single-line-for-each convert`:

        === SingleLineIf      preferForLoop, line 3       <- was silent
        === MultiLineIf       preferForLoop, line 3
        === FilterChain       exit=0, ZERO findings       <- still silent
        === SingleLinePlain   preferForLoop, line 3       <- was silent
        === MultiLinePlain    preferForLoop, line 3

    `swiftformat --rule-info preferForLoop` states the default: `--single-line-for-each  … "ignore" (default) or "convert"`. It also states the chain limit in its own examples — "Doesn't affect long multiline functional chains" — so no option reaches the `where` half.

    #### The option conflicts with nothing already shipped

    Both fixtures were run through the shipped script and through the same script plus the option:

    | the run | fail fixture | pass fixture |
    |---|---|---|
    | shipped | 42 findings, 21 distinct rules | 0 findings |
    | plus `--single-line-for-each convert` | 42 findings, the same 21 rules | 0 findings |

    Neither count moves, so the roster's measurement tables stand and the doctor fixture pair is unaffected. No prompt rule of `builtin/validators/swift/` sanctions a single-line `forEach` — searched, the only other `forEach` bullet in the whole set is `js-ts/naming-and-style`, which reads no `.swift` file.

    #### The decision

    BOTH, which is the `void` treatment the card asks for.

    1. **Enable `--single-line-for-each convert`.** It is what makes the tool decide the shape the bullet named, and it costs nothing measured.
    2. **Restore the `where` half as its own bullet in `idioms.md`**, saying which shape the tool misses. No option reaches it, so the prompt rule keeps it.

    The bullet is SPLIT at the requirement, exactly as `void` was: the `forEach` + `if` half is the tool's, the `filter`/`forEach` chain half stays in the prompt.

    #### The audit of every other probe body, both guard files

    `preferForLoop` is the only probe that stages a shape its bullet never named. Each row below was read against the deleted bullet word for word:

    | probe | the bullet's own DON'T | verdict |
    |---|---|---|
    | `SWIFT_IDIOMS_LONG_TYPE` | `Array<Int>`, `Dictionary<Key, Value>`, `Optional<String>` | the named shape, but ONE of the three |
    | `SWIFT_IDIOMS_PAREN_RETURN` | `func f() -> ()` | the named shape, word for word |
    | `SWIFT_IDIOMS_REDUNDANT_INIT` | the bullet names no literal; an internal memberwise `init` | the named shape |
    | `SWIFT_IDIOMS_FOR_EACH` | `forEach` + `if` | **WRONG — holds no `if`** |
    | `SWIFT_IDIOMS_HOISTED_LET` | `case let .point(x, y)` | the named shape, word for word |
    | `SWIFT_IDIOMS_OPEN_CLASS` | the bullet names no literal; a non-`final` `class` | the named shape |
    | `SWIFT_DISALLOWED_FORCE_UNWRAP` | `let name = user.name!` | the named shape |
    | `SWIFT_DISALLOWED_IMPLICITLY_UNWRAPPED` | `var session: URLSession!` | the named shape |
    | `SWIFT_DISALLOWED_FORCE_TRY` | `let data = try! Data(contentsOf: url)` | the named shape |
    | `SWIFT_DISALLOWED_FORCE_CAST` | `segue.destination as! DetailVC` | the named shape |
    | `SWIFT_UNCHECKED_SENDABLE_PLAIN` | `final class Counter: @unchecked Sendable { var n = 0 }` with no guard | the named shape |

    `SWIFT_IDIOMS_LONG_TYPE` is the second row worth an edit. The bullet enumerated THREE spellings and the probe stages one. All three were measured under the shipped script and each reports on its own line, so the probe is widened to hold all three rather than left proving a third of its bullet.
  timestamp: 2026-08-22T17:12:10.813607+00:00
- actor: claude-code
  id: 01m0n7zkghgd0y7pk02hxtdjdn
  text: |-
    ### Iteration 2 landed — the `forEach` bullet is split, and the option is on

    Written test-first. Three tests were RED before any fix, each for the right reason:

        the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took
          `preferForLoop` must report the defect `idioms.md` stated as `over `forEach` + `if``,
          or deleting that bullet left the requirement with no owner; the run reported []

        the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each        FAIL (tool silent)
        the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain FAIL (`idioms.md` states no such form)

    All three GREEN after the two changes below.

    #### 1. `--single-line-for-each convert` stands on the shipped command line

    It makes `preferForLoop` report the exact DON'T the deleted bullet named. Measured, and the measurement table now stands in `idioms-swift.md` under its own section, beside the `void` one:

    | the walk | shipped run | the same, without the option |
    |---|---|---|
    | `things.forEach { if $0 > 2 { print($0) } }`, on one line | reported | NO |
    | the same `forEach` and `if` over five lines | reported | reported |
    | `things.forEach { print($0) }`, on one line | reported | NO |
    | `values.forEach { value in print(value) }`, over three lines | reported | reported |
    | `things.filter { $0 > 2 }.forEach { thing in print(thing) }` | NO | NO |

    `the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each` holds row 1, so a run that drops the option goes quiet there rather than losing the shape without a word.

    Nothing shipped conflicts: the failing fixture reports the same **42 findings** carrying the same 21 rules with the option and without it, and the passing fixture reports 0 either way, so every count in the rule body stands as written.

    #### 2. The `where` half is back in `idioms.md`, as its own bullet

        - **Filter with a `where` clause, not with a `filter` chain feeding a `forEach`.**
          DON'T: `things.filter { $0 > 2 }.forEach { thing in print(thing) }`.
          DO: `for thing in things where thing > 2 { print(thing) }`. SwiftFormat's
          `preferForLoop` turns a `forEach` into a `for` loop and never suggests a
          `where` clause, so this half is this rule's alone.

    Row 5 above is why. No option reaches it — SwiftFormat states the limit itself, "Doesn't affect long multiline functional chains". `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain` holds BOTH sides: the gate stays silent on that chain, and `idioms.md` states the chain word for word, so a probe that stopped measuring a shape the prompt rule asks for fails by name.

    The bullet is terse on purpose, and the measurement lives in `idioms-swift.md`, which is the `void` shape exactly: the review prompt carries the prompt rule and never the tool rule body, so a first draft that carried the whole measurement in the bullet cost 63 more tokens on every validator task for words a person reads once.

    #### The guard probe, corrected

    `SWIFT_IDIOMS_FOR_EACH` is gone. `SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH` replaces it and stages `things.forEach { if $0 > 2 { print($0) } }` — the shape the bullet named. `words` moved from `Prefer a `for` loop` to ``over `forEach` + `if` ``, which is the half the tool owns and which the restored bullet does not state.

    #### The second bad probe the audit found

    `SWIFT_IDIOMS_LONG_TYPE` staged `Array<Int>` alone, and its bullet enumerated THREE DON'T spellings. Widened to hold `Array<Int>`, `Dictionary<String, Int>` and `Optional<String>`; measured under the shipped script, `typeSugar` reports each on its own line. Every other probe in both guard files stages the shape its bullet named — the audit table stands in the comment above.

    #### Everything the change made stale

    - `builtin/validators/code-hygiene/rules/idioms-swift.md` — the run block, the roster table (`preferForLoop` now names its option and reads "HALF of that bullet"), the paragraph that claimed the guard holds each row to "the rule reports a file holding that ONE defect" (it now says WRITTEN IN THE SHAPE THE BULLET NAMED), a new `preferForLoop` measurement section, and the supersedes section's counts.
    - `builtin/validators/swift/VALIDATOR.md` — "Ten bullets … and half of an eleventh" is now "NINE bullets … and HALF of each of two more".
    - `builtin/validators/code-hygiene/VALIDATOR.md` — the swift-idioms section now states two halves rather than one, and names the option.
    - Rust doc comments carrying the counts: `crates/swissarmyhammer-validators/src/builtin/mod.rs`, `src/review/tool_rules/tests.rs`, and the module doc plus the superseded-bullet table doc in `tests/shipped/idioms_swift.rs`.

    #### The token saving, recomputed

    `tiktoken` `o200k_base` over the markdown under each front matter, `f79727e2e^` against the working tree:

    | rule | before | after | saved |
    |---|---|---|---|
    | `VALIDATOR.md` | 174 | 403 | -229 |
    | `idioms.md` | 510 | 429 | 81 |
    | `value-semantics.md` | 280 | 242 | 38 |
    | `optionals.md` | 430 | 308 | 122 |
    | `error-handling.md` | 352 | 197 | 155 |
    | `concurrency.md` | 380 | 303 | 77 |
    | `casing.md` | 741 | 741 | 0 |
    | **TOTAL** | **2867** | **2623** | **244** |

    **244 tokens for each validator task, 8.5% of the set**, down from the 350 / 12.2% the first iteration reported. The rule bodies alone give up 473; `VALIDATOR.md` takes 229 back for the standing order the card asks it to state. The 106-token difference from iteration 1 is the restored `where` bullet, and it is the price of a requirement that has no tool.

    Verified: `cargo fmt --all -- --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo nextest run --workspace --no-fail-fast` 14183 run / 14183 passed / 0 skipped (baseline 14181 plus the two new tests).
  timestamp: 2026-08-22T17:23:34.033345+00:00
- actor: claude-code
  id: 01m0n80w3a7ft8vbv7aaqtk3nn
  text: |-
    ### implement — changed
    - evidence: 7 files — `builtin/validators/swift/rules/idioms.md`, `builtin/validators/swift/VALIDATOR.md`, `builtin/validators/code-hygiene/rules/idioms-swift.md`, `builtin/validators/code-hygiene/VALIDATOR.md`, `crates/swissarmyhammer-validators/src/builtin/mod.rs`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`.
    - the coverage hole: `--single-line-for-each convert` added to the shipped run (measured: the single-line `things.forEach { if $0 > 2 { print($0) } }` goes from ZERO findings to `preferForLoop`, and both fixtures are unmoved at 42 and 0), AND the `where` half restored to `idioms.md` as its own bullet, because no option reaches the `filter`/`forEach` chain.
    - probes corrected: `SWIFT_IDIOMS_FOR_EACH` replaced by `SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH`, which stages the shape the bullet named; `SWIFT_IDIOMS_LONG_TYPE` widened from one of its bullet's three DON'T spellings to all three. Every other probe in both guard files audited against its deleted bullet and correct — the table stands two comments above.
    - 2 new tests: `the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each` and `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain`. Both written RED first, beside the guard test failing on `the run reported []`.
    - token saving, recomputed: **244 for each validator task, 8.5% of the `swift` set** (was 350 / 12.2%). Rule bodies give up 473; `VALIDATOR.md` takes 229 back.
    - tests: `cargo nextest run --workspace --no-fail-fast` — 14183 run, 14183 passed, 0 skipped. `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - next: `/review`. The five findings stay unchecked for the reviewer to verify and close.
  timestamp: 2026-08-22T17:24:15.594717+00:00
- actor: claude-code
  id: 01m0n8aet74ehwt9dah2c25a5j
  text: |-
    ### test — green
    - evidence: `cargo nextest run --workspace` — 14183 tests run, 14183 passed, 0 failed, 0 skipped (baseline 14181 + 2 new tests, no drop). `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - confirmation 1: `the_shipped_swift_idioms_tool_rule_reports_a_file_beside_one_it_declined` (crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs) asserts the failing fixture reports exactly `SWIFT_IDIOMS_FAIL_FIXTURE_FINDINGS` = 42 findings. I ran the shipped script by hand with the real fixtures (`builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl` / `.pass.swift.tmpl`) against the installed swiftformat 0.62.1: fail fixture = 42 findings across 21 distinct rule_ids, pass fixture = 0 findings, exit 0. `every_shipped_idioms_tool_rule_passes_its_fixtures` (doctor's `verify_fixture_contract`) is the automated proof that pass=0/fail>0 holds for the shipped rule. Both tests pass.
    - confirmation 2: `SWIFT_IDIOMS_FOR_EACH` no longer exists anywhere in the repo (grep returns no matches). `SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH` is the replacement constant, used by `the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each` and in `SWIFT_IDIOMS_SUPERSEDED_BULLETS`.
    - confirmation 3: `SWIFT_IDIOMS_LONG_TYPE` now stages all three DON'T spellings the bullet enumerates — `Array<Int>`, `Dictionary<String, Int>`, and `Optional<String>` — confirmed by reading the constant body in idioms_swift.rs.
    - next: ready for review
    task: ^qs32yvp
  timestamp: 2026-08-22T17:29:29.671351+00:00
- actor: claude-code
  id: 01m0n8casjj025qayxjgj612tx
  text: |-
    ### commit — changed
    - evidence: 0182f3bfa54d31182ff26d1869213819622a09f5 fix(validators): restore where-clause bullet, fix forEach probe
    - next: none
  timestamp: 2026-08-22T17:30:31.090342+00:00
- actor: claude-code
  id: 01m0n8ymc82mz8nmrwx947wg3e
  text: |
    ### review — findings

    - evidence: engine `review sha HEAD~1..HEAD` reported 3 files reviewed, 4 excluded (`.kanban/`), findings 0 — the 4 markdown files of the commit are in NEITHER count (reproduction 4 on ^dyad426). Hand measurement on swiftformat 0.62.1 / swiftlint 0.65.0, with the shipped `run:` block extracted verbatim, gives 3 new findings: `builtin/validators/swift/rules/idioms.md:14`, `builtin/validators/code-hygiene/rules/idioms-swift.md:459`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:729`.
    - verified addressed: 4 of the 5 findings of 2026-08-22 11:58. Single-line `forEach` + `if` now reports `preferForLoop`; without the option it is silent. `SWIFT_IDIOMS_FOR_EACH` is gone with no reference left. The `void` treatment is applied. The ownership claim is corrected. The five-row table and the 42/21 and 0 fixture counts reproduce exactly.
    - still open: the restored `where` bullet covers the `filter` chain only. The deleted bullet asked for a `where` clause on the `for` LOOP. Measured: `for thing in things { if thing > 2 { print(thing) } }` reports NOTHING, and that is the shape `preferForLoop`'s own rewrite produces. So the shape the tool walks the author into has no owner.
    - next: widen the `idioms.md` bullet, add the row to the `idioms-swift.md` table, and guard the shape in the test.
  timestamp: 2026-08-22T17:40:30.728828+00:00
- actor: claude-code
  id: 01m0n90mb2csqtzrhww1tkm7e8
  text: |-
    ### finish iteration 2 — findings
    - implement: changed — 7 files; `--single-line-for-each convert` added, `where` half restored, 2 bad probes corrected
    - test: green — cargo nextest run --workspace, 14183 passed, 0 failed, 0 skipped; fmt clean; clippy clean; fixtures verified unmoved by hand against real swiftformat (fail 42/21, pass 0)
    - commit: 0182f3bf
    - review: findings — 3 new; prior findings 1, 3, 4, 5 verified addressed by measurement, prior finding 2 open in substance
    - engine coverage: 3 of 11 files; the 4 markdown files appear in NEITHER the reviewed count nor `skipped_files`. Reproduction 4 on ^dyad426.

    **The new finding is the same class as iteration 1, one shape over — and it is subtle.**

    The deleted bullet read: "Prefer a `for` loop (with a `where` clause when filtering) over `forEach` + `if`". The `where` clause is a requirement on the **`for` loop**. The deleted text named no `filter` chain at all. The restored bullet names the `filter` chain alone.

    Measured on swiftformat 0.62.1:
    - `for thing in things { if thing > 2 { print(thing) } }` → SILENT under the shipped run.
    - `preferForLoop` with `--single-line-for-each convert` rewrites `things.forEach { if $0 > 2 { print($0) } }` into `for thing in things { if thing > 2 { print(thing) } }` — NOT into `for thing in things where thing > 2`.
    - `idioms.md:14` is the only `where`-clause requirement anywhere in `builtin/validators/swift/rules/`.

    So an author who obeys the new `preferForLoop` finding, and applies SwiftFormat's own fix, lands on exactly the shape the deleted bullet forbade — and nothing now asks about it. **The tool's fix walks you into the hole.**

    That is worth naming as a general lesson for the remaining tool-validator work: when a tool AUTOCORRECTS, the prompt rule must cover the shape the tool's output lands on, not only the shape it started from.

    Converging, not stuck: iteration 1 found a deleted bullet with no coverage; iteration 2 found the restoration too narrow. Each round is a strictly narrower slice, not the same finding repeating.
  timestamp: 2026-08-22T17:41:36.226389+00:00
- actor: claude-code
  id: 01m0n9wp8bsbwa2h26ax709s8m
  text: |-
    ### Iteration 3 landed — the `where` half names both walks, and the autocorrect audit

    All eight checkboxes are flipped. The four the iteration-2 reviewer verified by measurement (items 1, 3, 4, 5 of 2026-08-22 11:58) were left unmarked because the review skill gives the marks to the implementer; item 2 and the three new findings close with this change.

    #### 1. `idioms.md:14` — the bullet names the nested-`if` `for` loop

    Written test-first. The new guard was RED for the right reason before the bullet moved:

        `idioms.md` must state `for thing in things { if thing > 2 { print(thing) } }`,
        because no rule of this roster decides it and the probe below would otherwise
        measure a shape nothing asks for

    The bullet now reads:

        - **A loop that filters says `where`.** DON'T:
          `for thing in things { if thing > 2 { print(thing) } }`, and DON'T:
          `things.filter { $0 > 2 }.forEach { thing in print(thing) }`. DO:
          `for thing in things where thing > 2 { print(thing) }`. SwiftFormat's
          `preferForLoop` WRITES the first DON'T as its own fix and never suggests a
          `where` clause, so this half is this rule's alone.

    The front-matter description moved with it — "where clauses over filter chains" became "where clauses over nested ifs and filter chains", because the old wording named the narrower half.

    Measured with the shipped `run:` block extracted verbatim, swiftformat 0.62.1, beside a `.swift-version` of `6.3`:

        for thing in things { if thing > 2 { print(thing) } }        exit=0, ZERO findings
        the same over 6 lines                                        exit=0, ZERO findings
        for thing in things where thing > 2 { print(thing) }         exit=0, ZERO findings   (the DO draws nothing)
        things.filter { $0 > 2 }.forEach { thing in print(thing) }   exit=0, ZERO findings
        things.forEach { if $0 > 2 { print($0) } }                   preferForLoop, line 3

    And the rewrite, read back from the tool:

        $ swiftformat --rules preferForLoop --single-line-for-each convert --quiet Probe.swift
        for thing in things { if thing > 2 { print(thing) } }

    Character for character the new DON'T. The tool's fix walks the author into the hole, and the bullet now stands in the hole.

    #### 2. `idioms-swift.md` — row 6, and line 427 corrected

    - The measurement table gained a sixth row: `for thing in things { if thing > 2 { print(thing) } }` | NO | NO. The five rows above it reproduce unchanged.
    - The requirement restatement is corrected. It now quotes the deleted bullet — "Prefer a `for` loop (with a `where` clause when filtering) over `forEach` + `if`" — and says "write a `where` clause ON THAT LOOP when it filters. The second requirement is about the LOOP; the deleted bullet named no `filter` chain."
    - "Row 5 is the `where` half" became "Rows 5 and 6 are the `where` half", with a paragraph stating that row 6 is what the tool's own fix writes.
    - The roster paragraph said the `preferForLoop` probe holds `forEach` + an `if` "and never a `filter` chain". That named one shape of the half; it now reads "never a walk of the `where` half".

    #### 3. The guard test, widened and renamed

    `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain` guarded the `filter` chain alone while its name read as the whole half. It is now `the_shipped_swift_idioms_tool_rule_decides_no_shape_of_the_where_half`, driven off a table `SWIFT_IDIOMS_WHERE_HALF_SHAPES` of both walks. Each row is held to the same load-bearing pair the old test stated: `idioms.md` must state the walk word for word, the probe must hold that form, and the gate must be SILENT on it. No reference to the old name is left in the repository.

    #### The autocorrect audit — the general lesson, applied to every deleted bullet

    The card asks the same question of each rule that rewrites code: does the tool's FIX produce a form our rules no longer discuss? Each was RUN, not read.

    | tool rule | what the fix WRITES | does a rule still discuss that form? |
    |---|---|---|
    | `typeSugar` | `[Int]`, `[String: Int]`, `String?` | yes — it is the DO the deleted bullet named, and `idioms.md`'s empty-collection bullet AGREES with it |
    | `void` | `-> Void` | yes — `idioms.md` still states "Omit the return clause entirely when it is `Void`. DON'T: `func f() -> Void {}`" |
    | `redundantMemberwiseInit` | a struct with no `init` | no rule forbids it; the tool's own message reads "explicit INTERNAL memberwise initializers", so a `public init` is untouched |
    | `hoistPatternLet` | `case .at(let x, let y)` | yes — it is the DO the deleted bullet named |
    | `preferFinalClasses` | `final class Worker` | yes — `value-semantics.md` still asks whether it should be a `class` at all |
    | `preferForLoop` | `for thing in things { if thing > 2 { print(thing) } }` | **NO, until this change** |
    | `noGuardInTests` | `let value = try XCTUnwrap(source)`, and `func` gains `throws` | yes — it is the DO of the `optionals.md` bullet that stays whole |

    `preferForLoop` was the only hole. But the audit turned up a second defect of the same class in the same file, and the implement skill says a finding names one example of a cause and the cause comes out of the whole file:

    **`void`'s output was stated but not GUARDED.** `idioms-swift.md` claims "Each half the gate misses has a test of its own, named in the section that measures it." That was true of `preferForLoop` and FALSE of `void` — the `void` section named no test, and nothing held `idioms.md` to keeping the omit-the-clause bullet. A future deletion of that bullet would have opened exactly the hole this card has now closed twice, because `-> Void` is what `void`'s fix writes.

    So `the_shipped_swift_idioms_tool_rule_decides_no_void_return_clause` was added, on the same pair. It was verified capable of failing: with `DON'T: `func f() -> Void {}`` temporarily cut out of the bullet it fails by name, and it passes with the bullet whole. Measured: `public static func f() -> Void {}` draws ZERO findings under the full shipped roster.

    The lesson is now stated once in each place that decides a split, rather than repeated:

    - `idioms-swift.md`, in the opening: "The author usually makes it with `swiftformat` itself, so every rule here AUTOCORRECTS. That decides what a split bullet keeps: the prompt rule has to state the shape the tool's FIX lands on, not only the shape the finding started from."
    - `swift/VALIDATOR.md`, on the split rule: "The tool AUTOCORRECTS, so the half that stays names the shape the FIX writes." — 13 tokens, because that file is carried on every validator task.
    - `code-hygiene/VALIDATOR.md`: "Each half stays because the tool's own FIX lands there", with both rewrites named. Its "so `idioms.md` keeps the `filter`-chain half" was the same mis-naming as `idioms-swift.md:427` and is now "the `where` half".

    #### Fixtures, unmoved

    Run by hand with the shipped script against the real fixtures on swiftformat 0.62.1:

    | fixture | findings | distinct rules |
    |---|---|---|
    | `idioms-swift.fail.swift.tmpl` | **42** | **21** |
    | `idioms-swift.pass.swift.tmpl` | **0** | — |

    #### The token saving, recomputed

    `tiktoken` `o200k_base` over the markdown under each front matter, `f79727e2e^` against the working tree:

    | rule | before | after | saved |
    |---|---|---|---|
    | `VALIDATOR.md` | 174 | 424 | -250 |
    | `idioms.md` | 510 | 435 | 75 |
    | `value-semantics.md` | 280 | 242 | 38 |
    | `optionals.md` | 430 | 308 | 122 |
    | `error-handling.md` | 352 | 197 | 155 |
    | `concurrency.md` | 380 | 303 | 77 |
    | `casing.md` | 741 | 741 | 0 |
    | **TOTAL** | **2867** | **2650** | **217** |

    **217 tokens for each validator task, 7.6% of the `swift` set**, down from the 244 / 8.5% iteration 2 reported. The rule bodies alone give up 467; `VALIDATOR.md` takes 250 back for the standing order and the split rule the card asks it to state. The 27-token difference from iteration 2 is 6 tokens for the widened bullet and 13 for the autocorrect clause in the manifest, plus 8 for the widened front-matter description — the price of a requirement no tool decides, and of the rule that stops the next split losing one.

    Verified: `cargo fmt --all -- --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo nextest run --workspace --no-fail-fast` 14184 run / 14184 passed / 0 skipped (baseline 14183 plus the new `void` guard; the `where` test was renamed rather than added).
  timestamp: 2026-08-22T17:56:55.691564+00:00
- actor: claude-code
  id: 01m0n9x5gmqrgv7kzba35p8fe0
  text: |-
    ### implement — changed
    - evidence: 5 files — `builtin/validators/swift/rules/idioms.md`, `builtin/validators/swift/VALIDATOR.md`, `builtin/validators/code-hygiene/rules/idioms-swift.md`, `builtin/validators/code-hygiene/VALIDATOR.md`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`.
    - all 8 checkboxes flipped: the 4 the iteration-2 reviewer verified by measurement, item 2, and the 3 new findings.
    - the defect: the `where` half now names the nested-`if` `for` loop beside the `filter` chain. Measured under the shipped script, `for thing in things { if thing > 2 { print(thing) } }` reports ZERO findings, and `swiftformat --rules preferForLoop --single-line-for-each convert` rewrites `things.forEach { if $0 > 2 { print($0) } }` into exactly that line.
    - `idioms-swift.md` gained row 6 (NO / NO), the requirement restatement now quotes the deleted bullet and says the `where` clause is on the LOOP, and "Row 5 is the `where` half" became "Rows 5 and 6".
    - the guard: `..._decides_no_filtering_for_each_chain` is now `..._decides_no_shape_of_the_where_half`, table-driven over both walks, each held to the silent-gate + stated-in-`idioms.md` pair. Written RED first.
    - autocorrect audit: every rewriting rule was RUN and its FIX output checked against the surviving rules. `preferForLoop` was the only prompt hole. `void` was the second defect of the same class — its output `-> Void` was STATED but not GUARDED, while `idioms-swift.md` claimed every missed half has a test. `..._decides_no_void_return_clause` closes it, verified capable of failing. `typeSugar`, `hoistPatternLet`, `redundantMemberwiseInit`, `preferFinalClasses` and `noGuardInTests` each land on a form a surviving rule still discusses.
    - fixtures unmoved: fail 42 findings / 21 distinct rules, pass 0, measured by hand against the real fixtures on swiftformat 0.62.1.
    - token saving, recomputed: **217 for each validator task, 7.6% of the `swift` set** (was 244 / 8.5%). Rule bodies give up 467; `VALIDATOR.md` takes 250 back.
    - tests: `cargo nextest run --workspace --no-fail-fast` — 14184 run, 14184 passed, 0 skipped. `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - next: `/review`. Task stays in `doing`.
  timestamp: 2026-08-22T17:57:11.316808+00:00
- actor: claude-code
  id: 01m0na68zk9x878htm5rq14zk3
  text: |
    ### test — green
    - evidence: `cargo nextest run --workspace` — 14184 tests run: 14184 passed, 0 failed, 0 skipped (28 slow, 1 leaky). `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean, 0 warnings.
    - baseline was 14183; result is 14184 — matches the implementer's report of one net new test plus one rename (count did not drop).
    - Confirmation 1: `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain` does not exist anywhere in the repo (grep: no matches). `the_shipped_swift_idioms_tool_rule_decides_no_shape_of_the_where_half` exists in `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` and passed in the nextest run.
    - Confirmation 2: `the_shipped_swift_idioms_tool_rule_decides_no_void_return_clause` exists in the same file and passed in the nextest run. It closes the defect where SwiftFormat's `void` fix writes `-> Void`, the surviving bullet's own DON'T.
    - Confirmation 3: ran the shipped `run:` block from `builtin/validators/code-hygiene/rules/idioms-swift.md` directly (not just trusting the test) against the real shipped fixtures (`builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl` and `idioms-swift.pass.swift.tmpl`) with `swiftformat --version` = 0.62.1. Failing fixture: 42 findings, 21 distinct rule ids, exit 0, no stderr. Passing fixture: 0 findings, exit 0, no stderr. Fixtures unmoved and counts match exactly.
    - Confirmation 4: `builtin/validators/swift/rules/idioms.md` literally contains `for thing in things { if thing > 2 { print(thing) } }` in the `where` bullet.
    - next: hand off for review.
  timestamp: 2026-08-22T18:02:09.779577+00:00
- actor: claude-code
  id: 01m0na8h7atsbpztgwwh2h7smj
  text: |-
    ### commit — changed
    - evidence: ae24e8474 fix(validators): cover the shape a tool's fix writes, not just its start shape
    - next: proceed to review
  timestamp: 2026-08-22T18:03:23.754244+00:00
- actor: claude-code
  id: 01m0narka4khdc8cdy07zyarc7
  text: |
    ### review — clean

    Scope: `review sha HEAD~1..HEAD` (commit `ae24e8474`). Iteration 3.

    **Engine coverage, reported apart from the hand measurement.** The engine reported "1 file(s) reviewed, 4 not reviewed" with `findings: 0`. The 1 reviewed file is the `.rs` file. The 4 not reviewed are `.kanban/`, excluded by `.reviewignore`. The **4 markdown files of this commit appear in neither count** — no line in the tally. Those 4 files hold most of the commit, so `findings: 0` is evidence about the Rust file only. This is reproduction 5 on ^dyad426.

    Every measurement below is a hand measurement, made with real swiftformat 0.62.1 and swiftlint 0.65.0, running the `run:` block extracted verbatim from the shipped `builtin/validators/code-hygiene/rules/idioms-swift.md`.

    #### The three findings of 2026-08-22 12:31 — measured verdicts

    1. **name the nested-`if` `for` loop as a DON'T — ADDRESSED.** `idioms.md:14` now reads "**A loop that filters says `where`.**" and holds both DON'Ts.
    2. **add row 6, correct line 427 — ADDRESSED.** Row 6 stands in the table as NO / NO, the text reads "Rows 5 and 6", and the restatement quotes the deleted bullet and says the `where` clause is a requirement ON THAT LOOP.
    3. **guard the nested-`if` `for` loop — ADDRESSED.** The test is table-driven over `SWIFT_IDIOMS_WHERE_HALF_SHAPES` and holds both halves of the pair for each of the two walks.

    All 8 prior checklist items measure as genuinely done.

    #### What this review measured by hand

    The bullet's claim is true:

    | walk | shipped gate |
    |---|---|
    | `for thing in things { if thing > 2 { print(thing) } }` (row 6) | SILENT |
    | `things.filter { $0 > 2 }.forEach { thing in print(thing) }` (row 5) | SILENT |
    | `things.forEach { if $0 > 2 { print($0) } }` (row 1) | reports `preferForLoop` |
    | `public static func f() -> Void {}` | SILENT |
    | `public static func run() -> () {}` | reports `void` |
    | `for thing in things where thing > 2 { print(thing) }` (the DO) | SILENT |
    | `public static func f() {}` (the DO) | SILENT |

    The autocorrect claims hold. `swiftformat --rules preferForLoop --single-line-for-each convert` rewrites row 1 into `for thing in things { if thing > 2 { print(thing) } }` — compared line to line, a character-for-character match with row 6. `swiftformat --rules void` rewrites `-> ()` into `-> Void`, which is the surviving bullet's own DON'T.

    **Claim 4 was performed, not asserted.** All five named rules were run independently, not the two asked for. Each fix output was measured under the shipped gate and read against the surviving prompt rules:

    | rule | what the fix wrote | gate on the fixed file | lands on |
    |---|---|---|---|
    | `typeSugar` | `[Int]`, `[String: Int]`, `String?` | SILENT | the DO of the bullet the rule took |
    | `hoistPatternLet` | `case .at(let x, let y)` | SILENT | the DO — one `let` per case variable |
    | `preferFinalClasses` | `public final class Worker` | SILENT | the DO of the `value-semantics.md` bullet |
    | `redundantMemberwiseInit` | the init removed | SILENT | the DO |
    | `noGuardInTests` | `let value = try #require(source)`, `throws` added | SILENT | the DO `optionals.md` states |

    No fix among the five lands on a shape a surviving rule forbids, and none lands on a shape no rule discusses. `preferForLoop` and `void` were the only gaps, as the commit states.

    Convergence, as a check on the whole defect class: applying the gate's own fix to the failing fixture drops the fixture from 42 findings to **0**. The gate is idempotent on its own output, so no enabled rule walks an author into a shape another enabled rule reports.

    Fixtures unmoved, staged as the guard stages them (as `Judged.swift`, with no `.swift-version`):

    | fixture | findings | distinct rules |
    |---|---|---|
    | `idioms-swift.fail.swift.tmpl` | 42 | 21 |
    | `idioms-swift.pass.swift.tmpl` | 0 | — |

    Both guards were verified RED by mutation, because a guard that cannot bite is the defect class this card kept hitting:

    - Removing `` `func f() -> Void {}` `` from `idioms.md` fails `the_shipped_swift_idioms_tool_rule_decides_no_void_return_clause` at `idioms_swift.rs:834`.
    - Removing the nested-`if` DON'T fails `the_shipped_swift_idioms_tool_rule_decides_no_shape_of_the_where_half` at `idioms_swift.rs:777`.

    The working tree was restored after each mutation. All 13 Swift idioms guard tests pass. `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain` and `SWIFT_IDIOMS_FOR_EACH` are retired with no reference left in `crates/` or `builtin/`.

    #### The same region, examined once more, and found covered

    The multi-line `forEach` + `if` (row 2 of the table) reports `preferForLoop`, and its fix writes a MULTI-LINE `for` loop holding a bare `if`, on which the gate is SILENT. `idioms.md:14` names only the one-line form of that walk.

    This is NOT a finding. The bullet's requirement is its headline — "A loop that filters says `where`" — which is stated at the level of the construct, and the multi-line form is the same construct with different line breaks. The iteration-2 hole was different in kind: there the headline itself read "Filter with a `where` clause, not with a `filter` chain feeding a `forEach`", so it named one construct and missed another. Asking the bullet to enumerate every line-breaking of one construct is not what the earlier findings asked for.

    **On whether the rule is faulty rather than the implementation:** it is not. The three iterations found three distinct gaps — a missing option, a bullet narrower than the half it restored, and the shape the fix writes — each real and each in the same area because that area was under-specified, not because the rule measures the wrong thing. This commit ends the recurrence structurally: both VALIDATOR.md files now state the general lesson at the place that decides a split ("The tool AUTOCORRECTS, so the half that stays names the shape the FIX writes"), and that generalization is what turned up the `void` defect by audit rather than by a fourth review round.

    - evidence: engine 0 findings over 1 of 5 files (4 markdown files uncounted); hand review of the 4 markdown files and the guard file found 0 findings; all 8 prior items verified done
    - next: none — task moves to done
  timestamp: 2026-08-22T18:12:10.180433+00:00
- actor: claude-code
  id: 01m0nasy6ezmz9avww4e4v72sb
  text: |-
    ### finish iteration 3 — clean
    - implement: changed — 5 files; `where` bullet widened to both walks; autocorrect audit found and closed a second defect (`void`)
    - test: green — cargo nextest run --workspace, 14184 passed, 0 failed, 0 skipped; fmt clean; clippy clean
    - commit: ae24e8474
    - review: clean — 0 findings; all 8 prior items verified done by measurement
    - engine coverage: 1 of 5 files; the 4 markdown files in neither count. Reproduction 5 on ^dyad426.
    - result: done in 3 iterations

    **The audit claim was performed, not asserted.** The reviewer ran all five named rewriting rules rather than the two spot-checks requested. Each fix output measured SILENT under the gate and lands on the DO of a surviving rule: `typeSugar` → `[Int]`/`[String: Int]`/`String?`; `hoistPatternLet` → `case .at(let x, let y)`; `preferFinalClasses` → `public final class`; `redundantMemberwiseInit` → init removed; `noGuardInTests` → `let value = try #require(source)`.

    **A structural convergence guarantee was added.** Applying the gate's own fix to the failing fixture drops it from 42 findings to **0**. So no enabled rule walks an author into a shape another enabled rule reports. That is a property of the whole gate, not a per-rule check, and it closes the defect class rather than its instances.

    **Both guards verified RED by mutation.** Removing the DON'T from `idioms.md` fails `..._decides_no_void_return_clause` and `..._decides_no_shape_of_the_where_half`. Working tree restored after each.

    **One measurement subtlety worth keeping.** The fixture constant 42/21 is measured with NO `.swift-version` staged. With `.swift-version` of 6.3, more rules go live and the same fixture yields 62/26. Anyone changing that constant needs to know which staging it assumes.

    **The rule is not faulty — the three iterations found three DISTINCT gaps:** a missing option, a bullet narrower than the half it restored, and the shape the fix writes. The recurrence is ended structurally: the general lesson is now stated in both VALIDATOR.md files, and that generalisation is what found the `void` defect by audit instead of by a fourth round.

    One shape in the same region was found and judged covered rather than a finding: the multi-line `forEach` + `if` whose fix writes a multi-line `for` loop holding a bare `if`. The bullet's headline states the requirement at construct level, so it is named. That judgment is recorded here so a later round does not relitigate it.
  timestamp: 2026-08-22T18:12:54.094013+00:00
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
- 01M0MVM2VZ71SBQ95754S2N0RX
- 01M0MVNQDK2G1J4X2SRT78SQR4
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffae80
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

## Review Findings (2026-08-22 11:58)

> Scope: `review sha HEAD~1..HEAD` (commit `f79727e2e`).
>
> **The engine did not review this commit.** It reported "5 file(s) reviewed, 10 not reviewed" over a 24-file commit and `findings: 0`. The 5 reviewed are the 5 `.rs` files; the 10 excluded are `.kanban/`. All **9 markdown files were silently dropped** — never opened, never accounted for in the tally. The whole substance of this commit is those 9 files, so the engine's `findings: 0` is not evidence of anything. Logged as reproduction 3 on ^dyad426.
>
> The findings below come from a hand review of the 9 markdown files plus the shipped tool invocations, run against real swiftformat 0.62.1 and swiftlint 0.65.0 using the `run:` blocks extracted verbatim from the shipped rule files.

- [x] `builtin/validators/swift/rules/idioms.md` (deleted bullet) `swift/idioms` — the `forEach` bullet was deleted whole, but shipped `preferForLoop` reads only part of it, so the `forEach` + `if` requirement now has no owner in either set. Measured under the shipped `idioms-swift` script on swiftformat 0.62.1: `things.forEach { if $0 > 2 { print($0) } }` — the exact DON'T shape the deleted bullet named — reports ZERO findings, as does `things.forEach { print($0) }`. Root cause: `preferForLoop` has a `--single-line-for-each` option whose default is `ignore`, and the shipped run block passes `--short-optionals always --pattern-let inline --guard-like-if-statements convert` but never `--single-line-for-each convert`. Adding that flag makes the probe report `preferForLoop: Convert functional forEach calls to for loops.` Either add the flag to the run block and re-measure, or restore the bullet and state which shape the tool misses.

- [x] `builtin/validators/swift/rules/idioms.md` (deleted bullet) `swift/idioms` — the "with a `where` clause when filtering" half of the deleted `forEach` bullet has no owner and cannot get one from this tool. `preferForLoop` only converts `forEach` into `for`; it never suggests a `where` clause, and it is silent on the filter-chain shape `things.filter { $0 > 2 }.forEach { thing in print(thing) }` by its own documented design ("Doesn't affect long multiline functional chains" in `swiftformat --rule-info preferForLoop`). No option changes this. This half must stay in `idioms.md`.

- [x] `builtin/validators/swift/VALIDATOR.md` — this commit's own new text states the rule the two findings above break: "A bullet stating ONE requirement the tool reads only partly stays here whole, and says which part the tool misses." The `void` bullet in this same commit follows that rule exactly — it is split, the surviving half is written as its own bullet, and `idioms-swift.md` carries a reported/NO measurement table for the three `Void` declaration shapes. The `forEach` bullet got neither the split nor the measurement. Apply the `void` treatment to `forEach`.

- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:675` `tool_rules/shipped` — the `SWIFT_IDIOMS_FOR_EACH` probe stages the multi-line `values.forEach { value in print(value) }`, which is the one `forEach` shape the shipped run does report, so `the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` passes without ever exercising the `forEach` + `if` shape the deleted bullet named. The guard's mechanism is sound — `verify_superseded_swift_bullet` asserts the tool-rule name appears in what the run actually REPORTED, not that the rule is enabled — but this probe body cannot prove the claim it is cited for. Stage the shape the deleted bullet named.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:99` — the added paragraph claims the guard test "holds each of the six taken rows to both halves of its own claim: the rule reports a file holding that ONE defect". That is not true of the `preferForLoop` row: its probe holds a `forEach` shape the bullet did not name, while the shape the bullet did name is unreported. Correct the claim, and mark the `preferForLoop` ownership-table row for the half it actually decides, as the `void` row is marked "HALF of that bullet".

### What was verified by hand, and passed

Every other claim in this commit was probed against the real tool and holds.

Deleted bullets — all FIRE, with the literal diagnostic observed:

| bullet | tool | rule | verdict |
|---|---|---|---|
| shorthand type sugar | swiftformat | `typeSugar` | FIRES on all three of `Array<Int>`, `Optional<String>`, `Dictionary<String, Int>` |
| no redundant memberwise init | swiftformat | `redundantMemberwiseInit` | FIRES |
| `Void` not `()` (the `()` half) | swiftformat | `void` | FIRES on `-> ()` and `(Int) -> ()` |
| own `let` per case variable | swiftformat | `hoistPatternLet` | FIRES |
| mark classes `final` | swiftformat | `preferFinalClasses` | FIRES |
| `for` loop over `forEach` | swiftformat | `preferForLoop` | **PARTIAL — see findings above** |
| no force unwrap | swiftlint | `force_unwrapping` | FIRES |
| no IUO | swiftlint | `implicitly_unwrapped_optional` | FIRES |
| no `try!` | swiftlint | `force_try` | FIRES |
| no `as!` | swiftlint | `force_cast` | FIRES |
| `@unchecked Sendable` invariant | swiftlint | `no_unchecked_sendable` | FIRES, and the documented `// swiftlint:disable:next` escape hatch genuinely suppresses it |

The three deliberately-KEPT bullets were each confirmed SILENT, so each is correctly the sole owner of its requirement:

- `noGuardInTests` is silent on shorthand `guard let source else { return }`. Control: the long form `guard let value = source else` fires twice on the same file, so the probe is sound and the shorthand gap is real.
- `void` is silent on `func typed() -> Void {}`, both in isolation and under the full shipped roster. Removing the clause is SwiftFormat's separate `redundantVoidReturnType`, which this roster does not name.
- Both shipped gates are silent on all four casing shapes — `MAX_RETRY_COUNT`, `kMaximumRetries`, `strName`, `bIsValid`. `identifier_name` is not in the shipped `only_rules` roster; run in isolation it would fire on `MAX_RETRY_COUNT` only, and as a charset check ("should only contain alphanumeric and other allowed characters"), not a casing check. It stays silent on the other three. Enabling it would not cover the bullet. Keeping `casing.md` whole is correct, and the diff confirms the file is unchanged.

`builtin/validators/swift/VALIDATOR.md` rewrite is accurate as to the bundle as it now stands:

- "Ten bullets that stood here are theirs now, and half of an eleventh" — counted against the diff: 10 whole bullets deleted across `idioms.md`, `value-semantics.md`, `optionals.md`, `error-handling.md`, `concurrency.md`, plus the `()` half of the `Void` bullet. Accurate.
- "Neither declares a `supersedes` key" — confirmed; both tool rules declare none and each carries a section explaining why.
- "there are no engine probes on this side" — confirmed; no rule in `builtin/validators/swift/rules/` carries a `run:` frontmatter key. The only textual match is prose in `access-control.md` ("runs no caller probe").

## Review Findings (2026-08-22 12:31)

> Scope: `review sha HEAD~1..HEAD` (commit `0182f3bf`). Iteration 2.
>
> **Engine coverage, reported apart from the hand measurement.** The engine reported "3 file(s) reviewed, 4 not reviewed" over an 11-file commit, with `findings: 0`. The 3 reviewed are the 3 `.rs` files. The 4 not reviewed are `.kanban/`, excluded by `.reviewignore`. The **4 markdown files of this commit appear in neither count** — dropped with no line in the tally. Those 4 files hold the substance of the commit, so `findings: 0` is evidence about the Rust files only. This is reproduction 4 on ^dyad426.
>
> Every measurement below is a hand measurement. Each one ran the `run:` block extracted verbatim from the shipped `builtin/validators/code-hygiene/rules/idioms-swift.md`, over `Probe.swift` beside a `.swift-version` of `6.3`, on swiftformat 0.62.1 and swiftlint 0.65.0.

### The five findings of 2026-08-22 11:58 — measured verdicts

The marks in the section above stay as the implementer left them. This section states what each item measures to now.

1. **single-line `forEach` + `if` reports — ADDRESSED.** `--single-line-for-each convert` stands on the shipped command line. Measured: `things.forEach { if $0 > 2 { print($0) } }` reports `preferForLoop`. Without the option the same file is SILENT.
2. **the `where` half stays in `idioms.md` — ADDRESSED IN THE LETTER, OPEN IN SUBSTANCE.** A `where` bullet is back in `idioms.md`, and the gap it claims is real: the filter chain measures SILENT with and without the option. The bullet is narrower than the half it restores. See the new finding below.
3. **apply the `void` treatment to `forEach` — ADDRESSED.** The bullet is split, the surviving half is its own bullet, and `idioms-swift.md` carries a five-row reported/NO measurement table.
4. **stage the shape the bullet named — ADDRESSED.** `SWIFT_IDIOMS_FOR_EACH` is gone, with no reference left anywhere in `crates/`. `SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH` holds the named shape and reports.
5. **correct the ownership claim — ADDRESSED.** The paragraph now reads "WRITTEN IN THE SHAPE THE BULLET NAMED", and the `preferForLoop` table row is marked "HALF of that bullet", as the `void` row is.

### New findings

- [x] `builtin/validators/swift/rules/idioms.md:14` `swift/idioms` — the restored bullet states the `where` half more narrowly than the deleted bullet stated it, so one shape of that half still has no owner in either set. The deleted bullet read "**Prefer a `for` loop (with a `where` clause when filtering) over `forEach` + `if`**". There the `where` clause is a requirement on the `for` LOOP, and the deleted text named no `filter` chain at all. The restored bullet names one DON'T only — `things.filter { $0 > 2 }.forEach { thing in print(thing) }` — and leaves out the `for` loop that filters with a nested `if`. Measured under the shipped script: `for thing in things { if thing > 2 { print(thing) } }` reports NOTHING. That shape is the one `preferForLoop` itself writes — `swiftformat --rules preferForLoop --single-line-for-each convert` rewrites `things.forEach { if $0 > 2 { print($0) } }` into `for thing in things { if thing > 2 { print(thing) } }`, not into `for thing in things where thing > 2`. So an author who obeys the new `preferForLoop` finding, and applies SwiftFormat's own fix, lands on a shape the deleted bullet forbade and no rule of either set now asks about. `idioms.md:14` is the only `where`-clause requirement in the whole of `builtin/validators/swift/rules/`. Name the nested-`if` `for` loop as a DON'T in that bullet, beside the `filter` chain.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:459` `code-hygiene/idioms-swift` — "Row 5 is the `where` half, and NO option reaches it" names row 5 as the whole of the `where` half, and row 5 is one shape of it. None of the five rows holds a `for` loop, so the section measures the half it claims to characterize only in part. Line 427 carries the same cause: "write a `where` clause rather than a `filter` chain when the loop filters" restates the deleted bullet's "(with a `where` clause when filtering)" as a rule about `filter` chains, which the deleted text never said. Add the row `for thing in things { if thing > 2 { print(thing) } }` — measured NO in both columns — state that `preferForLoop`'s own rewrite produces that row, and correct line 427 to the requirement the deleted bullet stated.

- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:729` `tool_rules/shipped` — `the_shipped_swift_idioms_tool_rule_decides_no_filtering_for_each_chain` holds the `filter` chain alone, so it guards one shape of the `where` half and the test name reads as if it guards the half. Its own doc comment states the load-bearing pair: the gate must stay silent, and `idioms.md` must state the shape word for word. Apply that pair to the nested-`if` `for` loop too — stage it, assert the gate is SILENT on it, and assert `idioms.md` states it — or the shape SwiftFormat's fix produces has a guard nowhere.

### What was measured and holds

The five-row table added to `idioms-swift.md` reproduces exactly, row for row and column for column:

| the walk | shipped run | without `--single-line-for-each convert` |
|---|---|---|
| `things.forEach { if $0 > 2 { print($0) } }`, one line | `preferForLoop` | SILENT |
| the same `forEach` and `if` over five lines | `preferForLoop` | `preferForLoop` |
| `things.forEach { print($0) }`, one line | `preferForLoop` | SILENT |
| `values.forEach { value in print(value) }`, three lines | `preferForLoop` | `preferForLoop` |
| `things.filter { $0 > 2 }.forEach { thing in print(thing) }` | SILENT | SILENT |

Fixtures are unmoved, with the option and without it:

| fixture | shipped run | without the option |
|---|---|---|
| `idioms-swift.fail.swift.tmpl` | 42 findings, 21 distinct rules | 42 findings, 21 distinct rules |
| `idioms-swift.pass.swift.tmpl` | 0 | 0 |

Claim 5 — "every other probe in both guard files stages the shape its bullet names" — was audited against the pre-deletion bullet text of `f79727e2e~1`. Every probe holds its bullet's own DON'T:

| probe | the DON'T the bullet named | the probe body | verdict |
|---|---|---|---|
| `SWIFT_IDIOMS_LONG_TYPE` | `Array<Int>`, `Dictionary<Key, Value>`, `Optional<String>` | all three, one per declaration | matches; `typeSugar` reports each on its own line |
| `SWIFT_IDIOMS_PAREN_RETURN` | `func f() -> ()` | `public static func run() -> () {}` | matches the half the gate decides |
| `SWIFT_IDIOMS_REDUNDANT_INIT` | a memberwise init identical to the synthesized one | `init(count: Int) { self.count = count }` | matches |
| `SWIFT_IDIOMS_HOISTED_LET` | `case let .point(x, y)` | `case let .at(x, y)` | matches |
| `SWIFT_IDIOMS_OPEN_CLASS` | a class not designed for subclassing, unmarked | `public class Worker` | matches |
| `SWIFT_DISALLOWED_FORCE_UNWRAP` | `let name = user.name!` | `name!` on a `String?` | matches |
| `SWIFT_DISALLOWED_IMPLICITLY_UNWRAPPED` | `var session: URLSession!` | `public var name: String!` | matches |
| `SWIFT_DISALLOWED_FORCE_TRY` | `let data = try! Data(contentsOf: url)` | `try! body()` | matches |
| `SWIFT_DISALLOWED_FORCE_CAST` | `segue.destination as! DetailVC` | `value as! Int` | matches |
| `SWIFT_UNCHECKED_SENDABLE_PLAIN` | `final class Counter: @unchecked Sendable` with no guard | `public final class Box: @unchecked Sendable` | matches |

The counts each file states are correct against the diff:

- `swift/VALIDATOR.md` "NINE bullets ... theirs whole, and HALF of each of two more" — counted: 3 whole from `idioms.md`, 1 from `value-semantics.md`, 2 from `optionals.md`, 2 from `error-handling.md`, 1 from `concurrency.md` = 9; halves are `void` and `preferForLoop` = 2. Accurate.
- `idioms-swift.md` "FOUR ... whole ... TWO more it decides in half ... The seventh stays there whole" — accurate over the seven swiftformat-decided bullets.
- `idioms-swift.md` "`idioms.md` still states the empty-collection declaration, the omit-the-clause half, the `where` half and the type-name repetition" — `idioms.md` holds exactly those four bullets.
- The doc comments in `builtin/mod.rs` and `tool_rules/tests.rs` were moved to the same count and say the same thing.

Two claims about SwiftFormat itself were read back from the tool and hold:

- `swiftformat --rule-info preferForLoop` states `--single-line-for-each … "ignore" (default) or "convert"`, and states "Doesn't affect long multiline functional chains".
- `preferForLoop` never writes a `where` clause. Its rewrite of both the one-line and the five-line `forEach` + `if` is a `for` loop with the `if` nested inside.