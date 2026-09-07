---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m1ydxkqe4nhzfynnk044mhss
  text: |-
    Research done. Measurements made on this machine, Apple Swift 6.4.

    New facts the card's table did not hold, each measured:

    1. A `--configuration` whose `rules` table names only some rules turns EVERY
       unnamed rule OFF. `AlwaysUseLowerCamelCase` is ON by default and reports
       `let Bad_One = 1`; under a `rules` table naming only
       `AlwaysUseLiteralForEmptyCollectionInit` it is silent. So one small JSON
       file states the whole rule set, and the other configuration keys
       (`lineLength`, `indentation`) keep their defaults.
    2. The nine unstoppable tags still come through that configuration:
       `[Indentation]` and `[LineLength]` each reported under it. The tag filter
       stays the gate.
    3. A DIRECTORY in a shared work list costs the WHOLE run: `swift format lint
       --strict Dirty.swift Adir.swift` exits 64, writes ZERO findings, and
       `Dirty.swift` is judged in none of it. That is the measured reason to keep
       one run for each file, and it is stronger than the reason the body records.
    4. The run's `--configuration` wins over a project `.swift-format`: a project
       file holding `"AlwaysUseLiteralForEmptyCollectionInit": false` did not stop
       the finding.
    5. `AlwaysUseLiteralForEmptyCollectionInit` reports `[Int]()` and
       `[String: Int]()`. It stays SILENT for `Set<String>()`.
    6. `NoVoidReturnOnFunctionSignature` reports `-> ()` AND `-> Void`, and stays
       silent for a closure parameter `(Int) -> ()`. That last shape belongs to
       `ReturnVoidInsteadOfEmptyTuple`, which the allowlist does not name.
    7. `printf '' | swift format lint --strict -` exits 0, so it is a doctor probe.
       `swift format --version` writes `main` and names no release, so the floor is
       read off the SUBCOMMAND standing at all.

    Blast radius the card did not list, found by reading the callers:

    - `tests/shipped/swift_guidelines_partial.rs` reads the shipped `idioms-swift`
      script for `swiftformat --lint` and five options. Those options go away with
      this move, so that half of
      `swift_partial_agrees_with_the_shipped_swift_tool_validators` loses its
      source and the test would fail. Task ^ywgq61n owns the partial rewrite; this
      task makes the minimum edit that keeps the test honest and green.
    - `tests/shipped/swift_judgment_rules.rs` uses `SWIFT_VERSION_SUPPORT`, which
      this task deletes.
    - `idioms.md` still states the empty-collection bullet and the omit-the-clause
      bullet, and the gate now decides both. That two-owner state is temporary and
      task ^2hd90th owns it, so no prompt bullet moves here.
    - `preferFinalClasses` has no successor rule in the toolchain, so the
      `value-semantics.md` `final` bullet has no owner until ^2hd90th returns it.
      Its row leaves the superseded-bullet table here.
  timestamp: 2026-09-07T17:16:11.886218+00:00
- actor: claude-code
  id: 01m1yfx1047wrp2taeh1c17ge8
  text: |-
    Implementation landed. What the gate is now:

    The script writes ONE list of seven rule tags, and that list does two jobs: a
    `sed` turns it into the `{"rules": {...}}` configuration the run passes with
    `--configuration`, and a `paste` turns it into the alternation the output
    filter reads. One list, so the two cannot drift.

    Per file: `[ ! -e "$file" ]` first, then
    `swift format lint --strict --configuration ... "$file" > /dev/null 2>err`
    with `|| status=$?`. A stderr line holding `: error: [` is a tagged finding;
    every other line is trouble, and a file that wrote one is declined whole with
    a `sah-diagnostic:` line. The findings pass through
    `grep -E ": error: \[($allowed)\] "` and a `sed` that writes
    `path:line: RuleName: reason`.

    TDD evidence, both directions measured:

    - RED, whole set: with the rule file and the two fixtures stashed back to the
      swiftformat version, `cargo nextest run -p swissarmyhammer-validators
      idioms_swift` answered 6 passed, 9 FAILED.
    - RED, the filter alone: with the allowlist alternation replaced by
      `[A-Za-z]*` in the shipped script, the same run answered 11 passed, 4
      FAILED, and
      `the_shipped_swift_idioms_tool_rule_drops_every_tag_outside_its_allowlist`
      failed with its own message — "the `[AddLines]` probe holds a layout defect
      and no idiom ... the run reported [\"AddLines\", \"AddLines\"]".
    - GREEN: 15 of 15 pass, and `cargo nextest run -p swissarmyhammer-validators
      swift` answers 86 of 86.

    Measured numbers now written into the body and held by a test:

    - The failing fixture: 11 findings, exit 0, carrying all 7 allowlist rules.
    - The passing fixture: 0 findings through the filter, while the raw
      `swift format lint --strict` over the same bytes writes 34 `[Indentation]`
      lines. The doctor's own fixture pair therefore measures the filter.
    - Each of the nine unstoppable tags: 0 findings through the shipped script.

    Two things this task did NOT do, both by the card's own order:

    - No prompt bullet moved. `idioms.md` still states the empty-collection bullet
      and the omit-the-clause half of the `Void` bullet, and the gate now decides
      both. The body names that two-owner state and names the task that ends it.
    - `UseWhereClausesInForLoops` stays OFF, and
      `the_shipped_swift_idioms_tool_rule_leaves_the_where_clause_rule_off` holds
      the shipped script to naming it NOWHERE — a configuration key alone would
      turn it on with no allowlist entry.

    `preferFinalClasses` left the superseded-bullet table, because the toolchain
    carries no rule like it. The `value-semantics.md` `final` bullet therefore has
    no owner until task ^2hd90th states it again. The body says so in as many
    words.
  timestamp: 2026-09-07T17:50:49.860398+00:00
- actor: claude-code
  id: 01m1yfxe5z161nce1fya740dmd
  text: |-
    One test item of the card is answered in part, and the gap is pre-existing.

    "Run `sah doctor`. The rule reports as healthy, and its fixture pair passes."

    The fixture pair passes. `every_shipped_idioms_tool_rule_passes_its_fixtures`
    drives the doctor's OWN `check_tool_rule` path over the shipped pair, and it
    passes.

    `sah doctor` writes no ROW for this rule. Measured over a probe Swift package,
    the table names `dead-code-swift`, `function-length-swift`,
    `magic-numbers-swift` and `missing-docs-swift`, and neither `idioms-swift` nor
    `disallowed-constructs-swift`. The same probe answers the same 4 rows with the
    working tree stashed back to commit `4fed2803d`, so the defect predates this
    change and is not its work. Task ^15vxtng records it, with what is already
    known and the hypothesis to measure first.

    Two more notes for the tasks that follow.

    - ^ywgq61n owns the partial. This task made ONE minimum edit to
      `swift_guidelines_partial.rs` to keep it honest: the half of
      `swift_partial_agrees_with_the_shipped_swift_tool_validators` that read five
      swiftformat options off the shipped script had no source left, so it now
      reads the LINT COMMAND HEAD off the shipped script and holds the partial to
      naming it. The partial already writes `swift format lint`, so no partial
      edit was needed. The empty-collection half is untouched.
    - ^37yt9xy owns the document sweep.
      `crates/swissarmyhammer-validators/src/review/tool_rules/tests.rs` still
      carries a doc comment saying this script intersects its roster with
      `swiftformat --rules`. That sentence is now false, and the card of ^37yt9xy
      lists that file, so it was left there.
  timestamp: 2026-09-07T17:51:03.359919+00:00
- actor: claude-code
  id: 01m1yfxq7etkx42z8m4bs6kvp0
  text: |-
    ### implement — changed
    - evidence: 8 files — builtin/validators/code-hygiene/rules/idioms-swift.md, builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl, builtin/validators/code-hygiene/fixtures/idioms-swift.pass.swift.tmpl, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_judgment_rules.rs, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/swift_guidelines_partial.rs, crates/swissarmyhammer-validators/src/builtin/mod.rs. `cargo nextest run -p swissarmyhammer-validators swift` — 86 run, 86 passed. `cargo nextest run -p swissarmyhammer-validators` — 854 run, 816 passed, 38 failed; the same 38 names fail at HEAD with the change stashed, so every one is pre-existing (missing go, revive, staticcheck, golangci-lint and eslint-plugin-sonarjs). `cargo clippy -p swissarmyhammer-validators --tests` — 0 warnings. `cargo fmt --check` — clean.
    - next: /review
  timestamp: 2026-09-07T17:51:12.622142+00:00
- actor: claude-code
  id: 01m1ygnfqt7bs6jgh3m5ab7qac
  text: |-
    ### review — findings
    - evidence: `review sha HEAD~1..HEAD` (checkpoint 7f5e52261) gave 0 findings over 5 Rust files. The engine matched no validator to `builtin/validators/code-hygiene/rules/idioms-swift.md`, which carries the gate script, so that file had no cover. Direct verification of the script found 3 items: idioms-swift.md:46, idioms-swift.md:40, and tests/shipped/idioms_swift.rs.
    - The cause is one: both greps match `: error: [` as text anywhere in the line, and `swift format` writes the file path at the head of each line. A path that holds that text carries a tag outside the allowlist through the gate, and it also stops the decline guard.
    - Measurements that hold as stated: fail fixture 11 findings over 7 rules; pass fixture 0 findings; 34 raw `[Indentation]` lines and 0 through the filter; two reporting files give the findings of both at exit 0; the run rewrites no file; `UseWhereClausesInForLoops` is OFF. `cargo nextest run -p swissarmyhammer-validators swift` — 86 passed.
    - next: anchor both greps to the true `path:line:column:` head, then add the probe named for a diagnostic message that `missing_docs.rs` already sets as the pattern.
  timestamp: 2026-09-07T18:04:11.386194+00:00
- actor: claude-code
  id: 01m1ygpdkxwwhrh77w4227chc1
  text: |-
    ### finish iteration 1 — findings
    - implement: changed — 8 files. The gate now runs `swift format lint`. One list of seven rule tags writes the configuration JSON and the output filter, thus the two cannot become different.
    - test: green — cargo nextest -p swissarmyhammer-validators swift, 86 run, 86 passed; cargo fmt --all --check clean; cargo clippy -p swissarmyhammer-validators --all-targets -D warnings clean. The driver ran these.
    - commit: 7f5e52261
    - review: findings — builtin/validators/code-hygiene/rules/idioms-swift.md:40, builtin/validators/code-hygiene/rules/idioms-swift.md:46, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs
    - The defect is one cause at two sites. Both `grep` patterns read `: error: [` at ANY position of the line, and `swift format` writes the path at the head of each line. Thus a file with the name `x: error: [UseShorthandTypeNames] y.swift` puts a tag through the filter that the allowlist does not name, and it stops the decline of a file the parser cannot read. The fix is to anchor both patterns to the true `path:line:column:` head.
    - Review confirmed every other measurement: 11 findings from the failing fixture carrying all 7 rules; 0 findings from the passing fixture through the filter; 34 raw `[Indentation]` lines from the same bytes; the file checksum does not change, thus the gate is read-only; `UseWhereClausesInForLoops` is absent from the written configuration.
  timestamp: 2026-09-07T18:04:41.981247+00:00
- actor: claude-code
  id: 01m1yhpnng83k1gedgtfakbs4q
  text: |
    All three review findings are answered. One cause, removed from the whole
    script.

    The cause: `swift format` writes the PATH FIRST on every line. Both readings of
    that output matched their text at ANY position, so a file NAME could give the
    pattern the reading looks for.

    The fix, at both sites: the script now builds the head of a real diagnostic
    before it reads anything.

        quoted="$(printf '%s' "$file" | sed -e 's/[][\\.^$*+?(){}|]/\\&/g')"
        diagnostic="^$quoted:[0-9]+:[0-9]+: error: "

    The `sed` puts a backslash in front of every pattern operator, so the path
    stands in the pattern as TEXT. The decline guard reads
    `grep -v -E "$diagnostic\[[A-Za-z]+\] "`, the tag filter reads
    `grep -E "$diagnostic\[($allowed)\] "`, and the rewrite is a `sed -E` anchored
    on the same head, with the path recaptured as group 1 rather than written back
    as text. The tool makes that head for a finding and for nothing else.

    TDD, both directions measured.

    - RED: the two new tests were written first and run against the shipped
      script. `..._measures_a_file_named_for_a_diagnostic_head` failed with "the
      `[Indentation]` probe holds a layout defect and no idiom ... the run reported
      [\"Indentation\"]". `..._declines_a_file_named_for_a_diagnostic_head` failed
      with three findings, each a raw parser line, and no marked line. 2 run, 0
      passed, 2 failed.
    - GREEN: the same two tests, 2 run, 2 passed.
    - `cargo nextest run -p swissarmyhammer-validators swift` — 88 run, 88 passed.
      86 before, plus the two new ones.

    Everything the review verified still holds, measured again with the shipped
    script over the same bytes:

    - the failing fixture: 11 findings, exit 0, carrying all 7 allowlist rules;
    - the passing fixture: 0 findings through the filter, beside 34 raw
      `[Indentation]` lines from the same bytes;
    - two reporting files in one run give the findings of both, at exit 0, beside
      one `sah-diagnostic:` line for a missing path;
    - the gate is read-only: `shasum` over both judged files is the same before and
      after the run;
    - `UseWhereClausesInForLoops` stands nowhere in the shipped script.

    Row 5 of the new body table is the half a reader could miss. An anchor tight
    enough to drop the crafted layout line could take a TRUE finding away as well,
    so `w: error: [UseShorthandTypeNames] s.swift`, holding one `Array<Int>`
    parameter, is measured to still report its `UseShorthandTypeNames` finding. The
    second half of `..._measures_a_file_named_for_a_diagnostic_head` holds it.

    Notes for the next agent:

    - The body carried the sentence that made the defect — "a line holding
      `: error: [` is a tagged finding". That sentence is now corrected, and the new
      section "Each reading is anchored on the head of a real diagnostic" states the
      measurement in five rows, with the loose script beside the shipped one.
    - `SwiftIdiomsFixtureRun` is renamed `SwiftIdiomsRun`, because a second helper
      `swift_idioms_staged_run` now answers the same shape over a probe file rather
      than over the fixture.
    - The `[Indentation]` probe and its tag are now named constants, so the
      tag-probe list and the new test read ONE file rather than two copies.
  timestamp: 2026-09-07T18:22:18.800167+00:00
- actor: claude-code
  id: 01m1yhpxfsjachmh5qtht0sttt
  text: |
    ### implement — changed
    - evidence: 2 files — builtin/validators/code-hygiene/rules/idioms-swift.md, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs. All 3 review findings flipped to `- [x]`. `cargo nextest run -p swissarmyhammer-validators swift` — 88 run, 88 passed (86 before, plus 2 new). `cargo nextest run -p swissarmyhammer-validators` — 856 run, 818 passed, 38 failed; the failing name set is IDENTICAL with the change stashed, so every one is pre-existing (missing go, revive, staticcheck, golangci-lint and eslint-plugin-sonarjs). `cargo fmt --all --check` clean. `cargo clippy -p swissarmyhammer-validators --all-targets -D warnings` clean.
    - next: /review
  timestamp: 2026-09-07T18:22:26.809491+00:00
- actor: claude-code
  id: 01m1yhrzj4tn0tjdmhx6gr5h0f
  text: |-
    ### finish iteration 2 — pending review
    - implement: changed — 2 files. All 3 findings closed. One cause removed from the whole script: each reading is now anchored on `^<path>:<line>:<column>: error: `, and the file name is escaped for the regular expression first.
    - test: green — cargo nextest -p swissarmyhammer-validators swift, 88 run, 88 passed (86 before, plus 2 new); cargo fmt --all --check clean. The driver ran these.
    - commit: 3d79443c9
    - review: pending
    - The two new tests came first. Both failed before the fix: one reported `["Indentation"]`, and the other gave three raw parser lines as findings with no marked line.
  timestamp: 2026-09-07T18:23:34.468853+00:00
- actor: claude-code
  id: 01m1yjf5bm5a4b03841cey53fg
  text: |-
    ### review — findings
    - evidence: `review sha HEAD~1..HEAD` (checkpoint 3d79443c9), round 2. Engine: 1 finding, 1 confirmed, 0 refuted; it again matched no validator to `builtin/validators/code-hygiene/rules/idioms-swift.md`. 3 more findings come from direct measurement of that uncovered gate script. Open at `builtin/validators/code-hygiene/rules/idioms-swift.md:41`, `builtin/validators/code-hygiene/rules/idioms-swift.md:49`, `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:281`, and `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` (anchor coverage).
    - The three findings of round 1 are genuinely fixed. Both new tests are RED against the old loose script and GREEN against the shipped one.
    - Two NEW defects in the same anchoring code: a path with a precomposed character (`café.swift`) is declined whole, because the tool writes the path in NFD and the argv spelling is NFC; and a path holding `|` leaves the line unrewritten, because `|` is the s-command delimiter of the `sed -E`. The escape set itself is complete, and the group-1 recapture does keep `&` out of the replacement.
    - Prior invariants hold: failing fixture 11 findings / 7 rules at exit 0; passing fixture 0 findings at exit 0 with 34 raw `[Indentation]` lines; two reporting files 22 findings at exit 0; both fixtures unchanged after the run; `UseWhereClausesInForLoops` absent from the allowlist. `cargo nextest run -p swissarmyhammer-validators swift` — 88 passed.
    - next: correct both readings in the gate script, correct the body sentence that says no file name can make the pattern, remove the duplication in the test helper, and add the two legitimate-name probes.
  timestamp: 2026-09-07T18:35:41.300860+00:00
- actor: claude-code
  id: 01m1yjgz78meqtcbvyr02skb6k
  text: |-
    ### finish iteration 2 — findings
    - implement: changed — 2 files; all 3 findings of round 1 closed
    - test: green — cargo nextest -p swissarmyhammer-validators swift, 88 run, 88 passed; fmt clean. The driver ran these.
    - commit: 3d79443c9
    - review: findings — builtin/validators/code-hygiene/rules/idioms-swift.md:41, builtin/validators/code-hygiene/rules/idioms-swift.md:49, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:281, and one more on the same test file for anchor coverage
    - Review proved the three findings of round 1 are truly closed. It put both readings back to the loose form and the two new tests failed by name, thus the tests are real.
    - The two new defects each LOSE a true finding. Neither one puts a tag through the gate.
      1. `swift format` writes the path back in NFD, and argv gives NFC. Thus `café.swift` matches the anchor nowhere, and the file is declined with every finding lost. This is a regression: the loose script reported that file.
      2. The rewrite uses `|` as the `sed` delimiter, and the escaper writes `|` as `\|`. BSD sed reads that as an escaped delimiter. Thus a path that holds `|` gives the raw tool line, unrewritten.
    - Context, not a finding: `swift format lint` has no `--` before the path, thus a path that starts with `-` is declined as an unknown option. That line is older than this commit.
  timestamp: 2026-09-07T18:36:40.552846+00:00
- actor: claude-code
  id: 01m1yk1qp7shm6n1d8xfhmscg8
  text: |-
    Round 3 research. Both defects reproduced with the SHIPPED script, Apple Swift 6.4:

    - `café.swift` holding one `Array<Int>` parameter: stdout empty, and
      `sah-diagnostic: idioms-swift declined café.swift: ...` on stderr. The true
      finding is lost. `ünïcodé.swift` the same.
    - `a|b.swift` the same bytes: the run writes the RAW tool line
      `a|b.swift:2:39: error: [UseShorthandTypeNames] use shorthand syntax ...`,
      unrewritten.

    The shape of the fix. Matching the argv path against the path the tool WRITES
    is the cause of both, so the new reading names the path NOWHERE. The script
    hands `swift format` one path for each run, so on any line the true head is
    the LAST `:<line>:<column>: error: ` of that line. One `awk` finds that last
    head, reads the `[<Tag>]` that follows it, and writes the finding with the
    path the ARGV spelled:

    - a line whose last head carries a tag is a finding of that tag;
    - every other line is trouble, and the file is declined;
    - the allowlist is read off the tag alone, which is `[A-Za-z]+`.

    That one reading answers both the decline guard and the tag filter, so the two
    cannot disagree. The path never enters a pattern and never enters a
    replacement, so neither normalization nor a regular-expression operator can
    reach either one.

    Measured with the new reading, over the same probes:

    | the file name | what it holds | the run |
    |---|---|---|
    | `Plain.swift` | one `Array<Int>` parameter | 1 `UseShorthandTypeNames` finding |
    | `café.swift` | the same | 1 `UseShorthandTypeNames` finding |
    | `ünïcodé.swift` | the same | 1 `UseShorthandTypeNames` finding |
    | `a|b.swift` | the same | 1 `UseShorthandTypeNames` finding, rewritten |
    | `w: error: [UseShorthandTypeNames] s.swift` | the same | 1 `UseShorthandTypeNames` finding |
    | `x: error: [UseShorthandTypeNames] y.swift` | one member indented 8 spaces | 0 findings |
    | `z: error: [UseShorthandTypeNames] q.swift` | Swift the parser cannot read | 0 findings, 1 marked line |

    Two SHARPER crafted names were measured as well, and the earlier anchored
    script had no answer for either. `yy:1:1: error: [UseShorthandTypeNames] y.swift`
    over an indented member gives 0 findings, and
    `zz:1:1: error: [UseShorthandTypeNames] q.swift` over Swift the parser cannot
    read is declined with one marked line. A name that carries a WHOLE diagnostic
    head therefore reaches nothing either.

    Invariants measured again with the new reading: failing fixture 11 findings
    carrying 7 rules at exit 0; passing fixture 0 findings at exit 0 beside 34 raw
    `[Indentation]` lines; two reporting files in one run give 12 findings beside
    one `sah-diagnostic:` line for a missing path; `shasum` over both judged files
    is the same before and after; a directory, an unreadable file and bytes that
    are not UTF-8 are each declined with the tool's own words, at exit 0.
  timestamp: 2026-09-07T18:45:49.895267+00:00
- actor: claude-code
  id: 01m1ym07hycbwkzsczpaqec5nd
  text: |-
    All four findings of round 2 are answered. One cause, removed from the whole
    script.

    The cause: the reading of the output named the PATH. The loose reading read the
    path by accident, because it matched its text at any position. The anchored
    reading read the path on purpose, and it then had to pass the path through a
    regular expression and through a substitution. Both readings break on a name
    the script does not control.

    The fix: the reading names the path NOWHERE. The script hands `swift format`
    one path for each run, so on any line the true head is the LAST
    `:<line>:<column>: error: ` of that line. One `awk` walks to that head:

        while (match(rest, /:[0-9]+:[0-9]+: error: /) > 0) {
          head = substr(rest, RSTART, RLENGTH)
          rest = substr(rest, RSTART + RLENGTH)
        }

    What stands AFTER the head decides the line. A `[<Tag>] ` makes the line a
    finding of that tag, and the allowlist is read off the tag alone, which is
    `[A-Za-z]+`. Every other line is trouble, and the file is declined whole. So
    ONE reading now answers the decline guard and the tag filter together, and the
    two cannot disagree. The finding carries the path the ARGUMENT LIST spelled and
    the line number off the head, so the path reaches no pattern and no
    replacement. The escaper `quoted=` is gone, and with it the `|` delimiter.

    TDD, both directions measured.

    - RED: the two new tests were written first and run against the SHIPPED script.
      `..._reports_a_file_whose_name_carries_an_accent` failed with "the gate must
      judge `café.swift` rather than decline it; it stated ["idioms-swift declined
      café.swift: cafe\u{301}.swift:2:39: error: ..."]" — the NFD spelling is
      visible in the message. `..._reports_a_file_whose_name_carries_an_alternation_bar`
      failed with "the run reported ["39", "39", "38"]", which is the COLUMN number
      of each raw tool line the rewrite did not touch. 2 run, 0 passed, 2 failed.
    - GREEN: the same two tests, 2 run, 2 passed.
    - `cargo nextest run -p swissarmyhammer-validators swift` — 90 run, 90 passed.
      88 before, plus the two new ones.

    The rule name is what makes the second test honest. The engine reads the FIRST
    `:<digits>:` of a line, so a raw tool line and a rewritten line give the SAME
    `path:line` row. The rule name separates them: the rewritten line carries
    `UseShorthandTypeNames`, and the raw line carries the column number.

    Every invariant measured again with the shipped script:

    - the failing fixture: 11 findings, exit 0, carrying all 7 allowlist rules;
    - the passing fixture: 0 findings through the filter, beside 34 raw
      `[Indentation]` lines from the same bytes;
    - two reporting files in one run give the findings of both, at exit 0, beside
      one `sah-diagnostic:` line for a missing path;
    - `shasum` over both judged files is the same before and after, so the gate is
      read-only;
    - `UseWhereClausesInForLoops` stands nowhere in the shipped script;
    - a directory, a file with no read permission and bytes that are not UTF-8 are
      each declined with the tool's own words, at exit 0.

    Two things the next agent should know.

    - The new reading is stronger than the anchored one against a CRAFTED name as
      well. `yy:1:1: error: [UseShorthandTypeNames] y.swift` over an indented member
      gives 0 findings, and `zz:1:1: error: [UseShorthandTypeNames] q.swift` over
      Swift the parser cannot read is declined with one marked line. A name that
      carries a WHOLE diagnostic head therefore reaches nothing. Both rows stand in
      the body table.
    - `awk` is the one tool the reading adds. `doctor.check_command` now reads
      `which swift mktemp awk sed sort paste`, and `grep` left that list because the
      script no longer runs it. The command was measured to exit 0 on this machine.

    The body section is rewritten and renamed "The reading finds the head of a
    diagnostic by its STRUCTURE". Its table now holds ELEVEN rows over THREE
    readings — loose, anchored and shipped — and every cell was measured. The two
    sentences the finding named, "the path stands in the pattern as TEXT" and "no
    file name can make one", are gone with the escaper they described.
  timestamp: 2026-09-07T19:02:29.182013+00:00
- actor: claude-code
  id: 01m1ym0gebjyf218q0y58re9vf
  text: |-
    ### implement — changed
    - evidence: 2 files — builtin/validators/code-hygiene/rules/idioms-swift.md, crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs (232 insertions, 71 deletions). All 4 findings of round 2 flipped to `- [x]`. `cargo nextest run -p swissarmyhammer-validators swift` — 90 run, 90 passed (88 before, plus 2 new). `cargo nextest run -p swissarmyhammer-validators` — 858 run, 819 passed, 39 failed; with the change stashed the same run gives the IDENTICAL failing name set, so every one is pre-existing (missing go, revive, staticcheck, golangci-lint and eslint-plugin-sonarjs). One Dart name, `the_shipped_dart_missing_docs_tool_rule_declines_a_file_it_may_not_read`, is flaky under load: it fails in one whole-crate run and passes 3 of 3 times on its own. `cargo fmt --all --check` clean. `cargo clippy -p swissarmyhammer-validators --all-targets -D warnings` clean.
    - next: /review
  timestamp: 2026-09-07T19:02:38.283621+00:00
depends_on:
- 01M1Y65VQ8V4TEWXDXD4Y2ZQNX
position_column: doing
position_ordinal: '8380'
title: Move idioms-swift to the toolchain swift format
---
## What

`builtin/validators/code-hygiene/rules/idioms-swift.md` runs `swiftformat`, the tool of Nick Lockwood. Only Homebrew installs that tool. The new policy is the Swift toolchain and its own `swift format`. Change this rule to run `swift format lint`.

The task "Measure the Swift toolchain rules the gate needs" is done. Its measurement table is in the body of the same file. **Read that table first, and build the script from it. Do not measure again.**

Files to change:
- `builtin/validators/code-hygiene/rules/idioms-swift.md` — the frontmatter script, the doctor block and the body
- `builtin/validators/code-hygiene/fixtures/idioms-swift.fail.swift.tmpl`
- `builtin/validators/code-hygiene/fixtures/idioms-swift.pass.swift.tmpl`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs`
- `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped.rs` — the `.swift-version` machinery `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT`, which exists only for the swiftformat version gate
- `crates/swissarmyhammer-validators/src/builtin/mod.rs`

### What the measurement decided, and the script must obey

| the fact | what the script must do |
|---|---|
| `swift format lint` writes NINE tags that no configuration can stop | Read the tag off each output line. KEEP only a tag of the gate allowlist. The configuration alone is not the gate. |
| Every finding and every error goes to **stderr**. stdout is always 0 bytes. | Read stderr, not stdout. |
| A path that holds no file exits **0** and writes nothing | The script must test the path itself, with `[ ! -e "$file" ]`, and write its own `sah-diagnostic:` line. The tool gives no error for it. |
| A file with findings exits **1**. A clean file exits 0. A directory exits 64. | Capture the status with `|| status=$?`. Read 1 as "findings, continue". `set -e` must not end the run. |
| The version floor is **Swift 6.0**, from the `swift format` subcommand | State that floor. Do not state 0.62.1, which measures swiftformat. |
| `swift format` reads **no** `.swift-version` file | Delete the section about `.swift-version` and the five version-gated rules. |
| The exemption directive is `// swift-format-ignore` | Rewrite the section "The directive an author writes". The bare form silences the pretty-printer tags as well; the named form does not. |

Keep the one-run-for-each-file shape, thus a refusing path costs only its own file.

### The allowlist

| the rule | what it decides |
|---|---|
| `AlwaysUseLiteralForEmptyCollectionInit` | the empty-collection bullet of `idioms.md`, in the correct direction. OFF by default; the configuration must make it ON |
| `NoVoidReturnOnFunctionSignature` | the `Void` return clause, BOTH halves |
| `ReplaceForEachWithForLoop` | the `forEach` half of the loop bullet |
| `UseLetInEveryBoundCaseVariable` | the pattern-let bullet |
| `UseShorthandTypeNames` | the shorthand type sugar bullet |
| `DontRepeatTypeInStaticProperties` | the type-name bullet. **Row 3 of the table proves it DOES report**, for `public static let redColor = Color()`. The member's own type must be the enclosing type. It is silent for `= 1`, which infers `Int`. Read row 3, and state in the body which shapes it misses |
| `UseSynthesizedInitializer` | the redundant memberwise initializer, the **internal** half only. Row 4 proves it is silent for a `public` initializer, and that answer is correct, because Swift synthesizes an internal initializer and never a public one |

**`UseWhereClausesInForLoops` stays OFF.** Row 5 of the table records that it contradicts `immutability.md`, and it states two answers and chooses neither. A person must choose. Do not make this rule ON in this task. The task "Rebalance the Swift prompt rules for the toolchain gate" carries that decision.

Do NOT make ON `NeverForceUnwrap`, `NeverUseForceTry` or `NeverUseImplicitlyUnwrappedOptionals`. `disallowed-constructs-swift` owns those three bullets, and swiftlint keeps the test-target split and the `ignored_literal_argument_functions` option that `swift format` does not have. One requirement takes one owner.

### What the gate loses

The gate loses the 22 rules the body lists — the performance group of 7, the Swift Testing group of 6, the SwiftUI group of 4, and 5 others — PLUS `preferFinalClasses` and `noGuardInTests`, which each own a prompt bullet. The task "Rebalance the Swift prompt rules for the toolchain gate" gives those bullets an owner. Do not delete a bullet here.

Remove the `--rules` intersection with `swiftformat --rules`, the `--min-version` test and the four command-line options. The toolchain gives a fixed set of 43 rules, and each rule is only ON or OFF.

## Acceptance Criteria
- [ ] The script names `swift format` and names `swiftformat` nowhere.
- [ ] The script holds an allowlist of rule tags, and it drops every output line whose tag is not in the allowlist.
- [ ] A Swift file that holds only layout defects gives 0 findings THROUGH the filter.
- [ ] A file that reports does not end the run. Over two files that both report, the run gives the findings of both, at exit 0.
- [ ] A path that holds no file writes one `sah-diagnostic:` line and costs only its own file.
- [ ] `doctor.check_command` tests the toolchain and the Swift 6.0 floor. `doctor.fix_hint` names the toolchain, because Homebrew does not install it.
- [ ] The failing fixture gives a STATED number of findings that carry a STATED number of rules.
- [ ] The passing fixture gives 0 findings and exit 0.
- [ ] The section about the exemption directive names `// swift-format-ignore` and names `swiftformat:disable` nowhere.
- [ ] The body holds no row that measures swiftformat, and nothing about `.swift-version`.
- [ ] `UseWhereClausesInForLoops` is OFF.

## Tests
- [ ] Update every test in `tests/shipped/idioms_swift.rs`. Delete the tests that hold the swiftformat roster and the 0.62.1 floor. Keep the measurement tests the previous task wrote.
- [ ] Write a test that holds a badly formatted probe file to 0 findings through the filter. A script that drops the filter fails it by name.
- [ ] Write a test that holds the allowlist to naming only rules that stand in `swift format dump-configuration`.
- [ ] Write a test that holds two reporting files in one run to giving the findings of both.
- [ ] Write a test that holds `UseWhereClausesInForLoops` to being OFF, and name row 5 as the reason.
- [ ] Delete or rewrite `SWIFT_PROBE_VERSION` and `SWIFT_VERSION_SUPPORT` in `tests/shipped.rs`.
- [ ] Update the bullet-ownership tests in `crates/swissarmyhammer-validators/src/builtin/mod.rs`.
- [ ] Run `cargo nextest run -p swissarmyhammer-validators swift`. Every test passes.
- [ ] Run `sah doctor`. The rule reports as healthy, and its fixture pair passes.

## Workflow
- Use `/tdd` — write the failing tests first, then write the code that makes them pass. #swift

## Review Findings (2026-09-07 13:05)

> Scope: `review sha HEAD~1..HEAD` (checkpoint 7f5e52261). The engine reviewed the 5 Rust files and returned 0 findings. It did NOT review `builtin/validators/code-hygiene/rules/idioms-swift.md`, which carries the gate script, because no validator matches `*.md`. It also excluded the two `.swift.tmpl` fixtures as validator fixtures. The items below come from direct verification of that uncovered script. Each one carries a reproduction.

> Verified as claimed: the failing fixture gives 11 findings that carry all 7 allowlist rules, at exit 0. The passing fixture gives 0 findings, at exit 0. A badly indented probe writes 34 raw `[Indentation]` lines and 0 through the filter. Two reporting files in one run give the findings of both, at exit 0, beside one `sah-diagnostic:` line for a missing path. The run never rewrites a file. `UseWhereClausesInForLoops` stands nowhere in the written configuration. `cargo nextest run -p swissarmyhammer-validators swift` — 86 tests, 86 passed.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:46` `gate-script/tag-filter` — the tag filter `grep -E ": error: \[($allowed)\] "` matches the allowlisted text ANYWHERE in the line, and `swift format` writes the file path at the head of every line, so a path that itself holds `: error: [<AllowedTag>] ` carries any tag through the gate. Measured: a file named `x: error: [UseShorthandTypeNames] y.swift` that holds one over-indented member reports `[Indentation]` in the raw run, and the shipped script writes `x: error: [UseShorthandTypeNames] y.swift:2: Indentation: unindent by 6 spaces` — an `Indentation` finding that the allowlist does not name. Anchor the match to the start of the line and to the true `path:line:column:` head, so that the written path cannot give the pattern.
- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:40` `gate-script/decline-guard` — the decline guard `grep -v ': error: \['` drops a line by the same unanchored text, so the same crafted path makes an UNTAGGED tool error look tagged, and the script never declines the file. Measured over a file the tool cannot parse: with an ordinary name the script correctly writes `sah-diagnostic: idioms-swift declined ...`; with the name `z: error: [UseShorthandTypeNames] q.swift` it writes no diagnostic and instead gives the two raw parser lines `error: expected expression in variable` and `error: expected '}' to end struct` as findings, which the `sed` leaves unrewritten. This is the same cause as the item above. Anchor both greps, not the tag filter alone.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` `tests/filter-coverage` — no test holds the tag filter against a path that carries a diagnostic-shaped name, although the sibling rule already sets that pattern: `missing_docs.rs` ships `the_shipped_swift_missing_docs_tool_rule_measures_a_file_named_for_the_configuration_message` and `the_shipped_swift_missing_docs_tool_rule_measures_a_file_named_for_the_decode_message`. Add the same probe here, so that a filter which reads the written path fails by name.

## Review Findings (2026-09-07 13:24)

> Scope: `review sha HEAD~1..HEAD` (checkpoint 3d79443c9). Round 2. The engine reviewed the Rust test file and returned 1 finding. It again did NOT review `builtin/validators/code-hygiene/rules/idioms-swift.md`, which carries the gate script, because no validator matches `*.md`. The gate-script items below come from direct verification of that uncovered script. Each one carries a reproduction, measured with Apple Swift 6.4, BSD sed and BSD grep 2.6.0-FreeBSD, under `LC_ALL=en_US.UTF-8`.

> The three findings of round 1 are GENUINELY FIXED. Measured with the shipped script: `x: error: [UseShorthandTypeNames] y.swift` holding one over-indented member gives 0 findings; `z: error: [UseShorthandTypeNames] q.swift` holding Swift the parser cannot read is declined with one `sah-diagnostic:` line; `w: error: [UseShorthandTypeNames] s.swift` holding one `Array<Int>` parameter still gives its one true `UseShorthandTypeNames` finding. The two new tests are genuinely RED against the OLD script: with the two readings put back to their loose form, `the_shipped_swift_idioms_tool_rule_measures_a_file_named_for_a_diagnostic_head` fails with `the run reported ["Indentation"]`, and `the_shipped_swift_idioms_tool_rule_declines_a_file_named_for_a_diagnostic_head` fails with three raw parser lines. Both pass against the shipped script.

> The escape set is COMPLETE for the regex dialect. Each of `{ } ( ) + ? ^ $ * [ ] . |` and each of `- / & % # @ = ~ , !` was driven through both consumers. `grep -E` reads every one of them correctly. `&` never reaches a replacement, because the rewrite recaptures the path as group 1, so the claim about group-1 recapture holds. A backslash in the path is read correctly.

> Prior invariants all still hold. The failing fixture gives 11 findings that carry all 7 allowlist rules, at exit 0. The passing fixture gives 0 findings at exit 0, and its raw run writes 34 `[Indentation]` lines. Two reporting files in one run give 22 findings at exit 0, beside one `sah-diagnostic:` line for a missing path. Both fixtures are byte for byte unchanged after the run, so the gate is read-only. `UseWhereClausesInForLoops` stands nowhere in the allowlist the script writes. `cargo nextest run -p swissarmyhammer-validators swift` — 88 tests, 88 passed.

- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:41` `gate-script/decline-guard` — the anchor compares the path as the ARGV spelled it with the path as `swift format` WRITES it, and those two are not the same bytes for a name that carries a precomposed character. The tool writes the path in NFD. Measured over `café.swift` holding one `Array<Int>` parameter: the argv name is `63 61 66 c3a9 2e 73 77 69 66 74`, and the head of the output line is `63 61 66 65 cc81 ...`, so `$diagnostic` matches no line. The `grep -v` then keeps the true tagged line as trouble, and the script writes `sah-diagnostic: idioms-swift declined café.swift: ...` and reports NOTHING. The same happens for `ünïcodé.swift`. `日本語.swift` survives, because those characters have no decomposed form, so the break is the precomposed Latin letters a real repository uses. This is a REGRESSION: the loose script reported the true finding over both names. Compare the head by a reading that does not depend on the spelling of the path — for example match the path by its LENGTH, or normalize both sides before the compare — so that a legitimate accented name keeps its findings.
- [x] `builtin/validators/code-hygiene/rules/idioms-swift.md:49` `gate-script/rewrite` — the rewrite is a `sed -E` that uses `|` as the s-command delimiter, and the escaper writes `|` as `\|`. BSD sed then reads `\|` as an escaped DELIMITER and not as a literal `|`, so the pattern matches no line and the substitution is silently skipped. Measured over `a|b.swift` holding one `Array<Int>` parameter: the `grep -E` matches, so the line IS reported, and the run writes the raw tool line `a|b.swift:2:15: error: [UseShorthandTypeNames] use shorthand syntax for this 'Array' type` instead of `a|b.swift:2: UseShorthandTypeNames: ...`. The same expression with a `,` delimiter rewrites the line correctly. An unrewritten raw line as a finding is the same failure round 1 named in its second item. Use a delimiter the path cannot hold, or escape the delimiter apart from the regex operators. `|` is the ONLY character of the 22 measured that defeats the rewrite. The body states at line 357 that "the path stands in the pattern as TEXT" and that "no file name can make one"; both sentences are false for `|`, so correct the body with the same edit.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs:281` `reuse/reuse` — The new function `swift_idioms_staged_run` duplicates the outcome-extraction and return-construction logic already in `swift_idioms_run` (lines 264–273). Both functions have identical sequences: drive the script, extract the outcome, build and return the same `SwiftIdiomsRun` struct. This pattern should have been extracted into a shared helper rather than written twice. Extract the common logic into a helper function — e.g., `fn swift_idioms_from_staging(staging: &ShippedStaging, files: &[&str]) -> SwiftIdiomsRun` — and have both `swift_idioms_run` and `swift_idioms_staged_run` set up their staging differently, then call this helper to do the run, outcome extraction, and return. This eliminates ~13 lines of duplication.
- [x] `crates/swissarmyhammer-validators/src/review/tool_rules/tests/shipped/idioms_swift.rs` `tests/anchor-coverage` — the two new tests hold the anchor against a CRAFTED name only. No test holds it against a LEGITIMATE name that the anchor breaks, so both defects above pass the suite. Add one probe for an accented name, such as `café.swift`, and one probe for a name holding `|`, each staged with the same `Array<Int>` parameter the crafted-name test already uses, and hold each run to the one true `UseShorthandTypeNames` finding in the REWRITTEN shape `<path>:<line>: UseShorthandTypeNames: ...`. A run that declines the file, and a run that writes the raw tool line, then each fail by name.