---
assignees:
- claude-code
comments:
- actor: claude-code
  id: 01m14cg0k2cs50y41chcyr68gq
  text: |-
    Research — the card's claims are all still true.

    All four files stand where the card says:
    - crates/swissarmyhammer-tools/src/mcp/server/agent_identity.rs::agent_deterministic_color (djb2, 10-colour AGENT_COLORS)
    - apps/kanban-app/src/state.rs::deterministic_color (djb2, 15-colour ACTOR_COLORS)
    - apps/kanban-app/ui/src/lib/actor-colors.ts::deriveActorColor (djb2 in BigInt, mirror of the 15-colour palette)
    - crates/swissarmyhammer-kanban/src/auto_color.rs::auto_color (FNV-1a, 16-colour tag palette)

    Both the tools crate and the kanban app already declare swissarmyhammer-kanban in Cargo.toml, so the shared hash needs no new dependency edge and ARCHITECTURE.md needs no change.

    The TypeScript copy is a fallback: components/avatar.tsx calls deriveActorColor while it renders, for an actor the entity store has not loaded or one with no stored `color`. The only route from the webview into Rust is an async Tauri command, one round trip per avatar, which a render cannot wait on. So the copy stays.

    Discovery, out of this card's scope: the TypeScript copy folds `charCodeAt` (UTF-16 code units) where Rust folds `bytes()` (UTF-8), so the two answer different colours for an id outside ASCII. Every id in play today is ASCII (agent ids are slugified, human ids are usernames), so no colour is wrong today. Filed as its own card.
  timestamp: 2026-08-28T14:31:02.498455+00:00
- actor: claude-code
  id: 01m14cwnb9eqpkmyhnknz3fgh0
  text: |-
    Implementation landed.

    One djb2 hash now stands in `swissarmyhammer-kanban/src/auto_color.rs`:
    `palette_color(palette, key)` folds the key with djb2 and picks the entry; the
    private `djb2` holds the seed and the multiplier. Both Rust call sites are one
    line that passes their own palette in. The three copies are one: `grep 5381`
    answers with the one Rust constant plus the documented TypeScript mirror.

    The FNV-1a fourth copy stays FNV-1a. A tag colour is written into the tag file
    when the tag is created, so a move to djb2 would re-colour the tags on every
    board that already exists — a visible change with nothing to gain. The reason
    stands in the `auto_color` doc comment.

    The TypeScript copy stays a copy, and the reason stands in the module comment:
    `Avatar` derives the colour while it renders, and the only route from the
    webview into Rust is an async Tauri command — one round trip per avatar, which
    a render cannot wait on. Its literals are named (DJB2_SEED, DJB2_MULTIPLIER,
    U64_MASK), and a new `actor-colors.test.ts` asserts the same table the Rust
    test asserts, so the two cannot drift apart in silence.

    Behaviour is preserved, and it was pinned BEFORE the refactor:
    - Three tables of input to colour were computed from the djb2 definition in
      Python, independent of the code under test.
    - All three passed against the un-refactored code (Rust 5/5, TypeScript 2/2).
    - Each table was proved live: one wrong colour in each Rust table made both
      Rust tests FAIL; multiplier 34n made the TypeScript test FAIL. All were
      restored.
    - After the refactor the same tables pass, so every existing actor keeps the
      colour it has.
  timestamp: 2026-08-28T14:37:56.969760+00:00
- actor: claude-code
  id: 01m14cww7rezb81dgrzy1e9xaw
  text: |-
    ### implement — changed
    - evidence: 5 files — crates/swissarmyhammer-kanban/src/auto_color.rs, crates/swissarmyhammer-tools/src/mcp/server/agent_identity.rs, apps/kanban-app/src/state.rs, apps/kanban-app/ui/src/lib/actor-colors.ts, apps/kanban-app/ui/src/lib/actor-colors.test.ts (new). `cargo nextest run --workspace`: 14264 tests run, 14264 passed, 0 skipped. `npm test` in apps/kanban-app/ui: 243 test files, 2253 tests, all passed, tsc clean. `cargo fmt --all --check` clean. `cargo clippy --workspace --all-targets -- -D warnings` clean.
    - next: ready for /review. New card ^gym7gr1 files the non-ASCII UTF-8 vs UTF-16 divergence the TypeScript copy carries.
  timestamp: 2026-08-28T14:38:04.024501+00:00
- actor: claude-code
  id: 01m14d7kwk908a7bjskkerwa7z
  text: |-
    ### finish iteration 1 — clean
    - implement: changed — 5 files. One shared `palette_color(palette, key)` in auto_color.rs over a private djb2 with named seed and multiplier; each palette stays with its caller.
    - test: green — cargo nextest run --workspace 14264 passed, 0 skipped; npm test in apps/kanban-app/ui 2253 tests passed with tsc clean; fmt and clippy clean
    - commit: 00a910dd0
    - review: clean — 0 findings, 1 candidate refuted, 9 validators attempted, 5 files reviewed
    - next: none — the card is in done

    The behaviour-preservation requirement was met the way it should be: input-to-colour tables were computed from the djb2 definition independently of the code, asserted at each site against the UN-refactored code, then each table was proved live by mutation — one wrong colour failed both Rust tests, and multiplier `34n` failed the TypeScript test. The same tables pass after the refactor, so no existing actor changes colour.

    Two decisions recorded rather than made silently:

    The FNV-1a fourth copy stays FNV-1a. A tag colour is written into the tag file at creation, so moving it to djb2 would re-colour tags on every existing board — a visible change with nothing to gain.

    The TypeScript copy stays a copy. `Avatar` derives the colour while it renders, and the only route from the webview into Rust is an async Tauri command — one round trip per avatar, which a render cannot wait on. The reason is in the module comment, and a new TypeScript test asserts the same table the Rust test asserts so the copy cannot drift in silence.

    One discovery filed separately as ^gym7gr1: the TypeScript copy folds `charCodeAt` (UTF-16 code units) where Rust folds `bytes()` (UTF-8), so the two disagree for a non-ASCII id. No id in play today is non-ASCII, and correcting it would change a colour, so it is out of this behaviour-preserving card.
  timestamp: 2026-08-28T14:43:55.923361+00:00
position_column: done
position_ordinal: ffffffffffffffffffffffffffffffffffffffffffb880
title: Share one djb2 actor-colour helper instead of three copies
---
## Problem

The same djb2 hash-to-palette helper is written three times, each with its own
palette:

- `crates/swissarmyhammer-tools/src/mcp/server/agent_identity.rs::agent_deterministic_color`
- `apps/kanban-app/src/state.rs::deterministic_color`
- `apps/kanban-app/ui/src/lib/actor-colors.ts::deriveActorColor`

`code_context find duplicates` measures the first two at 96.9% alike and the
third at 87.5%. A fourth, `crates/swissarmyhammer-kanban/src/auto_color.rs`,
does the same job with FNV-1a and a fourth palette.

## Task

Put one hash in `swissarmyhammer-kanban` — the crate both the tools crate and
the kanban app already depend on — and call it from each Rust site. Keep each
palette where it is; only the hash is shared. Decide whether the TypeScript
copy must stay a copy, and say why in the code if it must.

## Found by

Card ^hxd1r4r, while naming the djb2 constants in `agent_identity.rs`. Out of
that card's scope because the fix crosses three crates. #kanban