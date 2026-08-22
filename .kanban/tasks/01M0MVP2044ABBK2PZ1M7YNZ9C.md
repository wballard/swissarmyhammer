---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0nf4kt0a5t808ph6w37cy35
  text: |-
    Research done. Every claim measured on this machine, not read from docs.

    ## Toolchain measured

    | binary | path | version |
    |---|---|---|
    | `swift` | `/usr/bin/swift` | Apple Swift 6.4 (swiftlang-6.4.0.30.4) |
    | `swift format` | toolchain subcommand | reports `main` |
    | `swift-format` | `/opt/homebrew/bin/swift-format` | separate Homebrew copy |
    | `swiftformat` | `/opt/homebrew/bin/swiftformat` | 0.62.1 |
    | `swiftlint` | `/opt/homebrew/bin/swiftlint` | 0.65.0 |

    `swift format` is a subcommand of Apple's swift-format with `dump-configuration`, `format`, `lint` and a `--configuration` option that takes JSON. `swiftformat` is a different binary with `--rules`, `--lint`, `--options`. Confirmed different tools.

    ## The card's command lines: two are measurably wrong

    Probe: a real SPM package declaring `.package(url: "https://github.com/airbnb/swift", from: "1.0.0")`. Resolved to AirbnbSwift 1.2.0. `swift package plugin --list` names `'format' (plugin 'FormatSwift' in package 'AirbnbSwift')`.

    1. **`swift package format --lint` alone FAILS in a non-interactive shell.**
       ```
       error: Plugin 'FormatSwift' wants permission to write to the package directory.
       Stated reason: "Format Swift source files".
       Use `--allow-writing-to-package-directory` to allow this.
       EXIT: 1
       ```
       The plugin declares `permissions: [.writeToPackageDirectory(...)]` unconditionally in its own `Package.swift`, so SwiftPM demands the flag even in lint mode. Every documented line therefore has to carry `--allow-writing-to-package-directory`. The lint run still writes nothing — it prints `(lint mode - no files will be changed.)`.

    2. **The switch is `--target`, not `--targets`.**
       | run | exit |
       |---|---|
       | `... format --lint --target Probe` | 1, reports the real findings |
       | `... format --lint --targets Probe` | 1, `error: unknownError(exitCode: 64)` |

       The plugin source reads `argumentExtractor.extractOption(named: "target")`. `--paths`, `--exclude` and `--swift-version` are all correct as the card states them.

    ## Exit codes: the card's claim CONFIRMED

    | run | exit |
    |---|---|
    | `--lint`, formatting defects | 1, `error: lintFailure` |
    | `--lint`, clean | 0 |
    | fix mode, formatting defects | 0, and the files were rewritten |
    | fix mode, clean | 0 |
    | fix mode, a `print(...)` call (SwiftLint `no_direct_standard_out_logs`) | 1, `error: lintFailure` |
    | `--lint`, the same `print(...)` | 1 |

    Non-zero on any failure in lint mode; in fix mode non-zero only for a SwiftLint rule, because SwiftLint cannot autocorrect it.

    ## The contradiction the parent flagged is REAL

    `swiftformat --options` on 0.62.1 gives the defaults:

    | option | default | what `idioms-swift` pins |
    |---|---|---|
    | `--pattern-let` | `hoist` | `inline` |
    | `--short-optionals` | `preserve-struct-inits` | `always` |
    | `--single-line-for-each` | `ignore` | `convert` |
    | `--guard-like-if-statements` | `preserve` | `convert` |

    Measured, over `if case .some(let inner) = value` — the shape `idioms-swift` asks for:

    - bare `swiftformat` rewrote it to `if case let .some(inner) = value`
    - `idioms-swift` then reports that exact line: `hoistPatternLet: Reposition let or var bindings within pattern.`

    So a developer following the old bare `swiftformat .` line produces code our own review reports, and the tool's own fix walks them straight into the finding. Re-running with `--pattern-let inline --short-optionals always --guard-like-if-statements convert --single-line-for-each convert` left the gate's shape standing untouched.

    ## A SECOND contradiction, in the Airbnb plugin itself

    `airbnb.swiftformat` carries `--pattern-let inline`, `--short-optionals always`, `--single-line-for-each convert` — the same options `idioms-swift` pins. Good.

    But it ALSO carries `--property-types inferred`, which `idioms-swift` deliberately refused and `builtin/validators/swift/rules/idioms.md` names a validator error. Measured, the plugin's fix mode over a struct:

    ```
    -  public var items: [Int] = []          +  public var items = [Int]()
    -  public var table: [String: Int] = [:] +  public var table = [String: Int]()
    ```

    That turns the DO of `idioms.md` into its DON'T, word for word. The partial has to warn about this one line, or the plugin path silently fights our own prompt rule.

    ## Other defects found in the partial

    `swift test --help` states the filter form is `<test-target>.<test-case>` or `<test-target>.<test-case>/<test>`, and it is a regular expression. The partial says `swift test --filter <Suite>/<test>`, which drops the test-target component.

    ## Card overlap

    ^7fgdenq covers the config-detection half of the SAME two lines. Full overlap. This card absorbs it — see the separate comment.
  timestamp: 2026-08-22T19:28:38.208360+00:00
- actor: claude-code
  id: 01m0nf5p8pp1hwcvvyg29h5xy2
  text: |-
    ## Overlap with ^7fgdenq — this card absorbs it

    Both cards rewrite the SAME bullet, `builtin/_partials/project-types/swift.md` Format/Lint lines. They are not adjacent, they are the same edit.

    - ^m7ynz9c (this card) requires "split the two tools onto their own lines and **name the config file each reads**".
    - ^7fgdenq requires "**honor** an existing `.swift-format`/`.swiftformat`, tool defaults otherwise, never create a config as a side effect".

    Naming which config file a tool reads and then not saying to honor it is half a sentence. Doing them as two cards means writing the same two lines twice and risking the second pass contradicting the first.

    **Proposal: ^m7ynz9c absorbs ^7fgdenq.** This card delivers both halves in one edit, plus ^7fgdenq's second checkbox — the content regression test in `crates/swissarmyhammer-project-detection/src/types.rs` beside `spec_partial_matches_key`. ^7fgdenq's acceptance criteria are then all met here and it can be archived.

    One correction to carry over: ^7fgdenq's proposed text says "`swiftformat .` honors it likewise — pick the tool that matches the config file present". Measured, that is not enough. Honoring `.swiftformat` does NOT make bare `swiftformat` agree with our shipped `idioms-swift` gate — the four options that matter (`--pattern-let`, `--short-optionals`, `--single-line-for-each`, `--guard-like-if-statements`) all default the wrong way, and the project config only moves them if the project WROTE them. The absorbed text has to state the options, not only the config file.
  timestamp: 2026-08-22T19:29:13.494692+00:00
- actor: claude-code
  id: 01m0nfn18f2ekncrd1ggyamb7v
  text: |-
    ## What landed

    **`builtin/_partials/project-types/swift.md`** — the two loose lines are gone. In their place:

    - A three-row table naming each tool beside the config file it alone reads: Apple swift-format / `swift format` / `.swift-format` (JSON); SwiftFormat (Nick Lockwood) / `swiftformat` / `.swiftformat`; SwiftLint / `swiftlint` / `.swiftlint.yml`. The line above it states the first two are DIFFERENT programs.
    - Use the tool whose config file the repo already holds, and obey it. Never write a config file as a side effect. Never add style flags that disagree with a config already there. Tool defaults when no config exists.
    - The Airbnb plugin path, preferred when `Package.swift` declares the dependency, with both command lines, the switch list, and the exit codes.
    - Without the plugin: swiftformat 0.62.1+ with the four options spelled out, Apple swift-format with `-s` on the check run, swiftlint with `--fix`.
    - Write the project Swift version in `.swift-version`.

    **`crates/swissarmyhammer-templating/src/resolver.rs`** — `swift_partial_separates_the_two_formatters_and_documents_the_airbnb_plugin`, plus a `SWIFT_PARTIAL` name constant and a `builtin_partial` lookup helper, all inside the existing `#[cfg(test)] mod tests`.

    ## Where the test lives, and why not where ^7fgdenq said

    ^7fgdenq proposed `swissarmyhammer-project-detection`, reading the `.md` off disk through `CARGO_MANIFEST_DIR`. That crate holds only the partial's PATH string and never sees its content, so such a test measures a source file rather than the shipped text.

    `swissarmyhammer-templating` embeds every partial at build time (`build.rs` → `BuiltinGenerator` → `get_builtin_partials()`), and `PromptResolver::load_builtin_partials` registers this one as `_partials/project-types/swift` — the same string `partial!("swift")` builds in project-detection. The test reads that embedded table, so it measures the text an agent actually receives. `build.rs` emits `rerun-if-changed` on the partials directory, so editing the markdown rebuilds and re-checks. This follows the existing `test_load_all_prompts_registers_builtin_partials` precedent in the same module.

    ## Two corrections to the card's own command lines

    The card is the order and both requirements are delivered, but two literals on it do not run. Measured, not read:

    1. `swift package format --lint` alone exits 1 with `error: Plugin 'FormatSwift' wants permission to write to the package directory` in a non-interactive shell. The plugin declares `.writeToPackageDirectory` unconditionally in its own `Package.swift`, so lint mode needs the flag too. Both documented lines therefore carry `--allow-writing-to-package-directory`, and the partial says why. The check run still writes nothing.
    2. The switch is `--target` (singular). `--targets Probe` exits non-zero with `error: unknownError(exitCode: 64)`; the plugin source reads `extractOption(named: "target")`. `--paths`, `--exclude` and `--swift-version` are correct as the card states them.

    The card's exit-code claim is confirmed exactly as written.

    ## Consistency with the two shipped validators

    Both contradictions the parent warned about are real and are now written into the partial:

    - Bare `swiftformat .` fights `idioms-swift`. The four options default the wrong way; a bare run rewrote `if case .some(let inner)` into `if case let .some(inner)` and the gate reported that line.
    - The Airbnb plugin fights `builtin/validators/swift/rules/idioms.md`. Its config carries `--property-types inferred`, and fix mode rewrote `var items: [Int] = []` into `var items = [Int]()` — the DO of that rule turned into its DON'T. The partial carries a caution paragraph for it.

    Airbnb's config otherwise carries `--pattern-let inline`, `--short-optionals always` and `--single-line-for-each convert`, the same options `idioms-swift` pins, so the plugin path is the most consistent one available.

    ## Rest of the partial

    - ULID guidance verified by building a real package: `from: "1.2.0"` resolves (1.3.1), and `import ULID` + `ULID()` compiles. Left alone.
    - Test filter corrected. `swift test --help` states the form is `<test-target>.<test-case>` or `<test-target>.<test-case>/<test>`, and that it is a regular expression. The old line dropped the target component.
    - Build, run, deps and file-location lines checked and left alone.
  timestamp: 2026-08-22T19:37:36.271596+00:00
- actor: claude-code
  id: 01m0ng392wvh1yxh5agskcrw1y
  text: |-
    ## Rules pass, and what it changed

    Read the full `dump validators` output — 7 validators, 47 rules, for extensions `md` and `rs`. Two rules changed the code after the first draft:

    **`tdd` "One thing. 'and' in name? Split it."** The first draft was one test named `swift_partial_separates_the_two_formatters_and_documents_the_airbnb_plugin`, covering three separate requirements. Split into three:

    - `swift_partial_names_each_formatter_with_its_own_config_file`
    - `swift_partial_documents_the_airbnb_plugin_commands`
    - `swift_partial_agrees_with_the_shipped_swift_tool_validators`

    **`duplication/duplication`: "Two blocks that differ only by a value are one function with an argument."** Splitting one test into three would have triplicated the `for (needle, requirement)` loop and its assert body. Extracted `assert_swift_partial_states(&[(needle, requirement)])`; each test now passes only its own table. That is also the shape `code-hygiene/data-driven` asks for — a table interpreted once, never N parallel assert lines differing only by a constant.

    ## Rules checked and cleared, with the reason

    - **`reuse/reuse` "Reimplements a shared function/library".** Searched first. No helper anywhere fetches one builtin partial's embedded content by name. The nearest thing, `expand_partials` in `crates/swissarmyhammer-skills/tests/common/mod.rs`, expands `{% include %}` tags in a skill body and reads from disk — different operation, different crate, different contract. `reuse` carve-out: "same shape, different domain or contract is not a reuse miss", plus "Single-call-site helpers are not a reuse concern."
    - **`rust/error-handling` "Never panic on expected failure modes (bad input, missing files, network errors)."** This is why the test reads `get_builtin_partials()` and not the `.md` from disk. The embedded table is built by `build.rs`, so a missing entry is an internal invariant violation, which the same rule names as the one legitimate use of panic: "Panics are for bugs only — internal invariant violations." The disk-read approach ^7fgdenq proposed would have panicked on a missing FILE, which is one of the three forbidden modes.
    - **`test-integrity/no-test-cheating`.** No `#[ignore]`, real assertions, exact strings rather than "is not empty". The rule warns that an absence assertion on a string that was never present cannot fail, so that was proved separately — see the RED evidence below.
    - **`completeness/case-sensitivity-coverage`.** The needles are exact literals from a document this repo owns, and the domain is genuinely case-sensitive: `DIFFERENT programs` is deliberately capitalized and the command lines are case-sensitive. A case-sensitive match honors the format's real case contract, which is what the rule asks for.
    - **`completeness/inverse-operation-coverage` check 2.** No test name contains `roundtrip`, `symmetry`, `inverse`, `both directions`, or `read back` — the tests read only, so such a name would lie about scope.
    - **`code-hygiene/magic-numbers`.** Counted the sites, as the rule requires. Every needle appears exactly once, so the "genuinely one-off literals used exactly once" carve-out holds.
    - **`code-hygiene/function-length-rust`.** A `#[test]` is explicitly NOT exempt from the 250-code-line gate, and data lines count. The largest of the three tests is well under it.
    - **`code-hygiene/missing-docs-rust`.** Silent on `#[cfg(test)]` items, so doc comments are not required here — all four new items carry them anyway.
    - **`code-hygiene/dead-code-rust`.** Both helpers have callers; nothing is staged.
    - **`completeness/invariant-propagation` check 1** — "why only `swift.md` when sibling partials exist?" The token this guard protects is the Swift formatting guidance, which exists at exactly one site. No sibling partial documents two different formatters that share a name, so no sibling consumes the same token. The rule's own carve-out: "The token is genuinely handled at only one site (verify by searching, don't assume)." Verified by searching.

    ## RED → GREEN evidence

    | run | result |
    |---|---|
    | all three tests against the ORIGINAL partial | 3 failed, each on its own first requirement: missing `.swift-format`, missing `https://github.com/airbnb/swift`, missing `--pattern-let inline` |
    | new partial with `(or \`swiftformat .\`)` appended | `swift_partial_names_each_formatter_with_its_own_config_file` FAILED on the negative assertion; the other two passed |
    | new partial as shipped | 3 passed |

    The middle row is the one that matters for `no-test-cheating`: the absence assertion is provably able to fail, not a trivial assertion that always holds.
  timestamp: 2026-08-22T19:45:23.036320+00:00
- actor: claude-code
  id: 01m0ng3x4w354qt13vfwz77c3h
  text: |-
    ### implement — changed
    - evidence: 2 files — `builtin/_partials/project-types/swift.md`, `crates/swissarmyhammer-templating/src/resolver.rs`. `cargo nextest run` 14191 tests run, 14191 passed, 0 skipped. `cargo fmt --all -- --check` exit 0. `cargo clippy --workspace --all-targets -- -D warnings` exit 0. Three new tests verified RED against the original partial and GREEN against the new one; the negative assertion verified failing on its own by re-appending the old interchangeable phrasing.
    - acceptance: each tool named once beside its own config file (partial lines 48-50); plugin path documented with real command lines, switches and exit codes (lines 57-74); the DIFFERENT-programs statement stands and the `(or \`swiftformat .\`)` phrasing is gone and guarded by an assertion.
    - next: `/review`. Two literals on the card were corrected against measurement — `swift package format --lint` needs `--allow-writing-to-package-directory` in a non-interactive shell, and the switch is `--target`, not `--targets`. ^7fgdenq is fully satisfied by this work and can be archived.
  timestamp: 2026-08-22T19:45:43.580751+00:00
- actor: claude-code
  id: 01m0ngbeg1w4emfkvwsa38ew4x
  text: |
    ### test — green
    - evidence: `cargo nextest run --workspace` — 14191 tests run, 14191 passed, 0 failed, 0 skipped (baseline 14188 + 3 new). `cargo fmt --all -- --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - confirmation 1 (served partial, not disk file): confirmed from source. `crates/swissarmyhammer-templating/src/resolver.rs` test helper `builtin_partial()` calls `get_builtin_partials()`, the same function `PromptResolver::load_builtin_partials()` calls in production. `get_builtin_partials()` is codegen'd by `crates/swissarmyhammer-templating/build.rs` (`BuiltinGenerator::new("partials").source_dir("../../builtin/_partials")`), embedding the content at compile time. The tests measure the artifact an agent actually receives, not a raw file read.
    - confirmation 2 (tests fail on old partial): spot-checked all three. Reverted `builtin/_partials/project-types/swift.md` to `HEAD` content, ran `cargo nextest run -p swissarmyhammer-templating swift_partial` — all 3 tests FAILED (`swift_partial_names_each_formatter_with_its_own_config_file`, `swift_partial_documents_the_airbnb_plugin_commands`, `swift_partial_agrees_with_the_shipped_swift_tool_validators`), each panic naming the missing requirement (e.g. missing ".swift-format", missing "--pattern-let inline"). Restored the file byte-for-byte (verified via `git diff --stat` matching pre-check state) and re-ran — all 3 PASS. Full workspace suite confirmed green after restore.
    - next: none, ready for review
  timestamp: 2026-08-22T19:49:50.721552+00:00
position_column: doing
position_ordinal: '8280'
project: swift-validator
title: 'swift partial: prefer `swift package format --lint` when the Airbnb plugin is a dependency'
---
`builtin/_partials/project-types/swift.md:40-41` names three tools loosely and commits to none:

```
- Format: `swift format -i -r Sources Tests` (or `swiftformat .`) — run before committing
- Lint: `swiftlint` (if present)
```

`swift format` and `swiftformat` are DIFFERENT TOOLS. The first is Apple's `swift-format`, shipped with the toolchain. The second is Nick Lockwood's SwiftFormat, installed separately, with a different config format and a different rule set. Offering them as interchangeable alternatives in one line is wrong and will send an agent to the wrong config file.

## Two fixes

1. Split the two tools onto their own lines. Say which config file each reads: `swift format` reads `.swift-format`; `swiftformat` reads `.swiftformat`.

2. Detect the Airbnb Swift Package Manager plugin and prefer it when present. A package that declares

   ```swift
   .package(url: "https://github.com/airbnb/swift", from: "1.0.0")
   ```

   gets one command that runs SwiftFormat AND SwiftLint with a shared config and one exit code:

   ```
   swift package format --lint          # check, do not write
   swift package --allow-writing-to-package-directory format   # non-interactive fix
   ```

   Useful switches: `--paths Sources Tests Package.swift`, `--targets <Target>`, `--exclude Tests`, `--swift-version 6.2`.

   Exit codes: non-zero on any failure in `--lint` mode; in fix mode, non-zero only for SwiftLint rules, because those cannot autocorrect.

   For a project with this dependency, one install and one command beats wiring two tools ourselves.

## Related

Card ^7fgdenq — "guidelines: swift format instruction — honor an existing .swift-format/.swiftformat config, defaults otherwise" — covers the config-detection half. Read it before starting. These two cards may merge; if they do, close one and say so.

## Acceptance

- The partial names each tool once, with its own config file.
- The plugin path is documented with the real command lines above.
- No line implies `swift format` and `swiftformat` are the same tool. #tool-validators