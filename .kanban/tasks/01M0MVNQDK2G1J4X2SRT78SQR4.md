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
depends_on:
- 01M0MVKKJN6S08JSCDH3FX5BNY
position_column: doing
position_ordinal: '8280'
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