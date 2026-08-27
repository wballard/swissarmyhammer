//! The task operations and their optional parameters.
//!
//! These tests hold the task CRUD, movement, assignment and query operations.
//! They also hold the optional parameters of add task, update task, move task
//! and next task, and each filter that list tasks accepts.

use super::*;

// ------------------------------------------------------------------
// Task operations (additional)
// ------------------------------------------------------------------

#[tokio::test]
async fn dispatch_get_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Get me"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "get task", "id": task_id})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Get me");
    assert_eq!(result["id"].as_str().unwrap(), task_id);
}

#[tokio::test]
async fn dispatch_update_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Original title"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "update task", "id": task_id, "title": "Updated title", "description": "New desc"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    // The update response is the thin ack — the effect is asserted via
    // `get task`, the agreed escape hatch.
    crate::task_helpers::assert_task_mutation_ack(&result, &task_id);

    let task = get_task(&ctx, &task_id).await;
    assert_eq!(task["title"], "Updated title");
    assert_eq!(task["description"], "New desc");
}

#[tokio::test]
async fn dispatch_delete_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Delete me"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "delete task", "id": task_id})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["deleted"], true);
}

#[tokio::test]
async fn dispatch_complete_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Complete me"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "complete task", "id": task_id})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(get_task(&ctx, &task_id).await["position"]["column"], "done");
}

#[tokio::test]
async fn dispatch_move_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Move me"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();
    assert_eq!(r["position"]["column"], "todo");

    let ops = parse_input(json!({"op": "move task", "id": task_id, "column": "doing"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(
        get_task(&ctx, &task_id).await["position"]["column"],
        "doing"
    );
}

#[tokio::test]
async fn dispatch_assign_and_unassign_task() {
    let (_temp, ctx) = setup().await;

    // Create actor and task
    let ops =
        parse_input(json!({"op": "add actor", "id": "frank", "name": "Frank", "type": "human"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Assign me"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    // Assign — thin ack; the effect is asserted via `get task`
    let ops =
        parse_input(json!({"op": "assign task", "id": task_id, "assignee": "frank"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(result["id"], task_id);
    let assignees = get_task(&ctx, &task_id).await["assignees"]
        .as_array()
        .unwrap()
        .clone();
    assert!(
        assignees.iter().any(|a| a == "frank"),
        "frank should be assigned"
    );

    // Unassign — thin ack; the effect is asserted via `get task`
    let ops =
        parse_input(json!({"op": "unassign task", "id": task_id, "assignee": "frank"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["ok"], true);
    assert!(
        get_task(&ctx, &task_id).await["assignees"]
            .as_array()
            .unwrap()
            .is_empty(),
        "frank should be unassigned"
    );
}

#[tokio::test]
async fn dispatch_next_task() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Next one"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "next task"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Next one");
}

#[tokio::test]
async fn dispatch_tag_and_untag_task() {
    let (_temp, ctx) = setup().await;

    // Add task
    let ops = parse_input(json!({"op": "add task", "title": "Tagged task"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    // Tag the task — TagTask auto-creates the tag and returns the thin
    // ack; the effect is asserted via `get task`
    let ops = parse_input(json!({"op": "tag task", "id": task_id, "tag": "feature"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["ok"], true);
    assert_eq!(result["id"], task_id);
    assert!(get_task(&ctx, &task_id).await["tags"]
        .as_array()
        .unwrap()
        .contains(&json!("feature")));

    // Untag — thin ack; the effect is asserted via `get task`
    let ops = parse_input(json!({"op": "untag task", "id": task_id, "tag": "feature"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["ok"], true);
    assert!(get_task(&ctx, &task_id).await["tags"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn dispatch_list_tasks_with_filters() {
    let (_temp, ctx) = setup().await;

    // Add tasks in different columns
    let ops = parse_input(json!({"op": "add task", "title": "Todo task"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops =
        parse_input(json!({"op": "add task", "title": "Doing task", "column": "doing"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // Filter by column
    let ops = parse_input(json!({"op": "list tasks", "column": "doing"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["count"], 1);
    assert_eq!(result["tasks"][0]["title"], "Doing task");
}

// ------------------------------------------------------------------
// Activity operations
// ------------------------------------------------------------------

// ------------------------------------------------------------------
// Dispatch: add task with optional fields
// ------------------------------------------------------------------

#[tokio::test]
async fn dispatch_add_task_with_description() {
    let (_temp, ctx) = setup().await;

    let ops =
        parse_input(json!({"op": "add task", "title": "Described", "description": "Some detail"}))
            .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Described");
    // The add response is slim (no description echo) — assert the stored
    // description via `get task`.
    let task_id = result["id"].as_str().unwrap();
    assert_eq!(get_task(&ctx, task_id).await["description"], "Some detail");
}

#[tokio::test]
async fn dispatch_add_task_with_ordinal() {
    // Caller-supplied ordinals must be well-formed FractionalIndex
    // encodings — legacy strings like "a5" are rejected at the
    // validation boundary rather than silently stored.
    let (_temp, ctx) = setup().await;

    let ordinal = Ordinal::DEFAULT_STR;
    let ops =
        parse_input(json!({"op": "add task", "title": "Ordered", "ordinal": ordinal})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Ordered");
    assert_eq!(result["position"]["ordinal"], ordinal);
}

#[tokio::test]
async fn dispatch_add_task_with_assignees_array() {
    let (_temp, ctx) = setup().await;

    // Add an actor
    let ops =
        parse_input(json!({"op": "add actor", "id": "alice", "name": "Alice", "type": "human"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Assigned", "assignees": ["alice"]}))
        .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Assigned");
    let assignees = result["assignees"].as_array().unwrap();
    assert!(assignees.iter().any(|a| a == "alice"));
}

#[tokio::test]
async fn dispatch_add_task_with_single_assignee() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add actor", "id": "bob", "name": "Bob", "type": "human"}))
        .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Single Assignee", "assignee": "bob"}))
        .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Single Assignee");
    let assignees = result["assignees"].as_array().unwrap();
    assert!(assignees.iter().any(|a| a == "bob"));
}

#[tokio::test]
async fn dispatch_add_task_with_actor_auto_assigns() {
    let (_temp, ctx) = setup().await;

    // Add actor first
    let ops =
        parse_input(json!({"op": "add actor", "id": "agent", "name": "Agent", "type": "agent"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // Provide actor in the operation itself (not in assignees)
    let mut op = crate::types::Operation::new(crate::types::Verb::Add, crate::types::Noun::Task, {
        let mut m = serde_json::Map::new();
        m.insert("title".into(), json!("Auto-assigned"));
        m
    });
    op.actor = Some("agent".into());
    let result = execute_operation(&ctx, &op).await.unwrap();
    let assignees = result["assignees"].as_array().unwrap();
    assert!(
        assignees.iter().any(|a| a == "agent"),
        "actor should be auto-assigned when no explicit assignees"
    );
}

#[tokio::test]
async fn dispatch_add_task_with_depends_on() {
    let (_temp, ctx) = setup().await;

    // Add first task
    let ops = parse_input(json!({"op": "add task", "title": "Dep target"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let dep_id = r["id"].as_str().unwrap().to_string();

    // Add task depending on first
    let ops = parse_input(json!({"op": "add task", "title": "Dependent", "depends_on": [dep_id]}))
        .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    let deps = result["depends_on"].as_array().unwrap();
    assert!(deps.iter().any(|d| d.as_str() == Some(&dep_id)));
}

// ------------------------------------------------------------------
// Dispatch: update task with optional fields
// ------------------------------------------------------------------

#[tokio::test]
async fn dispatch_update_task_with_assignees() {
    let (_temp, ctx) = setup().await;

    let ops =
        parse_input(json!({"op": "add actor", "id": "zara", "name": "Zara", "type": "human"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Reassign"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "update task", "id": task_id, "assignees": ["zara"]})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let task = get_task(&ctx, &task_id).await;
    let assignees = task["assignees"].as_array().unwrap();
    assert!(assignees.iter().any(|a| a == "zara"));
}

#[tokio::test]
async fn dispatch_update_task_with_depends_on() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Target dep"})).unwrap();
    let r1 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let dep_id = r1["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "add task", "title": "Updatable"})).unwrap();
    let r2 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r2["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "update task", "id": task_id, "depends_on": [dep_id]})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let task = get_task(&ctx, &task_id).await;
    let deps = task["depends_on"].as_array().unwrap();
    assert!(deps.iter().any(|d| d.as_str() == Some(&dep_id)));
}

// ------------------------------------------------------------------
// Dispatch: move task with optional fields
// ------------------------------------------------------------------

#[tokio::test]
async fn dispatch_move_task_with_ordinal() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Ordinal move"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "move task", "id": task_id, "column": "doing", "ordinal": "z9"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let task = get_task(&ctx, &task_id).await;
    assert_eq!(task["position"]["column"], "doing");
    // Ordinal is passed through to MoveTask
    assert!(task["position"]["ordinal"].as_str().is_some());
}

#[tokio::test]
async fn dispatch_move_task_with_before_id() {
    let (_temp, ctx) = setup().await;

    // Add two tasks in doing column
    let ops = parse_input(json!({"op": "add task", "title": "First", "column": "doing"})).unwrap();
    let r1 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id1 = r1["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "add task", "title": "Second", "column": "doing"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // Add a task in todo, then move before id1
    let ops = parse_input(json!({"op": "add task", "title": "Mover"})).unwrap();
    let r3 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id3 = r3["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "move task", "id": id3, "column": "doing", "before_id": id1}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(get_task(&ctx, &id3).await["position"]["column"], "doing");
}

#[tokio::test]
async fn dispatch_move_task_with_after_id() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Anchor", "column": "doing"})).unwrap();
    let r1 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id1 = r1["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "add task", "title": "After mover"})).unwrap();
    let r2 = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id2 = r2["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "move task", "id": id2, "column": "doing", "after_id": id1}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(get_task(&ctx, &id2).await["position"]["column"], "doing");
}

// ------------------------------------------------------------------
// Dispatch: next task with filters
// ------------------------------------------------------------------

#[tokio::test]
async fn dispatch_next_task_with_tag_filter() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Untagged"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Tagged task"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "tag task", "id": task_id, "tag": "priority"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "next task", "filter": "#priority"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Tagged task");
}

#[tokio::test]
async fn dispatch_next_task_with_assignee_filter() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add actor", "id": "dev", "name": "Dev", "type": "human"}))
        .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Assigned next"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({"op": "assign task", "id": task_id, "assignee": "dev"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "next task", "filter": "@dev"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["title"], "Assigned next");
}

// ------------------------------------------------------------------
// Dispatch: list tasks with all filter types
// ------------------------------------------------------------------

/// Board size for the `tag` reproducer. It is larger than the default page
/// size of 10, so a `tag` the listing drops shows up as a full first page
/// instead of as a plausible small answer.
const TAG_FILTER_BOARD_SIZE: usize = 20;

/// How many of the [`TAG_FILTER_BOARD_SIZE`] tasks carry the filtered tag.
const TAG_FILTER_MATCHES: usize = 3;

/// Add `TAG_FILTER_BOARD_SIZE` tasks and put `#bug` on the first
/// `TAG_FILTER_MATCHES` of them. Returns the ids of the tagged tasks.
async fn seed_partly_tagged_board(ctx: &KanbanContext) -> Vec<String> {
    let mut tagged = Vec::new();
    for i in 0..TAG_FILTER_BOARD_SIZE {
        let title = format!("Task {i:02}");
        let ops = parse_input(json!({"op": "add task", "title": title})).unwrap();
        let r = execute_operation(ctx, &ops[0]).await.unwrap();
        if i < TAG_FILTER_MATCHES {
            let id = r["id"].as_str().unwrap().to_string();
            let ops = parse_input(json!({"op": "tag task", "id": id, "tag": "bug"})).unwrap();
            execute_operation(ctx, &ops[0]).await.unwrap();
            tagged.push(id);
        }
    }
    tagged
}

/// Read the `id` of every task in a `list tasks` result, in order.
fn listed_ids(result: &serde_json::Value) -> Vec<String> {
    result["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect()
}

/// `{"op": "list tasks", "tag": "bug"}` must return ONLY the tagged tasks.
/// Before the fix the `tag` param was accepted and dropped, so the call
/// answered a narrow question with a page of the whole board.
#[tokio::test]
async fn dispatch_list_tasks_with_tag_filter() {
    let (_temp, ctx) = setup().await;
    let tagged = seed_partly_tagged_board(&ctx).await;

    let ops = parse_input(json!({"op": "list tasks", "tag": "bug"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["total"], TAG_FILTER_MATCHES,
        "tag must scope the listing to the tagged tasks, not the whole board"
    );
    assert_eq!(result["count"], TAG_FILTER_MATCHES);
    let mut listed = listed_ids(&result);
    listed.sort();
    let mut expected = tagged;
    expected.sort();
    assert_eq!(listed, expected);
}

/// The card's acceptance: `tag: X` and `filter: "#X"` answer identically.
#[tokio::test]
async fn dispatch_list_tasks_tag_param_matches_filter_atom() {
    let (_temp, ctx) = setup().await;
    seed_partly_tagged_board(&ctx).await;

    let ops = parse_input(json!({"op": "list tasks", "tag": "bug"})).unwrap();
    let by_param = execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "filter": "#bug"})).unwrap();
    let by_filter = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(listed_ids(&by_param), listed_ids(&by_filter));
    assert_eq!(by_param["total"], by_filter["total"]);
}

/// `tag` + an explicit `filter` apply both (AND semantics).
#[tokio::test]
async fn dispatch_list_tasks_tag_param_intersects_with_filter() {
    let (_temp, ctx) = setup().await;

    // Carries both tags — the only match.
    let ops = parse_input(json!({"op": "add task", "title": "Urgent bug"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let both_id = r["id"].as_str().unwrap().to_string();
    for tag in ["bug", "urgent"] {
        let ops = parse_input(json!({"op": "tag task", "id": both_id, "tag": tag})).unwrap();
        execute_operation(&ctx, &ops[0]).await.unwrap();
    }

    // Carries only the `tag` param's tag — excluded by the filter.
    let ops = parse_input(json!({"op": "add task", "title": "Plain bug"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let bug_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "tag task", "id": bug_id, "tag": "bug"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // Carries only the filter's tag — excluded by the `tag` param.
    let ops = parse_input(json!({"op": "add task", "title": "Plain urgent"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let urgent_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "tag task", "id": urgent_id, "tag": "urgent"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "tag": "bug", "filter": "#urgent"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["total"], 1,
        "tag + filter must intersect, matching only the doubly tagged task"
    );
    assert_eq!(result["tasks"][0]["title"], "Urgent bug");
}

/// A `tag` naming no tag yields an empty listing, not the whole board.
#[tokio::test]
async fn dispatch_list_tasks_unknown_tag_returns_empty() {
    let (_temp, ctx) = setup().await;
    seed_partly_tagged_board(&ctx).await;

    let ops = parse_input(json!({"op": "list tasks", "tag": "nonexistent"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["total"], 0,
        "an unknown tag must yield an empty list, not the whole board"
    );
}

/// A `tag` carrying several values cannot become one filter atom. It is an
/// explicit error naming `filter`, never a silently unfiltered listing.
#[tokio::test]
async fn dispatch_list_tasks_multi_valued_tag_errors() {
    let (_temp, ctx) = setup().await;
    seed_partly_tagged_board(&ctx).await;

    let ops = parse_input(json!({"op": "list tasks", "tag": ["bug", "urgent"]})).unwrap();
    let err = execute_operation(&ctx, &ops[0]).await.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("tag") && msg.contains("filter"),
        "the error must name the param and point at `filter`: {msg}"
    );
}

/// A one-element array is unambiguous and resolves to that one value, the
/// same shape tolerance every other ref-style param carries.
#[tokio::test]
async fn dispatch_list_tasks_single_element_tag_array_filters() {
    let (_temp, ctx) = setup().await;
    seed_partly_tagged_board(&ctx).await;

    let ops = parse_input(json!({"op": "list tasks", "tag": ["bug"]})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(result["total"], TAG_FILTER_MATCHES);
}

/// `{"op": "list tasks", "assignee": "worker"}` must return ONLY that
/// actor's tasks. The board holds a second, unassigned task so the
/// assertion can fail when the param is dropped.
#[tokio::test]
async fn dispatch_list_tasks_with_assignee_filter() {
    let (_temp, ctx) = setup().await;

    let ops =
        parse_input(json!({"op": "add actor", "id": "worker", "name": "Worker", "type": "human"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Worker task"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let task_id = r["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "assign task", "id": task_id, "assignee": "worker"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "add task", "title": "Nobody's task"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "assignee": "worker"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["total"], 1,
        "assignee must scope the listing to the assigned task"
    );
    assert_eq!(result["tasks"][0]["title"], "Worker task");
}

/// `exclude_done: false` widens the listing to the done column, which is
/// otherwise dropped when no `column` is named.
#[tokio::test]
async fn dispatch_list_tasks_exclude_done_false_includes_done() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Open"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let ops = parse_input(json!({"op": "add task", "title": "Finished"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let done_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "move task", "id": done_id, "column": "done"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "exclude_done": false})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(
        result["total"], 2,
        "exclude_done: false must keep the done task in the listing"
    );

    let ops = parse_input(json!({"op": "list tasks", "exclude_done": true})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(result["total"], 1);
    assert_eq!(result["tasks"][0]["title"], "Open");
}

/// MCP transports that carry every argument as a string must not lose
/// `exclude_done` to a silent type mismatch.
#[tokio::test]
async fn dispatch_list_tasks_exclude_done_accepts_string_boolean() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Open"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let ops = parse_input(json!({"op": "add task", "title": "Finished"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let done_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "move task", "id": done_id, "column": "done"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // A stringified boolean carries whatever case its source language
    // writes, so the spelling is matched case-insensitively.
    for spelling in ["false", "False", "FALSE"] {
        let ops = parse_input(json!({"op": "list tasks", "exclude_done": spelling})).unwrap();
        let result = execute_operation(&ctx, &ops[0]).await.unwrap();
        assert_eq!(result["total"], 2, "exclude_done: {spelling:?}");
    }
}

/// A non-boolean `exclude_done` is an explicit error, never a listing that
/// quietly kept the default.
#[tokio::test]
async fn dispatch_list_tasks_non_boolean_exclude_done_errors() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Open"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "exclude_done": "yes please"})).unwrap();
    let err = execute_operation(&ctx, &ops[0]).await.unwrap_err();
    assert!(
        err.to_string().contains("exclude_done"),
        "the error must name the param: {err}"
    );
}

#[tokio::test]
async fn dispatch_list_tasks_with_ready_filter() {
    let (_temp, ctx) = setup().await;

    // Add a task with a dependency (not ready)
    let ops = parse_input(json!({"op": "add task", "title": "Blocker"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let blocker_id = r["id"].as_str().unwrap().to_string();

    let ops =
        parse_input(json!({"op": "add task", "title": "Blocked", "depends_on": [blocker_id]}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // List only ready tasks
    let ops = parse_input(json!({"op": "list tasks", "filter": "#READY"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();
    // Only the blocker should be ready
    let titles: Vec<&str> = result["tasks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["title"].as_str())
        .collect();
    assert!(titles.contains(&"Blocker"), "Blocker should be ready");
    assert!(
        !titles.contains(&"Blocked"),
        "Blocked task should not be ready"
    );
}

// ------------------------------------------------------------------
// Dispatch: list tasks `project` param folds into the `$<project>`
// filter. Regression for the silent-ignore bug where `project` was
// dropped and the whole board was returned.
// ------------------------------------------------------------------

/// `{"op": "list tasks", "project": "<id>"}` must return ONLY that
/// project's tasks. Before the fix the `project` param was silently
/// ignored and the whole board (both tasks) came back.
#[tokio::test]
async fn dispatch_list_tasks_project_param_scopes_to_project() {
    let (_temp, ctx) = setup().await;

    let ops =
        parse_input(json!({"op": "add project", "id": "myproj", "name": "My Project"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops =
        parse_input(json!({"op": "add task", "title": "In project", "project": "myproj"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    let ops = parse_input(json!({"op": "add task", "title": "Out of project"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "project": "myproj"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["count"], 1,
        "project param must scope the listing to the in-project task only"
    );
    assert_eq!(result["tasks"][0]["title"], "In project");
}

/// `project` + an explicit `filter` apply both (AND semantics): only a
/// task that is BOTH in the project AND carries the tag is returned.
#[tokio::test]
async fn dispatch_list_tasks_project_param_intersects_with_filter() {
    let (_temp, ctx) = setup().await;

    let ops =
        parse_input(json!({"op": "add project", "id": "myproj", "name": "My Project"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // In project AND tagged #bug — the only match.
    let ops =
        parse_input(json!({"op": "add task", "title": "Bug in project", "project": "myproj"}))
            .unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let bug_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "tag task", "id": bug_id, "tag": "bug"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // In project but NOT tagged — excluded by the filter.
    let ops =
        parse_input(json!({"op": "add task", "title": "Plain in project", "project": "myproj"}))
            .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    // Tagged #bug but NOT in project — excluded by the project.
    let ops = parse_input(json!({"op": "add task", "title": "Bug outside"})).unwrap();
    let r = execute_operation(&ctx, &ops[0]).await.unwrap();
    let outside_id = r["id"].as_str().unwrap().to_string();
    let ops = parse_input(json!({"op": "tag task", "id": outside_id, "tag": "bug"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops =
        parse_input(json!({"op": "list tasks", "project": "myproj", "filter": "#bug"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["count"], 1,
        "project + filter must intersect (AND), matching only the in-project tagged task"
    );
    assert_eq!(result["tasks"][0]["title"], "Bug in project");
}

/// A `project` value naming no existing project yields an empty listing
/// (normal `$` filter semantics), not the whole board.
#[tokio::test]
async fn dispatch_list_tasks_unknown_project_returns_empty() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Some task"})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({"op": "list tasks", "project": "nonexistent"})).unwrap();
    let result = execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        result["count"], 0,
        "an unknown project must yield an empty list, not the whole board"
    );
}
