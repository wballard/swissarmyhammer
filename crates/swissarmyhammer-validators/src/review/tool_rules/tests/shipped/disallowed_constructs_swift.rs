//! Acceptance tests for the shipped `disallowed-constructs-swift` tool rule.
//!
//! Each test drives the SHIPPED script over a probe repository and reads what
//! the real swiftlint reported.
//!
//! Four answers stand apart from every other Swift rule of this set, and one
//! test holds each. The gate names TWELVE rules, and swiftlint answers a name
//! it does not know with a warning and a healthy exit, so a misspelling would
//! cost one rule and read as a clean run. Three of the twelve stay off inside a
//! test target, and swiftlint carries no per-rule path filter for a stock rule,
//! so the SCRIPT partitions its own paths — from the `Tests/` convention and
//! from the test targets the package manifest names. And the three custom
//! regex rules read an identifier alone, so a comment and a string literal
//! holding the same words stay silent.

use super::*;

/// Acceptance: every shipped disallowed-construct tool rule passes its fixture
/// pair in doctor, and supersedes nothing.
///
/// The pass fixture is the load-bearing half. It holds the same declarations
/// the fail fixture holds, each written in the form its rule asks for, and two
/// of them behind the inline directive instead — so a swiftlint release that
/// changed what one rule asks for, or that stopped honouring the directive,
/// makes the pair fail and takes the rule out of the review rather than moving
/// a finding without a word.
#[test]
#[serial_test::serial(cwd)]
fn every_shipped_disallowed_constructs_tool_rule_passes_its_fixtures() {
    verify_shipped_tool_rules_pass_fixtures(
        SHIPPED_DISALLOWED_CONSTRUCTS_RULES,
        DISALLOWED_CONSTRUCTS_RULE_KIND,
    );
}

/// What the shipped script writes before each name of its `only_rules:` list.
///
/// The list is written as the argument list of one `printf`, one quoted YAML
/// line for each entry, so the text between this head and the quote that
/// closes it IS a name the gate enables. Reading the names off the shipped
/// script is what makes the guard below hold the rule that ships rather than a
/// copy this file wrote.
const SWIFT_DISALLOWED_ROSTER_HEAD: &str = "'  - ";

/// The quote that closes one entry of that list.
const SWIFT_DISALLOWED_ROSTER_END: char = '\'';

/// How many names the shipped `only_rules:` list holds.
///
/// Nine stock swiftlint rules, plus `custom_rules`, which is the identifier
/// that enables the three regex rules the script defines under it. The count
/// is the assertion that a name added or dropped later reaches this guard
/// rather than moving the gate without a word.
const SWIFT_DISALLOWED_ROSTER_SIZE: usize = 10;

/// The command that asks swiftlint which rules it knows.
const SWIFT_LINT_TOOL: &str = "swiftlint";

/// The subcommand that makes swiftlint write its whole rule list.
const SWIFT_LINT_RULES_COMMAND: &str = "rules";

/// The character swiftlint writes around each column of that list.
const SWIFT_LINT_TABLE_EDGE: char = '|';

/// The first column's heading, which is the one row of the table that names no
/// rule.
const SWIFT_LINT_TABLE_HEADING: &str = "identifier";

/// The names the shipped script's `only_rules:` list holds, read off the
/// script itself.
fn swift_disallowed_roster(script: &str) -> Vec<String> {
    script
        .split(SWIFT_DISALLOWED_ROSTER_HEAD)
        .skip(1)
        .filter_map(|entry| entry.split_once(SWIFT_DISALLOWED_ROSTER_END))
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Every rule identifier the installed swiftlint knows.
///
/// `swiftlint rules` writes a table, one row for each identifier, with the
/// identifier in the first column. The heading row is dropped by name.
fn swift_lint_known_rules() -> Vec<String> {
    let listed = std::process::Command::new(SWIFT_LINT_TOOL)
        .arg(SWIFT_LINT_RULES_COMMAND)
        .output()
        .expect("the installed swiftlint must write its rule list");

    String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix(SWIFT_LINT_TABLE_EDGE))
        .filter_map(|row| row.split(SWIFT_LINT_TABLE_EDGE).next())
        .map(str::trim)
        .filter(|name| !name.is_empty() && *name != SWIFT_LINT_TABLE_HEADING)
        .map(str::to_string)
        .collect()
}

/// Acceptance: every rule the shipped script enables is one the installed
/// swiftlint knows.
///
/// swiftlint answers a name it does not know with a warning and a HEALTHY
/// exit. Measured on 0.65.0 over one file holding a force unwrap, beside a
/// configuration naming `no_such_rule_at_all`: exit 0, the finding of the rule
/// swiftlint does know still reported, and `warning: 'no_such_rule_at_all' is
/// not a valid rule identifier` on stderr. So a misspelling costs one rule and
/// reads as a clean run.
///
/// This is the guard on that. Every name of the shipped `only_rules:` list
/// must stand in swiftlint's own list, so a misspelling and a rule swiftlint
/// renames each fail here by name.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_names_only_rules_swiftlint_knows() {
    let loader = builtin_loader();
    require_tool_installed(
        &loader,
        SWIFT_PROJECT_TYPES,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
    );
    let shipped = required_shipped_tool_rule(&loader, SWIFT_DISALLOWED_CONSTRUCTS_RULE);

    let roster = swift_disallowed_roster(&shipped.script);
    assert_eq!(
        roster.len(),
        SWIFT_DISALLOWED_ROSTER_SIZE,
        "the shipped roster must name {SWIFT_DISALLOWED_ROSTER_SIZE} rules; it names {roster:?}"
    );

    let known = swift_lint_known_rules();
    assert!(
        !known.is_empty(),
        "`{SWIFT_LINT_TOOL} {SWIFT_LINT_RULES_COMMAND}` wrote no rule name, so the \
         comparison below holds nothing and this test cannot fail"
    );

    let unknown: Vec<&String> = roster.iter().filter(|name| !known.contains(name)).collect();

    assert!(
        unknown.is_empty(),
        "the shipped script names these rules and the installed swiftlint knows none of \
         them, so swiftlint warns and lints on and the gate stops measuring each one: \
         {unknown:?}"
    );
}

/// The shipped failing fixture, as [`copy_shipped_fixture`] asks for it.
const SWIFT_DISALLOWED_FAIL_FIXTURE: &str = "disallowed-constructs-swift.fail.swift";

/// Where a probe stages the failing fixture.
const SWIFT_DISALLOWED_JUDGED_PATH: &str = "Judged.swift";

/// Every rule the shipped failing fixture reports.
///
/// The fixture holds one declaration for each of the twelve rules the gate
/// enables, so the run reports each name exactly one time. Measured with the
/// shipped script and swiftlint 0.65.0: 12 findings, exit 0, and 0 bytes on
/// stderr.
const SWIFT_DISALLOWED_FAIL_FIXTURE_RULES: &[&str] = &[
    "implicitly_unwrapped_optional",
    "force_unwrapping",
    "force_try",
    "force_cast",
    "unused_optional_binding",
    "unowned_variable_capture",
    "legacy_constructor",
    "legacy_nsgeometry_functions",
    "legacy_cggeometry_functions",
    "no_direct_standard_out_logs",
    "no_file_literal",
    "no_unchecked_sendable",
];

/// Drives the shipped script over the failing fixture, and answers the rule
/// name of each finding it reported.
///
/// The fixture is staged through `prepare` so the run reads the bytes the set
/// ships. A test that wrote its own copy would answer for the copy.
fn swift_disallowed_fixture_rules() -> Vec<String> {
    let loader = builtin_loader();
    require_tool_installed(
        &loader,
        SWIFT_PROJECT_TYPES,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
    );

    let stage_fixture = |repo: &Path| {
        copy_shipped_fixture(
            &loader,
            repo,
            SWIFT_DISALLOWED_FAIL_FIXTURE,
            SWIFT_DISALLOWED_JUDGED_PATH,
        );
    };
    let staging = ShippedStaging {
        staged: NO_SUPPORT_FILES,
        links: NO_PROBE_LINKS,
        outside: NO_PROBE_OUTSIDE,
        prepare: &stage_fixture,
        restore: &stage_nothing_more,
    };

    let run = drive_shipped_script_whole(
        &loader,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
        &staging,
        &[SWIFT_DISALLOWED_JUDGED_PATH],
    );
    let outcome = run
        .outcome
        .expect("the shipped Swift disallowed-constructs script must judge the fixture and exit 0");

    finding_rule_names(&outcome, &run.repo_root)
}

/// Acceptance: the shipped failing fixture reports every one of the twelve
/// rules the gate enables, and reports each one time.
///
/// swiftlint keeps linting past a name it does not know, so the roster guard
/// above cannot say that an enabled rule still REPORTS. This says it: a
/// release that stops reporting one construct fails here by name, and the
/// doctor then marks the rule unusable through the same fixture.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_reports_every_enabled_rule() {
    let reported = swift_disallowed_fixture_rules();

    for rule in SWIFT_DISALLOWED_FAIL_FIXTURE_RULES {
        assert_eq!(
            reported.iter().filter(|name| *name == rule).count(),
            1,
            "the shipped failing fixture holds one declaration for `{rule}`, so the run \
             must report it exactly one time; it reported {reported:?}"
        );
    }
    assert_eq!(
        reported.len(),
        SWIFT_DISALLOWED_FAIL_FIXTURE_RULES.len(),
        "the failing fixture must report the twelve enabled rules and nothing else; \
         it reported {reported:?}"
    );
}

/// Where a probe stages the Swift file it measures one rule over.
const SWIFT_DISALLOWED_PROBE_PATH: &str = "Probe.swift";

/// Drives the shipped script over `source` staged at
/// [`SWIFT_DISALLOWED_PROBE_PATH`], and answers the rule name of each finding
/// it reported.
fn swift_disallowed_reporting_rules(source: &str) -> Vec<String> {
    let loader = builtin_loader();
    require_tool_installed(
        &loader,
        SWIFT_PROJECT_TYPES,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
    );

    drive_shipped_script(
        &loader,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
        &ShippedStaging::of(&[(SWIFT_DISALLOWED_PROBE_PATH, source)]),
        &[SWIFT_DISALLOWED_PROBE_PATH],
        finding_rule_names,
    )
    .expect("the shipped Swift disallowed-constructs script must judge the probe file and exit 0")
}

/// The rule that reports an `@unchecked Sendable` conformance.
const SWIFT_UNCHECKED_SENDABLE_RULE: &str = "no_unchecked_sendable";

/// A type that declares `@unchecked Sendable` and states no invariant.
const SWIFT_UNCHECKED_SENDABLE_PLAIN: &str = concat!(
    "public final class Box: @unchecked Sendable {\n",
    "    public let value: Int = 0\n",
    "}\n",
);

/// The same type behind the directive, with the invariant after it.
///
/// The two constants differ in that one line and in nothing else, so the
/// directive is the only thing the test below can be measuring.
const SWIFT_UNCHECKED_SENDABLE_ANNOTATED: &str = concat!(
    "// swiftlint:disable:next no_unchecked_sendable  the value is a `let` of a value \
     type, so no write can race a read\n",
    "public final class Box: @unchecked Sendable {\n",
    "    public let value: Int = 0\n",
    "}\n",
);

/// Acceptance: the annotation IS the documentation, and the shipped rule
/// honours it.
///
/// `concurrency.md` states the `@unchecked Sendable` requirement as pure
/// judgment — the smell is the ABSENCE of a lock and a comment. No tool can
/// answer "is there a documented invariant". A tool CAN require an explicit
/// annotation, and the text of that annotation is the invariant, which is what
/// Airbnb's own message asks the author to write.
///
/// The two halves are both load-bearing. Without the directive the conformance
/// reports, and with it the same bytes report nothing, so the escape hatch is
/// the only difference between the two runs. A swiftlint release that stopped
/// honouring the directive turns every documented exception back into a
/// finding no edit can satisfy.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_reads_the_unchecked_sendable_directive() {
    let plain = swift_disallowed_reporting_rules(SWIFT_UNCHECKED_SENDABLE_PLAIN);
    assert!(
        plain.contains(&SWIFT_UNCHECKED_SENDABLE_RULE.to_string()),
        "`{SWIFT_UNCHECKED_SENDABLE_RULE}` must report a conformance that states no \
         invariant; the run reported {plain:?}"
    );

    let annotated = swift_disallowed_reporting_rules(SWIFT_UNCHECKED_SENDABLE_ANNOTATED);
    assert!(
        annotated.is_empty(),
        "the same conformance under `// swiftlint:disable:next \
         {SWIFT_UNCHECKED_SENDABLE_RULE}` must report nothing, because the text after the \
         directive IS the documented invariant; the run reported {annotated:?}"
    );
}

/// The rule that reports a call writing to standard out.
const SWIFT_STANDARD_OUT_RULE: &str = "no_direct_standard_out_logs";

/// One real `print` call, beside the same word inside a string literal, inside
/// a line comment and inside a doc comment.
///
/// `match_kinds: [identifier]` is what tells the four apart. Measured with the
/// same regex and no `match_kinds`: all four report.
const SWIFT_STANDARD_OUT_PROBE: &str = concat!(
    "public func announce(_ message: String) {\n",
    "    print(message)\n",
    "    let quoted = \"print(this is a string, not a call)\"\n",
    "    // print(this is a comment)\n",
    "    _ = quoted\n",
    "}\n\n",
    "/// print(this is a doc comment)\n",
    "public func documented() {}\n",
);

/// Acceptance: the three custom regex rules read an identifier, and never a
/// comment or a string literal.
///
/// A regex over source text reads all three alike. `match_kinds` is
/// swiftlint's own answer, and no reading of the regex could replace it — the
/// same words stand in each of the four positions.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_reads_no_comment_and_no_string_literal() {
    let reported = swift_disallowed_reporting_rules(SWIFT_STANDARD_OUT_PROBE);

    assert_eq!(
        reported,
        vec![SWIFT_STANDARD_OUT_RULE.to_string()],
        "`{SWIFT_STANDARD_OUT_RULE}` must report the one real call and none of the three \
         copies of the same words in a comment or a string literal; the run reported \
         {reported:?}"
    );
}

/// A force unwrap, which `force_unwrapping` reports in a product target and
/// never inside a test target.
const SWIFT_FORCE_UNWRAP_SOURCE: &str = concat!(
    "public func unwrap(_ value: Int?) -> Int {\n",
    "    return value!\n",
    "}\n",
);

/// The line [`SWIFT_FORCE_UNWRAP_SOURCE`] holds its force unwrap on.
const SWIFT_FORCE_UNWRAP_LINE: usize = 2;

/// Where a probe stages the product-target copy of those bytes.
const SWIFT_PRODUCT_TARGET_PATH: &str = "Sources/Probe/Forced.swift";

/// Where a probe stages the test-target copy of the SAME bytes, under the
/// `Tests/` directory SPM lays a package out with.
const SWIFT_TESTS_DIRECTORY_PATH: &str = "Tests/ProbeTests/Forced.swift";

/// Where a probe stages a third copy, under a test target the package manifest
/// names at a path no convention would find.
const SWIFT_MANIFEST_TARGET_PATH: &str = "Custom/OddTests/Forced.swift";

/// The manifest of the probe package.
///
/// It declares one product target and two test targets, the second of them at
/// an explicit `path:` outside `Tests/`. That second target is the whole
/// reason the run reads the manifest at all.
const SWIFT_PROBE_MANIFEST: &str = concat!(
    "// swift-tools-version:5.9\n",
    "import PackageDescription\n\n",
    "let package = Package(\n",
    "    name: \"Probe\",\n",
    "    targets: [\n",
    "        .target(name: \"Probe\"),\n",
    "        .testTarget(name: \"ProbeTests\", dependencies: [\"Probe\"]),\n",
    "        .testTarget(name: \"OddTests\", dependencies: [\"Probe\"], path: \"Custom/OddTests\"),\n",
    "    ]\n",
    ")\n",
);

/// Where a package states its manifest.
const SWIFT_PACKAGE_MANIFEST_PATH: &str = "Package.swift";

/// Drives the shipped script over `files`, staged with the same force unwrap
/// at each path, and answers each finding as the `path:line` row the work-list
/// holds.
fn swift_disallowed_force_rows(staged: &[(&str, &str)], files: &[&str]) -> Vec<String> {
    let loader = builtin_loader();
    require_tool_installed(
        &loader,
        SWIFT_PROJECT_TYPES,
        SWIFT_DISALLOWED_CONSTRUCTS_RULE,
    );

    shipped_script_findings(&loader, SWIFT_DISALLOWED_CONSTRUCTS_RULE, staged, files)
        .expect("the shipped Swift disallowed-constructs script must judge the probe files")
}

/// Acceptance: a force unwrap reports in a product target and stays silent in
/// a test target, for the SAME bytes.
///
/// `optionals.md` says "in non-test code" and means it, and swiftlint carries
/// no per-rule path filter for a stock rule — measured on 0.65.0, an
/// `excluded:` key under `force_unwrapping:` is answered with
/// `warning: Configuration for 'force_unwrapping' rule contains the invalid
/// key(s) 'excluded'.` and both files still report. So the SCRIPT partitions
/// its own paths.
///
/// The two halves are both load-bearing. The product path must report, or the
/// partition swallowed a real finding; the test path must stay silent, or the
/// carve-out the prompt rule states was never honoured. The two files hold
/// identical bytes, so the PATH is the only difference between them.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_keeps_the_force_rules_out_of_a_test_target() {
    let rows = swift_disallowed_force_rows(
        &[
            (SWIFT_PRODUCT_TARGET_PATH, SWIFT_FORCE_UNWRAP_SOURCE),
            (SWIFT_TESTS_DIRECTORY_PATH, SWIFT_FORCE_UNWRAP_SOURCE),
        ],
        &[SWIFT_PRODUCT_TARGET_PATH, SWIFT_TESTS_DIRECTORY_PATH],
    );

    assert_eq!(
        rows,
        vec![format!(
            "{SWIFT_PRODUCT_TARGET_PATH}:{SWIFT_FORCE_UNWRAP_LINE}"
        )],
        "the force unwrap under `{SWIFT_PRODUCT_TARGET_PATH}` must report and the same \
         bytes under `{SWIFT_TESTS_DIRECTORY_PATH}` must not; the run reported {rows:?}"
    );
}

/// Acceptance: the run reads the test targets the package manifest names, so a
/// test target outside `Tests/` keeps the carve-out too.
///
/// The `Tests/` convention answers SPM's default layout and nothing else. A
/// package is free to state an explicit `path:`, and `swift package describe`
/// is the fact that names it — measured over this manifest, it reports
/// `Tests/ProbeTests` and `Custom/OddTests`.
///
/// The product path stands in the same run, so the test says the manifest
/// widened the carve-out rather than switched the gate off.
///
/// The probe stages a file under `Tests/ProbeTests` as well, and names it in
/// no argument list. SPM refuses to describe a package whose declared target
/// directory holds no source — measured, `swift package describe` then exits 1
/// with `error: Source files for target ProbeTests should be located under
/// 'Tests/ProbeTests'`, and the run falls back to the convention, which is the
/// answer this test must not be given for free.
#[test]
fn the_shipped_swift_disallowed_constructs_tool_rule_reads_a_test_target_the_manifest_names() {
    let rows = swift_disallowed_force_rows(
        &[
            (SWIFT_PACKAGE_MANIFEST_PATH, SWIFT_PROBE_MANIFEST),
            (SWIFT_PRODUCT_TARGET_PATH, SWIFT_FORCE_UNWRAP_SOURCE),
            (SWIFT_TESTS_DIRECTORY_PATH, SWIFT_FORCE_UNWRAP_SOURCE),
            (SWIFT_MANIFEST_TARGET_PATH, SWIFT_FORCE_UNWRAP_SOURCE),
        ],
        &[SWIFT_PRODUCT_TARGET_PATH, SWIFT_MANIFEST_TARGET_PATH],
    );

    assert_eq!(
        rows,
        vec![format!(
            "{SWIFT_PRODUCT_TARGET_PATH}:{SWIFT_FORCE_UNWRAP_LINE}"
        )],
        "the manifest names `{SWIFT_MANIFEST_TARGET_PATH}`'s directory as a test target, \
         so the force unwrap there must stay silent while the one under \
         `{SWIFT_PRODUCT_TARGET_PATH}` reports; the run reported {rows:?}"
    );
}
