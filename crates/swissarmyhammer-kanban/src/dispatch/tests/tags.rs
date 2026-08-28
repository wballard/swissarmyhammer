//! The `tags` parameter on add task, on update task, on tag task and on
//! untag task.
//!
//! These tests hold each shape that `tags` accepts, the tag entities that
//! dispatch creates, the errors for a reference that it cannot resolve, and
//! the markers that it writes into the description.

use super::*;

// -----------------------------------------------------------------------
// `tags` on add task / update task
// -----------------------------------------------------------------------

#[tokio::test]
async fn dispatch_add_task_tags_array_applies() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Tagged at birth",
        "tags": ["bug", "kanban"],
    }))
    .unwrap();
    let created = execute_operation(&ctx, &ops[0]).await.unwrap();

    let id = created["id"].as_str().unwrap();
    assert_eq!(stored_tags(&ctx, id).await, vec!["bug", "kanban"]);
}

#[tokio::test]
async fn dispatch_update_task_tags_array_replaces_the_set() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Retag me",
        "description": "body carries #stale",
    }))
    .unwrap();
    let created = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(stored_tags(&ctx, &id).await, vec!["stale"]);

    let ops = parse_input(json!({
        "op": "update task",
        "id": id,
        "tags": ["bug", "init", "mirdan"],
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        stored_tags(&ctx, &id).await,
        vec!["bug", "init", "mirdan"],
        "`tags` on update replaces the whole set"
    );
}

/// The equivalence contract: one `add task {tags:[a,b,c]}` and one
/// `add task` followed by three `tag task` calls must land on the same
/// stored tag set. This is what makes the plural form a real alias for
/// the singular op instead of a second, drifting implementation.
#[tokio::test]
async fn dispatch_add_task_tags_equivalent_to_three_tag_task_calls() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Plural",
        "tags": ["bug", "init", "mirdan"],
    }))
    .unwrap();
    let plural = execute_operation(&ctx, &ops[0]).await.unwrap();
    let plural_id = plural["id"].as_str().unwrap().to_string();

    let singular_id = add_one_task(&ctx, "Singular").await;
    for tag in ["bug", "init", "mirdan"] {
        let ops = parse_input(json!({"op": "tag task", "id": singular_id, "tag": tag})).unwrap();
        execute_operation(&ctx, &ops[0]).await.unwrap();
    }

    assert_eq!(
        stored_tags(&ctx, &plural_id).await,
        stored_tags(&ctx, &singular_id).await,
        "a tags array must equal one `tag task` per tag"
    );
}

#[tokio::test]
async fn dispatch_add_task_tags_single_string_applies() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({"op": "add task", "title": "Scalar tag", "tags": "bug"})).unwrap();
    let created = execute_operation(&ctx, &ops[0]).await.unwrap();

    let id = created["id"].as_str().unwrap();
    assert_eq!(stored_tags(&ctx, id).await, vec!["bug"]);
}

#[tokio::test]
async fn dispatch_update_task_tags_stringified_array_applies() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Stringified").await;

    let ops = parse_input(json!({
        "op": "update task",
        "id": id,
        "tags": "[\"bug\",\"kanban\"]",
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(stored_tags(&ctx, &id).await, vec!["bug", "kanban"]);
}

/// The singular `tag` is an alias for the same list and takes every shape
/// `tags` takes — it names the key, it does not narrow the shape. The
/// stringified array is the shape a client with no array type-hint sends, so
/// drive it under both keys and require the same stored set. `tag task`
/// already holds this shape; `update task` reaches it through the same
/// `tag_refs` helper, and measuring it here keeps the documented claim
/// resting on a fixture rather than on reading the code.
#[tokio::test]
async fn dispatch_update_task_singular_and_plural_tag_keys_agree_on_a_stringified_array() {
    let (_temp, ctx) = setup().await;
    let stringified = serde_json::to_string(&["bug", "kanban"]).unwrap();

    let singular = add_one_task(&ctx, "Singular key").await;
    let ops =
        parse_input(json!({"op": "update task", "id": singular, "tag": &stringified})).unwrap();
    execute_operation(&ctx, &ops[0])
        .await
        .expect("the singular key must read a stringified array");

    let plural = add_one_task(&ctx, "Plural key").await;
    let ops =
        parse_input(json!({"op": "update task", "id": plural, "tags": &stringified})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        stored_tags(&ctx, &singular).await,
        vec!["bug", "kanban"],
        "a stringified array under `tag` is one tag per element"
    );
    assert_eq!(
        stored_tags(&ctx, &singular).await,
        stored_tags(&ctx, &plural).await,
        "the singular key must not narrow the shapes the plural key takes"
    );
}

/// A tag ref given as the tag entity's full ULID resolves to that tag's
/// name — the exact form that was silently dropped.
#[tokio::test]
async fn dispatch_add_task_tags_full_ulid_resolves_to_tag_name() {
    let (_temp, ctx) = setup().await;
    let bug_id = add_one_tag(&ctx, "bug").await;
    let kanban_id = add_one_tag(&ctx, "kanban").await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "By ulid",
        "tags": [bug_id, kanban_id],
    }))
    .unwrap();
    let created = execute_operation(&ctx, &ops[0]).await.unwrap();

    let id = created["id"].as_str().unwrap();
    assert_eq!(stored_tags(&ctx, id).await, vec!["bug", "kanban"]);
}

/// Short id and `^<short>` both resolve, mirroring every other id-taking
/// param on the board.
#[tokio::test]
async fn dispatch_update_task_tags_short_id_and_caret_resolve() {
    let (_temp, ctx) = setup().await;
    let bug_id = add_one_tag(&ctx, "bug").await;
    let kanban_id = add_one_tag(&ctx, "kanban").await;
    let id = add_one_task(&ctx, "By short id").await;

    let ops = parse_input(json!({
        "op": "update task",
        "id": id,
        "tags": [
            crate::types::short_id(&bug_id),
            format!("^{}", crate::types::short_id(&kanban_id)),
        ],
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(stored_tags(&ctx, &id).await, vec!["bug", "kanban"]);
}

/// An unresolvable tag id ref is an error and creates nothing — the same
/// rule `depends_on` already states.
#[tokio::test]
async fn dispatch_add_task_tags_unresolvable_ulid_errors_and_creates_nothing() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Doomed",
        "tags": ["01KJZEPKJ35S76KF7E9HS5742J"],
    }))
    .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await;

    assert!(
        result.is_err(),
        "an unresolvable tag ref must error, not silently drop"
    );

    let ops = parse_input(json!({"op": "list tasks"})).unwrap();
    let listed = execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(
        listed["tasks"].as_array().unwrap().len(),
        0,
        "the failed add must not leave a task behind"
    );
}

#[tokio::test]
async fn dispatch_update_task_tags_unresolvable_ulid_errors_without_changing_tags() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Keep my tags",
        "tags": ["keep"],
    }))
    .unwrap();
    let created = execute_operation(&ctx, &ops[0]).await.unwrap();
    let id = created["id"].as_str().unwrap().to_string();

    let ops = parse_input(json!({
        "op": "update task",
        "id": id,
        "tags": ["01KJZEPKJ35S76KF7E9HS5742J"],
    }))
    .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await;

    assert!(result.is_err(), "an unresolvable tag ref must error");
    assert_eq!(
        stored_tags(&ctx, &id).await,
        vec!["keep"],
        "a rejected update must leave the tag set untouched"
    );
}

#[tokio::test]
async fn dispatch_update_task_tags_empty_array_clears_the_set() {
    let (_temp, ctx) = setup().await;

    // Seed through the singular op so the pre-state holds regardless of
    // whether the plural form works — the clear is what's under test.
    let id = add_one_task(&ctx, "Clear me").await;
    for tag in ["bug", "kanban"] {
        let ops = parse_input(json!({"op": "tag task", "id": id, "tag": tag})).unwrap();
        execute_operation(&ctx, &ops[0]).await.unwrap();
    }
    assert_eq!(stored_tags(&ctx, &id).await, vec!["bug", "kanban"]);

    let ops = parse_input(json!({"op": "update task", "id": id, "tags": []})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert!(
        stored_tags(&ctx, &id).await.is_empty(),
        "an explicit empty tags array replaces the set with nothing"
    );
}

/// A malformed `tags` value (neither string nor array) errors instead of
/// being dropped — on update a silent drop would look like "no change".
#[tokio::test]
async fn dispatch_update_task_tags_malformed_scalar_errors() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Malformed tags").await;

    let ops = parse_input(json!({"op": "update task", "id": id, "tags": 42})).unwrap();
    assert!(
        execute_operation(&ctx, &ops[0]).await.is_err(),
        "a non-string, non-array tags value must error"
    );
}

/// Auto-created tag entities must exist after a plural apply, exactly as
/// `tag task` guarantees — otherwise `list tags` and the UI disagree with
/// the task's own tag list.
#[tokio::test]
async fn dispatch_add_task_tags_auto_creates_tag_entities() {
    let (_temp, ctx) = setup().await;

    let ops = parse_input(json!({
        "op": "add task",
        "title": "Auto create",
        "tags": ["brand-new"],
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let names = board_tag_names(&ctx).await;
    assert!(
        names.iter().any(|name| name == "brand-new"),
        "plural tags must auto-create the Tag entity, got: {names:?}"
    );
}

// -----------------------------------------------------------------------
// `tag` / `tags` on tag task / untag task
// -----------------------------------------------------------------------

/// Punctuation that a tag name must never hold. A name carrying one of
/// these is a fragment of prose or of code that a joined write left behind,
/// never a tag a caller asked for.
const FRAGMENT_PUNCTUATION: [char; 8] = [',', '"', '\'', '[', ']', '(', ')', ':'];

/// The two refs of the bug report, applied in one call.
const TWO_REFS: [&str; 2] = ["tool-validators", "objectivity"];

/// The stored tag set the two refs must produce — sorted, because
/// `parse_tags` reports a sorted set.
const TWO_TAGS: [&str; 2] = ["objectivity", "tool-validators"];

/// The defect: one `tag task` carrying two refs wrote a single tag named
/// `tool-validators-objectivity`. The param was read as a scalar, so the
/// slug normalizer collapsed the array's punctuation into hyphens. Two refs
/// are two tags.
#[tokio::test]
async fn dispatch_tag_task_array_applies_one_tag_per_element() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Two tags in one call").await;

    let ops = parse_input(json!({"op": "tag task", "id": id, "tag": TWO_REFS})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(
        stored_tags(&ctx, &id).await,
        TWO_TAGS,
        "an array of refs is one tag per element, never one joined name"
    );
}

/// `tags` is the canonical key and `tag` is its alias, taking every shape the
/// plural key takes, so `tag task` must answer to both.
#[tokio::test]
async fn dispatch_tag_task_accepts_the_plural_tags_key() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Plural key").await;

    let ops = parse_input(json!({"op": "tag task", "id": id, "tags": TWO_REFS})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(stored_tags(&ctx, &id).await, TWO_TAGS);
}

/// The exact wire shape from the bug report: a client with no array
/// type-hint sends the array as one string. It still splits.
#[tokio::test]
async fn dispatch_tag_task_stringified_array_applies_one_tag_per_element() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Stringified refs").await;

    let ops = parse_input(json!({
        "op": "tag task",
        "id": id,
        "tag": serde_json::to_string(&TWO_REFS).unwrap(),
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert_eq!(stored_tags(&ctx, &id).await, TWO_TAGS);
}

/// `untag task` is the inverse of `tag task` and takes the same shapes, so
/// one call removes both tags a single `tag task` applied.
#[tokio::test]
async fn dispatch_untag_task_array_removes_one_tag_per_element() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Untag both").await;

    let ops = parse_input(json!({"op": "tag task", "id": id, "tag": TWO_REFS})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();
    assert_eq!(stored_tags(&ctx, &id).await, TWO_TAGS);

    let ops = parse_input(json!({"op": "untag task", "id": id, "tags": TWO_REFS})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    assert!(
        stored_tags(&ctx, &id).await.is_empty(),
        "an array of refs removes one tag per element"
    );
}

/// One unresolvable ref rejects the whole call. A partial apply, or a tag
/// named after the failed ULID, are both silent writes the caller never
/// asked for.
#[tokio::test]
async fn dispatch_tag_task_unresolvable_ulid_errors_and_applies_nothing() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "One bad ref").await;

    let ops = parse_input(json!({
        "op": "tag task",
        "id": id,
        "tag": ["bug", "01KJZEPKJ35S76KF7E9HS5742J"],
    }))
    .unwrap();
    let result = execute_operation(&ctx, &ops[0]).await;

    assert!(result.is_err(), "an unresolvable tag ref must error");
    assert!(
        stored_tags(&ctx, &id).await.is_empty(),
        "a rejected call must apply none of its refs"
    );
}

/// An empty list has no tag to apply, so an `ok` would report a write that
/// never happened. `tag task` and `untag task` both refuse it.
#[tokio::test]
async fn dispatch_tag_and_untag_task_empty_list_errors() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Nothing to apply").await;

    for op in ["tag task", "untag task"] {
        let ops = parse_input(json!({"op": op, "id": id, "tags": []})).unwrap();
        assert!(
            execute_operation(&ctx, &ops[0]).await.is_err(),
            "{op} with an empty list must error instead of acking a write it never made"
        );
    }
}

/// The board invariant behind the wreckage the bug report lists: a tag name
/// holds no comma, quotation mark, bracket, or colon. Every ref shape the
/// tagging ops accept is driven through, including the shapes that used to
/// be joined into one name.
#[tokio::test]
async fn dispatch_tagging_writes_no_tag_name_holding_punctuation() {
    let (_temp, ctx) = setup().await;
    let id = add_one_task(&ctx, "Hostile refs").await;

    let ops = parse_input(json!({"op": "tag task", "id": id, "tag": TWO_REFS})).unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({
        "op": "tag task",
        "id": id,
        "tags": serde_json::to_string(&TWO_REFS).unwrap(),
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let ops = parse_input(json!({
        "op": "update task",
        "id": id,
        "tags": ["Coverage: gap", "[serial(cwd)]);", "he said \"go\""],
    }))
    .unwrap();
    execute_operation(&ctx, &ops[0]).await.unwrap();

    let names = board_tag_names(&ctx).await;
    assert!(!names.is_empty(), "the drive must have created tags");
    for name in &names {
        assert!(
            !name.contains(FRAGMENT_PUNCTUATION),
            "tag name {name:?} holds fragment punctuation, board holds: {names:?}"
        );
    }
}
