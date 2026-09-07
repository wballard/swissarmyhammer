//! A card whose stored front matter holds a `---` run must survive a write.
//!
//! Kanban stores every comment in the front matter of the card's `.md` file.
//! Agents put markdown tables in comments, and a table separator row is
//! `|---|---|`, which holds the bare run `---`. A reader that looks for that
//! run as a SUBSTRING stops inside the table instead of at the closing
//! delimiter line. The card then loses its title and part of its body.
//!
//! These tests drive the production write path -- the same commands the MCP
//! ops call -- and read the card back from a fresh context, so they cover the
//! whole read-write round trip and not only one parser.

use serde_json::json;
use swissarmyhammer_kanban::{
    board::InitBoard, comment::AddComment, entity::UpdateEntityField, task::AddTask, KanbanContext,
    KanbanOperationProcessor, OperationProcessor,
};
use tempfile::TempDir;

/// The title of the reduced card. It must survive the round trip verbatim.
const CARD_TITLE: &str = "swiftlint declines a file it cannot read";

/// A comment whose text holds a markdown table. The separator row carries the
/// bare `---` run that truncates a substring-based front matter split.
const TABLE_COMMENT: &str = "\
RAW swiftlint, judged file beside the refusing path:

| the refusing path | status | stdout | stderr |
|---|---|---|---|
| a path that holds no file | 2 | 1 entry | 0 bytes |

The status is 2, and NOT the 0 the card carried over.";

/// A description whose text holds a `---` horizontal rule on its own line.
const RULE_DESCRIPTION: &str = "\
Before the rule.

---

After the rule.
";

/// Build a temp board and the processor that drives it.
async fn setup() -> (TempDir, KanbanContext, KanbanOperationProcessor) {
    let temp = TempDir::new().unwrap();
    let kanban_dir = temp.path().join(".kanban");
    let ctx = KanbanContext::new(&kanban_dir);
    let processor = KanbanOperationProcessor::new();

    processor
        .process(&InitBoard::new("Test Board"), &ctx)
        .await
        .unwrap();

    (temp, ctx, processor)
}

#[tokio::test]
async fn a_comment_holding_a_markdown_table_survives_a_task_write() {
    let (temp, ctx, processor) = setup().await;

    let added = processor
        .process(
            &AddTask::new(CARD_TITLE).with_description("The card description.\n"),
            &ctx,
        )
        .await
        .unwrap();
    let task_id = added["id"].as_str().unwrap().to_string();

    processor
        .process(&AddComment::new(task_id.as_str(), TABLE_COMMENT), &ctx)
        .await
        .unwrap();

    // Any field write rewrites the whole card, so it replays the read that
    // truncates and then persists the truncation.
    processor
        .process(
            &UpdateEntityField::new("task", &task_id, "position_column", json!("doing")),
            &ctx,
        )
        .await
        .unwrap();

    let kanban_dir = temp.path().join(".kanban");
    let reopened = KanbanContext::new(&kanban_dir);
    let ectx = reopened.entity_context().await.unwrap();
    let task = ectx.read("task", &task_id).await.unwrap();

    assert_eq!(
        task.get_str("title"),
        Some(CARD_TITLE),
        "the title is written after the comment; a substring split loses it"
    );
    assert_eq!(
        task.get_str("body"),
        Some("The card description.\n"),
        "the body must not start inside the comment's table row"
    );

    let comments = task.get("comments").expect("the comment must survive");
    let text = comments[0]["text"].as_str().expect("comment text");
    assert_eq!(
        text, TABLE_COMMENT,
        "the comment text must survive verbatim"
    );
}

#[tokio::test]
async fn a_description_holding_a_horizontal_rule_survives_a_task_write() {
    let (temp, ctx, processor) = setup().await;

    let added = processor
        .process(
            &AddTask::new(CARD_TITLE).with_description(RULE_DESCRIPTION),
            &ctx,
        )
        .await
        .unwrap();
    let task_id = added["id"].as_str().unwrap().to_string();

    processor
        .process(
            &UpdateEntityField::new("task", &task_id, "position_column", json!("doing")),
            &ctx,
        )
        .await
        .unwrap();

    let kanban_dir = temp.path().join(".kanban");
    let reopened = KanbanContext::new(&kanban_dir);
    let ectx = reopened.entity_context().await.unwrap();
    let task = ectx.read("task", &task_id).await.unwrap();

    assert_eq!(task.get_str("title"), Some(CARD_TITLE));
    assert_eq!(
        task.get_str("body"),
        Some(RULE_DESCRIPTION),
        "a horizontal rule in the body must survive the round trip"
    );
}
