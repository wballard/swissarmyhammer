//! What the shipped Swift guidelines partial has to agree with.
//!
//! `builtin/_partials/project-types/swift.md` is the Swift guidance an agent
//! receives before it writes a line of Swift, and this project ships gates
//! that judge the Swift it then writes. Guidance that disagrees with a gate
//! walks the author into a finding from this project's own review, so the
//! agreement is a requirement rather than a coincidence.
//!
//! The partial is read through `get_builtin_partials`, the table
//! `PromptResolver::load_builtin_partials` serves from, so an assertion here
//! measures the text an agent actually RECEIVES rather than a file on disk.
//! What the partial is HELD TO is read the same way. The lint command comes
//! out of the shipped `idioms-swift` script, and the two empty-collection
//! forms come out of the shipped `idioms` prompt rule, so each value has ONE
//! owner and a change to either gate fails a test here rather than leaving the
//! partial to drift.
//!
//! This module stands in this crate rather than beside the partial in
//! `swissarmyhammer-templating`, because that crate cannot read a validator:
//! `swissarmyhammer-validators` depends on `swissarmyhammer-templating`, so
//! the edge the other way is a dependency cycle. This crate reaches both
//! sides already.

use super::*;

use swissarmyhammer_templating::resolver::get_builtin_partials;

/// The name `get_builtin_partials` serves the Swift project-type guidelines
/// under.
///
/// `PromptResolver::load_builtin_partials` registers it as `_partials/{name}`,
/// which is the same path `swissarmyhammer-project-detection` builds for the
/// `swift` project type through its `partial!` macro.
const SWIFT_GUIDELINES_PARTIAL: &str = "project-types/swift";

/// The Swift project-type guidelines, as an agent receives them.
fn swift_guidelines_partial() -> &'static str {
    get_builtin_partials()
        .into_iter()
        .find(|(name, _)| *name == SWIFT_GUIDELINES_PARTIAL)
        .map(|(_, content)| content)
        .unwrap_or_else(|| {
            panic!("the builtin partial `{SWIFT_GUIDELINES_PARTIAL}` must be embedded")
        })
}

/// Assert the Swift guidelines partial carries each phrase it must keep.
///
/// Each row pairs the exact text the partial has to hold with the requirement
/// that text satisfies, so a failure names the requirement rather than only
/// the missing string.
fn assert_swift_partial_states(requirements: &[(&str, &str)]) {
    let partial = swift_guidelines_partial();
    for (needle, requirement) in requirements {
        assert!(
            partial.contains(needle),
            "the swift guidelines partial must {requirement} (missing {needle:?})"
        );
    }
}

/// Acceptance: the guidelines name each formatter beside the config file it
/// alone reads.
///
/// `swift format` and `swiftformat` are DIFFERENT programs. The first is
/// Apple's swift-format, which ships with the toolchain and reads
/// `.swift-format`. The second is Nick Lockwood's SwiftFormat, installed
/// separately, which reads `.swiftformat`. The guidelines once offered the two
/// as interchangeable alternatives on one line, which sends an agent to the
/// wrong config file.
///
/// The PAIRING is what the requirement is about. Guidelines holding all three
/// file names somewhere state nothing about which tool reads which file, so a
/// row that swapped `.swift-format` and `.swiftformat` would send an agent to
/// the wrong one and read as correct. Each needle below therefore carries the
/// command cell and the config cell of ONE table row.
#[test]
fn swift_partial_names_each_formatter_with_its_own_config_file() {
    assert_swift_partial_states(&[
        (
            "`swift format` (in the toolchain) | `.swift-format`",
            "pair Apple's swift-format with the config file it alone reads",
        ),
        (
            "`swiftformat` | `.swiftformat`",
            "pair SwiftFormat with the config file it alone reads",
        ),
        (
            "`swiftlint` | `.swiftlint.yml`",
            "pair SwiftLint with the config file it alone reads",
        ),
        (
            "DIFFERENT programs",
            "state that the two formatters are not one tool",
        ),
        (
            "Do not write a config file as a side effect",
            "forbid creating a config file while formatting",
        ),
    ]);

    let partial = swift_guidelines_partial();
    assert!(
        !partial.contains("(or `swiftformat .`)"),
        "the swift guidelines partial must never offer `swift format` and \
         `swiftformat` as interchangeable alternatives — they are different \
         tools reading different config files"
    );
}

/// Acceptance: the guidelines carry the Airbnb plugin command lines that RUN.
///
/// A package that depends on `github.com/airbnb/swift` gets one command
/// running SwiftFormat and SwiftLint together. Both command lines carry
/// `--allow-writing-to-package-directory`: the plugin always asks for write
/// permission, so a bare `swift package format --lint` stops with a permission
/// error in a non-interactive shell.
#[test]
fn swift_partial_documents_the_airbnb_plugin_commands() {
    assert_swift_partial_states(&[
        (
            "https://github.com/airbnb/swift",
            "show the dependency that supplies the Airbnb plugin",
        ),
        (
            "swift package --allow-writing-to-package-directory format --lint\n",
            "give the plugin check command, which needs the write permission too",
        ),
        (
            "swift package --allow-writing-to-package-directory format\n",
            "give the plugin non-interactive fix command",
        ),
        (
            "--target <Target>",
            "give the singular --target switch the plugin accepts",
        ),
        ("--paths Sources Tests", "list the plugin paths switch"),
        ("--exclude Tests", "list the plugin exclude switch"),
        (
            "--swift-version 6.2",
            "list the plugin Swift version switch",
        ),
        (
            "Fix mode exits non-zero only for a SwiftLint rule",
            "state which failures each mode reports through its exit code",
        ),
    ]);
}

/// The head of the lint command the shipped `idioms-swift` script runs.
///
/// The gate moved from Nick Lockwood's `swiftformat` to the toolchain's own
/// `swift format`, and the guidelines have to send an author to the SAME tool.
/// Reading the head off the shipped script is what keeps the two together: a
/// gate that moved again fails the test below rather than leaving the partial
/// to name a tool no review runs.
const SWIFT_IDIOMS_LINT_HEAD: &str = "swift format lint";

/// What the shipped `idioms` prompt rule writes before the swiftformat option
/// that rewrites its DO into its DON'T.
///
/// The mark opens with the quote the rule wraps the option in, so the reading
/// below lands on the option's own value rather than on the prose after it.
const SWIFT_IDIOMS_PROPERTY_TYPES_MARK: &str = "`--property-types ";

/// What that rule writes before the empty-collection form it asks for.
const SWIFT_IDIOMS_DO_MARK: &str = "DO: `";

/// What that rule writes before the empty-collection form it refuses.
const SWIFT_IDIOMS_DONT_MARK: &str = "DON'T: `";

/// What closes a quoted span of a prompt rule body.
const SWIFT_PROMPT_RULE_QUOTE: char = '`';

/// The one bullet of `body` that decides how an empty collection is declared.
///
/// The bullet is found by the swiftformat option it names, so the DO form, the
/// DON'T form and the option all come off the SAME sentence rather than out of
/// whichever bullet of the rule happened to write `DO:` first.
fn swift_empty_collection_bullet(body: &str) -> &str {
    body.lines()
        .find(|line| line.contains(SWIFT_IDIOMS_PROPERTY_TYPES_MARK))
        .unwrap_or_else(|| {
            panic!(
                "the shipped `{SWIFT_IDIOMS_PROMPT_RULE}` prompt rule must name \
                 {SWIFT_IDIOMS_PROPERTY_TYPES_MARK:?}"
            )
        })
}

/// What `bullet` writes between `mark` and the quote that closes the span
/// holding it.
///
/// The prompt rules quote every declaration and every option they name, so one
/// reading serves each of them.
fn stated_after(bullet: &str, mark: &str) -> String {
    let opened = bullet
        .split_once(mark)
        .unwrap_or_else(|| {
            panic!("the empty-collection bullet must write {mark:?}");
        })
        .1;
    opened
        .split_once(SWIFT_PROMPT_RULE_QUOTE)
        .unwrap_or_else(|| panic!("the span the bullet opens after {mark:?} must be closed"))
        .0
        .to_string()
}

/// Acceptance: the guidelines state what the shipped Swift gates measure, read
/// out of those gates.
///
/// Guidance that disagrees with a gate walks an author into a finding from
/// this project's own review, and two shipped rules could disagree with what
/// the guidelines say:
///
/// - `builtin/validators/code-hygiene/rules/idioms-swift.md` runs the
///   toolchain's own `swift format`, so an author the guidelines send to
///   another formatter writes code that gate reports.
/// - `builtin/validators/swift/rules/idioms.md` wants the annotated
///   empty-collection literal that the Airbnb plugin's own
///   `--property-types inferred` rewrites away.
///
/// Every value below is READ from the shipped rule rather than written here,
/// so the tool name and the two declaration forms each have one owner. Change
/// either gate and this test fails.
#[test]
fn swift_partial_agrees_with_the_shipped_swift_tool_validators() {
    let loader = builtin_loader();

    let script = required_shipped_tool_rule(&loader, SWIFT_IDIOMS_RULE).script;
    assert!(
        script.contains(SWIFT_IDIOMS_LINT_HEAD),
        "the shipped `{SWIFT_IDIOMS_RULE}` script must run `{SWIFT_IDIOMS_LINT_HEAD}`, or the \
         guidelines below name a tool no review runs"
    );
    assert_swift_partial_states(&[(
        SWIFT_IDIOMS_LINT_HEAD,
        "name the lint command the idioms-swift gate itself runs",
    )]);

    let idioms = swift_prompt_rule_body(&loader, SWIFT_IDIOMS_PROMPT_RULE);
    let bullet = swift_empty_collection_bullet(&idioms);
    let inferred = format!(
        "--property-types {}",
        stated_after(bullet, SWIFT_IDIOMS_PROPERTY_TYPES_MARK)
    );
    assert_swift_partial_states(&[
        (
            inferred.as_str(),
            "name the option that rewrites the annotated empty collection away",
        ),
        (
            stated_after(bullet, SWIFT_IDIOMS_DO_MARK).as_str(),
            "show the empty-collection form the idioms prompt rule wants back",
        ),
        (
            stated_after(bullet, SWIFT_IDIOMS_DONT_MARK).as_str(),
            "show the empty-collection form a plugin fix run writes",
        ),
    ]);
}
