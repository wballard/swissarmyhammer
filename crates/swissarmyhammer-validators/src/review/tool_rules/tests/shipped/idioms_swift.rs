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
/// Twenty-eight, which is Airbnb's list narrowed to the rules that decide an
/// idiom. The count is the assertion that a name added or dropped later
/// reaches this guard rather than moving the gate without a word.
const SWIFT_IDIOMS_ROSTER_SIZE: usize = 28;

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
/// repository that states no `.swift-version`: 37 findings, at exit 0. The
/// count states that a refusing path beside the fixture costs the run nothing,
/// because the run reports the same 37 either way.
const SWIFT_IDIOMS_FAIL_FIXTURE_FINDINGS: usize = 37;

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

/// What the script writes between a finding's rule name and swiftformat's own
/// sentence.
const SWIFT_IDIOMS_CLAIM_SEPARATOR: &str = ": ";

/// The rule name of each finding of `outcome`, in the order the run stated
/// them.
///
/// The script writes each finding's claim as `<rule_id>: <reason>`, so the
/// name stands before the first separator. A probe of WHICH rule fired reads
/// this rather than the `path:line` row [`finding_rows`] answers.
fn finding_rule_names(outcome: &ScriptOutcome, _repo_root: &Path) -> Vec<String> {
    outcome
        .findings
        .iter()
        .map(|finding| {
            finding
                .claim
                .split_once(SWIFT_IDIOMS_CLAIM_SEPARATOR)
                .map_or_else(|| finding.claim.clone(), |(name, _)| name.to_string())
        })
        .collect()
}

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
