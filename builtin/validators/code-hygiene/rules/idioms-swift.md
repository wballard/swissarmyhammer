---
name: idioms-swift
description: Swift declarations written in a non-preferred form when an equivalent preferred form exists — checked by swiftformat, not by prompt.
match:
  files:
    - "**/*.swift"
  project_types:
    - swift
tool:
  scope: files
  run: |
    set -e
    if [ "$#" -eq 0 ]; then
      exit 0
    fi
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    printf '%s\n' typeSugar void redundantMemberwiseInit preferForLoop \
      hoistPatternLet preferFinalClasses preferCountWhere isEmpty \
      preferContains preferFirstWhere preferMinOverSorted preferFlatMap \
      preferLazyMap swiftTestingTestCaseNames redundantSwiftTestingSuite \
      testSuiteAccessControl validateTestCases noForceUnwrapInTests \
      noForceTryInTests noGuardInTests environmentEntry redundantEmptyView \
      redundantViewBuilder redundantSwiftUIGroup redundantEquatable \
      genericExtensions opaqueGenericParameters ifExpressions \
      conditionalAssignment | sort -u > "$work/named"
    swiftformat --rules 2>/dev/null |
      sed -e 's/([^)]*)//' -e 's/[[:space:]]//g' -e '/^$/d' | sort -u > "$work/known"
    enabled="$(comm -12 "$work/named" "$work/known" | paste -s -d, -)"
    if [ -z "$enabled" ]; then
      printf 'idioms-swift: swiftformat names none of the rules this gate states, so no file was judged\n' >&2
      exit 1
    fi
    for file in "$@"; do
      if [ ! -e "$file" ]; then
        printf 'sah-diagnostic: idioms-swift found no file at %s, so its declarations are unread\n' "$file" >&2
        continue
      fi
      status=0
      swiftformat --lint --quiet --reporter json --cache ignore \
        --min-version 0.62.1 --rules "$enabled" \
        --short-optionals always --pattern-let inline \
        --guard-like-if-statements convert --single-line-for-each convert \
        "$file" > "$work/report.json" 2> "$work/lint.err" || status=$?
      if [ "$status" -gt 1 ]; then
        if grep -q '^error: Project specifies SwiftFormat --min-version' "$work/lint.err"; then
          printf 'idioms-swift: the installed swiftformat is older than 0.62.1, so this gate cannot run\n' >&2
          exit 1
        fi
        reason="$(sed -n 's/^error: //p' "$work/lint.err" | sed -n '1p')"
        printf 'sah-diagnostic: idioms-swift declined %s: %s\n' "$file" "${reason:-swiftformat exited $status}" >&2
        continue
      fi
      jq -c --arg file "$file" \
        '.[] | {file: $file, line: .line, message: "\(.rule_id): \(.reason)"}' \
        "$work/report.json"
    done
  doctor:
    check_command: "which swiftformat jq mktemp sed sort comm paste grep && printf '' | swiftformat --lint --quiet --min-version 0.62.1"
    check_version_command: "swiftformat --version"
    fix_hint: "brew install swiftformat"
---

# Idioms — Swift

`swiftformat` decides, in one run, a set of Swift idioms a reader would
otherwise judge by eye. Each rule this gate enables rewrites one shape into an
equivalent shape, so the tool's answer is a fact about the source rather than a
preference: the two forms compile to the same program, and the tool names which
one the project writes.

The gate runs swiftformat in `--lint` mode. It never rewrites a file. Review is
read-only, so a finding names the line and the author makes the edit.

The author usually makes it with `swiftformat` itself, so every rule here
AUTOCORRECTS. That decides what a split bullet keeps: the prompt rule has to
state the shape the tool's FIX lands on, not only the shape the finding started
from. A gate that corrects into a shape no rule of either set discusses walks
the author into a hole. Each of the two split sections below measures the shape
its fix writes, and names the test that holds it.

Every measurement below was made with SwiftFormat 0.62.1.

## Which rules the gate enables

The roster is Airbnb's, taken from
`Sources/AirbnbSwiftFormatTool/airbnb.swiftformat` on their `master` branch,
and narrowed to the rules that decide an IDIOM. Twenty-nine names stand in the
script, in five groups.

Seven of them decide a bullet a prompt rule of `builtin/validators/swift/` used
to state. FOUR of those bullets are out of the prompt text whole and this gate
is their ONE owner. TWO more it decides in half, and each of those two stays in
the prompt text as the half the gate misses. The seventh stays there whole,
because this gate reads only part of it and no option moves the rest.

| the rule | the option it needs | the bullet it took |
|---|---|---|
| `typeSugar` | `--short-optionals always` | `idioms.md` shorthand type sugar |
| `void` | | `idioms.md` return `Void`, not `()` — HALF of that bullet |
| `redundantMemberwiseInit` | | `idioms.md` no memberwise init identical to the synthesized one |
| `preferForLoop` | `--single-line-for-each convert` | `idioms.md` a `for` loop over `forEach` + `if` — HALF of that bullet |
| `hoistPatternLet` | `--pattern-let inline` | `idioms.md` bind each case variable with its own `let` |
| `preferFinalClasses` | | `value-semantics.md` mark classes `final` |
| `noGuardInTests` | `--guard-like-if-statements convert` | `optionals.md` never `guard` in a test — the bullet STAYS |

`the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` holds each of the
six taken rows to both halves of its own claim: the rule reports a file holding
that ONE defect, WRITTEN IN THE SHAPE THE BULLET NAMED, and the prompt rule
states that requirement no longer. A rule that went silent, and a bullet
written back into the prompt text, each fail it by name. The two HALF rows
carry the half the gate decides and nothing more: the `void` probe holds
`-> ()` and never `-> Void`, and the `preferForLoop` probe holds `forEach` + an
`if` and never a walk of the `where` half. Each half the gate misses has a test
of its own, named in the section that measures it.

The other twenty-two decide a question no shipped prompt rule asks:

- Performance: `preferCountWhere`, `isEmpty`, `preferContains`,
  `preferFirstWhere`, `preferMinOverSorted`, `preferFlatMap`, `preferLazyMap`.
- Swift Testing: `swiftTestingTestCaseNames`, `redundantSwiftTestingSuite`,
  `testSuiteAccessControl`, `validateTestCases`, `noForceUnwrapInTests`,
  `noForceTryInTests`.
- SwiftUI: `environmentEntry`, `redundantEmptyView`, `redundantViewBuilder`,
  `redundantSwiftUIGroup`.
- Other: `redundantEquatable`, `genericExtensions`, `opaqueGenericParameters`,
  `ifExpressions`, `conditionalAssignment`.

## Why the script intersects its roster with the installed tool

Two of the twenty-nine names — `preferLazyMap` and `ifExpressions` — stand in
Airbnb's file and in NO released SwiftFormat. Airbnb's `master` tracks
SwiftFormat's `main` branch rather than a release. The eight newest SwiftFormat
releases were read, 0.59.1 through 0.62.1, and neither name appears in any of
them; `swiftformat --rules` on 0.62.1 lists 153 rules and holds 27 of the 29.

A name SwiftFormat does not know breaks the WHOLE run, and no flag turns that
off. Measured on 0.62.1, over one file:

| the spelling | status | what it wrote |
|---|---|---|
| `--rules typeSugar,preferLazyMap` | 70 | `error: Unknown rule 'preferLazyMap'. Did you mean 'preferFlatMap'?` |
| the same, plus `--unknown-rules ignore` | 70 | the same line |
| a `--config` file holding the two `--rules` lines | 70 | the same line |
| the same, plus `--unknown-rules ignore` | 70 | the same line |
| `--disable all --enable typeSugar,preferLazyMap` | 70 | the same line |
| the same, plus `--unknown-rules ignore` | 70 | the same line |

`--unknown-rules ignore` moves nothing on the command line in 0.62.1, so a
script that named the two would report NOTHING for any Swift file, at a status
the engine reads as a broken tool.

The script therefore asks `swiftformat --rules` which rules the installed tool
knows, and enables the INTERSECTION of that answer with the twenty-nine names.
Measured on 0.62.1: 27 rules are enabled, and the two unreleased names are
absent. The day SwiftFormat ships either one, the same script enables it with
no edit here.

The intersection is what makes the roster forward-compatible, and it is also
what could swallow a typo. The acceptance test
`the_shipped_swift_idioms_tool_rule_names_only_rules_swiftformat_knows` closes
that: it reads the roster out of the shipped script, subtracts the two names
this section records as unreleased, and holds every remaining name to standing
in `swiftformat --rules`. A misspelling and a rule SwiftFormat renames each
fail it by name.

## The version floor, and where doctor reads it

`preferFinalClasses`, `redundantMemberwiseInit` and `redundantEquatable` are
recent SwiftFormat additions, and an older tool would drop them through the
intersection above without a word. The floor is **0.62.1** — the version every
rule of the roster was measured present in, and the newest release.

`--min-version` is SwiftFormat's own gate for this, and it is stated twice.

- `doctor.check_command` ends in `printf '' | swiftformat --lint --quiet
  --min-version 0.62.1`. Measured: exit 0 at the floor, and exit 70 with
  `error: Project specifies SwiftFormat --min-version of 0.99.0.` above it. A
  machine whose swiftformat is too old therefore reports as a missing tool in
  `sah doctor`, with `brew install swiftformat` as the fix, and no review ever
  reaches the run.
- The run states `--min-version 0.62.1` as well, so a tool that changed under
  the doctor check still cannot under-report. The script reads that one error
  line apart from every other status-70 line and exits 1 for it, because a too
  old tool is a broken tool for every file rather than one path it declined.

The rule declares no install commands. Homebrew is the supported way to install
swiftformat and it installs the current version only, so a Homebrew command
cannot pin one — the same reason `function-length-swift`, `missing-docs-swift`
and `dead-code-swift` each state. The `doctor.fix_hint` names the Homebrew
command instead. `sah doctor` shows that hint as the fix; the install lifecycle
never runs it.

## Why the script runs swiftformat once for each file

One refusing path costs a single swiftformat run EVERY finding it made. This is
where swiftformat differs from swiftlint, and it is what shapes the loop.
Measured on 0.62.1, each run over `Dirty.swift`, which holds one `typeSugar`
finding, beside one refusing path:

| the refusing path | status | findings | stderr |
|---|---|---|---|
| a path that holds no file | 70 | 0 | `error: File not found at <path>.` |
| a file with no read permission | 70 | 0 | `error: Failed to read file <path>.` |
| a file whose bytes are not UTF-8 | 70 | 0 | the same line |
| a file the parser recovers nothing from | 70 | 0 | `error: Unexpected token @@@ at 2:5 in <path>.` |

Every row reports ZERO, and `Dirty.swift` is judged in none of them.
`builtin/validators/README.md` refuses that answer in as many words: "A nonzero
exit fails the WHOLE run, so one unjudged path throws away every finding the
run did make."

So the script hands swiftformat ONE path for each run. A refusing path then
costs its own file and nothing more, and the script writes one line opening
`sah-diagnostic:` that names the path and carries swiftformat's own words.
Measured with the shipped script over the four refusing paths above, each
staged beside the failing fixture, which holds 42 findings:

| the run | findings | marked lines | exit |
|---|---|---|---|
| the failing fixture alone | 42 | 0 | 0 |
| the passing fixture alone | 0 | 0 | 0 |
| a path that holds no file, beside the failing fixture | 42 | 1 | 0 |
| a file with no read permission, beside it | 42 | 1 | 0 |
| a file whose bytes are not UTF-8, beside it | 42 | 1 | 0 |
| a file the parser recovers nothing from, beside it | 42 | 1 | 0 |
| a path that holds no file, alone | 0 | 1 | 0 |

The `[ ! -e "$file" ]` test stands before swiftformat, because a path that
holds no file is the one shape a run can answer without starting the tool. The
other three take swiftformat's own message: the script reads the first line
opening `error: ` out of stderr and writes it after the path, so the marked
line says WHICH file and WHY. A status over 1 that wrote no `error: ` line at
all still writes a marked line, naming the status.

## What the project's own `.swiftformat` decides, and what it cannot

swiftformat reads a project `.swiftformat` on its own, and the command line
this script writes wins the parts that matter. Measured on 0.62.1 over
`Sources/Dirty.swift` and `Generated/Dirty.swift`, which hold the same bytes,
beside a project `.swiftformat` holding `--disable typeSugar` and
`--exclude Generated`:

| the run | `Sources/` | `Generated/` |
|---|---|---|
| the shipped shape | 1 finding | 0 findings |
| the same, plus `--config` naming a file of the rule's own | 1 finding | 1 finding |

So the project cannot turn a rule of this gate off: measured against a project
that states `--disable typeSugar`, `--enable indent` and `--rules indent`, the
run reports the same 16 findings it reports with no project file at all. And
the project's `--exclude` list still holds, which is the generated-code
carve-out `function-length-swift` records for swiftlint, reached the same way.

The script therefore names no `--config`. A `--config` path takes the project's
own file out of the run whole, and with it the one thing the project should
own.

## The Swift language version, which the project states

Five rules of the roster read the Swift language version, and swiftformat takes
it from a `.swift-version` file beside the code. Measured over the two probe
files that carry the failing fixture's idiom and performance groups:

| the run | rules that reported |
|---|---|
| beside `.swift-version` holding `6.3` | `preferCountWhere`, `environmentEntry`, `genericExtensions`, `opaqueGenericParameters`, `conditionalAssignment`, and the rest |
| with no `.swift-version` | the rest alone |

The run passes NO `--swift-version` of its own. A project that never stated its
Swift version would then be told to write `values.count(where:)`, which needs
Swift 6.0 — a finding its own toolchain cannot satisfy. swiftformat declines to
suggest an API the version may lack, and that is the correct answer. A project
that wants those five rules writes its version in `.swift-version`, which is
swiftformat's own mechanism and a fact the project already owns.

Measured: `Package.swift` is not read for this. A directory holding
`// swift-tools-version:5.9` and no `.swift-version` still gets the warning
swiftformat writes when no version is stated.

## The two Airbnb options that contradicted a prompt rule

Airbnb's configuration states two answers this project already answered in
`builtin/validators/swift/`. A tool and a prompt rule that disagree produce
churn on every review round, so each was measured and decided before the roster
took it.

### `noGuardInTests` is ENABLED, and `optionals.md` gained the test carve-out

`optionals.md` asked for `guard let … else { return }` as the early exit and
carved out nothing for a test. Airbnb bans `guard` inside a test.

Both are right, and they are right about different code. In production a
`guard` protects the happy path. In a test a `guard` that returns takes the
test out BEFORE its assertions run, so a broken program reads as a pass. That
is a defect, not a style. `optionals.md` already knew tests are different, and
it now states the test bullet as well.

So the roster takes `noGuardInTests`, and the prompt rule states the same
requirement in words. Measured on 0.62.1 over one XCTest suite, the rule
reports:

| the declaration | reported |
|---|---|
| `guard let value = source else { return }` in a `throws` test | yes |
| the same in a test that is not `throws` | yes, on the `guard` and on the `func` line, because the fix adds `throws` |
| `guard let value = source else { XCTFail("missing"); return }` | yes |
| `guard 1 == 1 else { return }` | yes |
| `guard let value else { return }`, the shorthand | NO |
| `guard let source else { return 0 }` in a `private` helper of the suite | NO |

The shorthand row is why that bullet STAYS in `optionals.md` whole. It binds
the same name from the same optional and the bullet reads for both shapes, so
the prompt half carries the one this gate misses. Every other bullet of the
roster table came out of the prompt text; this one did not.

The rule reads a TEST, not a file name: a production method named
`testConnection` holding a `guard` draws nothing, measured over a file that
imports no test framework.

`--guard-like-if-statements convert` stands on the command line with it. The
option is the only difference between two measured runs over the same probe: a
trailing `if let value = source { … }` that wraps the assertions of a test
reports THREE lines under the option and NOTHING without it. It is the same
defect the guard half decides — the assertions never run when the binding
fails, and the test passes anyway. An `if let` a test asserts AFTER does not
trail the body, and stays silent. Where the intent really is a conditional
failure report, the message names `XCTAssert(...)` beside `#require`, and the
assertion that matches the intent is the edit.

### `--property-types inferred` is NOT enabled, because it loses

Airbnb sets `--property-types inferred`. `idioms.md` states the opposite for an
empty collection, and calls the reverse a validator error.

Measured on 0.62.1, `swiftformat --rules propertyTypes --property-types
inferred` over a struct holding `var items: [Int] = []`,
`var ids: Set<String> = []` and `var table: [String: Int] = [:]` reports all
three and rewrites them:

| what the file held | what the run wrote |
|---|---|
| `public var items: [Int] = []` | `public var items = [Int]()` |
| `public var ids: Set<String> = []` | `public var ids = Set<String>()` |
| `public var table: [String: Int] = [:]` | `public var table = [String: Int]()` |
| `public var name: String = "probe"` | unchanged |

Each rewrite turns the DO of `idioms.md` into its DON'T, word for word. The
empty literal names no type, so the rule was expected to leave it alone; it
does not. `idioms.md` is the deliberate house style and it is already
load-bearing — it warns in as many words against flip-flopping between the two
forms across review rounds.

So the roster names NEITHER `propertyTypes` NOR the option, and the gate
decides neither form of an empty-collection declaration. `idioms.md` owns that
bullet alone.

Two acceptance tests hold that, one for each direction the option can take.
Measured on 0.62.1 over the two forms:

| the option | the DO of `idioms.md` | the DON'T |
|---|---|---|
| `--property-types inferred` | 2 findings | silent |
| `--property-types explicit` | silent | 2 findings |

`the_shipped_swift_idioms_tool_rule_agrees_with_the_swift_prompt_rules` holds
the DO to drawing no finding, so `inferred` fails there.
`the_shipped_swift_idioms_tool_rule_decides_no_empty_collection_declaration`
holds the DON'T to the same, so `explicit` fails there. Neither direction can
be added without a test naming it.

## `testSuiteAccessControl` reports the declaration `validateTestCases` reads

The two rules read the SAME declaration — an internal method a test suite holds
whose name says test and whose attribute does not — and in a run holding both,
`testSuiteAccessControl` reports it and `validateTestCases` does not. Measured
on 0.62.1 over three shapes, each run three ways:

| the declaration | the whole roster | `validateTestCases` alone | the two together |
|---|---|---|---|
| `func alsoATest()` beside `override func setUp()` in an `XCTestCase` | `testSuiteAccessControl` | 1 finding | `testSuiteAccessControl` |
| `func verifiesTheThing() throws` in an `XCTestCase` | `testSuiteAccessControl` | 1 finding | `testSuiteAccessControl` |
| `func testAnother()` in a `struct` beside a `@Test` method | `testSuiteAccessControl` | 1 finding | `testSuiteAccessControl` |

Making the method `private` satisfies `testSuiteAccessControl`, and measured, it
silences `validateTestCases` as well: a private method is a helper, and neither
rule then reads a test in it. So the author who takes the finding at face value
never learns the method wanted a `@Test`.

That is SwiftFormat's own behaviour and no option moves it. Both rules are
enabled, because the card that states this roster names both and because a
SwiftFormat that separates them later needs no edit here. The acceptance test
`the_shipped_swift_idioms_tool_rule_reads_the_rule_swiftformat_reports_first`
holds the measurement, so a release that changes it fails a test rather than
moving a finding without a word.

## A rule that removes a range reports one finding for each line of it

`redundantMemberwiseInit` reports the whole span it would delete, so ONE
redundant initializer is five findings. Measured over a file holding two
structs, each with an explicit memberwise `init`: 10 findings, at rows 25 to 29
and 35 to 39.

The findings are reported as swiftformat writes them. Collapsing a run of
consecutive lines carrying one rule was measured and refused: `typeSugar` on
the failing fixture reports rows 13, 14 and 15 for THREE distinct properties,
and a collapse would report one of them. No field of the report tells the two
shapes apart, so every line stands, and the author who fixes the initializer
clears all five at once.

## `void` decides half of the bullet it took, and which half

`idioms.md` stated two requirements in one bullet: write `Void` rather than
`()`, and omit the return clause entirely when it is `Void`. SwiftFormat's
`void` rule decides the FIRST and not the second. Measured on 0.62.1 over one
file:

| the declaration | reported |
|---|---|
| `public static func run() -> ()` | yes |
| `public static func handler(_ body: (Int) -> ())` | yes |
| `public static func typed() -> Void {}` | NO |

`swiftformat --rules void` over the same file rewrites `-> ()` into `-> Void`
and leaves `-> Void` standing. Removing the clause is SwiftFormat's separate
`redundantVoidReturnType` rule, which this roster does not name.

Row 3 is what the tool's own FIX writes, so the half that stays is the half the
author reaches by taking the finding.

So the `()` half came out of `idioms.md` and the omit-the-clause half stays
there, written as its own bullet, and
`the_shipped_swift_idioms_tool_rule_decides_no_void_return_clause` holds both
sides of it: the gate stays silent on row 3, and `idioms.md` states the row-3
declaration word for word. A roster that later takes `redundantVoidReturnType`
takes that bullet with it.

## `preferForLoop` decides half of the bullet it took, under an option

`idioms.md` stated two requirements in one bullet as well — "Prefer a `for` loop
(with a `where` clause when filtering) over `forEach` + `if`". Write a `for`
loop rather than `forEach` + `if` when the code needs control flow, and write a
`where` clause ON THAT LOOP when it filters. The second requirement is about
the LOOP; the deleted bullet named no `filter` chain. SwiftFormat's
`preferForLoop` decides the FIRST and not the second, and the first only under
an option the rule has to name. Measured on 0.62.1, each row one file, under the
shipped script:

| the walk | shipped run | the same, without `--single-line-for-each convert` |
|---|---|---|
| `things.forEach { if $0 > 2 { print($0) } }`, on one line | reported | NO |
| the same `forEach` and `if` written over five lines | reported | reported |
| `things.forEach { print($0) }`, on one line | reported | NO |
| `values.forEach { value in print(value) }`, over three lines | reported | reported |
| `things.filter { $0 > 2 }.forEach { thing in print(thing) }` | NO | NO |
| `for thing in things { if thing > 2 { print(thing) } }` | NO | NO |

Row 1 is the shape the bullet named, word for word, and the shipped run before
this option was silent on it. Row 4 is the body the coverage-guard row used to
stage: it reports without the option and it holds no `if`, so it proved
ownership of a shape the bullet never named while row 1 went unreported.
`swiftformat --rule-info preferForLoop` states
why: `--single-line-for-each` takes `"ignore" (default) or "convert"`, so a
single-line closure is out of reach until the run says otherwise. The option
therefore stands on the command line, and
`the_shipped_swift_idioms_tool_rule_reads_a_single_line_for_each` holds row 1,
so a run that dropped the option goes quiet there rather than losing the shape
without a word.

The option costs nothing already measured. The failing fixture reports the same
**42 findings** carrying the same 21 rules with it and without it, and the
passing fixture reports 0 either way, so every count in this file stands as
written.

Rows 5 and 6 are the `where` half, and NO option reaches either. SwiftFormat
states the limit on row 5 in its own rule information — "Doesn't affect long
multiline functional chains" — and `preferForLoop` never suggests a `where`
clause even where it does convert.

Row 6 is what the tool's own FIX writes. Measured on 0.62.1,
`swiftformat --rules preferForLoop --single-line-for-each convert` rewrites row
1 into row 6, character for character, and never into
`for thing in things where thing > 2`. So the author who takes the row-1 finding
and applies SwiftFormat's own correction lands on a walk the deleted bullet
forbade, and row 6 is why the `where` half has to name that walk.

So that half stays in `idioms.md`, written as its own bullet holding both
DON'Ts, and `the_shipped_swift_idioms_tool_rule_decides_no_shape_of_the_where_half`
holds both sides of each row: the gate stays silent on rows 5 and 6, and
`idioms.md` states each declaration word for word.

## Why this rule supersedes nothing

`supersedes` names a whole prompt rule, and the engine skips that rule whole
when the tool is healthy. This gate decides SEVEN bullets spread across three
prompt rules — five of `builtin/validators/swift/rules/idioms.md`, one of
`value-semantics.md` and one of `optionals.md` — and no one of those rules is
only those bullets. Naming any of them here would take its OTHER bullets out of
every review the moment swiftformat is installed. `idioms.md` still states the
empty-collection declaration, the omit-the-clause half, the `where` half and
the type-name repetition; `value-semantics.md` still states four bullets;
`optionals.md` still states three.

The prompt half has landed. Six of the seven bullets are this gate's — four
whole, and the `void` and `preferForLoop` rows for the halves the two sections
above measure. The seventh, `optionals.md` never `guard` in a test, stays whole
for the reason the shorthand row records. One requirement takes one owner, and
`the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` is what holds it
that way.

`stuttering-name-go` and `unused-dependencies-rust` are the two shipped tool
rules that already declare no `supersedes`, so an empty key is the stated shape
rather than an omission.

## The directive an author writes

Selection in the roster is attribution, not exemption. To exempt one
declaration, write `// swiftformat:disable:next <rule>` on the line DIRECTLY
above it, and name the rule the finding names — the message carries it, because
the script writes `<rule_id>: <reason>` as the finding text:

    // swiftformat:disable:next preferFinalClasses  subclassed by the test double
    public class Worker {

`preferFinalClasses` states three more escapes of its own, in its own message:
put `Base` in the class name, write a doc comment mentioning "base class" or
"subclass", or make the class `open`.

`// swiftformat:disable <rule>` runs to a matching `// swiftformat:enable`, and
a `.swiftformat` at the project root carries an `--exclude` list for a whole
directory. The `--exclude` list is the generated-code carve-out, and the
section above states the measurement behind it.

## What the fixture pair holds

The failing fixture holds one declaration for each of the five groups, each
written in the form its rule reports, under a `// MARK:` heading that names the
group. Measured with the shipped script, over a directory that states no
`.swift-version`: **42 findings**, exit 0, carrying 21 of the 27 enabled rules.
The six that stay silent are the five the section above records as
version-gated, and `validateTestCases`, which the section above records as
unreachable behind `testSuiteAccessControl`.

The passing fixture holds the SAME declarations, each written in the form its
rule asks for. Measured with the shipped script: 0 findings, exit 0, and 0
bytes on stderr.

The doctor materializes the fixture directory flat and hands this rule one
path, so neither fixture reads the other. The fixtures carry no
`.swift-version`, so the five version-gated rules stay silent in the doctor's
run; the acceptance tests stage a probe repository of their own and hold those
five to reporting beside a `.swift-version` and to staying silent without one.

## What the Swift toolchain measures, and what it cannot

`swift format` is a SUBCOMMAND of the Swift toolchain, spelled with a space. It
is a different program from `swiftformat`, which is what this gate runs today.
A move from one to the other needs facts, and the eight sections below are
those facts. **This section changes no gate. `swiftformat` still runs the gate.**

Every measurement below was made with the toolchain of this machine and of the
self-hosted macOS CI runner:

    Apple Swift version 6.4 (swiftlang-6.4.0.33.1 clang-2100.3.33.1)
    Target: arm64-apple-macosx27.0.0

`swift format --version` writes `main`, so it names no release. The Swift
version above is the only version a measurement can carry. The binary stands at
`/Applications/Xcode-beta.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/swift-format`.

`swift format dump-configuration` writes the configuration the tool would read.
It holds 43 keys under `rules`, and each key is one rule.

## The tags no configuration can stop

`swift format lint` writes one line for each finding, and each line carries a
`[<Name>]` tag. Eight of those tags come from the PRETTY-PRINTER, not from the
43 rules. No key of the configuration reaches them.

Each row below was measured over its own probe file, with every one of the 43
rules switched OFF:

    swift format dump-configuration > all-off.json    # then each rule set to false
    swift format lint --strict --configuration all-off.json <probe>.swift

| the tag | what draws it | what the run wrote |
|---|---|---|
| `AddLines` | two members on the brace line of a `struct` | `add 1 line break`, at 1:15 and at 2:16 |
| `EndOfLineComment` | a trailing comment that runs past column 100 | `move end-of-line comment that exceeds the line length` |
| `Indentation` | a member indented 8 spaces where 2 are asked for | `unindent by 6 spaces` |
| `LineLength` | a declaration that runs past column 100 | `line is too long` |
| `RemoveLine` | four blank lines between two declarations | `remove line break`, at 1:14 and at 2:1 |
| `Spacing` | `let alpha = 1+2` | `add 1 space`, at 1:14 and at 1:15 |
| `TrailingComma` | an array literal over several lines, with no trailing comma | `add trailing comma to the last element in multiline collection literal` |
| `TrailingWhitespace` | three spaces after a declaration | `remove trailing whitespace` |

The eight names come from the tool itself. `PrettyPrintFindingCategory` stands
in the binary, and the reflection data beside it lists the same eight cases the
eight probes draw.

So the configuration is only a SECOND gate. A gate built on `swift format` must
read the tag off each output line, and it must KEEP only a tag that stands in an
allowlist the script states. A gate that kept every line would report layout.

`the_shipped_swift_idioms_rule_body_names_every_tag_swift_format_writes` holds
this table. It reads the tag column out of this body, and it drives one probe
for each tag through the live toolchain. A release that stops writing a tag
fails that test BY NAME.

## What the shipped passing fixture draws from the pretty-printer

The leak is not a corner case. `missing-docs-swift.pass.swift.tmpl` is a fixture
this set ships, and every declaration in it is correct:

    swift format lint --strict missing-docs-swift.pass.swift.tmpl

| the configuration | findings | the tags |
|---|---|---|
| all 43 rules OFF | 2 | `[Indentation]`, at 9:1 and at 10:1 |
| the default, all 43 rules as shipped | 2 | `[Indentation]`, at the same two rows |

The fixture is written with 4-space indentation. `swift format` asks for 2. The
two rows are the two members of `DocumentedStructure`. A gate with no allowlist
would fail this set's own passing fixture.

## Every status the lint run writes

`swift format lint --strict` was run over each shape below. The run states no
configuration, so it reads the 43 rules as shipped:

    swift format lint --strict Probe.swift

| the run | status | stdout | stderr |
|---|---|---|---|
| a file with findings | 1 | 0 bytes | `Probe.swift:1:14: error: [Spacing] add 1 space`, twice |
| a file with no finding | 0 | 0 bytes | 0 bytes |
| a path that holds no file | 0 | 0 bytes | 0 bytes |
| a file with no read permission | 1 | 0 bytes | `<unknown>: error: Unable to open Probe.swift: file is not readable or does not exist` |
| a file whose bytes are not UTF-8 | 1 | 0 bytes | `<unknown>: error: Unable to lint Probe.swift: file is not readable or does not exist.` |
| a file the parser cannot read | 1 | 0 bytes | `Probe.swift:2:4: error: expected name in attribute`, three times |
| a path that names a directory | 64 | 0 bytes | `Error: 'Probe.swift' is a path to a directory, not a Swift source file.` |

Three answers in that table decide how a gate is shaped.

**Row 3 is the dangerous one.** A path that holds no file exits 0 and writes
NOTHING. That reads exactly like a clean pass over a file the run never opened.
A gate must test the path itself before it starts the tool, the way the shipped
script already does with `[ ! -e "$file" ]`.

**Every line stands on STDERR.** Findings and errors share that one channel, and
stdout holds 0 bytes in every row. A gate that read stdout would read an empty
channel for a file that holds findings.

**A refusing path costs its own file alone.** This is where `swift format`
differs from `swiftformat`. Measured with each refusing path beside a file that
holds two findings: the run reports both findings each time, and it exits 1.

The section "Why the script runs swiftformat once for each file" holds four
rows for the same four refusing paths, and it records the opposite answer: under
`swiftformat`, one refusing path costs the WHOLE run and every finding with it.
**Those four rows are about `swiftformat`, and they stay true for the gate that
ships today. The table above replaces them the day the gate moves to
`swift format`, and not before.** A gate that moves therefore needs no loop of
one path for each run, because `swift format` takes a whole work list and keeps
the findings of every path it could read.

`the_shipped_swift_idioms_rule_body_records_every_status_the_lint_run_writes`
holds this table. It reads the run column and the status column out of this
body, and it drives each shape through the live toolchain.

## `DontRepeatTypeInStaticProperties` reports one shape

The rule DOES report. An earlier probe found it silent, because that probe named
a property of type `Int`.

Each row below is a `public struct Color` that holds one member, over three
lines, with that one rule ON:

| the declaration | reported |
|---|---|
| `public static let redColor = Color()` | YES — `remove the suffix 'Color' from the name of the variable 'redColor'` |
| `public static let redColor = 1` | NO |
| `public static let redColor: Color = Color()` | NO |
| `public static var redColor: Color { Color() }` | NO |
| `public static let colorRed = Color()` | NO |
| `public static let red = Color()` | NO |
| `public static func makeColor() -> Color { Color() }` | NO |
| `public static let bluePalette = Palette()`, in `public enum Palette` | YES |
| `public static let darkTheme = Theme()`, in `public final class Theme` | YES |
| `public static let greenColor = Color()`, in `extension Color` | YES |
| `static var redColor: Color { get }`, in `public protocol Color` | NO |

The rule reads three facts at one time. The member must be `static` or `class`.
Its TYPE must be the type that holds it, and the rule reads that type from an
initializer that CALLS the type. And the member name must end in the type name
as a SUFFIX: `colorRed` and `redColor2` each pass.

An annotation makes the rule silent when an initializer stands beside it. Row 3
holds `redColor: Color = Color()` and draws nothing, and row 1 holds the same
member with the annotation off and reports.

Two more shapes were measured, and neither stands in the table, because Swift
compiles neither. `struct Sandwich { static let bolognaSandwich: Sandwich }`
written over three lines reports
`remove the suffix 'Sandwich' from the name of the variable 'bolognaSandwich'`,
and the same declaration written on ONE line draws nothing. `swiftc -typecheck`
refuses both: `'static let' declaration requires an initializer expression or an
explicitly stated getter`.

So `idioms.md` KEEPS the type-name bullet. The bullet states `static let redColor`
on `Color` and it names no type. The tool decides that bullet only where the
member's own type is the enclosing type, which is one shape of many.

## `UseSynthesizedInitializer` reads the access level

Each row below is a `struct Point` that holds `var x: Int` and `var y: Int`,
beside one explicit memberwise initializer, with that one rule ON. The
`swiftformat` column names what the gate that ships reports over the same file:

| the initializer | `swift format` | `swiftformat --rules redundantMemberwiseInit` |
|---|---|---|
| an `internal` `init` of an `internal` struct | YES | 5 findings |
| a `public` `init` of a `public` struct | NO | 0 findings |
| an `internal` `init` of a `public` struct | YES | 5 findings |
| a `private` `init` of an `internal` struct | NO | 0 findings |
| an `init` whose parameter carries a default value | YES | 0 findings |
| a `public` `init` of a `public` struct that holds `internal` properties | NO | 0 findings |
| an `init` whose body writes `self.x = max(0, x)` | NO | 0 findings |

The message reads `remove this explicit initializer, which is identical to the
compiler-synthesized initializer`.

The silent half is correct, and it is not a hole. Swift synthesizes an INTERNAL
memberwise initializer, never a `public` one, so a `public init` is not identical
to it. Deleting a `public init` takes a declaration off the package surface.

The rule is therefore the OWNER of the internal half and of nothing more. A
prompt bullet that wants the public half must state it in words, and it must
state that half alone: a bullet that named both halves would fight the tool on
every review round.

Row 5 is where the two tools differ. `swift format` reports the default-value
form, and `swiftformat` stays silent about it.

## `UseWhereClausesInForLoops` and `immutability.md` disagree

`builtin/validators/swift/rules/immutability.md` names TWO shapes as a DON'T, in
one bullet, and its fix is `map` or `filter`:

```swift
// the first DON'T
for user in users where user.isActive { names.append(user.name) }
// the second DON'T
for user in users { if user.isActive { names.append(user.name) } }
```

Measured with `UseWhereClausesInForLoops` ON, over one file for each shape:

| the walk | reported | the fix `swift format --in-place` wrote |
|---|---|---|
| the second DON'T, the nested `if` | YES — `replace this 'if' statement with a 'where' clause` | the FIRST DON'T, character for character |
| the first DON'T, the `where` clause | NO | — |
| `let names = users.filter(\.isActive).map(\.name)`, the DO | NO | — |

The conflict is real and it is measured. An author who takes the finding, and who
applies the tool's own correction, lands on the first DON'T of `immutability.md`,
word for word. The rule is `false` in the shipped configuration, so a gate that
turned it ON would create this conflict rather than find it.

This is the same class of conflict this repository already refused for
`--property-types inferred`. A tool and a prompt rule that disagree make churn on
every review round.

A person must choose one of two answers:

1. **Keep `UseWhereClausesInForLoops` OFF.** `idioms.md` and `immutability.md`
   then keep the whole question, and the tool decides nothing here.
2. **Rewrite the carve-out of `immutability.md`** so the two rules cannot
   disagree. The bullet would then stop naming the `where` form as a DON'T.

**This body chooses neither.** The evidence stands above. The choice is a
person's.

## Which rule reports a filter chain

The `where` bullet of `idioms.md` names a filter chain as its second DON'T:

```swift
things.filter { $0 > 2 }.forEach { thing in print(thing) }
```

Measured over one file for each walk, each rule ON alone and then both together:

| the walk | `UseWhereClausesInForLoops` | `ReplaceForEachWithForLoop` | the two together |
|---|---|---|---|
| `things.filter { $0 > 2 }.forEach { thing in print(thing) }` | NO | YES, at 2:28 | `ReplaceForEachWithForLoop` |
| `things.forEach { if $0 > 2 { print($0) } }` | NO | YES, at 2:10 | `ReplaceForEachWithForLoop` |
| `for thing in things where thing > 2 { print(thing) }` | NO | NO | NO |

`ReplaceForEachWithForLoop` reports the chain. Its message reads `replace use of
'.forEach { ... }' with for-in loop`.

The shape its fix lands on is NOTHING. Measured: `swift format --in-place` with
that rule ON leaves the file byte for byte as it was, with the rule alone and
with both rules together. The rule REPORTS and it CORRECTS nothing.

That is the opposite of `preferForLoop`, which the section "`preferForLoop`
decides half of the bullet it took, under an option" records. `preferForLoop`
rewrites the walk, and this body has to name the shape it writes. A rule that
writes no fix needs no such row: the author writes the edit, and the message
names the shape.

## The directive an author writes for `swift format`

The shipped documents tell an author to write `// swiftformat:disable:next <rule>`.
`swift format` reads no such comment.

Each row below is one file holding `let Bad_One = 1`, with
`AlwaysUseLowerCamelCase` ON:

| the directive | where it stands | reported |
|---|---|---|
| `// swiftformat:disable:next AlwaysUseLowerCamelCase` | the line above | YES — the comment does nothing |
| `// swift-format-ignore` | the line above | NO |
| `// swift-format-ignore: AlwaysUseLowerCamelCase` | the line above | NO |
| `// swift-format-ignore-file` | the top of the file | NO, for every declaration |
| `// swift-format-ignore-file: AlwaysUseLowerCamelCase` | the top of the file | NO, for every declaration |
| `// swift-format-ignore` | above the first of two declarations | the SECOND declaration alone |
| `// swift-format-ignore` | after the declaration, on the same line | YES — a trailing comment does nothing |

So both forms work, and each covers ONE declaration. The line form is
`// swift-format-ignore` on the line above the declaration, and
`// swift-format-ignore: <RuleName>` names one rule. The file form is
`// swift-format-ignore-file` at the top, and it takes the whole file out.

A rule name the tool does not know is accepted, and it silences nothing.
Measured: `// swift-format-ignore: NoSuchRuleName` above `let Bad_One = 1` still
reports.

The directive reaches the pretty-printer as well, and only in its bare form. Each
row below is a file with a member indented 8 spaces, with all 43 rules OFF:

| the directive | the `[Indentation]` finding |
|---|---|
| none | YES |
| `// swift-format-ignore` above the declaration | NO |
| `// swift-format-ignore: Indentation` above the declaration | YES |
| `// swift-format-ignore-file` at the top | NO |

A tag is not a rule name, so the named form cannot reach one. An author who wants
to keep one layout finding out must write the bare form, and that form takes
every rule off the declaration with it.

## The Swift version floor, and the version CI runs

The floor is **Swift 6.0**, and the `swift format` SUBCOMMAND sets it. Neither
rule of this section sets it, because each shipped long before the subcommand
did.

| the question | how it was measured | the answer |
|---|---|---|
| when `UseWhereClausesInForLoops` first shipped | `Sources/SwiftFormatRules/UseWhereClausesInForLoops.swift` on `release/5.6` | HTTP 200, so Swift 5.6 or older |
| when `AlwaysUseLiteralForEmptyCollectionInit` first shipped | the same directory on `release/5.9`, then `Sources/SwiftFormat/Rules/` on `release/5.10` | HTTP 404, then HTTP 200, so Swift 5.10 |
| when `swift format` first ran | the `README.md` of `release/5.10` and of `release/6.0` | Swift 6.0 |

Each row is one `curl` against the `swiftlang/swift-format` repository, over the
release branch of each Swift version. The `README.md` of `release/6.0` states the
answer to row 3 in its own words: "Xcode 16 and above include swift-format in the
toolchain. You can run `swift-format` from anywhere on the system using
`swift format` (notice the space instead of dash)." The `README.md` of
`release/5.10` carries no such line.

**`swift format` reads no `.swift-version` file.** Measured over two directories
that hold the same file, with `AlwaysUseLiteralForEmptyCollectionInit` ON:

| the directory | findings |
|---|---|
| beside a `.swift-version` holding `6.0` | 2 |
| with no `.swift-version` | 2 |

`swift format lint --help` names no `--swift-version` option, and the binary
carries no `.swift-version` string. This is the opposite of `swiftformat`, which
the section "The Swift language version, which the project states" records as
reading that file for five rules of the roster.

The rule under measurement writes the DO of `idioms.md` as its fix:
`replace '[Int]()' with ': [Int] = []'`, and
`replace '[String: Int]()' with ': [String: Int] = [:]'`. That is the opposite
direction from `swiftformat --property-types inferred`, which the section "Why
`--property-types inferred` is NOT enabled" records.

**The CI runner runs the same toolchain.** The self-hosted macOS runner writes
`swift --version` in the step "Ensure the language toolchains the roster tests
need are available". Read from run `34129568075`, at `2026-09-07T14:09:18Z`:

    Apple Swift version 6.4 (swiftlang-6.4.0.33.1 clang-2100.3.33.1)
    Target: arm64-apple-macosx27.0.0
    swift-driver version: 1.168.6

6.4 stands over the floor of 6.0, so a gate that moved to `swift format` would
not turn CI red for want of a toolchain.
