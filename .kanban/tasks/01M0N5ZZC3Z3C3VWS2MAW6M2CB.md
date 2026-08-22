---
assignees:
- claude-code
position_column: todo
position_ordinal: fff180
project: swift-validator
title: 'review: dump validators drops the VALIDATOR.md guidance the real prompt carries'
---
Found while working ^qs32yvp, which had to know what a validator body costs a review run.

`dump validators` does NOT render a validator set's `VALIDATOR.md` body, and the real review prompt DOES. So an agent that preloads the dump reads a different rule set than the one the reviewer is held to.

## The two shapes, measured by reading the code

The review prompt renders the manifest body once per validator task, in the same block of bytes as the rule bodies:

- `crates/swissarmyhammer-validators/src/validators/parser.rs` — `parse_ruleset_directory` puts the `VALIDATOR.md` body under the front matter into `RuleSet.manifest_body`.
- `crates/swissarmyhammer-validators/src/review/fleet/render.rs` — `render_suffix` calls `render_validator_guidance(&mut out, ruleset.manifest_body())`, which emits a `## Guidance` block between the mandate and the rules, then writes each non-tool rule body under `## Rules`.
- `render_suffix` is the single body behind both the warm path (`render_validator_suffix`) and the degraded path (`render_fleet_prompt`), so no review shape misses it.

The dump does not:

- `crates/swissarmyhammer-tools/src/mcp/tools/review/validators.rs` — `render_rules_markdown` writes the ruleset NAME, its `description` and its source layer, then each rule body. It never reads `manifest_body`.

## Two differences, not one

1. **The dump is missing the `## Guidance` block.** Validator-wide intent, scope and blanket carve-outs authored in `VALIDATOR.md` are invisible to anything preloading the dump. The `swift` set states "if a tool can decide it, the tool owns it" there, and a preloading agent never reads it.
2. **The dump INCLUDES tool rules.** `rule_details` iterates `ruleset.rules` with no `is_tool_rule()` filter, while the prompt filters them out. So the dump shows rule bodies no reviewer ever reads, which is the opposite error.

## Why it matters here

The `implement` skill preloads `dump validators` so an author writes to the rules review will enforce. Both differences break that promise in opposite directions: the author misses guidance a reviewer is given, and reads tool-rule bodies a reviewer is not.

## One more measurement worth carrying

`validator_suffix_framing_bytes` runs the same `render_suffix`, and `prompt_framing` takes the MAX over validators. So a long `VALIDATOR.md` on ONE validator shrinks the file-payload budget for EVERY batch in the run. That is an argument for keeping a manifest body short, and it belongs in `builtin/validators/README.md` where an author reads it.

## Acceptance

- `dump validators` renders the manifest body in the same place the review prompt renders it, and filters tool rules the same way — or states in `builtin/validators/README.md` why the two shapes differ on purpose.
- A test asserts the dump and the review suffix agree on which bodies each carries. `renderer.rs` already holds `validator_suffix_emits_the_manifest_body_after_mandate_before_rules`; nothing holds the dump to anything.

#tool-validators