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
        --guard-like-if-statements convert \
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

Every measurement below was made with SwiftFormat 0.62.1.

## Which rules the gate enables

The roster is Airbnb's, taken from
`Sources/AirbnbSwiftFormatTool/airbnb.swiftformat` on their `master` branch,
and narrowed to the rules that decide an IDIOM. Twenty-nine names stand in the
script, in five groups.

Seven of them decide a bullet a prompt rule of `builtin/validators/swift/` used
to state. Six of those bullets are out of the prompt text and this gate is
their ONE owner. The seventh stays there whole, because this gate reads only
part of it.

| the rule | the option it needs | the bullet it took |
|---|---|---|
| `typeSugar` | `--short-optionals always` | `idioms.md` shorthand type sugar |
| `void` | | `idioms.md` return `Void`, not `()` — HALF of that bullet |
| `redundantMemberwiseInit` | | `idioms.md` no memberwise init identical to the synthesized one |
| `preferForLoop` | | `idioms.md` `for` loop over `forEach` |
| `hoistPatternLet` | `--pattern-let inline` | `idioms.md` bind each case variable with its own `let` |
| `preferFinalClasses` | | `value-semantics.md` mark classes `final` |
| `noGuardInTests` | `--guard-like-if-statements convert` | `optionals.md` never `guard` in a test — the bullet STAYS |

`the_shipped_swift_idioms_tool_rule_owns_each_bullet_it_took` holds each of the
six taken rows to both halves of its own claim: the rule reports a file holding
that ONE defect, and the prompt rule states the bullet no longer. A rule that
went silent, and a bullet written back into the prompt text, each fail it by
name.

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

So the `()` half came out of `idioms.md` and the omit-the-clause half stays
there, written as its own bullet. A roster that later takes
`redundantVoidReturnType` takes that bullet with it.

## Why this rule supersedes nothing

`supersedes` names a whole prompt rule, and the engine skips that rule whole
when the tool is healthy. This gate decides SEVEN bullets spread across three
prompt rules — five of `builtin/validators/swift/rules/idioms.md`, one of
`value-semantics.md` and one of `optionals.md` — and no one of those rules is
only those bullets. Naming any of them here would take its OTHER bullets out of
every review the moment swiftformat is installed. `idioms.md` still states the
empty-collection declaration, the omit-the-clause half above and the
type-name repetition; `value-semantics.md` still states four bullets;
`optionals.md` still states three.

The prompt half has landed. Six of the seven bullets are this gate's — five
whole, and the `void` row for the half the section above measures. The seventh,
`optionals.md` never `guard` in a test, stays whole for the reason the
shorthand row records. One requirement takes one owner, and
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
