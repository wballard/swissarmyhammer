//! Enforces that the `review` skill tells the model to review only the
//! repository that contains the current working directory.
//!
//! The rule is written inline in `builtin/skills/review/SKILL.md`. It has one
//! reader — the `review` skill — so it is not a partial. The `reviewer` agent
//! gets it by loading that skill, which is why the agent must NOT repeat the
//! text; `assert_guidance_single_source` fails if it does.
//!
//! Failing this test means the scope rule was dropped or duplicated, so an agent
//! can again review a path, a clone, or a pull request outside this repository.

use std::path::Path;

mod common;
use common::{assert_guidance_single_source, rendered_builtin_instructions};

/// Sentences that carry the rule. Each is pinned to one `builtin/` file.
const CANONICAL_SCOPE: &[&str] = &[
    "Review only the repository that contains the current working directory.",
    "Get the repository root with `git rev-parse --show-toplevel`.",
    "the target is outside this repository, and stop.",
];

#[test]
fn review_skill_renders_repository_scope() {
    let body = rendered_builtin_instructions("review");
    for sentence in CANONICAL_SCOPE {
        assert!(
            body.contains(sentence),
            "builtin skill 'review' must state the scope rule: {sentence}"
        );
    }
}

#[test]
fn repository_scope_has_single_source_of_truth() {
    for sentence in CANONICAL_SCOPE {
        assert_guidance_single_source(sentence, Path::new("skills/review/SKILL.md"));
    }
}
