//! Deterministic color assignment: a name in, a palette entry out.
//!
//! Two hashes stand here, because two are already in the wild:
//!
//! - [`auto_color`] colors a tag with FNV-1a over this module's own palette.
//! - [`palette_color`] colors anything with djb2 over a palette the caller
//!   owns. The kanban app colors a person with it, and the MCP server colors a
//!   connecting agent with it, each over its own palette.
//!
//! A color is written onto the entity when the entity is created, and it is
//! read back for as long as that entity lives. So neither hash may change the
//! answer it gives: a new answer re-colors entities that already exist. The
//! two hashes stay separate for that reason, and not because either is better.

/// Curated palette of 16 tag colors (6-char hex without `#`).
///
/// These are chosen to be distinct, readable as pill backgrounds with white or
/// dark text, and visually pleasant in a kanban UI.
const PALETTE: &[&str] = &[
    "d73a4a", // red
    "e36209", // orange
    "f9c513", // yellow
    "0e8a16", // green
    "006b75", // teal
    "1d76db", // blue
    "5319e7", // purple
    "b60205", // dark red
    "d876e3", // pink
    "0075ca", // ocean
    "7057ff", // violet
    "008672", // sea green
    "e4e669", // lime
    "bfd4f2", // light blue
    "c5def5", // periwinkle
    "fbca04", // gold
];

/// Return a deterministic color for a tag slug.
///
/// Uses a simple FNV-1a hash mapped to the palette index. Tags keep FNV-1a
/// while actors use [`palette_color`]: every tag that carries a color got it
/// from this hash, and a tag color is written into the tag file, so a move to
/// djb2 would re-color the tags on every board that already exists.
pub fn auto_color(slug: &str) -> &'static str {
    let hash = fnv1a(slug);
    let idx = (hash as usize) % PALETTE.len();
    PALETTE[idx]
}

/// FNV-1a hash (32-bit) for short strings.
fn fnv1a(s: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in s.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// The djb2 starting hash. Part of the published algorithm, not a tunable.
const DJB2_SEED: u64 = 5381;

/// The djb2 per-byte multiplier. Part of the published algorithm, not a tunable.
const DJB2_MULTIPLIER: u64 = 33;

/// Return a deterministic entry of `palette` for `key`.
///
/// `key` is folded with djb2 and the digest picks the entry, so the same key
/// always picks the same entry and a caller derives a color instead of storing
/// one. This is the one hash, not the one palette: each caller keeps the
/// palette its own surface was designed around and passes it in.
///
/// # Panics
///
/// Panics when `palette` is empty. A palette to pick from is the caller's to
/// supply, and there is no entry to answer with.
pub fn palette_color(palette: &[&'static str], key: &str) -> &'static str {
    assert!(
        !palette.is_empty(),
        "palette_color needs a palette to pick from"
    );
    palette[(djb2(key) as usize) % palette.len()]
}

/// djb2 hash (64-bit) for short strings.
fn djb2(s: &str) -> u64 {
    s.bytes().fold(DJB2_SEED, |hash, byte| {
        hash.wrapping_mul(DJB2_MULTIPLIER).wrapping_add(byte as u64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_color_deterministic() {
        let c1 = auto_color("bug");
        let c2 = auto_color("bug");
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_auto_color_different_tags_differ() {
        let c1 = auto_color("bug");
        let c2 = auto_color("feature");
        // Not guaranteed to differ, but very likely with 16 colors
        // Just ensure they're valid hex
        assert_eq!(c1.len(), 6);
        assert_eq!(c2.len(), 6);
        // At minimum, both should be from the palette
        assert!(PALETTE.contains(&c1));
        assert!(PALETTE.contains(&c2));
    }

    #[test]
    fn test_auto_color_valid_hex() {
        for slug in &["bug", "feature", "docs", "urgent", "low-priority", "v2"] {
            let color = auto_color(slug);
            assert_eq!(color.len(), 6);
            assert!(color.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }

    /// A palette of four named entries, so a wrong index is a wrong name.
    const PROBE_PALETTE: &[&str] = &["first", "second", "third", "fourth"];

    /// The entry djb2 picks out of [`PROBE_PALETTE`] for each key. Computed
    /// from the djb2 definition alone, not from this module's code, so the
    /// table fails whenever the two stop agreeing.
    const PINNED_PROBE_COLORS: &[(&str, &str)] = &[
        ("alice", "fourth"),
        ("bob", "first"),
        ("carol", "third"),
        ("dave", "second"),
        ("", "second"),
    ];

    #[test]
    fn test_palette_color_matches_pinned_table() {
        for (key, expected) in PINNED_PROBE_COLORS {
            assert_eq!(
                palette_color(PROBE_PALETTE, key),
                *expected,
                "the entry picked for {key:?} changed"
            );
        }
    }

    #[test]
    fn test_palette_color_keeps_the_callers_palette() {
        let other = &["only"];
        assert_eq!(palette_color(other, "alice"), "only");
    }

    #[test]
    #[should_panic(expected = "palette_color needs a palette to pick from")]
    fn test_palette_color_rejects_an_empty_palette() {
        palette_color(&[], "alice");
    }

    #[test]
    fn test_palette_coverage() {
        // With enough tags, we should hit multiple palette entries
        let mut seen = std::collections::HashSet::new();
        for i in 0..100 {
            let slug = format!("tag-{}", i);
            seen.insert(auto_color(&slug));
        }
        // Should hit at least half the palette
        assert!(seen.len() >= 8, "Only hit {} palette entries", seen.len());
    }
}
