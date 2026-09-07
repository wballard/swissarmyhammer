//! TagTask command — appends `#tag` to task description

use crate::context::KanbanContext;
use crate::error::KanbanError;
use crate::task::tags::{apply_tag_refs_to_task, TagApply};
use crate::types::TaskId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use swissarmyhammer_operations::{async_trait, operation, Execute, ExecutionResult};

/// Add tags to a task by appending a `#tag` marker for each to its
/// description.
///
/// Each entry of `tags` is a forgiving tag reference — a tag name/slug (e.g.
/// "bug"), a full tag ULID, `^<short>`, or a 7-char short id. See
/// [`crate::task::tags::apply_tag_refs`] for the resolution rules shared with
/// the `tags` parameter on `add task` / `update task`. If the Tag object
/// doesn't exist yet, it is auto-created with an auto-color; an id reference
/// that names no tag is an error.
///
/// One call applies one tag per entry. An empty list is an error, because
/// there is nothing to apply and an ack would report a write that never
/// happened.
#[operation(verb = "tag", noun = "task", description = "Add a tag to a task")]
#[derive(Debug, Deserialize, Serialize)]
pub struct TagTask {
    /// The task ID to tag
    pub id: TaskId,
    /// The tag references to add, one tag for each entry (e.g. `["bug"]`)
    pub tags: Vec<String>,
}

impl TagTask {
    /// Create a new TagTask command for the given task and one tag reference.
    pub fn new(id: impl Into<TaskId>, tag: impl Into<String>) -> Self {
        Self::with_tags(id, vec![tag.into()])
    }

    /// Create a TagTask that applies every reference in `tags` in one call.
    pub fn with_tags(id: impl Into<TaskId>, tags: Vec<String>) -> Self {
        Self {
            id: id.into(),
            tags,
        }
    }
}

#[async_trait]
impl Execute<KanbanContext, KanbanError> for TagTask {
    async fn execute(&self, ctx: &KanbanContext) -> ExecutionResult<Value, KanbanError> {
        // One shared path with `untag task` and with `add task`/`update task`:
        // resolve the refs, append a `#slug` to the body for each, mint the Tag
        // entity for a name that is new. Only the mode differs from
        // `untag task`.
        match apply_tag_refs_to_task(ctx, self.id.as_str(), &self.tags, TagApply::Append).await {
            Ok(value) => ExecutionResult::Success { value },
            Err(error) => ExecutionResult::Failed { error },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::InitBoard;
    use crate::task::{AddTask, GetTask};
    use crate::task_helpers::assert_task_mutation_ack;
    use serde_json::json;
    use tempfile::TempDir;

    async fn setup() -> (TempDir, KanbanContext) {
        let temp = TempDir::new().unwrap();
        let kanban_dir = temp.path().join(".kanban");
        let ctx = KanbanContext::new(kanban_dir);

        InitBoard::new("Test")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        (temp, ctx)
    }

    /// `tag task` returns exactly the thin ack; the tag's presence is
    /// asserted via `get task` (stored state, not response echo).
    #[tokio::test]
    async fn test_tag_task_returns_thin_ack() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Tag me")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let task_id = add_result["id"].as_str().unwrap();

        let result = TagTask::new(task_id, "bug")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        assert_task_mutation_ack(&result, task_id);

        let task = GetTask::new(task_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        assert!(
            task["tags"].as_array().unwrap().contains(&json!("bug")),
            "tag must be applied to the stored task, got: {}",
            task["tags"]
        );
    }

    /// A tag ref shaped like a ULID must name a real tag. Before the shared
    /// resolver landed this silently created a tag literally called
    /// `01KJZEPKJ35S76KF7E9HS5742J`.
    #[tokio::test]
    async fn test_tag_task_unknown_tag_ulid_errors() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Bad tag ref")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let task_id = add_result["id"].as_str().unwrap();

        let result = TagTask::new(task_id, "01KJZEPKJ35S76KF7E9HS5742J")
            .execute(&ctx)
            .await
            .into_result();

        assert!(result.is_err(), "an unresolvable tag ULID must error");
        let task = GetTask::new(task_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        assert!(
            task["tags"].as_array().unwrap().is_empty(),
            "the rejected tag must not land, got: {}",
            task["tags"]
        );
    }

    /// Tagging by an existing tag's ULID applies that tag's name.
    #[tokio::test]
    async fn test_tag_task_by_tag_ulid_applies_the_name() {
        let (_temp, ctx) = setup().await;

        let tag = crate::tag::AddTag::new("bug")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let tag_id = tag["id"].as_str().unwrap();

        let add_result = AddTask::new("Tag by id")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let task_id = add_result["id"].as_str().unwrap();

        TagTask::new(task_id, tag_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        let task = GetTask::new(task_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        assert_eq!(task["tags"], json!(["bug"]));
    }

    /// Re-tagging with the same tag is idempotent and still returns the ack.
    #[tokio::test]
    async fn test_tag_task_idempotent_returns_thin_ack() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Tag me twice")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let task_id = add_result["id"].as_str().unwrap();

        TagTask::new(task_id, "bug")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let result = TagTask::new(task_id, "bug")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        assert_task_mutation_ack(&result, task_id);

        let task = GetTask::new(task_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        assert_eq!(
            task["tags"].as_array().unwrap().len(),
            1,
            "duplicate tag must not be appended twice"
        );
    }
}
