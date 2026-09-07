//! End-to-end coverage for the forgiving list arguments of the `kanban` binary.
//!
//! `--tags`, `--assignees`, `--depends_on` and `--attachments` all take a list
//! of refs, and a shell argument carries exactly one string. A caller who means
//! several refs therefore writes them as a JSON array inside that one value —
//! `--tags '["red","blue"]'` — which is the form every one of those flags
//! documents in its own help text.
//!
//! These tests launch the compiled binary against a board in a temp directory,
//! so they measure the whole path: the generated clap tree, the argument
//! extractor, the operation dispatcher, and the board on disk. A test at any one
//! of those layers cannot see a defect in the layer above it.
//!
//! The binary path comes from `env!("CARGO_BIN_EXE_kanban")`, which Cargo (and
//! nextest) populate for an integration test beside a `[[bin]]` target.

use serde_json::Value;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Absolute path to the compiled `kanban` binary, injected by Cargo.
const KANBAN_BIN: &str = env!("CARGO_BIN_EXE_kanban");

/// Run `kanban <args>` inside `dir` and return its stdout parsed from YAML.
///
/// Panics when the command fails. The CLI writes its error text to
/// `.kanban/mcp.<pid>.log` rather than to stderr, so the panic names the log
/// beside whatever the process did print.
fn run(dir: &Path, args: &[&str]) -> Value {
    let output = Command::new(KANBAN_BIN)
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to launch kanban binary");

    assert!(
        output.status.success(),
        "`kanban {}` exited with {}\nstdout: {}\nstderr: {}\n(the error text is in {}/.kanban/mcp.<pid>.log)",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        dir.display(),
    );

    let stdout = String::from_utf8(output.stdout).expect("kanban stdout is UTF-8");
    serde_yaml_ng::from_str(&stdout)
        .unwrap_or_else(|e| panic!("kanban stdout is not YAML: {e}\n{stdout}"))
}

/// Create a board in a fresh temp directory, ready for task commands.
fn board() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    run(dir.path(), &["board", "init", "--name", "Forgiving lists"]);
    dir
}

/// Add a task carrying `extra` flags, returning its id.
fn add_task(dir: &Path, extra: &[&str]) -> String {
    let mut args = vec!["task", "add", "--title", "Subject", "--description", "body"];
    args.extend_from_slice(extra);
    run(dir, &args)["id"]
        .as_str()
        .expect("`task add` reports the new task's id")
        .to_string()
}

/// Read one list-valued field of a task as sorted strings.
fn task_field(dir: &Path, id: &str, field: &str) -> Vec<String> {
    let task = run(dir, &["task", "get", "--id", id]);
    let mut values: Vec<String> = task[field]
        .as_array()
        .unwrap_or_else(|| panic!("task field `{field}` is not a list: {task}"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("task field `{field}` holds a non-string: {value}"))
                .to_string()
        })
        .collect();
    values.sort();
    values
}

/// Read the names of every tag entity the board holds, sorted.
fn board_tag_names(dir: &Path) -> Vec<String> {
    let listed = run(dir, &["tags", "list"]);
    let mut names: Vec<String> = listed["tags"]
        .as_array()
        .expect("`tags list` reports a tags array")
        .iter()
        .map(|tag| {
            tag["name"]
                .as_str()
                .expect("every tag carries a name")
                .to_string()
        })
        .collect();
    names.sort();
    names
}

/// `task add --tags '["red","blue"]'` applies two tags, not one joined name.
#[test]
fn add_task_applies_every_tag_of_a_stringified_array() {
    let dir = board();

    let id = add_task(dir.path(), &["--tags", r#"["red","blue"]"#]);

    assert_eq!(task_field(dir.path(), &id, "tags"), ["blue", "red"]);
}

/// The board mints one tag entity for each element, and no joined name.
///
/// A joined name is worse than a dropped one: it leaves debris on the board
/// that nobody asked for and that a later sweep has to find.
#[test]
fn add_task_mints_no_joined_tag_entity() {
    let dir = board();

    add_task(dir.path(), &["--tags", r#"["red","blue"]"#]);

    assert_eq!(board_tag_names(dir.path()), ["blue", "red"]);
}

/// `task update --tags` replaces the tag set with every element.
#[test]
fn update_task_replaces_with_every_tag_of_a_stringified_array() {
    let dir = board();
    let id = add_task(dir.path(), &["--tags", "stale"]);

    run(
        dir.path(),
        &[
            "task",
            "update",
            "--id",
            &id,
            "--tags",
            r#"["green","yellow"]"#,
        ],
    );

    assert_eq!(task_field(dir.path(), &id, "tags"), ["green", "yellow"]);
}

/// `task tag --tags` appends every element to the tags already there.
#[test]
fn tag_task_adds_every_tag_of_a_stringified_array() {
    let dir = board();
    let id = add_task(dir.path(), &["--tags", "alpha"]);

    run(
        dir.path(),
        &["task", "tag", "--id", &id, "--tags", r#"["beta","gamma"]"#],
    );

    assert_eq!(
        task_field(dir.path(), &id, "tags"),
        ["alpha", "beta", "gamma"]
    );
}

/// `task untag --tags` removes every element it names.
#[test]
fn untag_task_removes_every_tag_of_a_stringified_array() {
    let dir = board();
    let id = add_task(dir.path(), &["--tags", r#"["keep","drop-one","drop-two"]"#]);

    run(
        dir.path(),
        &[
            "task",
            "untag",
            "--id",
            &id,
            "--tags",
            r#"["drop-one","drop-two"]"#,
        ],
    );

    assert_eq!(task_field(dir.path(), &id, "tags"), ["keep"]);
}

/// `task add --assignees '["alice","bob"]'` assigns both actors.
#[test]
fn add_task_assigns_every_actor_of_a_stringified_array() {
    let dir = board();
    run(
        dir.path(),
        &["actor", "add", "--id", "alice", "--name", "Alice"],
    );
    run(
        dir.path(),
        &["actor", "add", "--id", "bob", "--name", "Bob"],
    );

    let id = add_task(dir.path(), &["--assignees", r#"["alice","bob"]"#]);

    assert_eq!(task_field(dir.path(), &id, "assignees"), ["alice", "bob"]);
}

/// `task update --depends_on '["<id>","<id>"]'` records both dependencies.
#[test]
fn update_task_depends_on_every_task_of_a_stringified_array() {
    let dir = board();
    let first = add_task(dir.path(), &[]);
    let second = add_task(dir.path(), &[]);
    let subject = add_task(dir.path(), &[]);

    let refs = format!(r#"["{first}","{second}"]"#);
    run(
        dir.path(),
        &["task", "update", "--id", &subject, "--depends_on", &refs],
    );

    let mut expected = [first, second];
    expected.sort();
    assert_eq!(task_field(dir.path(), &subject, "depends_on"), expected);
}

/// `task update --attachments '["a.txt","b.txt"]'` attaches both files.
///
/// `attachments` is typed as a plain string rather than a list, so it reaches
/// the dispatcher whole. This test holds that contract in place beside its
/// list-typed siblings.
#[test]
fn update_task_attaches_every_file_of_a_stringified_array() {
    let dir = board();
    std::fs::write(dir.path().join("first.txt"), "first").expect("write attachment");
    std::fs::write(dir.path().join("second.txt"), "second").expect("write attachment");
    let id = add_task(dir.path(), &[]);

    run(
        dir.path(),
        &[
            "task",
            "update",
            "--id",
            &id,
            "--attachments",
            r#"["first.txt","second.txt"]"#,
        ],
    );

    let task = run(dir.path(), &["task", "get", "--id", &id]);
    let mut names: Vec<&str> = task["attachments"]
        .as_array()
        .expect("the task carries an attachments array")
        .iter()
        .map(|attachment| {
            attachment["name"]
                .as_str()
                .expect("every attachment carries a name")
        })
        .collect();
    names.sort();
    assert_eq!(names, ["first.txt", "second.txt"]);
}
