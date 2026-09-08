//! Enforces that the builtin `sah-help` skill teaches a person what
//! SwissArmyHammer makes available, and that it finds the inventory at run
//! time instead of from a fixed list.
//!
//! The skill must send the agent to the live discovery ops. A fixed list of
//! skills, agents, or validators in the skill body goes stale the next time a
//! builtin is added. The discovery ops always show the current set, and they
//! include the user's own skills, agents, and validators.
//!
//! Failing this test means the skill body drifted away from live discovery,
//! or dropped one of the areas a person must learn about.

mod common;
use common::rendered_builtin_instructions;

/// Lists every skill, builtin and user-added, with its description.
const LIST_SKILLS_OP: &str = "list skill";

/// Finds a skill by keyword when the person asks about one topic.
const SEARCH_SKILLS_OP: &str = "search skill";

/// Lists every subagent the main agent can delegate to.
const LIST_AGENTS_OP: &str = "list agent";

/// Lists every review validator and the files it applies to.
const LIST_VALIDATORS_OP: &str = "list validators";

/// The CLI command that lists the MCP tools with their descriptions.
const CLI_TOOL_LIST: &str = "sah tool list";

/// The CLI command that shows the top-level command set.
const CLI_HELP: &str = "sah --help";

/// The lifecycle skills a new person must learn first.
const LIFECYCLE_SKILLS: &[&str] = &[
    "/plan",
    "/implement",
    "/test",
    "/review",
    "/commit",
    "/finish",
];

/// The directories a person edits to add their own skills, agents, and
/// validators.
const CUSTOMIZE_DIRS: &[&str] = &[".skills/", ".agents/", ".validators/"];

#[test]
fn sah_help_discovers_the_inventory_at_run_time() {
    let body = rendered_builtin_instructions("sah-help");

    for op in [
        LIST_SKILLS_OP,
        SEARCH_SKILLS_OP,
        LIST_AGENTS_OP,
        LIST_VALIDATORS_OP,
    ] {
        assert!(
            body.contains(op),
            "builtin skill 'sah-help' must use the `{op}` op so the inventory is live, not a fixed list"
        );
    }
}

#[test]
fn sah_help_names_the_cli_entry_points() {
    let body = rendered_builtin_instructions("sah-help");

    for cmd in [CLI_TOOL_LIST, CLI_HELP] {
        assert!(
            body.contains(cmd),
            "builtin skill 'sah-help' must name `{cmd}` so a person can explore from the terminal"
        );
    }
}

#[test]
fn sah_help_teaches_the_lifecycle_skills() {
    let body = rendered_builtin_instructions("sah-help");

    for skill in LIFECYCLE_SKILLS {
        assert!(
            body.contains(skill),
            "builtin skill 'sah-help' must name `{skill}` in the lifecycle tour"
        );
    }
}

#[test]
fn sah_help_shows_where_to_add_your_own() {
    let body = rendered_builtin_instructions("sah-help");

    for dir in CUSTOMIZE_DIRS {
        assert!(
            body.contains(dir),
            "builtin skill 'sah-help' must name `{dir}` so a person knows where to add their own"
        );
    }
}

#[test]
fn sah_help_takes_an_optional_topic() {
    let body = rendered_builtin_instructions("sah-help");

    assert!(
        body.contains("$ARGUMENTS"),
        "builtin skill 'sah-help' must accept an optional topic through $ARGUMENTS"
    );
}
