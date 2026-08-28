/**
 * Canonical actor color palette and deterministic hash.
 *
 * This is a hand-kept copy of Rust, and it stays a copy. The Rust hash lives in
 * `swissarmyhammer-kanban/src/auto_color.rs` (`palette_color`) and the palette
 * below is copied verbatim from `ACTOR_COLORS` in `kanban-app/src/state.rs`,
 * which stays the source of truth for both. Any change is made in Rust first,
 * then mirrored here.
 *
 * The copy cannot be replaced by a call into Rust: `Avatar` derives the color
 * while it renders, for an actor the entity store has not loaded or one that
 * carries no stored `color`, and the only route from the webview into Rust is
 * an async Tauri command — one round trip per avatar, which a render cannot
 * wait on. So the copy stays, and `actor-colors.test.ts` holds it to the same
 * table the Rust `test_deterministic_color_matches_pinned_table` asserts on.
 *
 * One difference is known and is out of this module's hands: Rust folds the
 * UTF-8 bytes of the id and this folds UTF-16 code units, so the two answer
 * different colors for an id outside ASCII. Every id in play today is ASCII —
 * agent ids are slugified, human ids are usernames.
 */

/** 15-color palette matching `ACTOR_COLORS` in state.rs. */
export const ACTOR_COLORS: readonly string[] = [
  "e53e3e",
  "dd6b20",
  "d69e2e",
  "38a169",
  "319795",
  "3182ce",
  "5a67d8",
  "805ad5",
  "d53f8c",
  "2b6cb0",
  "c05621",
  "2f855a",
  "2c7a7b",
  "6b46c1",
  "b83280",
];

/** The djb2 starting hash. Part of the published algorithm, not a tunable. */
const DJB2_SEED = 5381n;

/** The djb2 per-byte multiplier. Part of the published algorithm, not a tunable. */
const DJB2_MULTIPLIER = 33n;

/** Every bit a Rust `u64` holds, so `&` reproduces its wrapping arithmetic. */
const U64_MASK = 0xffff_ffff_ffff_ffffn;

/**
 * Derive a deterministic hex color from a string using djb2 hash.
 *
 * Matches `palette_color(ACTOR_COLORS, id)` in Rust exactly:
 *   hash = bytes.fold(5381, |h, b| h.wrapping_mul(33).wrapping_add(b))
 *   palette[hash % len]
 *
 * We use BigInt to faithfully reproduce Rust's u64 wrapping arithmetic.
 */
export function deriveActorColor(id: string): string {
  let hash = DJB2_SEED;
  for (let index = 0; index < id.length; index++) {
    hash = (hash * DJB2_MULTIPLIER + BigInt(id.charCodeAt(index))) & U64_MASK;
  }
  return ACTOR_COLORS[Number(hash % BigInt(ACTOR_COLORS.length))];
}
