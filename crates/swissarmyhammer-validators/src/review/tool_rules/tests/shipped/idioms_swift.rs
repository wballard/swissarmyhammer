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

/// What a shell line writes to join the line under it.
const SWIFT_IDIOMS_LINE_JOIN: char = '\\';

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

/// The file a project states its Swift language version in.
const SWIFT_VERSION_PATH: &str = ".swift-version";

/// The Swift language version the version probe states, which is the version
/// Airbnb's own SwiftFormat configuration pins.
const SWIFT_IDIOMS_PROBE_VERSION: &str = "6.3\n";

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
    let loader = builtin_loader();
    require_tool_installed(&loader, SWIFT_PROJECT_TYPES, SWIFT_IDIOMS_RULE);

    let mut staged: Vec<(&str, &str)> = vec![(SWIFT_IDIOMS_PROBE_PATH, source)];
    staged.extend_from_slice(support);

    drive_shipped_script(
        &loader,
        SWIFT_IDIOMS_RULE,
        &ShippedStaging::of(&staged),
        &[SWIFT_IDIOMS_PROBE_PATH],
        finding_rule_names,
    )
    .expect("the shipped Swift idioms script must judge the probe file and exit 0")
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
        &[(SWIFT_VERSION_PATH, SWIFT_IDIOMS_PROBE_VERSION)],
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
             {SWIFT_IDIOMS_PROBE_VERSION:?}; the run reported {with:?}"
        );
    }
}

/// The support files a probe stages to make every enabled rule live.
///
/// Five rules read the Swift language version, so a probe that stated none
/// would buy a clean run from the version gate rather than from the source it
/// staged.
const SWIFT_IDIOMS_VERSION_SUPPORT: &[(&str, &str)] =
    &[(SWIFT_VERSION_PATH, SWIFT_IDIOMS_PROBE_VERSION)];

/// The validator set the Swift prompt rules stand in.
const SWIFT_VALIDATOR_SET: &str = "swift";

/// The prompt rule that decides how a test unwraps an optional.
const SWIFT_OPTIONALS_PROMPT_RULE: &str = "optionals";

/// The prompt rule that decides how an empty collection is declared.
const SWIFT_IDIOMS_PROMPT_RULE: &str = "idioms";

/// The markdown body the shipped Swift prompt rule `rule` carries.
///
/// The body is read where the set ships it, so a test measures the SHIPPED
/// words rather than a copy this file wrote.
fn swift_prompt_rule_body(loader: &ValidatorLoader, rule: &str) -> String {
    loader
        .get_ruleset(SWIFT_VALIDATOR_SET)
        .unwrap_or_else(|| panic!("the `{SWIFT_VALIDATOR_SET}` validator set must ship"))
        .rules
        .iter()
        .find(|shipped| shipped.name == rule)
        .unwrap_or_else(|| {
            panic!("the `{SWIFT_VALIDATOR_SET}` set must carry a `{rule}` prompt rule")
        })
        .body
        .clone()
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

    let answered =
        swift_idioms_reporting_rules(SWIFT_PROMPT_RULE_ANSWER, SWIFT_IDIOMS_VERSION_SUPPORT);
    assert!(
        answered.is_empty(),
        "Swift written the way the prompt rules ask for must draw no finding from this \
         gate; the run reported {answered:?}"
    );

    let refused =
        swift_idioms_reporting_rules(SWIFT_PROMPT_RULE_REFUSAL, SWIFT_IDIOMS_VERSION_SUPPORT);
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
        swift_idioms_reporting_rules(SWIFT_IDIOMS_INFERRED_PROPERTY, SWIFT_IDIOMS_VERSION_SUPPORT);

    assert!(
        reported.is_empty(),
        "the gate must report neither form of an empty-collection declaration, because \
         `idioms.md` decides that bullet and `propertyTypes` would decide it the other \
         way; the run reported {reported:?}"
    );
}
