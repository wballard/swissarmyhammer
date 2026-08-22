//! Acceptance tests for the Swift JUDGMENT rules — the bullets the `swift`
//! prompt set keeps because no shipped tool decides them.
//!
//! `builtin/validators/swift/VALIDATOR.md` states the standing order: if a tool
//! can decide it, the tool owns it. A prompt bullet a tool ALSO decides is two
//! owners for one requirement, which produces churn on every review round. A
//! prompt bullet no tool decides is the set's own to state.
//!
//! A prompt rule has no tool to run, so a fixture pair cannot measure it the
//! way `builtin/validators/README.md` measures a tool rule. What CAN be
//! measured is the other half of the contract, and this module measures it: the
//! DON'T of each new bullet draws no finding from ANY shipped Swift gate, so
//! the bullet has exactly one owner; and the DO draws none either, so the shape
//! the prompt rule asks the author for does not walk into a tool finding.
//!
//! Each form the probes are written in is held to standing in the body of the
//! prompt rule that states it. Without that, an edit to either side would leave
//! this module measuring a house style nothing ships.

use super::*;

/// The prompt rule that decides what `init` does and where a body that
/// branches lives.
const SWIFT_INITIALIZATION_PROMPT_RULE: &str = "initialization";

/// The prompt rule that decides where a value may be mutable.
const SWIFT_IMMUTABILITY_PROMPT_RULE: &str = "immutability";

/// The prompt rule that decides how a scope states its requirements.
const SWIFT_PRECONDITIONS_PROMPT_RULE: &str = "preconditions";

/// The prompt rule that decides how a declaration is named.
const SWIFT_NAMING_CLARITY_PROMPT_RULE: &str = "naming-clarity";

/// The prompt rule that decides the access level of a declaration.
const SWIFT_ACCESS_CONTROL_PROMPT_RULE: &str = "access-control";

/// Every shipped Swift gate that reads a FILE list.
///
/// The five together are the whole deterministic half for Swift, less
/// `dead-code-swift`, which loads a built package rather than a file and
/// decides whether a declaration is USED — a question no bullet here asks.
const SWIFT_FILE_GATES: &[&str] = &[
    SWIFT_IDIOMS_RULE,
    SWIFT_DISALLOWED_CONSTRUCTS_RULE,
    SWIFT_FUNCTION_LENGTH_RULE,
    SWIFT_MAGIC_NUMBERS_RULE,
    SWIFT_MISSING_DOCS_RULE,
];

/// Where a judgment probe stands inside its probe repository.
const SWIFT_JUDGMENT_PROBE_PATH: &str = "Judgment.swift";

/// One judgment rule, with the Swift that shows the shape it refuses and the
/// Swift that shows the shape it asks for.
struct SwiftJudgmentRule {
    /// The prompt rule, by its name in the `swift` set.
    prompt_rule: &'static str,

    /// Swift written the way the rule states NOT to.
    refusal: &'static str,

    /// The same subjects written the way the rule asks for.
    answer: &'static str,

    /// Each form the two probes are written in. Every one must stand in the
    /// body of [`Self::prompt_rule`] and in one of the two probes, so the
    /// probes measure the RULE's own words rather than a shape this file
    /// invented.
    forms: &'static [&'static str],
}

/// The `init` bullets written the way `initialization.md` states NOT to.
///
/// Four subjects, one for each bullet: an `init` that opens a database, a
/// property a later `configure` method fills in, a `didSet` that branches, and
/// a callback closure that branches.
const SWIFT_INITIALIZATION_REFUSAL: &str = concat!(
    "import Foundation\n\n",
    "struct Row {}\n\n",
    "enum Database {\n",
    "    static func open(_ path: String) -> Handle { Handle() }\n\n",
    "    struct Handle {\n",
    "        func allRows() -> [Row] { [] }\n",
    "    }\n",
    "}\n\n",
    "struct Session {\n",
    "    init(token: String) {}\n",
    "}\n\n",
    "struct Photo {\n",
    "    let isPlaceholder: Bool\n",
    "}\n\n",
    "final class ReportStore {\n",
    "    private let rows: [Row]\n\n",
    "    init(path: String) { rows = Database.open(path).allRows() }\n",
    "}\n\n",
    "final class Uploader {\n",
    "    private var session: Session?\n\n",
    "    init() {}\n\n",
    "    func configure(token: String) {\n",
    "        session = Session(token: token)\n",
    "    }\n",
    "}\n\n",
    "final class Badge {\n",
    "    static let cap = 99\n",
    "    var shown: [Photo] = []\n\n",
    "    var count: Int = 0 {\n",
    "        didSet {\n",
    "            if count == 0 {\n",
    "                shown = []\n",
    "            } else if count > Badge.cap {\n",
    "                shown = Array(shown.prefix(Badge.cap))\n",
    "            }\n",
    "        }\n",
    "    }\n",
    "}\n\n",
    "final class Gallery {\n",
    "    private var photos: [Photo] = []\n\n",
    "    func reload(loader: (@escaping ([Photo]) -> Void) -> Void) {\n",
    "        loader { loaded in\n",
    "            var kept: [Photo] = []\n",
    "            for photo in loaded where !photo.isPlaceholder {\n",
    "                kept.append(photo)\n",
    "            }\n",
    "            self.photos = kept\n",
    "        }\n",
    "    }\n",
    "}\n",
);

/// The same four subjects written the way `initialization.md` asks for.
const SWIFT_INITIALIZATION_ANSWER: &str = concat!(
    "import Foundation\n\n",
    "struct Row {}\n\n",
    "enum Database {\n",
    "    static func open(_ path: String) -> Handle { Handle() }\n\n",
    "    struct Handle {\n",
    "        func allRows() -> [Row] { [] }\n",
    "    }\n",
    "}\n\n",
    "struct Session {\n",
    "    init(token: String) {}\n",
    "}\n\n",
    "struct Photo {\n",
    "    let isPlaceholder: Bool\n",
    "}\n\n",
    "final class ReportStore {\n",
    "    private let path: String\n",
    "    private var rows: [Row] = []\n\n",
    "    init(path: String) { self.path = path }\n\n",
    "    func load() { rows = Database.open(path).allRows() }\n",
    "}\n\n",
    "final class Uploader {\n",
    "    private let session: Session\n\n",
    "    init(token: String) {\n",
    "        session = Session(token: token)\n",
    "    }\n",
    "}\n\n",
    "final class Badge {\n",
    "    static let cap = 99\n",
    "    var shown: [Photo] = []\n\n",
    "    var count: Int = 0 { didSet { countDidChange() } }\n\n",
    "    private func countDidChange() {\n",
    "        if count == 0 {\n",
    "            shown = []\n",
    "        } else if count > Badge.cap {\n",
    "            shown = Array(shown.prefix(Badge.cap))\n",
    "        }\n",
    "    }\n",
    "}\n\n",
    "final class Gallery {\n",
    "    private var photos: [Photo] = []\n\n",
    "    func reload(loader: (@escaping ([Photo]) -> Void) -> Void) {\n",
    "        loader { loaded in self.photosDidLoad(loaded) }\n",
    "    }\n\n",
    "    private func photosDidLoad(_ loaded: [Photo]) {\n",
    "        photos = loaded.filter { !$0.isPlaceholder }\n",
    "    }\n",
    "}\n",
);

/// The immutability bullets written the way `immutability.md` states NOT to.
///
/// The accumulator loop carries the nested `if` on purpose: measured on
/// SwiftFormat 0.62.1, `swiftformat --rules preferForLoop
/// --single-line-for-each convert` rewrites
/// `users.forEach { if $0.isActive { names.append($0.name) } }` into exactly
/// this line, so the shape here is where the tool's own correction LANDS.
const SWIFT_IMMUTABILITY_REFUSAL: &str = concat!(
    "import Foundation\n\n",
    "struct User {\n",
    "    let name: String\n",
    "    let isActive: Bool\n",
    "}\n\n",
    "enum Roster {\n",
    "    static var attemptLimit = 3\n\n",
    "    static func activeNames(of users: [User]) -> [String] {\n",
    "        var names: [String] = []\n",
    "        for user in users { if user.isActive { names.append(user.name) } }\n",
    "        return names\n",
    "    }\n",
    "}\n\n",
    "func formatDuration(_ seconds: Int) -> String {\n",
    "    \"\\(seconds)s\"\n",
    "}\n",
);

/// The same subjects written the way `immutability.md` asks for.
const SWIFT_IMMUTABILITY_ANSWER: &str = concat!(
    "import Foundation\n\n",
    "struct User {\n",
    "    let name: String\n",
    "    let isActive: Bool\n",
    "}\n\n",
    "enum Roster {\n",
    "    static let attemptLimit = 3\n",
    "    static let backoffSeconds = 2\n",
    "    static var totalBudgetSeconds: Int { attemptLimit * backoffSeconds }\n\n",
    "    static func activeNames(of users: [User]) -> [String] {\n",
    "        users.filter(\\.isActive).map(\\.name)\n",
    "    }\n",
    "}\n\n",
    "extension Int {\n",
    "    var formattedDuration: String {\n",
    "        \"\\(self)s\"\n",
    "    }\n",
    "}\n",
);

/// The precondition bullets written the way `preconditions.md` states NOT to.
const SWIFT_PRECONDITIONS_REFUSAL: &str = concat!(
    "import Foundation\n\n",
    "protocol Transport {\n",
    "    func write(_ message: String)\n",
    "}\n\n",
    "struct Channel {\n",
    "    let transport: Transport\n",
    "    let isConnected: Bool\n\n",
    "    func send(_ message: String) { if isConnected { if !message.isEmpty ",
    "{ transport.write(message) } } }\n",
    "}\n\n",
    "struct Settings {\n",
    "    let table: [String: String]\n\n",
    "    func value(for key: String) -> String {\n",
    "        guard let found = table[key] else { fatalError(\"no value for \\(key)\") }\n",
    "        return found\n",
    "    }\n",
    "}\n",
);

/// The same subjects written the way `preconditions.md` asks for.
const SWIFT_PRECONDITIONS_ANSWER: &str = concat!(
    "import Foundation\n",
    "import os\n\n",
    "protocol Transport {\n",
    "    func write(_ message: String)\n",
    "}\n\n",
    "struct Channel {\n",
    "    let transport: Transport\n",
    "    let isConnected: Bool\n\n",
    "    func send(_ message: String) {\n",
    "        guard isConnected else { return }\n",
    "        guard !message.isEmpty else { return }\n",
    "        transport.write(message)\n",
    "    }\n",
    "}\n\n",
    "struct Settings {\n",
    "    let table: [String: String]\n",
    "    let logger: Logger\n\n",
    "    func value(for key: String) -> String {\n",
    "        guard let found = table[key] else {\n",
    "            assertionFailure(\"no value for \\(key)\")\n",
    "            logger.error(\"settings key is missing; the empty default stands\")\n",
    "            return \"\"\n",
    "        }\n",
    "        return found\n",
    "    }\n",
    "}\n",
);

/// Event handlers named the way `naming-clarity.md` states NOT to.
const SWIFT_NAMING_CLARITY_REFUSAL: &str = concat!(
    "import Foundation\n\n",
    "final class EditorScreen {\n",
    "    func handleSaveTap() {}\n\n",
    "    func onCancelPressed() {}\n",
    "}\n",
);

/// The same handlers named the way `naming-clarity.md` asks for.
const SWIFT_NAMING_CLARITY_ANSWER: &str = concat!(
    "import Foundation\n\n",
    "final class EditorScreen {\n",
    "    func didTapSave() {}\n\n",
    "    func didTapCancel() {}\n",
    "}\n",
);

/// A SwiftUI view whose access levels are the reverse of what
/// `access-control.md` asks for.
const SWIFT_ACCESS_CONTROL_REFUSAL: &str = concat!(
    "import SwiftUI\n\n",
    "struct BannerView: View {\n",
    "    private let title: String\n",
    "    private let subtitle: String\n",
    "    @State var isExpanded = false\n\n",
    "    var body: some View {\n",
    "        Text(isExpanded ? title : subtitle)\n",
    "    }\n",
    "}\n",
);

/// The same view with the access levels `access-control.md` asks for.
const SWIFT_ACCESS_CONTROL_ANSWER: &str = concat!(
    "import SwiftUI\n\n",
    "struct BannerView: View {\n",
    "    let title: String\n",
    "    let subtitle: String\n",
    "    @State private var isExpanded = false\n\n",
    "    var body: some View {\n",
    "        Text(isExpanded ? title : subtitle)\n",
    "    }\n",
    "}\n",
);

/// Every judgment rule this module measures, with its probe pair.
const SWIFT_JUDGMENT_RULES: &[SwiftJudgmentRule] = &[
    SwiftJudgmentRule {
        prompt_rule: SWIFT_INITIALIZATION_PROMPT_RULE,
        refusal: SWIFT_INITIALIZATION_REFUSAL,
        answer: SWIFT_INITIALIZATION_ANSWER,
        forms: &[
            "init(path: String) { rows = Database.open(path).allRows() }",
            "init(path: String) { self.path = path }",
            "private var session: Session?",
            "private let session: Session",
            "didSet { countDidChange() }",
            "self.photosDidLoad(loaded)",
        ],
    },
    SwiftJudgmentRule {
        prompt_rule: SWIFT_IMMUTABILITY_PROMPT_RULE,
        refusal: SWIFT_IMMUTABILITY_REFUSAL,
        answer: SWIFT_IMMUTABILITY_ANSWER,
        forms: &[
            "for user in users { if user.isActive { names.append(user.name) } }",
            r"users.filter(\.isActive).map(\.name)",
            "static var attemptLimit = 3",
            "static let attemptLimit = 3",
            "static var totalBudgetSeconds: Int { attemptLimit * backoffSeconds }",
            "func formatDuration(_ seconds: Int) -> String",
            "var formattedDuration: String",
        ],
    },
    SwiftJudgmentRule {
        prompt_rule: SWIFT_PRECONDITIONS_PROMPT_RULE,
        refusal: SWIFT_PRECONDITIONS_REFUSAL,
        answer: SWIFT_PRECONDITIONS_ANSWER,
        forms: &[
            "if isConnected { if !message.isEmpty { transport.write(message) } }",
            "guard isConnected else { return }",
            "guard !message.isEmpty else { return }",
            r#"fatalError("no value for \(key)")"#,
            r#"assertionFailure("no value for \(key)")"#,
            "settings key is missing; the empty default stands",
        ],
    },
    SwiftJudgmentRule {
        prompt_rule: SWIFT_NAMING_CLARITY_PROMPT_RULE,
        refusal: SWIFT_NAMING_CLARITY_REFUSAL,
        answer: SWIFT_NAMING_CLARITY_ANSWER,
        forms: &[
            "func handleSaveTap()",
            "func onCancelPressed()",
            "func didTapSave()",
            "func didTapCancel()",
        ],
    },
    SwiftJudgmentRule {
        prompt_rule: SWIFT_ACCESS_CONTROL_PROMPT_RULE,
        refusal: SWIFT_ACCESS_CONTROL_REFUSAL,
        answer: SWIFT_ACCESS_CONTROL_ANSWER,
        forms: &[
            "private let title: String",
            "@State var isExpanded = false",
            "@State private var isExpanded = false",
        ],
    },
];

/// Drives the shipped script of `gate` over `source`, staged alone at
/// [`SWIFT_JUDGMENT_PROBE_PATH`], and answers the rule name of each finding it
/// reported.
fn swift_gate_reporting_rules(gate: &str, source: &str) -> Vec<String> {
    let loader = builtin_loader();
    require_tool_installed(&loader, SWIFT_PROJECT_TYPES, gate);

    drive_shipped_script(
        &loader,
        gate,
        &ShippedStaging::of(&[(SWIFT_JUDGMENT_PROBE_PATH, source)]),
        &[SWIFT_JUDGMENT_PROBE_PATH],
        finding_rule_names,
    )
    .unwrap_or_else(|_| panic!("the shipped `{gate}` script must judge the probe file and exit 0"))
}

/// Acceptance: every form the probes below are written in stands in the body of
/// the prompt rule that states it.
///
/// This test costs no tool run, and it is what makes the measured test that
/// follows measure the SHIPPED rules. A form the rule stopped stating, and a
/// form no probe holds, each fail here by name.
#[test]
fn every_swift_judgment_rule_states_the_forms_its_probes_hold() {
    let loader = builtin_loader();

    for rule in SWIFT_JUDGMENT_RULES {
        let body = swift_prompt_rule_body(&loader, rule.prompt_rule);

        for form in rule.forms {
            assert!(
                body.contains(form),
                "`{}.md` must state `{form}`, or the probes below measure a form no \
                 prompt rule asks for",
                rule.prompt_rule
            );
            assert!(
                rule.refusal.contains(form) || rule.answer.contains(form),
                "`{form}` stands in `{}.md` and in neither probe, so nothing measures it",
                rule.prompt_rule
            );
        }
    }
}

/// Acceptance: no shipped Swift gate decides any judgment rule of the `swift`
/// set, in either direction.
///
/// Both halves are load-bearing, and they answer the two ways a judgment rule
/// can go wrong.
///
/// A gate that reports the DON'T is a SECOND owner for a requirement the prompt
/// rule states, and two owners produce churn on every review round — the tool
/// finding and the prompt finding arrive together, and the author cannot tell
/// which edit satisfies both. That bullet belongs to the tool instead, and the
/// tool rule records the deletion.
///
/// A gate that reports the DO is worse: the prompt rule then walks the author
/// into a tool finding no edit can satisfy while the bullet stands.
///
/// Measured over the five shipped Swift gates that read a file list, with
/// swiftformat 0.62.1 and swiftlint 0.65.0: 10 probes, 50 runs, 0 findings.
#[test]
fn no_shipped_swift_gate_decides_a_swift_judgment_rule() {
    for rule in SWIFT_JUDGMENT_RULES {
        for gate in SWIFT_FILE_GATES {
            let refused = swift_gate_reporting_rules(gate, rule.refusal);
            assert!(
                refused.is_empty(),
                "`{gate}` reports the shape `{}.md` refuses, so that requirement has two \
                 owners; the run reported {refused:?}",
                rule.prompt_rule
            );

            let answered = swift_gate_reporting_rules(gate, rule.answer);
            assert!(
                answered.is_empty(),
                "`{gate}` reports the shape `{}.md` ASKS for, so the prompt rule walks the \
                 author into a tool finding; the run reported {answered:?}",
                rule.prompt_rule
            );
        }
    }
}
