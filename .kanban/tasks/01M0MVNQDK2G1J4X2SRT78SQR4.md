---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0n331qqkg6zhgnddbxxk1q0
  text: |-
    ## Conflict 2 probe — `--property-types inferred`, recorded verbatim

    The probe ran against the installed swiftformat. The output below is copied from
    the terminal, not written from memory.

    ```
    $ swiftformat --version
    0.62.1

    $ cat Probe.swift
    public struct Holder {
        public var items: [Int] = []
        public var ids: Set<String> = []
    }

    $ swiftformat --lint --quiet --reporter json --cache ignore --rules propertyTypes --property-types inferred Probe.swift
    Source input did not pass lint check.
    [
      {
        "file" : "/tmp/claude-501/-Users-wballard-github-swissarmyhammer-swissarmyhammer/817c01c3-bcce-422d-9cc4-d73e733381f9/scratchpad/probe2/Probe.swift",
        "line" : 2,
        "reason" : "Convert property declarations to use inferred types (let foo = Foo()) or explicit types (let foo: Foo = .init()).",
        "rule_id" : "propertyTypes"
      },
      {
        "file" : "/tmp/claude-501/-Users-wballard-github-swissarmyhammer-swissarmyhammer/817c01c3-bcce-422d-9cc4-d73e733381f9/scratchpad/probe2/Probe.swift",
        "line" : 3,
        "reason" : "Convert property declarations to use inferred types (let foo = Foo()) or explicit types (let foo: Foo = .init()).",
        "rule_id" : "propertyTypes"
      }
    ](exit 1)

    $ swiftformat --quiet --cache ignore --rules propertyTypes --property-types inferred Probe.swift && cat Probe.swift
    public struct Holder {
        public var items = [Int]()
        public var ids = Set<String>()
    }
    ```

    An earlier run of the same probe carried a dictionary and a string as well. It
    reported the dictionary and left the string alone:

    ```
    $ cat Probe.swift
    public struct Holder {
        public var items: [Int] = []
        public var ids: Set<String> = []
        public var table: [String: Int] = [:]
        public var name: String = "probe"
    }
    $ swiftformat --quiet --cache ignore --rules propertyTypes --property-types inferred Rewritten.swift
    $ cat Rewritten.swift
    public struct Holder {
        public var items = [Int]()
        public var ids = Set<String>()
        public var table = [String: Int]()
        public var name: String = "probe"
    }
    ```

    READING: swiftformat REWRITES `var items: [Int] = []` to `var items = [Int]()`
    and `var ids: Set<String> = []` to `var ids = Set<String>()`. Those are the two
    forms `builtin/validators/swift/rules/idioms.md` names as its DON'T, word for
    word. The card's UNVERIFIED guess — that the empty literal names no type so the
    rule would leave it alone — is WRONG. The conflict is real.

    By the card's own branch: the option loses. `propertyTypes` and
    `--property-types inferred` stay OUT of the `idioms-swift` roster, and the
    reason is written into the tool rule.
  timestamp: 2026-08-22T15:58:04.023766+00:00
- actor: claude-code
  id: 01m0n33mw431cde8ed1hpmeytx
  text: |-
    ## Conflict 1 probe — `noGuardInTests`, recorded verbatim

    swiftformat 0.62.1 lists both names, each off by default, so `--rules` must name
    them:

    ```
    $ swiftformat --rules | grep -E 'propertyTypes|noGuardInTests|redundantType'
     noGuardInTests (disabled)
     propertyTypes (disabled)
     redundantType
     redundantTypedThrows
    ```

    ### What the rule reports, over one XCTest suite

    Probe source `Matrix.swift`, run as
    `swiftformat --lint --quiet --reporter json --cache ignore --rules noGuardInTests Matrix.swift`.
    Reported lines: 12, 16, 18, 24, 26, 27, 32. Every finding carries the same
    reason:

    ```
    "Convert guard statements and trailing if statements in unit tests to try #require(...) or try XCTUnwrap(...) / XCTAssert(...)."
    ```

    Mapped back to the source:

    | the declaration | line | reported |
    |---|---|---|
    | `guard let value else { return }` (shorthand, no `=`) in a `throws` test | 6 | NO |
    | `guard let value = source else { return }` in a `throws` test | 12 | YES |
    | the same in a non-`throws` test | 16, 18 | YES — the `func` line too, because the fix adds `throws` |
    | `guard let value = source else { XCTFail("missing"); return }` | 24, 26, 27 | YES |
    | `guard 1 == 1 else { return }` | 32 | YES |
    | `guard let source else { return 0 }` inside a `private` helper of the suite | 43 | NO |

    The rewrite it would make, for the same file:

    ```
        func testBindingGuardLet() throws {
            let source: Int? = 1
            let value = try XCTUnwrap(source)
            XCTAssertEqual(value, 1)
        }

        func testBindingGuardLetNonThrowing() throws {
            let source: Int? = 1
            let value = try XCTUnwrap(source)
            XCTAssertEqual(value, 1)
        }

        func testGuardWithFail() throws {
            let source: Int? = 1
            let value = try XCTUnwrap(source, "missing")
            XCTAssertEqual(value, 1)
        }

        func testBooleanGuard() {
            XCTAssert(1 == 1)
            XCTAssertTrue(true)
        }

        private func helper(_ source: Int?) -> Int {
            guard let source else { return 0 }
            return source
        }
    ```

    ### Swift Testing suites

    ```
    $ swiftformat --quiet --cache ignore --rules noGuardInTests --guard-like-if-statements convert ST.swift && cat ST.swift
    import Testing

    struct SwiftTestingProbe {
        @Test func theProbeUnwraps() throws {
            let source: Int? = 1
            let value = try #require(source)
            #expect(value == 1)
        }

        @Test func theProbeGuardsABool() throws {
            try #require(1 == 1)
            #expect(true)
        }
    }
    ```

    ### It reads a test, not a file name

    Production code stays untouched, and so does a production method whose NAME says
    test:

    ```
    $ cat NotATest.swift
    public struct Runner {
        public func testConnection(_ source: Int?) -> Int {
            guard let value = source else { return 0 }
            return value
        }
    }
    $ swiftformat --lint --quiet --reporter json --cache ignore --rules noGuardInTests --guard-like-if-statements convert NotATest.swift
    [

    ] exit=0
    ```

    ### What `--guard-like-if-statements convert` adds

    The option is the only difference between these two runs over `TrailingIf.swift`:

    ```
    $ swiftformat --lint ... --rules noGuardInTests TrailingIf.swift
    [

    ] exit=0
    $ swiftformat --lint ... --rules noGuardInTests --guard-like-if-statements convert TrailingIf.swift
    [ three findings, lines 4, 6, 7 ] exit=1
    ```

    The reported shape is a trailing `if let value = source { ...assertions... }`
    that wraps the whole rest of a test. `if flag { ... }` with a plain Bool is NOT
    reported. So the option decides ONE more shape, and it is the same defect the
    guard half decides: the assertions inside never run when the binding fails, and
    the test passes anyway.
  timestamp: 2026-08-22T15:58:23.620495+00:00
- actor: claude-code
  id: 01m0n3zvz5de2phnx58k32pybt
  text: |-
    ## The two decisions, and where each is written down

    ### Conflict 1 — `noGuardInTests`: ENABLED, and `optionals.md` gained the carve-out

    I evaluated the card's recommendation rather than taking it, and it holds. A
    `guard` that returns in a test takes the test out BEFORE its assertions run, so
    a broken program reads as a pass. That is a defect, not a style. `optionals.md`
    already knew tests are different — it sanctions an implicitly unwrapped optional
    for a fixture set in `setUp()` — so the carve-out is the thinking the file
    already had, applied where it was missing.

    Two facts the card did not have, and both matter:

    1. **The rule reads a TEST, not a file name.** A production method named
       `testConnection` holding a `guard` draws NOTHING, and a `private` helper
       inside a test suite keeps its `guard`. So there is no false positive on
       production code, and the prompt bullet is scoped the same way.
    2. **The rule MISSES the shorthand.** `guard let value else { return }` is not
       reported; only `guard let value = source else { return }` is. So the tool
       decides part of the bullet and the prompt half carries the rest. This is
       recorded in the "Why this rule supersedes nothing" section, because it is
       exactly why `supersedes` stays empty.

    `--guard-like-if-statements convert` is enabled with it. The option is the only
    difference between two measured runs: a trailing `if let` that wraps the
    assertions of a test reports three lines with it and nothing without it. Same
    defect — the assertions never run when the binding fails. An `if let` a test
    asserts AFTER does not trail the body and stays silent, so the option decides
    one shape rather than every `if`.

    Written down in: `builtin/validators/swift/rules/optionals.md` (a new bullet),
    `builtin/validators/code-hygiene/rules/idioms-swift.md` (a new section, "The two
    Airbnb options that contradicted a prompt rule"), and a cross-reference in
    `builtin/validators/swift/rules/error-handling.md`, whose force-cast bullet also
    recommends `guard`.

    ### Conflict 2 — `--property-types inferred`: NOT enabled

    The probe (recorded verbatim in an earlier comment) settles it. swiftformat
    REWRITES `var items: [Int] = []` into `var items = [Int]()`, which is the DON'T
    `idioms.md` names word for word. The card's guess that the empty literal names
    no type, so the rule would leave it alone, is WRONG. The conflict is real, and
    by the card's own branch the option loses.

    Extra measurement that made the test design better: the two directions of the
    option are caught by DIFFERENT halves.

    | the option | the DO of `idioms.md` | the DON'T |
    |---|---|---|
    | `--property-types inferred` | 2 findings | silent |
    | `--property-types explicit` | silent | 2 findings |

    So one test holding the DO clean would NOT catch `explicit`. Two tests ship, one
    for each direction.

    Written down in: `builtin/validators/code-hygiene/rules/idioms-swift.md` (same
    new section) and `builtin/validators/swift/rules/idioms.md` (the empty-collection
    bullet now names the rule and option that must not be added).

    ## The tests, and the RED I watched

    - `the_shipped_swift_idioms_tool_rule_agrees_with_the_swift_prompt_rules` — the
      card's acceptance test. It reads the BODIES of `optionals.md` and `idioms.md`
      off the shipped loader and holds every form its probes are written in to
      standing in them, so the probe is the prompt rules' own answer rather than a
      shape the test invented. Then one Swift file written that way draws ZERO
      findings from the tool, and the shape the prompt rules refuse draws
      `noGuardInTests`. The probe stages a `.swift-version`, so a clean run cannot
      be one the version gate bought.
    - `the_shipped_swift_idioms_tool_rule_reads_a_trailing_if_in_a_test` — the guard
      on `--guard-like-if-statements convert` standing on the command line.
    - `the_shipped_swift_idioms_tool_rule_decides_no_empty_collection_declaration` —
      the guard on `propertyTypes` staying out, in the `explicit` direction.

    RED was watched, not assumed. With `noGuardInTests` and the option removed from
    the shipped script:

    ```
    FAIL the_shipped_swift_idioms_tool_rule_reads_a_trailing_if_in_a_test
    FAIL the_shipped_swift_idioms_tool_rule_agrees_with_the_swift_prompt_rules
      `noGuardInTests` must report the `guard` `optionals.md` refuses in a test; the run reported []
    ```

    With `propertyTypes` and `--property-types inferred` added:

    ```
    FAIL the_shipped_swift_idioms_tool_rule_agrees_with_the_swift_prompt_rules
      Swift written the way the prompt rules ask for must draw no finding from this gate; the run reported ["propertyTypes", "propertyTypes"]
    PASS the_shipped_swift_idioms_tool_rule_decides_no_empty_collection_declaration
    ```

    That PASS is what showed the two directions need two tests, and the doc now
    states which test catches which.

    ## Counts that moved, and every place that stated them

    The roster went 28 → 29 names, 26 → 27 enabled, and the failing fixture 37 → 42
    findings. The "25 of the 26 enabled rules" line in the tool rule was ALREADY
    wrong before this change — the doctor run states no `.swift-version`, so five
    version-gated rules plus `validateTestCases` stay silent and the real answer is
    21 of 27. It now reads what was measured.

    The "six bullets across two prompt rules" claim stood in THREE places. All three
    now read seven across three: the tool rule body,
    `crates/.../review/tool_rules/tests.rs` (`SHIPPED_IDIOMS_RULES`), and
    `crates/.../builtin/mod.rs` (`CODE_HYGIENE_IDIOMS_TOOL_RULES`).

    `supersedes` stays empty on both Swift tool rules, as the card required.
  timestamp: 2026-08-22T16:13:48.389967+00:00
- actor: claude-code
  id: 01m0n4282vp1jrramav4p43b36
  text: |-
    ### implement — changed

    - evidence: 9 files.
      - `builtin/validators/code-hygiene/rules/idioms-swift.md` — roster gains `noGuardInTests`, command line gains `--guard-like-if-statements convert`, new section "The two Airbnb options that contradicted a prompt rule" carries both decisions with their measurements, counts restated (28→29 named, 26→27 enabled, 37→42 fixture findings, and the pre-existing wrong "25 of the 26" corrected to the measured 21 of 27).
      - `builtin/validators/swift/rules/optionals.md` — new bullet: never `guard` in a test, unwrap with an assertion.
      - `builtin/validators/swift/rules/idioms.md` — the empty-collection bullet names `propertyTypes` / `--property-types inferred` as the rule and option that must not be added.
      - `builtin/validators/swift/rules/error-handling.md` — its force-cast bullet also recommends `guard`, so it now points at `optionals.md` for the test case.
      - `builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl` and `.pass.swift.tmpl` — the guard shape and the trailing-`if let` shape, each in the form its rule reports and the form it asks for.
      - `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` — roster size 29, fixture findings 42, three new acceptance tests.
      - `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs` and `crates/swissarmyhammer-validators/src/builtin/mod.rs` — the "six bullets across two prompt rules" claim, which stood in three places counting the rule body.
    - `cargo fmt --all -- --check`: clean.
    - `cargo clippy --workspace --all-targets -- -D warnings`: zero errors, zero warnings.
    - `cargo nextest run -p swissarmyhammer-validators -p mirdan --no-fail-fast`: `Summary [ 70.531s] 1340 tests run: 1340 passed (27 slow), 0 skipped`.
    - RED watched for all three new tests before GREEN; the exact failure messages are in the comment above.
    - next: `/review`.
  timestamp: 2026-08-22T16:15:06.331479+00:00
- actor: claude-code
  id: 01m0n4a7ac8859k92h9s2z18gr
  text: |-
    ### commit — changed
    - evidence: c097c1548 fix(validators): enable noGuardInTests, keep house style for property types
    - next: local checkpoint only. No push.
  timestamp: 2026-08-22T16:19:27.692827+00:00
- actor: claude-code
  id: 01m0n4q5zd388fkt6sae0aqe9x
  text: |-
    ### review — clean

    - scope: `review sha HEAD~1..HEAD`, which is c097c1548.
    - engine counts: 0 findings, 0 confirmed, 0 refuted, 7 attempted, 0 failed.
    - The engine read the three Rust files. It did not count the four prompt-rule
      markdown files. I read those four files and I measured each number in them
      again with swiftformat 0.62.1.

    **Every measurement in the commit is correct. I made each one again:**

    - The roster holds 29 names. swiftformat 0.62.1 knows 27 of them.
    - The failing fixture gives 42 findings, which carry 21 of the 27 enabled
      rules. The six silent rules are `preferCountWhere`, `opaqueGenericParameters`,
      `environmentEntry`, `genericExtensions`, `conditionalAssignment` and
      `validateTestCases`. This agrees with the rule body word for word. The old
      "25 of the 26" line was wrong. The new "21 of 27" line is correct.
    - The table of the bullets holds seven rows: five for `idioms.md`, one for
      `value-semantics.md`, one for `optionals.md`. The "seven across three" count
      is correct in the rule body, in `tests.rs` and in `builtin/mod.rs`.

    **Decision 1. The carve-out agrees with the tool.** I measured
    `noGuardInTests` again over one XCTest suite. It reports the binding `guard`
    and the boolean `guard`. It stays silent on the shorthand
    `guard let value else`, on a `private` helper of the suite, and on a
    production method with the name `testConnection`. The bullet in `optionals.md`
    speaks about a test, not about a file name, so it agrees at each of these
    points. The bullet is wider than the tool at one point only: the shorthand.
    This is not a contradiction, because the tool asks for nothing there. The
    rule body gives this gap as the reason that `supersedes` stays empty. The two
    halves cannot fight.

    **Decision 2. The probe supports the decision.** swiftformat 0.62.1 with
    `--property-types inferred` writes `var items = [Int]()`,
    `var ids = Set<String>()` and `var table = [String: Int]()`. It does not change
    `var name: String = "probe"`. These are the DON'T forms of `idioms.md`, word
    for word. The conflict is real, and the option loses.

    **The two new guards are real guards. I measured each one:**

    - Trailing `if let` in a test: 0 findings without
      `--guard-like-if-statements convert`, 3 findings with it. An `if let` that a
      test asserts after gives 0 findings with the option. So the test fails if the
      option comes off the command line, and the negative half separates the two
      shapes.
    - Empty collection: the DON'T form gives 2 findings under `explicit` and 0
      under `inferred`. The DO form gives 2 findings under `inferred` and 0 under
      `explicit`. So the two tests catch the two directions, as the rule body
      states. Neither direction can enter without a test failure.

    **The coupling of the acceptance test is sound.** The test reads the shipped
    bodies off the loader, so an edit to a prompt rule cannot leave the probe
    measuring a form that nothing ships. The coupled strings are the DO and DON'T
    code forms, not prose, so a change to them is a change to the rule itself. The
    test reads presence, not polarity, but the tool half of the same test closes
    that gap: the answer file must report nothing and the refusal file must report
    `noGuardInTests`, and both come from real swiftformat.

    - next: none. Task moves to `done`.
  timestamp: 2026-08-22T16:26:32.301852+00:00
- actor: claude-code
  id: 01m0n4rxn9dp22ygxm6xey4j9z
  text: |-
    ### finish iteration 1 — clean
    - implement: changed — 9 files; both conflicts decided by measurement
    - test: green — cargo nextest run --workspace, 14179 passed, 0 failed, 0 skipped (14176 baseline + 3 new tests); fmt clean; clippy clean
    - commit: c097c1548
    - review: clean — 0 findings, 7 attempted, 0 failed; every documented measurement independently reproduced against swiftformat 0.62.1
    - result: done in one iteration

    **Decision 1 — `noGuardInTests` + `--guard-like-if-statements convert` ENABLED**, with a test carve-out added to `optionals.md`. A `guard` that returns in a test takes the test out before its assertions run, so a broken program reads as a pass.

    Re-measured by the reviewer over an XCTest suite: the rule reports the binding `guard` and the boolean `guard`, and stays silent on the shorthand `guard let value else`, on a `private` helper inside the suite, and on a production method named `testConnection`. The carve-out keys on a test, not a filename, so it agrees at every one of those points. It is wider than the tool at exactly one place — the shorthand — and the tool asks for nothing there, so the two halves cannot fight. That gap is the documented reason `supersedes` stays empty.

    **Decision 2 — `--property-types inferred` NOT enabled.** The probe disproved the card's guess. swiftformat rewrites `items`, `ids` and `table` into `[Int]()`, `Set<String>()`, `[String: Int]()` — the DON'T forms of `idioms.md` verbatim — and leaves `name: String` alone. House style stands.

    **Two wrong measurements in already-committed work were corrected and re-verified:**
    - The `idioms-swift` rule said "25 of the 26 enabled rules". True figure is **21 of 27**. Roster is 29 names, 27 known to swiftformat 0.62.1; the failing fixture yields 42 findings carrying 21 distinct rules. The six silent ones are exactly `preferCountWhere`, `opaqueGenericParameters`, `environmentEntry`, `genericExtensions`, `conditionalAssignment`, `validateTestCases`.
    - "Six bullets across two prompt rules" was wrong and had propagated into three files. True figure is **seven bullets across three rules** — 5 in `idioms.md`, 1 in `value-semantics.md`, 1 in `optionals.md`.

    **Both new guards discriminate.** The trailing-`if` probe gives 0 findings without `--guard-like-if-statements convert` and 3 with it. The property guards catch opposite directions: the DON'T form reports 2 under `explicit` and 0 under `inferred`; the DO form reports 2 under `inferred` and 0 under `explicit`.

    One known limit, recorded not hidden: the acceptance test's `body.contains(form)` check proves presence, not polarity. A bullet inverted while keeping the same code string would pass that half. The real-swiftformat assertions in the same test cover it.
  timestamp: 2026-08-22T16:27:29.321358+00:00
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffad80
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