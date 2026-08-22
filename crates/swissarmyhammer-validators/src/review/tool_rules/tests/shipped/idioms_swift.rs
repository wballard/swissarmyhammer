//! Acceptance tests for the shipped `idioms-swift` tool rule.
//!
//! Each test drives the SHIPPED script over a probe repository and reads what
//! the real swiftformat reported.
//!
//! Four answers stand apart from every other Swift rule of this set, and one
//! test holds each. The script INTERSECTS its own rule roster with
//! `swiftformat --rules`, because a name SwiftFormat does not know breaks the
//! whole run. It hands swiftformat ONE path for each run, because one refusing
//! path otherwise costs the run every finding it made. Two enabled rules read
//! the same declaration, and only the first of them reports it. And five
//! enabled rules read the Swift language version, which the project states in
//! a `.swift-version` file.
//!
//! One more test stands apart because of what it protects rather than what it
//! measures. This gate took six requirements out of the Swift prompt rules —
//! four whole bullets, and half of each of two more — and a requirement
//! deleted without a tool that reads it is one the set states nowhere. The
//! last test in this file holds each of those six to being reported by its
//! tool, over Swift written in the SHAPE the bullet named, AND stated by no
//! prompt rule. The half of each split bullet the gate misses has a test of
//! its own beside it.

use super::*;

/// Acceptance: every shipped idioms tool rule passes its fixture pair in
/// doctor, and supersedes nothing.
///
/// The pass fixture is the load-bearing half. It holds the same declarations
/// the fail fixture holds, each written in the form its rule asks for, so a
/// SwiftFormat release that changed what one rule asks for makes the pair
/// fail and takes the rule out of the review rather than moving a finding
/// without a word.
#[test]
#[serial_test::serial(cwd)]
fn every_shipped_idioms_tool_rule_passes_its_fixtures() {
    verify_shipped_tool_rules_pass_fixtures(SHIPPED_IDIOMS_RULES, IDIOMS_RULE_KIND);
}

/// The line of the shipped script that opens its rule roster.
///
/// The roster is the argument list of one `printf`, so the words between this
/// head and the pipe under it ARE the names the gate enables. Reading them off
/// the shipped script is what makes the guard below hold the rule that ships
/// rather than a copy this file wrote.
const SWIFT_IDIOMS_ROSTER_HEAD: &str = r"printf '%s\n' ";

/// The word that closes the roster: the pipe the `printf` writes into.
const SWIFT_IDIOMS_ROSTER_END: &str = "|";

/// How many rule names the shipped roster holds.
///
/// Twenty-nine, which is Airbnb's list narrowed to the rules that decide an
/// idiom. The count is the assertion that a name added or dropped later
/// reaches this guard rather than moving the gate without a word.
const SWIFT_IDIOMS_ROSTER_SIZE: usize = 29;

/// The roster names no released SwiftFormat knows.
///
/// Both stand in Airbnb's `airbnb.swiftformat` on `master`, which tracks
/// SwiftFormat's `main` branch. Measured against SwiftFormat 0.62.1, the
/// newest release: `swiftformat --rules` lists 153 rules and neither of these
/// two is among them. The script intersects its roster with that answer, so
/// each takes effect the day SwiftFormat ships it, with no edit.
const SWIFT_IDIOMS_UNRELEASED_RULES: &[&str] = &["preferLazyMap", "ifExpressions"];

/// The command that asks swiftformat which rules it knows.
const SWIFT_FORMAT_TOOL: &str = "swiftformat";

/// The flag that makes swiftformat write its whole rule list.
const SWIFT_FORMAT_RULES_FLAG: &str = "--rules";

/// What swiftformat writes after the name of a rule it leaves off by default.
///
/// The script strips it, because `--rules` enables a rule whatever its default
/// is: measured on 0.62.1, `isEmpty` reads ` isEmpty (disabled)` in the list
/// and reports when the run names it.
const SWIFT_FORMAT_DISABLED_MARK: &str = "(disabled)";

/// The rule names the shipped `idioms-swift` script enables, read off the
/// script itself.
///
/// The words stand between [`SWIFT_IDIOMS_ROSTER_HEAD`] and the pipe under it,
/// over as many lines as the script writes, each continuation line joined with
/// a trailing backslash.
fn swift_idioms_roster(script: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut inside = false;

    for line in script.lines() {
        let trimmed = line.trim();
        let rest = match trimmed.strip_prefix(SWIFT_IDIOMS_ROSTER_HEAD) {
            Some(rest) => {
                inside = true;
                rest
            }
            None if inside => trimmed,
            None => continue,
        };
        for word in rest
            .trim_end_matches(SWIFT_IDIOMS_LINE_JOIN)
            .split_whitespace()
        {
            if word == SWIFT_IDIOMS_ROSTER_END {
                return names;
            }
            names.push(word.to_string());
        }
    }

    names
}

/// Every rule name the installed swiftformat knows.
///
/// The reading matches the script's own: the `(disabled)` mark comes off, and
/// the surrounding space with it.
fn swift_format_known_rules() -> Vec<String> {
    let listed = std::process::Command::new(SWIFT_FORMAT_TOOL)
        .arg(SWIFT_FORMAT_RULES_FLAG)
        .output()
        .expect("the installed swiftformat must write its rule list");

    String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(|line| {
            line.replace(SWIFT_FORMAT_DISABLED_MARK, "")
                .trim()
                .to_string()
        })
        .filter(|name| !name.is_empty())
        .collect()
}

/// Acceptance: every rule the shipped script names is one the installed
/// swiftformat knows, but for the two no release carries yet.
///
/// The script enables the INTERSECTION of its roster with
/// `swiftformat --rules`, because `--unknown-rules ignore` moves nothing on
/// the command line and an unknown name breaks the whole run at status 70.
/// That intersection is what makes the roster forward-compatible, and it is
/// also what would swallow a misspelling: a name no swiftformat knows drops
/// out and the gate stops measuring it, silently.
///
/// This is the guard on that. Every name but the two
/// [`SWIFT_IDIOMS_UNRELEASED_RULES`] records must stand in swiftformat's own
/// list, so a misspelling and a rule SwiftFormat renames each fail by name.
#[test]
fn the_shipped_swift_idioms_tool_rule_names_only_rules_swiftformat_knows() {
    let loader = builtin_loader();
    require_tool_installed(&loader, SWIFT_PROJECT_TYPES, SWIFT_IDIOMS_RULE);
    let shipped = required_shipped_tool_rule(&loader, SWIFT_IDIOMS_RULE);

    let roster = swift_idioms_roster(&shipped.script);
    assert_eq!(
        roster.len(),
        SWIFT_IDIOMS_ROSTER_SIZE,
        "the shipped roster must name {SWIFT_IDIOMS_ROSTER_SIZE} rules; it names {roster:?}"
    );

    let known = swift_format_known_rules();
    assert!(
        !known.is_empty(),
        "`{SWIFT_FORMAT_TOOL} {SWIFT_FORMAT_RULES_FLAG}` wrote no rule name, so the \
         comparison below holds nothing and this test cannot fail"
    );

    let unknown: Vec<&String> = roster
        .iter()
        .filter(|name| !SWIFT_IDIOMS_UNRELEASED_RULES.contains(&name.as_str()))
        .filter(|name| !known.contains(name))
        .collect();

    assert!(
        unknown.is_empty(),
        "the shipped script names these rules and the installed swiftformat knows none \
         of them, so the intersection drops each one and the gate stops measuring it: \
         {unknown:?}"
    );
}

/// The probe repository stages no file as a `(path, text)` pair.
///
/// Each probe below stages the shipped fixture through `prepare` instead, so
/// the run measures the SHIPPED bytes rather than a copy this file wrote.
const NO_STAGED_TEXT: &[(&str, &str)] = &[];

/// The shipped failing fixture, as [`copy_shipped_fixture`] asks for it.
const SWIFT_IDIOMS_FAIL_FIXTURE: &str = "idioms-swift.fail.swift";

/// Where a probe stages the failing fixture.
const SWIFT_IDIOMS_JUDGED_PATH: &str = "Judged.swift";

/// A path the probe repository holds no file at.
const SWIFT_IDIOMS_ABSENT_PATH: &str = "Absent.swift";

/// How many findings the shipped failing fixture reports.
///
/// Measured with the shipped script and SwiftFormat 0.62.1, over a probe
/// repository that states no `.swift-version`: 42 findings, at exit 0. The
/// count states that a refusing path beside the fixture costs the run nothing,
/// because the run reports the same 42 either way.
const SWIFT_IDIOMS_FAIL_FIXTURE_FINDINGS: usize = 42;

/// What the marked line the script writes for a path it could not judge opens
/// with, after the engine takes the `sah-diagnostic:` marker off.
const SWIFT_IDIOMS_DECLINED_HEAD: &str = "idioms-swift found no file at";

/// Drives the shipped script over the failing fixture beside `files`, and
/// answers the findings and the declined items of that one run.
///
/// The fixture is staged through `prepare` so the run reads the bytes the set
/// ships. `files` is the argument list the run carries, which is where a probe
/// names a path the repository holds no file at.
fn swift_idioms_run(files: &[&str]) -> (Vec<String>, Vec<String>) {
    let loader = builtin_loader();
    require_tool_installed(&loader, SWIFT_PROJECT_TYPES, SWIFT_IDIOMS_RULE);

    let stage_fixture = |repo: &Path| {
        copy_shipped_fixture(
            &loader,
            repo,
            SWIFT_IDIOMS_FAIL_FIXTURE,
            SWIFT_IDIOMS_JUDGED_PATH,
        );
    };
    let staging = ShippedStaging {
        staged: NO_STAGED_TEXT,
        links: NO_PROBE_LINKS,
        outside: NO_PROBE_OUTSIDE,
        prepare: &stage_fixture,
        restore: &stage_nothing_more,
    };

    let run = drive_shipped_script_whole(&loader, SWIFT_IDIOMS_RULE, &staging, files);
    let outcome = run
        .outcome
        .expect("the shipped Swift idioms script must judge the probe files and exit 0");

    (
        finding_rows(&outcome, &run.repo_root),
        script_diagnostics(&outcome, &run.repo_root),
    )
}

/// Acceptance: a path the shipped Swift idioms rule cannot judge costs the run
/// that one path, and no finding of the files it did judge.
///
/// swiftformat throws away the WHOLE run for one refusing path. Measured on
/// 0.62.1 over a file holding one finding beside a path that holds no file:
/// status 70, ZERO findings, and the healthy file judged in none of it. That
/// is the answer `builtin/validators/README.md` refuses — "a nonzero exit
/// fails the WHOLE run, so one unjudged path throws away every finding the run
/// did make".
///
/// The script therefore hands swiftformat one path for each run. The two
/// halves of this test are both load-bearing: the run keeps every finding of
/// the file it judged, AND it states the path it declined on the marked
/// stderr channel. A run that lost either half is a run that answered for a
/// file it never read.
#[test]
fn the_shipped_swift_idioms_tool_rule_reports_a_file_beside_one_it_declined() {
    let (judged, _) = swift_idioms_run(&[SWIFT_IDIOMS_JUDGED_PATH]);
    assert_eq!(
        judged.len(),
        SWIFT_IDIOMS_FAIL_FIXTURE_FINDINGS,
        "the shipped failing fixture must report \
         {SWIFT_IDIOMS_FAIL_FIXTURE_FINDINGS} findings on its own; it reported {judged:?}"
    );

    let (beside, declined) =
        swift_idioms_run(&[SWIFT_IDIOMS_ABSENT_PATH, SWIFT_IDIOMS_JUDGED_PATH]);

    assert_eq!(
        beside, judged,
        "a path the run cannot judge must cost that path alone; the run beside \
         `{SWIFT_IDIOMS_ABSENT_PATH}` reported another finding list than the run without it"
    );
    assert_eq!(
        declined.len(),
        1,
        "the run must state the one path it declined; it stated {declined:?}"
    );
    assert!(
        declined[0].starts_with(SWIFT_IDIOMS_DECLINED_HEAD)
            && declined[0].contains(SWIFT_IDIOMS_ABSENT_PATH),
        "the marked line must name the path it declined; it reads `{}`",
        declined[0]
    );
}

/// Where a probe stages the Swift file it measures a rule set over.
const SWIFT_IDIOMS_PROBE_PATH: &str = "Probe.swift";

/// A test suite holding one internal method whose NAME says test and whose
/// attribute does not.
///
/// `testSuiteAccessControl` reads it as a member that should be `private`, and
/// `validateTestCases` reads it as a test that should carry `@Test`. The two
/// rules read the same declaration.
const SWIFT_IDIOMS_TEST_COLLISION: &str = concat!(
    "import Testing\n\n",
    "struct ProbeTests {\n",
    "    @Test func `the probe holds`() {\n",
    "        #expect(true)\n",
    "    }\n\n",
    "    func testAnother() {\n",
    "        #expect(true)\n",
    "    }\n",
    "}\n",
);

/// The rule that reports the declaration of [`SWIFT_IDIOMS_TEST_COLLISION`].
const SWIFT_IDIOMS_REPORTING_RULE: &str = "testSuiteAccessControl";

/// The rule that reads the same declaration and never reports it.
const SWIFT_IDIOMS_SILENT_RULE: &str = "validateTestCases";

/// A `filter { }.count`, which `preferCountWhere` reports, and a constrained
/// generic parameter, which `opaqueGenericParameters` reports.
///
/// Both rules read the Swift language version, so both stay silent when the
/// project states none.
const SWIFT_IDIOMS_VERSION_GATED: &str = concat!(
    "public enum Readings {\n",
    "    public static func overLimit(_ values: [Int]) -> Int {\n",
    "        values.filter { $0 > 10 }.count\n",
    "    }\n",
    "}\n\n",
    "public func handle<T: Collection>(_ value: T) {\n",
    "    print(value.count)\n",
    "}\n",
);

/// The rules [`SWIFT_IDIOMS_VERSION_GATED`] reports only when the project
/// states its Swift language version.
const SWIFT_IDIOMS_VERSION_GATED_RULES: &[&str] = &["preferCountWhere", "opaqueGenericParameters"];

/// Drives the shipped script over `source` staged at
/// [`SWIFT_IDIOMS_PROBE_PATH`], beside `support`, and answers the rule name of
/// each finding it reported.
fn swift_idioms_reporting_rules(source: &str, support: &[(&str, &str)]) -> Vec<String> {
    swift_gate_reporting_rules(SWIFT_IDIOMS_RULE, SWIFT_IDIOMS_PROBE_PATH, source, support)
}

/// Acceptance: the shipped Swift idioms rule reports the rule swiftformat
/// reports first, and states the one it therefore never reaches.
///
/// `testSuiteAccessControl` and `validateTestCases` read the SAME declaration,
/// and in a run holding both, only the first of them reports it. Measured on
/// SwiftFormat 0.62.1 over three shapes, `validateTestCases` alone reports one
/// finding on each, and the pair reports `testSuiteAccessControl` on each.
///
/// The rule enables both, because the roster it takes names both and because a
/// SwiftFormat that separates them later needs no edit. This test is what
/// makes that measured rather than assumed: a release that changed the order
/// fails here rather than moving a finding without a word.
#[test]
fn the_shipped_swift_idioms_tool_rule_reads_the_rule_swiftformat_reports_first() {
    let reported = swift_idioms_reporting_rules(SWIFT_IDIOMS_TEST_COLLISION, NO_SUPPORT_FILES);

    assert!(
        reported.contains(&SWIFT_IDIOMS_REPORTING_RULE.to_string()),
        "`{SWIFT_IDIOMS_REPORTING_RULE}` must report the internal test-named method; \
         the run reported {reported:?}"
    );
    assert!(
        !reported.contains(&SWIFT_IDIOMS_SILENT_RULE.to_string()),
        "`{SWIFT_IDIOMS_SILENT_RULE}` reads the same declaration and \
         `{SWIFT_IDIOMS_REPORTING_RULE}` reports it first, so a run holding both must \
         report it once; the run reported {reported:?}"
    );
}

/// Acceptance: the shipped Swift idioms rule reads the Swift language version
/// out of the project's own `.swift-version`, and asks for no API the project
/// may lack without one.
///
/// Five rules of the roster read that version. The run passes no
/// `--swift-version` of its own, because a project that never stated one would
/// then be told to write `values.count(where:)`, which needs Swift 6.0 — a
/// finding its own toolchain cannot satisfy.
///
/// The two halves are both load-bearing. Without the file the version-gated
/// rules stay silent, and with it they report, so the file is the only
/// difference between the two runs.
#[test]
fn the_shipped_swift_idioms_tool_rule_reads_the_project_swift_version() {
    let without = swift_idioms_reporting_rules(SWIFT_IDIOMS_VERSION_GATED, NO_SUPPORT_FILES);
    let with = swift_idioms_reporting_rules(
        SWIFT_IDIOMS_VERSION_GATED,
        &[(SWIFT_VERSION_PATH, SWIFT_PROBE_VERSION)],
    );

    for rule in SWIFT_IDIOMS_VERSION_GATED_RULES {
        assert!(
            !without.contains(&(*rule).to_string()),
            "`{rule}` reads the Swift language version, so it must stay silent for a \
             project that states none; the run reported {without:?}"
        );
        assert!(
            with.contains(&(*rule).to_string()),
            "`{rule}` must report beside a `{SWIFT_VERSION_PATH}` of \
             {SWIFT_PROBE_VERSION:?}; the run reported {with:?}"
        );
    }
}

/// A Swift file written the way the two Swift prompt rules ASK for.
///
/// Every declaration is a DO one of them states: the empty-collection
/// properties carry the type annotation `idioms.md` requires, and the test
/// unwraps with `#require` rather than with the `guard` `optionals.md` now
/// refuses in a test.
const SWIFT_PROMPT_RULE_ANSWER: &str = concat!(
    "import Testing\n\n",
    "public struct Holder {\n",
    "    public var items: [Int] = []\n",
    "    public var ids: Set<String> = []\n",
    "}\n\n",
    "struct HolderTests {\n",
    "    @Test func `the holder holds its readings`() throws {\n",
    "        let holder = Holder()\n",
    "        let value = try #require(holder.items.first)\n",
    "        #expect(value == 1)\n",
    "    }\n",
    "}\n",
);

/// The same suite written the way `optionals.md` states NOT to: a `guard`
/// that returns takes the test out before its assertion runs.
const SWIFT_PROMPT_RULE_REFUSAL: &str = concat!(
    "import Testing\n\n",
    "struct HolderTests {\n",
    "    @Test func `the holder holds its readings`() {\n",
    "        let source: Int? = 1\n",
    "        guard let value = source else { return }\n",
    "        #expect(value == 1)\n",
    "    }\n",
    "}\n",
);

/// The rule that reports the `guard` of [`SWIFT_PROMPT_RULE_REFUSAL`].
const SWIFT_NO_GUARD_IN_TESTS_RULE: &str = "noGuardInTests";

/// Each form the probes above are written in, with the prompt rule that
/// states it.
///
/// The pairs are what make [`SWIFT_PROMPT_RULE_ANSWER`] and
/// [`SWIFT_PROMPT_RULE_REFUSAL`] the PROMPT RULES' own answer rather than a
/// shape this file invented: every form must stand, word for word, in the body
/// of the rule that names it.
const SWIFT_PROMPT_RULE_FORMS: &[(&str, &str)] = &[
    (SWIFT_IDIOMS_PROMPT_RULE, "var items: [Int] = []"),
    (SWIFT_IDIOMS_PROMPT_RULE, "var ids: Set<String> = []"),
    (SWIFT_OPTIONALS_PROMPT_RULE, "try #require("),
    (
        SWIFT_OPTIONALS_PROMPT_RULE,
        "guard let value = source else { return }",
    ),
];

/// Acceptance: the two halves agree — Swift written the way the prompt rules
/// ask for draws no finding from this gate, and the shape they refuse does.
///
/// A tool and a prompt rule that disagree produce churn on every review round,
/// so agreement is a fact to measure rather than one to assume. Both halves
/// are load-bearing. The answer file must report NOTHING, or the gate fights
/// the prompt rule the author is reading; the refusal file must report
/// `noGuardInTests`, or the carve-out `optionals.md` states is a bullet no
/// tool backs.
///
/// The probe stages a `.swift-version`, so every enabled rule is live and a
/// clean run cannot be one the version gate bought.
///
/// Each form the two probes are written in is held to standing in the body of
/// the prompt rule that states it. Without that, an edit to either prompt rule
/// would leave this test measuring a house style nothing ships.
#[test]
fn the_shipped_swift_idioms_tool_rule_agrees_with_the_swift_prompt_rules() {
    let loader = builtin_loader();

    for (rule, form) in SWIFT_PROMPT_RULE_FORMS {
        let body = swift_prompt_rule_body(&loader, rule);
        assert!(
            body.contains(form),
            "`{rule}.md` must state `{form}`, or the probes below measure a form no \
             prompt rule asks for"
        );
        assert!(
            SWIFT_PROMPT_RULE_ANSWER.contains(form) || SWIFT_PROMPT_RULE_REFUSAL.contains(form),
            "`{form}` stands in `{rule}.md` and in neither probe, so nothing measures it"
        );
    }

    let answered = swift_idioms_reporting_rules(SWIFT_PROMPT_RULE_ANSWER, SWIFT_VERSION_SUPPORT);
    assert!(
        answered.is_empty(),
        "Swift written the way the prompt rules ask for must draw no finding from this \
         gate; the run reported {answered:?}"
    );

    let refused = swift_idioms_reporting_rules(SWIFT_PROMPT_RULE_REFUSAL, SWIFT_VERSION_SUPPORT);
    assert!(
        refused.contains(&SWIFT_NO_GUARD_IN_TESTS_RULE.to_string()),
        "`{SWIFT_NO_GUARD_IN_TESTS_RULE}` must report the `guard` `optionals.md` refuses \
         in a test; the run reported {refused:?}"
    );
}

/// A test whose assertion stands inside an `if let` that trails the body.
const SWIFT_IDIOMS_TRAILING_IF: &str = concat!(
    "import Testing\n\n",
    "struct TrailingTests {\n",
    "    @Test func `the reading stands`() {\n",
    "        let source: Int? = 1\n",
    "        if let value = source {\n",
    "            #expect(value == 1)\n",
    "        }\n",
    "    }\n",
    "}\n",
);

/// The same suite with one assertion written after the `if let`, so the `if`
/// no longer trails the body.
const SWIFT_IDIOMS_UNTRAILED_IF: &str = concat!(
    "import Testing\n\n",
    "struct TrailingTests {\n",
    "    @Test func `the reading stands`() {\n",
    "        let source: Int? = 1\n",
    "        if let value = source {\n",
    "            #expect(value == 1)\n",
    "        }\n",
    "        #expect(source != nil)\n",
    "    }\n",
    "}\n",
);

/// Acceptance: the shipped Swift idioms rule reads a trailing `if let` in a
/// test, which is the shape `--guard-like-if-statements convert` decides.
///
/// The option is what puts that shape in reach: measured on SwiftFormat
/// 0.62.1 over the trailing probe, `noGuardInTests` alone reports NOTHING and
/// the same rule under `--guard-like-if-statements convert` reports three
/// lines. So this test is the guard on the option standing on the shipped
/// command line — a run that dropped it goes quiet here rather than losing a
/// shape without a word.
///
/// The two halves are both load-bearing. The trailing `if` must report,
/// because the assertions inside it never run when the binding fails and the
/// test passes anyway. The untrailed `if` must NOT, because an `if` a test
/// asserts after is a branch rather than an early exit. The two probes differ
/// in that one line and in nothing else.
#[test]
fn the_shipped_swift_idioms_tool_rule_reads_a_trailing_if_in_a_test() {
    let trailing = swift_idioms_reporting_rules(SWIFT_IDIOMS_TRAILING_IF, NO_SUPPORT_FILES);
    assert!(
        trailing.contains(&SWIFT_NO_GUARD_IN_TESTS_RULE.to_string()),
        "`{SWIFT_NO_GUARD_IN_TESTS_RULE}` must report a test whose assertion stands \
         inside a trailing `if let`; the run reported {trailing:?}"
    );

    let untrailed = swift_idioms_reporting_rules(SWIFT_IDIOMS_UNTRAILED_IF, NO_SUPPORT_FILES);
    assert!(
        !untrailed.contains(&SWIFT_NO_GUARD_IN_TESTS_RULE.to_string()),
        "an `if let` a test asserts AFTER does not trail the body, so \
         `{SWIFT_NO_GUARD_IN_TESTS_RULE}` must stay silent; the run reported {untrailed:?}"
    );
}

/// The empty-collection form `idioms.md` names as its DON'T.
///
/// It differs from the two properties of [`SWIFT_PROMPT_RULE_ANSWER`] in that
/// each annotated literal is written as a constructor call instead.
const SWIFT_IDIOMS_INFERRED_PROPERTY: &str = concat!(
    "public struct Holder {\n",
    "    public var items = [Int]()\n",
    "    public var ids = Set<String>()\n",
    "}\n",
);

/// Acceptance: the gate decides NEITHER form of an empty-collection
/// declaration, so `idioms.md` owns that bullet alone.
///
/// SwiftFormat's `propertyTypes` rule under `--property-types inferred` is
/// Airbnb's answer here, and it rewrites the DO of `idioms.md` into its DON'T
/// — measured on 0.62.1, `var items: [Int] = []` becomes
/// `var items = [Int]()` and `var ids: Set<String> = []` becomes
/// `var ids = Set<String>()`. Enabling it would set the tool against the
/// prompt rule on every review round, so the roster names neither the rule nor
/// the option, and the rule body carries the measurement.
///
/// This test and the one above are what make that decision fail loudly, and
/// each catches one direction of the option. Measured on 0.62.1 over the two
/// forms:
///
/// | the option | the DO of `idioms.md` | the DON'T |
/// |---|---|---|
/// | `--property-types inferred` | 2 findings | silent |
/// | `--property-types explicit` | silent | 2 findings |
///
/// So `inferred` fails the test above, on the DO, and `explicit` fails this
/// one, on the DON'T. Neither direction can be added without a test naming it.
#[test]
fn the_shipped_swift_idioms_tool_rule_decides_no_empty_collection_declaration() {
    let reported =
        swift_idioms_reporting_rules(SWIFT_IDIOMS_INFERRED_PROPERTY, SWIFT_VERSION_SUPPORT);

    assert!(
        reported.is_empty(),
        "the gate must report neither form of an empty-collection declaration, because \
         `idioms.md` decides that bullet and `propertyTypes` would decide it the other \
         way; the run reported {reported:?}"
    );
}

/// The rule that converts a `forEach` call into a `for` loop.
const SWIFT_PREFER_FOR_LOOP_RULE: &str = "preferForLoop";

/// A `forEach` whose closure holds an `if`, written on ONE line.
///
/// This is the shape the deleted `idioms.md` bullet named as its DON'T:
/// `forEach` + `if` where the author needs control flow, and `forEach` can
/// `break`, `continue` or `return` out of nothing.
const SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH: &str = concat!(
    "public enum Looping {\n",
    "    public static func walk(_ things: [Int]) {\n",
    "        things.forEach { if $0 > 2 { print($0) } }\n",
    "    }\n",
    "}\n",
);

/// The same walk written as a `filter` chain feeding a `forEach`, which is
/// the shape a `for … where` clause replaces.
///
/// The line stands, character for character, in the `idioms.md` bullet that
/// states the `where` half.
const SWIFT_IDIOMS_FILTER_CHAIN: &str = concat!(
    "public enum Filtering {\n",
    "    public static func walk(_ things: [Int]) {\n",
    "        things.filter { $0 > 2 }.forEach { thing in print(thing) }\n",
    "    }\n",
    "}\n",
);

/// The DON'T of the `idioms.md` bullet that keeps the `where` half, word for
/// word as [`SWIFT_IDIOMS_FILTER_CHAIN`] writes it.
const SWIFT_IDIOMS_FILTER_CHAIN_FORM: &str =
    "things.filter { $0 > 2 }.forEach { thing in print(thing) }";

/// The same walk written as a `for` loop that filters with a nested `if`.
///
/// This is the line `preferForLoop` itself WRITES. Measured on 0.62.1,
/// `swiftformat --rules preferForLoop --single-line-for-each convert` rewrites
/// [`SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH`] into exactly this, and never into a
/// `where` clause. So the author who takes the finding
/// [`the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each`] holds
/// the gate to, and applies the tool's own fix, lands here.
const SWIFT_IDIOMS_NESTED_IF_FOR_LOOP: &str = concat!(
    "public enum Nesting {\n",
    "    public static func walk(_ things: [Int]) {\n",
    "        for thing in things { if thing > 2 { print(thing) } }\n",
    "    }\n",
    "}\n",
);

/// The other DON'T of the `idioms.md` bullet that keeps the `where` half, word
/// for word as [`SWIFT_IDIOMS_NESTED_IF_FOR_LOOP`] writes it.
const SWIFT_IDIOMS_NESTED_IF_FOR_LOOP_FORM: &str =
    "for thing in things { if thing > 2 { print(thing) } }";

/// Every shape of the `where` half `idioms.md` keeps, beside the form the
/// bullet writes it in.
///
/// The half is a requirement on the LOOP — filter with a `where` clause — and
/// two walks break it. A `filter` chain feeding a `forEach` never becomes a
/// loop at all, and a `for` loop that filters with a nested `if` is a loop that
/// says no `where`. A table holding one of them would guard one shape and read
/// as if it guarded the half.
const SWIFT_IDIOMS_WHERE_HALF_SHAPES: &[(&str, &str)] = &[
    (SWIFT_IDIOMS_FILTER_CHAIN, SWIFT_IDIOMS_FILTER_CHAIN_FORM),
    (
        SWIFT_IDIOMS_NESTED_IF_FOR_LOOP,
        SWIFT_IDIOMS_NESTED_IF_FOR_LOOP_FORM,
    ),
];

/// Acceptance: the shipped Swift idioms rule reads a single-line `forEach`,
/// which is the shape `--single-line-for-each convert` decides.
///
/// The option is what puts that shape in reach, and its default is the other
/// way: `swiftformat --rule-info preferForLoop` states
/// `--single-line-for-each … "ignore" (default) or "convert"`. Measured on
/// SwiftFormat 0.62.1 under the shipped script, over the probe below:
/// without the option the run reports NOTHING, and with it the run reports
/// `preferForLoop`. So this test is the guard on the option standing on the
/// shipped command line — a run that dropped it goes quiet here rather than
/// losing a shape without a word.
///
/// The shape matters more than the option. `forEach` + `if` written on one
/// line is exactly what the deleted `idioms.md` bullet named, so a run that
/// stays silent here leaves that bullet with no owner in either set.
#[test]
fn the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each() {
    let reported =
        swift_idioms_reporting_rules(SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH, SWIFT_VERSION_SUPPORT);

    assert!(
        reported.contains(&SWIFT_PREFER_FOR_LOOP_RULE.to_string()),
        "`{SWIFT_PREFER_FOR_LOOP_RULE}` must report a single-line `forEach` holding an \
         `if`, which is the shape the deleted `idioms.md` bullet named; the run \
         reported {reported:?}"
    );
}

/// Acceptance: the gate decides NO shape of the `where` half, so `idioms.md`
/// owns that half of the bullet alone.
///
/// `preferForLoop` converts a `forEach` into a `for` loop and never suggests a
/// `where` clause, and SwiftFormat states the limit in its own rule
/// information: "Doesn't affect long multiline functional chains". Measured on
/// 0.62.1 under the shipped script, and under the same script with
/// `--single-line-for-each convert`, both walks below report NOTHING either
/// way. No option reaches either one.
///
/// The nested-`if` `for` loop is the shape the tool's own fix WRITES, so it is
/// the one an author reaches by obeying the finding
/// [`the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each`] holds
/// the gate to. A guard that measured the `filter` chain alone would leave the
/// tool free to walk the author into a shape no rule of either set asks about.
///
/// Both halves of each row are load-bearing. The gate must stay silent, or the
/// bullet has two owners and every review round produces churn. And
/// `idioms.md` must state the walk word for word, or the probe measures a
/// shape no prompt rule asks for.
#[test]
fn the_shipped_swift_idioms_tool_rule_decides_no_shape_of_the_where_half() {
    let loader = builtin_loader();
    let body = swift_prompt_rule_body(&loader, SWIFT_IDIOMS_PROMPT_RULE);

    for (probe, form) in SWIFT_IDIOMS_WHERE_HALF_SHAPES {
        assert!(
            body.contains(form),
            "`{SWIFT_IDIOMS_PROMPT_RULE}.md` must state `{form}`, because no rule of this \
             roster decides it and the probe below would otherwise measure a shape nothing \
             asks for"
        );
        assert!(
            probe.contains(form),
            "the probe must hold the form `{SWIFT_IDIOMS_PROMPT_RULE}.md` states, or the two \
             measure different shapes"
        );

        let reported = swift_idioms_reporting_rules(probe, SWIFT_VERSION_SUPPORT);
        assert!(
            reported.is_empty(),
            "the gate must report no `{form}`, because \
             `{SWIFT_IDIOMS_PROMPT_RULE}.md` decides that half and \
             `{SWIFT_PREFER_FOR_LOOP_RULE}` never suggests a `where` clause; the run \
             reported {reported:?}"
        );
    }
}

/// A function whose return clause names `Void` where it could name nothing.
///
/// This is the line `void` itself WRITES. Measured on 0.62.1,
/// `swiftformat --rules void` rewrites [`SWIFT_IDIOMS_PAREN_RETURN`] into
/// exactly this and stops there, so the author who takes the `void` finding
/// and applies the tool's own fix lands here.
const SWIFT_IDIOMS_VOID_RETURN: &str = concat!(
    "public enum Typed {\n",
    "    public static func f() -> Void {}\n",
    "}\n",
);

/// The DON'T of the `idioms.md` bullet that keeps the omit-the-clause half,
/// word for word as [`SWIFT_IDIOMS_VOID_RETURN`] writes it.
const SWIFT_IDIOMS_VOID_RETURN_FORM: &str = "func f() -> Void {}";

/// Acceptance: the gate decides no `Void` return clause, so `idioms.md` owns
/// the omit-the-clause half of that bullet alone.
///
/// `void` rewrites `()` INTO `Void` and stops there. Removing the clause is
/// SwiftFormat's separate `redundantVoidReturnType` rule, which this roster
/// does not name, so the shape the fix lands on is the shape the surviving
/// half forbids. Measured on 0.62.1 under the shipped script, the probe below
/// reports NOTHING.
///
/// Both halves are load-bearing, and they are the same pair the `where` half
/// stands on: the gate must stay silent, or the bullet has two owners, and
/// `idioms.md` must state the declaration word for word, or the probe measures
/// a shape no prompt rule asks for.
#[test]
fn the_shipped_swift_idioms_tool_rule_decides_no_void_return_clause() {
    let loader = builtin_loader();
    let body = swift_prompt_rule_body(&loader, SWIFT_IDIOMS_PROMPT_RULE);

    assert!(
        body.contains(SWIFT_IDIOMS_VOID_RETURN_FORM),
        "`{SWIFT_IDIOMS_PROMPT_RULE}.md` must state `{SWIFT_IDIOMS_VOID_RETURN_FORM}`, \
         because no rule of this roster decides it and the probe below would otherwise \
         measure a shape nothing asks for"
    );
    assert!(
        SWIFT_IDIOMS_VOID_RETURN.contains(SWIFT_IDIOMS_VOID_RETURN_FORM),
        "the probe must hold the form `{SWIFT_IDIOMS_PROMPT_RULE}.md` states, or the two \
         measure different shapes"
    );

    let reported = swift_idioms_reporting_rules(SWIFT_IDIOMS_VOID_RETURN, SWIFT_VERSION_SUPPORT);

    assert!(
        reported.is_empty(),
        "the gate must report no `Void` return clause, because \
         `{SWIFT_IDIOMS_PROMPT_RULE}.md` decides that half and this roster names no \
         `redundantVoidReturnType`; the run reported {reported:?}"
    );
}

/// One declaration for each long spelling the deleted `idioms.md` bullet
/// named as its DON'T.
///
/// The bullet enumerated three — `Array<Int>`, `Dictionary<Key, Value>` and
/// `Optional<String>` — so a probe holding one of them would prove a third of
/// it. Measured on 0.62.1 under the shipped script, `typeSugar` reports each of
/// the three on its own line.
const SWIFT_IDIOMS_LONG_TYPE: &str = concat!(
    "public enum Sugar {\n",
    "    public static func read(_ values: Array<Int>) -> Int {\n",
    "        values.count\n",
    "    }\n\n",
    "    public static func table(_ pairs: Dictionary<String, Int>) -> Int {\n",
    "        pairs.count\n",
    "    }\n\n",
    "    public static func name(_ value: Optional<String>) -> Int {\n",
    "        value?.count ?? 0\n",
    "    }\n",
    "}\n",
);

/// A function whose return clause names the empty tuple rather than `Void`.
const SWIFT_IDIOMS_PAREN_RETURN: &str = concat!(
    "public enum Returns {\n",
    "    public static func run() -> () {}\n",
    "}\n",
);

/// A struct carrying an internal initializer the compiler would synthesize.
const SWIFT_IDIOMS_REDUNDANT_INIT: &str = concat!(
    "public struct Reading {\n",
    "    public var count: Int\n",
    "\n",
    "    init(count: Int) {\n",
    "        self.count = count\n",
    "    }\n",
    "}\n",
);

/// A case whose `let` binds the whole pattern rather than each variable.
const SWIFT_IDIOMS_HOISTED_LET: &str = concat!(
    "public enum Corner {\n",
    "    case at(Int, Int)\n",
    "}\n",
    "\n",
    "public func describe(_ corner: Corner) -> Int {\n",
    "    switch corner {\n",
    "    case let .at(x, y):\n",
    "        return x + y\n",
    "    }\n",
    "}\n",
);

/// A class that is neither `final` nor a stated extension point.
const SWIFT_IDIOMS_OPEN_CLASS: &str = concat!(
    "public class Worker {\n",
    "    public var count = 0\n",
    "\n",
    "    public init() {}\n",
    "}\n",
);

/// Every requirement this gate took out of a Swift prompt rule.
///
/// Each `defect` is written in the SHAPE the deleted bullet named, so a row
/// reports what the bullet was about rather than a neighbouring shape the same
/// rule happens to read.
///
/// Two rows are HALF of the bullet `idioms.md` used to state, and each stands
/// beside a test of its own that holds the other half to the prompt rule.
///
/// The `void` row: measured on 0.62.1, `void` reports `func run() -> ()` and
/// stays SILENT on `func run() -> Void {}`: it rewrites `()` INTO `Void` and
/// stops there. SwiftFormat removes the clause under `redundantVoidReturnType`,
/// which this roster does not name. So the `()` half came out of `idioms.md`
/// and the omit-the-clause half stays there.
///
/// The `preferForLoop` row: measured on 0.62.1 under the shipped script,
/// `preferForLoop` reports `forEach` + `if` — on one line under
/// `--single-line-for-each convert`, and over several lines with or without it
/// — and stays SILENT on `things.filter { … }.forEach { … }`, by SwiftFormat's
/// own documented design. So the `forEach` + `if` half came out of `idioms.md`
/// and the `where`-clause half stays there.
///
/// Two bullets this gate touches are NOT here, and each was measured before it
/// was left alone. `optionals.md` never `guard` in a test stays whole, because
/// `noGuardInTests` reports `guard let value = source else { return }` and
/// stays silent on the shorthand `guard let value else { return }`, which binds
/// the same name from the same optional. `idioms.md` empty-collection
/// declarations stay whole, because the roster names neither `propertyTypes`
/// nor its option, for the reason the test above states.
const SWIFT_IDIOMS_SUPERSEDED_BULLETS: &[SupersededSwiftBullet] = &[
    SupersededSwiftBullet {
        prompt_rule: SWIFT_IDIOMS_PROMPT_RULE,
        tool_rule: "typeSugar",
        defect: SWIFT_IDIOMS_LONG_TYPE,
        words: "shorthand type sugar",
    },
    SupersededSwiftBullet {
        prompt_rule: SWIFT_IDIOMS_PROMPT_RULE,
        tool_rule: "void",
        defect: SWIFT_IDIOMS_PAREN_RETURN,
        words: "Return `Void`, not `()`",
    },
    SupersededSwiftBullet {
        prompt_rule: SWIFT_IDIOMS_PROMPT_RULE,
        tool_rule: "redundantMemberwiseInit",
        defect: SWIFT_IDIOMS_REDUNDANT_INIT,
        words: "memberwise initializer identical to the synthesized one",
    },
    SupersededSwiftBullet {
        prompt_rule: SWIFT_IDIOMS_PROMPT_RULE,
        tool_rule: SWIFT_PREFER_FOR_LOOP_RULE,
        defect: SWIFT_IDIOMS_SINGLE_LINE_FOR_EACH,
        words: "over `forEach` + `if`",
    },
    SupersededSwiftBullet {
        prompt_rule: SWIFT_IDIOMS_PROMPT_RULE,
        tool_rule: "hoistPatternLet",
        defect: SWIFT_IDIOMS_HOISTED_LET,
        words: "Bind each case variable with its own `let`",
    },
    SupersededSwiftBullet {
        prompt_rule: SWIFT_VALUE_SEMANTICS_PROMPT_RULE,
        tool_rule: "preferFinalClasses",
        defect: SWIFT_IDIOMS_OPEN_CLASS,
        words: "Mark classes not designed for subclassing",
    },
];

/// Acceptance: this gate is the ONE owner of every bullet it took out of a
/// Swift prompt rule.
///
/// A bullet deleted from the prompt text without a tool that reads it is a
/// requirement the set states nowhere, and nothing downstream reports the hole.
/// A bullet the prompt rule states beside a tool that decides it is two owners,
/// which produce churn on every review round. This test is what stands between
/// the deletion and each of those.
///
/// Every probe holds ONE KIND of defect, so the rule the run names is the rule
/// that read the shape rather than a neighbour that read the same file, and it
/// holds that defect in the SHAPE the bullet named — a probe staging some other
/// shape the same rule happens to report proves ownership of a requirement the
/// bullet never stated. The probes stage a `.swift-version`, so no answer here
/// is one the version gate bought.
#[test]
fn the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took() {
    let loader = builtin_loader();

    for bullet in SWIFT_IDIOMS_SUPERSEDED_BULLETS {
        let reported = swift_idioms_reporting_rules(bullet.defect, SWIFT_VERSION_SUPPORT);
        verify_superseded_swift_bullet(&loader, bullet, &reported);
    }
}
