---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m11zwmt4yh962n9fbt9qnkfa
  text: |-
    Picked up. Research done, and the picture differs from the card in one part.

    ## The read side is already whole-line, and the named card survives

    `swissarmyhammer_common::frontmatter::split_frontmatter_body` compares a WHOLE
    line to `---` (`is_delimiter_line`, `line_content(raw) == "---"`). It landed in
    commit 3523b4594 on 2026-08-01, with `60a173bf2` and `8dee9cb0f` after it.

    Both entity readers call that one function:
    - `crates/swissarmyhammer-entity/src/io.rs::parse_frontmatter_body`
    - `crates/swissarmyhammer-entity/src/store.rs::deserialize`

    I drove the REAL bytes of `.kanban/tasks/01M076YBBHE5ZQJCM518491BB0.md` -- the
    card the report names -- through the production path in a temp board: `update
    task`, then a `delete tag` sweep. The card kept its title, its body and every
    comment, table row and all. So the current source does not reproduce the
    corruption on that card.

    The write path is `store.rs::serialize`, NOT `io.rs::format_frontmatter_body`
    (kanban registers a store for every entity type in `context.rs::register_entity_stores`,
    so `io::write_entity` is the no-store fallback only).

    ## The board is clean

    `grep -rl "^title: Untitled$" .kanban/tasks/` answers 3 files:
    - this card itself, whose description quotes the string as prose;
    - two files under `.kanban/tasks/.trash/`, each a genuinely empty task
      (`position_column`, `position_ordinal`, `title: Untitled`, no body).

    So no live card needs restoring from its changelog. The earlier sweep was
    reverted with git, which took the `.jsonl` changelogs back with it.

    ## What IS still open, and it is the other half of the card

    The write side carries NO delimiter test at all. Both writers emit
    `format!("---\n{}---\n{}", frontmatter_yaml, body)` and TRUST the YAML emitter
    never to put a bare `---` at column 0, and never to hand back YAML that does
    not end in a newline. Nothing checks it. If either ever holds, the card is
    written corrupt and nothing reports it until a later read loses the title.
    `io.rs` pins the emitter with `format_writes_exactly_two_delimiter_lines`, but
    `store.rs` -- the LIVE path -- has no such test and no markdown-table test.

    That is what the card asks for: "the delimiter test must be a whole-line test
    on both the read and the write side." I am implementing the write-side half.
  timestamp: 2026-08-27T16:12:16.068896+00:00
- actor: claude-code
  id: 01m120n8g133bc82ntb5jhbw79
  text: |-
    Implementation landed. The write side now carries the whole-line delimiter test.

    ## What changed

    `swissarmyhammer_common::frontmatter::join_frontmatter_body(frontmatter_yaml, body)`
    is the new write-side twin of `split_frontmatter_body`, and it holds the SAME
    rule: a delimiter is a WHOLE line equal to `---`. It answers the two conditions
    that let a written file split back into different halves than it was built from:

    1. A line of the frontmatter YAML that is itself a delimiter line. The write is
       REFUSED with the new `DelimiterInFrontmatter` error, which names the line.
       Before, both writers emitted `format!("---\n{}---\n{}", ..)` and TRUSTED the
       YAML emitter. Nothing measured it, so a card written that way was destroyed
       on the next read and kept no record of what it held.
    2. Frontmatter YAML that does not end in a newline. That put the closing `---`
       on the end of the last YAML line, where it is not a whole line and does not
       delimit at all. The terminator is now added.

    Both writers call it, so the two sides cannot drift:
    - `swissarmyhammer-entity/src/store.rs::serialize` -- the LIVE kanban path.
    - `swissarmyhammer-entity/src/io.rs::format_frontmatter_body` -- the no-store
      fallback.

    The hand-rolled `format!` is gone from both.

    ## Errors

    - `StoreError::Serialize(#[source] Box<dyn Error + Send + Sync>)`. The source is
      BOXED, not flattened to a string, so `Error::source()` still reaches the real
      cause. `swissarmyhammer-store` is a Tier 0 leaf with zero workspace
      dependencies (ARCHITECTURE.md), so it cannot name the error type directly.
    - `EntityError::FrontmatterDelimiter { path, source }`. Blast radius: the
      exhaustive match in `apps/kanban-cli/src/commands/serve.rs::classify_entity_error_kind`
      now classifies it as `Internal`.

    ## RED before GREEN

    I replaced the guard body with the unguarded `format!` both writers carried
    before, and ran the crate. Two of the new tests FAILED:
    - `adds_the_terminator_the_closing_delimiter_needs` --
      `left: "---\ntitle: x---\nbody\n"`, `right: "---\ntitle: x\n---\nbody\n"`.
    - `refuses_frontmatter_holding_a_delimiter_line`.
    Then I restored the guard and both went GREEN.

    ## Tests

    - `crates/swissarmyhammer-common/src/frontmatter.rs`: 6 new tests for the join.
      A markdown table row `|---|---|`, an indented block-scalar run, an inline
      `a --- b`, and a `----` run must all JOIN and split back to themselves.
    - `crates/swissarmyhammer-entity/src/store.rs`: the LIVE path had NO
      markdown-table test and no write-side pin at all -- only `io.rs`, the dead
      fallback, carried them. Added
      `test_a_markdown_table_in_a_comment_survives_the_round_trip` and
      `test_serialize_writes_exactly_two_delimiter_lines` (11 emitter-style shapes).
    - `crates/swissarmyhammer-kanban/tests/frontmatter_delimiter_roundtrip.rs`: NEW
      production-path file. It drives `AddTask` -> `AddComment` -> `UpdateEntityField`,
      re-opens the board from disk, and asserts the title, the description and the
      comment text all survive. One test for a comment holding `|---|---|---|---|`,
      one for a description holding a `---` rule on its own line.

    ## The sweep: zero cards

    `grep -rl "^title: Untitled$" .kanban/tasks/` answers only this card, whose
    description quotes the string as prose. The two `.trash` matches are genuinely
    empty tasks, not corruption. Nothing to restore.

    ## Dead end, so the next agent does not repeat it

    I spent the first half of this card hunting a live substring splitter. There
    is none in the kanban path. I copied the REAL bytes of the named card into a
    temp board and ran `update task` and a `delete tag` sweep over it through the
    production ops: the title, body and every table row survived. The read-side
    fix landed on 2026-08-01 (3523b4594). Do not go looking for it again.

    Other crates DO still carry substring splits -- `swissarmyhammer-validators`
    `parser.rs::extract_frontmatter`, `swissarmyhammer-skills` and
    `swissarmyhammer-agents` `split_frontmatter` (`.find("\n---")`),
    `swissarmyhammer-templating` `frontmatter.rs` (`line.trim() == "---"`, which an
    indented block-scalar run closes early, plus a CRLF byte-offset undercount), and
    `markdowndown`. None of them read kanban cards, so they are outside this card.
    They deserve their own.

    ## Verification

    - `cargo nextest run --workspace`: 14234 passed, 0 failed, 0 skipped.
    - `cargo clippy --workspace --all-targets -- -D warnings`: clean.
    - `cargo fmt --check`: clean.
  timestamp: 2026-08-27T16:25:42.657396+00:00
- actor: claude-code
  id: 01m120nht0sympzdacvghks5hy
  text: |-
    ### implement — changed
    - evidence: 6 files — crates/swissarmyhammer-common/src/frontmatter.rs (new `join_frontmatter_body` + `DelimiterInFrontmatter`, 6 tests), crates/swissarmyhammer-store/src/error.rs (`StoreError::Serialize` with a boxed source), crates/swissarmyhammer-entity/src/store.rs (live writer calls the shared join; 2 new tests), crates/swissarmyhammer-entity/src/io.rs (fallback writer calls it), crates/swissarmyhammer-entity/src/error.rs (`EntityError::FrontmatterDelimiter`), apps/kanban-cli/src/commands/serve.rs (exhaustive match arm), plus NEW crates/swissarmyhammer-kanban/tests/frontmatter_delimiter_roundtrip.rs (2 production-path tests). RED first: with the guard replaced by the old unguarded `format!`, `adds_the_terminator_the_closing_delimiter_needs` and `refuses_frontmatter_holding_a_delimiter_line` both FAILED; GREEN after restoring it. `cargo nextest run --workspace` gives 14234 passed, 0 failed, 0 skipped. `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean. Board sweep: 0 cards read `title: Untitled`, so nothing needed restoring.
    - next: /review
  timestamp: 2026-08-27T16:25:52.192751+00:00
- actor: claude-code
  id: 01m12144fbejtnh65vs9mm4jw9
  text: |-
    ### review — findings
    - evidence: 2 findings (2 confirmed, 0 refuted) from `review sha HEAD~1..HEAD` — crates/swissarmyhammer-entity/src/error.rs:24, crates/swissarmyhammer-entity/src/error.rs:25
    - next: document every field of the struct in crates/swissarmyhammer-entity/src/error.rs, then re-run review. Task stays in review.
  timestamp: 2026-08-27T16:33:50.059011+00:00
- actor: claude-code
  id: 01m1214rqjt6h5644cw4geg1a8
  text: |-
    ### finish iteration 1 — findings
    - implement: changed — 7 files; new `join_frontmatter_body` write-side guard; RED verified before GREEN
    - test: green — cargo nextest run --workspace, 14234 passed, 0 failed, 0 skipped; clippy and fmt clean
    - commit: c743d620b
    - review: findings — crates/swissarmyhammer-entity/src/error.rs:24, crates/swissarmyhammer-entity/src/error.rs:25 (code-hygiene/missing-docs-rust)
    - next: document every field of that struct in error.rs, not only the two named lines
  timestamp: 2026-08-27T16:34:10.802196+00:00
- actor: claude-code
  id: 01m121hwggazws5gpscv7tszwf
  text: |
    Pulled the card back to `doing` and worked the two open findings.

    ## The cause, and the whole file

    The finding names `error.rs:24` and `error.rs:25` -- the two fields of the new
    `FrontmatterDelimiter` variant. The cause is broader: `EntityError` is a public
    enum, so EVERY field of EVERY struct variant is a public item, and
    `missing_docs` asks each one for a doc comment.

    I measured the file with the rule's own tool instead of reading it by eye:

    ```
    RUSTFLAGS="-W missing_docs" cargo clippy -p swissarmyhammer-entity \
      --message-format=json | jq -r 'select(.reason=="compiler-message")
      | select(.message.code.code=="missing_docs")
      | "\(.message.spans[0].file_name):\(.message.spans[0].line_start) \(.message.message)"' \
      | sort -u | grep 'swissarmyhammer-entity/src/error.rs'
    ```

    That answered 21 lines, not 2. `sort -u` on `spans[0]` collapses two fields
    that share one line, so the real count of undocumented fields is higher still.
    Only `TransactionPartialFailure` already documented its fields.

    ## What changed

    Every field of every struct variant in `crates/swissarmyhammer-entity/src/error.rs`
    now carries a doc comment: `NotFound`, `InvalidFrontmatter`,
    `FrontmatterDelimiter`, `Yaml`, `UnknownEntityType`, `ValidationFailed`,
    `ComputeError`, `StaleChange`, `UnsupportedUndoOp`, `ChangelogEntryNotFound`,
    `RestoreFromTrashFailed`, `InvalidPath`, `AttachmentSourceNotFound`,
    `AttachmentNotFound` and `AttachmentTooLarge`. The single-line variants became
    multi-line to hold the comments. No behavior changed, and no signature changed,
    so the blast radius is empty: the variants keep their names, their fields and
    their field types.

    ## After

    The same command answers 0 lines for that file.

    ## For the next agent

    The workspace holds many more files with the same cause -- the run over
    `-p swissarmyhammer-entity` alone reported 189 lines across
    `swissarmyhammer-fields/src/types.rs`, `swissarmyhammer-js/src/error.rs`,
    `swissarmyhammer-operations/src/parameter.rs` and more. Review only reports
    findings in CHANGED files, so those stay silent until someone touches them.
    They are outside this card.

    ## Verification

    - `cargo nextest run --workspace`: 14234 passed, 0 failed, 0 skipped.
    - `cargo clippy --workspace --all-targets -- -D warnings`: clean.
    - `cargo fmt --check`: clean.
  timestamp: 2026-08-27T16:41:20.656538+00:00
- actor: claude-code
  id: 01m121j2tbp7n2dprbbxm2s9w2
  text: |
    ### implement — changed
    - evidence: 1 file — crates/swissarmyhammer-entity/src/error.rs. Documented every field of every struct variant of `EntityError`, not only the two lines the findings named. Measured with the rule's own tool: `RUSTFLAGS="-W missing_docs" cargo clippy -p swissarmyhammer-entity --message-format=json | jq ... | grep error.rs` answered 21 lines before and 0 lines after. `cargo nextest run --workspace` gives 14234 passed, 0 failed, 0 skipped. `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean. Both `## Review Findings` items are now `- [x]`.
    - next: /review
  timestamp: 2026-08-27T16:41:27.115055+00:00
position_column: doing
position_ordinal: '8380'
title: kanban task write corrupts a card whose front matter holds a `---` run
---
A read-write round trip of a task destroys the card when the stored front
matter holds a `---` run. The card loses its title and part of its body.

## What happened

`01M076YBBHE5ZQJCM518491BB0` carries a comment whose text holds a markdown
table. One row of that table is `|---|---|---|---|`. The comment is stored in
the front matter of `.kanban/tasks/01M076YBBHE5ZQJCM518491BB0.md`.

One `update task` on the card rewrote the file as:

```
    | the refusing path | status | stdout | stderr |
    |
title: Untitled
---
|---|---|---|
```

The reader stopped the front matter at the `---` INSIDE the table row. It
then read no `title`, so the writer wrote `title: Untitled`, and the rest of
the table became the first line of the body. The card's real title and the
first part of its description were lost.

The delimiter test must be a WHOLE LINE that equals `---`. It reads a
substring today.

## The blast radius

The sweep on ^4nzhg4s wrote 104 cards and broke 13 of them this way. One
`grep -rl "title: Untitled" .kanban/tasks/*.md` over the board before that
sweep already answered 1 card, so the defect has struck before.

Every op that writes a task carries it: `update task`, `tag task`,
`untag task`, `delete tag` (which rewrites every task that holds the tag),
`rename tag`.

## Work

- Make the front-matter delimiter a whole-line test on both the read and the
  write side.
- Add a test that round-trips a task whose comment text holds `|---|---|`
  and asserts the title, the description, and the comment all survive.
- Add a test that round-trips a task whose DESCRIPTION holds a `---`
  horizontal rule on its own line.
- Sweep the board for the cards this already broke: find each
  `title: Untitled` card and restore it from its changelog.

## Done when

A card whose front matter holds a `---` run survives a write unchanged, and
no card on the board reads `title: Untitled`. #bug #kanban

## Review Findings (2026-08-27 11:27)

> Scope: `review sha HEAD~1..HEAD` — reviewed the diffs only — lines this change added or modified. 7 file(s) reviewed, 4 not reviewed.

> 4 file(s) not reviewed — excluded by an ignore rule:
> - `.kanban/ (from .reviewignore)` — 4 file(s)

- [x] `crates/swissarmyhammer-entity/src/error.rs:24` `code-hygiene/missing-docs-rust` — missing documentation for a struct field.
- [x] `crates/swissarmyhammer-entity/src/error.rs:25` `code-hygiene/missing-docs-rust` — missing documentation for a struct field.