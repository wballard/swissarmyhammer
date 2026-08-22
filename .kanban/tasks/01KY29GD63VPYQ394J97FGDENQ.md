---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m0nfm8e28fg6e0n6hk55n7cj
  text: |-
    ## Superseded by ^m7ynz9c — do not work this card as written

    ^m7ynz9c ("swift partial: prefer `swift package format --lint` when the Airbnb plugin is a dependency") rewrites the SAME Format/Lint bullet in `builtin/_partials/project-types/swift.md`. Both halves of this card landed there:

    - **Checkbox 1 (the Format bullet).** The partial now names each tool beside the config file it alone reads (`swift format` → `.swift-format`, `swiftformat` → `.swiftformat`, `swiftlint` → `.swiftlint.yml`), states plainly that the first two are DIFFERENT programs, says to use the tool whose config file the repo already holds and to obey it, forbids writing a config file as a side effect of formatting, forbids style flags that disagree with a config already there, and states the tool defaults apply when no config exists.
    - **Checkbox 2 (the content regression test).** Landed as `swift_partial_separates_the_two_formatters_and_documents_the_airbnb_plugin` in `crates/swissarmyhammer-templating/src/resolver.rs`. Verified RED against the old partial, GREEN against the new one.

    Two departures from this card's text, both driven by measurement:

    1. **The test reads the SERVED content, not the file on disk.** This card proposed a test in `crates/swissarmyhammer-project-detection/src/types.rs` reading the `.md` through `CARGO_MANIFEST_DIR`. That crate holds only the partial's PATH string; it never sees the content. `swissarmyhammer-templating` embeds the partial at build time via `get_builtin_partials()` and `PromptResolver::load_builtin_partials` registers it as `_partials/project-types/swift` — the same path `partial!("swift")` builds. Asserting there measures the text an agent actually receives. `spec_partial_matches_key` is untouched and still passes.

    2. **"`swiftformat .` honors `.swiftformat` likewise" is not sufficient advice.** Measured on swiftformat 0.62.1, the four options our shipped `idioms-swift` validator pins all default the other way:

       | option | default | what `idioms-swift` pins |
       |---|---|---|
       | `--pattern-let` | `hoist` | `inline` |
       | `--short-optionals` | `preserve-struct-inits` | `always` |
       | `--single-line-for-each` | `ignore` | `convert` |
       | `--guard-like-if-statements` | `preserve` | `convert` |

       A bare `swiftformat .` rewrote `if case .some(let inner) = value` into `if case let .some(inner) = value`, and `idioms-swift` then reported that exact line. So the partial states the four options, and states that a repo `.swiftformat` already carrying them makes a bare run correct.

    Config discovery was measured, not assumed: a root `.swift-format` reached a file two directories down (7-space indentation applied, no `--configuration` flag), and a root `.swiftformat` did the same.

    This card can be archived.
  timestamp: 2026-08-22T19:37:10.850616+00:00
position_column: todo
position_ordinal: b280
title: 'guidelines: swift format instruction — honor an existing .swift-format/.swiftformat config, defaults otherwise'
---
## What

The Swift project guidelines partial — `builtin/_partials/project-types/swift.md`, served through `detect projects` via the `partial!("swift")` entry in `crates/swissarmyhammer-project-detection/src/types.rs` (~line 261) — currently gives one bare formatting line (line 40):

> `- Format: \`swift format -i -r Sources Tests\` (or \`swiftformat .\`) — run before committing`

That says nothing about configuration, so an agent working in someone else's repo can steamroll the project's own style. Update it to make config-honoring explicit:

- [ ] **Replace the line-40 Format bullet** in `builtin/_partials/project-types/swift.md` with guidance to this effect (wording may be polished, substance fixed):
  - If the repo has a `.swift-format` config (Apple's `swift-format`), run `swift format -i -r Sources Tests` — the tool discovers and honors the config automatically (it searches each file's directory and its parents). Never pass ad-hoc style flags or `--configuration` overrides that fight it, and never edit or regenerate the config as a side effect of formatting.
  - If the repo instead has a `.swiftformat` config (Nick Lockwood's SwiftFormat), use `swiftformat .`, which honors it likewise — pick the tool that matches the config file present.
  - If NO formatter config exists, format with the tool defaults (`swift format -i -r Sources Tests`) and do NOT create a config file as a side effect.
- [ ] **Add a content regression test** next to `spec_partial_matches_key` (~line 586) in `crates/swissarmyhammer-project-detection/src/types.rs` `#[cfg(test)]`: read `../../builtin/_partials/project-types/swift.md` via `CARGO_MANIFEST_DIR` (the same relative-root convention `swissarmyhammer-validators/src/builtin/mod.rs` uses for `../../builtin/validators`) and assert the Format guidance (a) mentions honoring an existing `.swift-format` and `.swiftformat`, (b) states defaults apply when no config exists, and (c) forbids creating/overriding a config as a formatting side effect — so the instruction can't silently regress to the bare one-liner.

**Deploy note (not part of this card's code change):** builtin content changes need the usual rebuild + redeploy (`just sah` + `sah init`) before deployed copies serve the new text.

## Acceptance Criteria

- [ ] `builtin/_partials/project-types/swift.md` no longer contains the bare `- Format: \`swift format -i -r Sources Tests\` (or \`swiftformat .\`) — run before committing` line; the replacement covers all three cases (`.swift-format` present → honor it; `.swiftformat` present → `swiftformat .` honors it; neither → tool defaults) and forbids creating or overriding a config as a side effect.
- [ ] The file's frontmatter (`title` / `description` / `partial: true`) and all other sections (ULID guidance, Testing, other Common commands, File locations) are unchanged.
- [ ] The new content regression test fails against the pre-change file and passes after.
- [ ] `spec_partial_matches_key` still passes (partial path untouched).

## Tests

- [ ] New test in `crates/swissarmyhammer-project-detection/src/types.rs` `#[cfg(test)]` asserting the swift partial's Format guidance covers config-honoring, defaults fallback, and the no-config-creation rule.
- [ ] `cargo nextest run -p swissarmyhammer-project-detection` — green.

## Workflow
- Use `/tdd` — write failing tests first, then implement to make them pass.