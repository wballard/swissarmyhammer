/**
 * Unit tests for {@link deriveActorColor} — the avatar fallback colour.
 *
 * `deriveActorColor` is a hand-kept copy of the Rust `deterministic_color` in
 * `kanban-app/src/state.rs`. The copy earns its keep only while the two answer
 * the same colour for the same actor id, so the table below is the same table
 * `test_deterministic_color_matches_pinned_table` asserts on the Rust side.
 * Change one, and this test tells you the other must change too.
 */
import { describe, it, expect } from "vitest";
import { ACTOR_COLORS, deriveActorColor } from "./actor-colors";

/** Actor id to the colour the Rust `deterministic_color` answers for it. */
const PINNED_ACTOR_COLORS: readonly (readonly [string, string])[] = [
  ["alice", "b83280"],
  ["bob", "3182ce"],
  ["will", "5a67d8"],
  ["claude-code", "e53e3e"],
  ["carol", "38a169"],
  ["dave", "b83280"],
  ["", "2f855a"],
  ["a", "c05621"],
  ["z", "3182ce"],
];

describe("deriveActorColor", () => {
  it("answers the colour Rust answers for the same actor id", () => {
    for (const [id, expected] of PINNED_ACTOR_COLORS) {
      expect(deriveActorColor(id), `the colour of ${id} changed`).toBe(
        expected,
      );
    }
  });

  it("carries the palette Rust carries, in Rust's order", () => {
    expect(ACTOR_COLORS).toEqual([
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
    ]);
  });
});
