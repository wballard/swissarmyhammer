//! UntagTask command — removes `#tag` from task description

use crate::context::KanbanContext;
use crate::error::KanbanError;
use crate::task::tags::{apply_tag_refs_to_task, TagApply};
use crate::types::TaskId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use swissarmyhammer_operations::{async_trait, operation, Execute, ExecutionResult};

/// Remove tags from a task by removing their `#tag` markers from its
/// description.
///
/// Each entry of `tags` is a forgiving tag reference resolved exactly as `tag
/// task` resolves it — a tag name/slug (e.g. "bug"), a full tag ULID,
/// `^<short>`, or a 7-char short id. Removing a tag the task does not carry is
/// a no-op; an id reference that names no tag is an error.
///
/// One call removes one tag per entry. An empty list is an error, because
/// there is nothing to remove and an ack would report a write that never
/// happened.
#[operation(
    verb = "untag",
    noun = "task",
    description = "Remove a tag from a task"
)]
#[derive(Debug, Deserialize, Serialize)]
pub struct UntagTask {
    /// The task ID to untag
    pub id: TaskId,
    /// The tag references to remove, one tag for each entry
    pub tags: Vec<String>,
}

impl UntagTask {
    /// Create a new UntagTask command for the given task and one tag
    /// reference.
    pub fn new(id: impl Into<TaskId>, tag: impl Into<String>) -> Self {
        Self::with_tags(id, vec![tag.into()])
    }

    /// Create an UntagTask that removes every reference in `tags` in one call.
    pub fn with_tags(id: impl Into<TaskId>, tags: Vec<String>) -> Self {
        Self {
            id: id.into(),
            tags,
        }
    }
}

#[async_trait]
impl Execute<KanbanContext, KanbanError> for UntagTask {
    async fn execute(&self, ctx: &KanbanContext) -> ExecutionResult<Value, KanbanError> {
        // Same shared path as `tag task`, inverse mode: strip each `#slug` from
        // the body instead of appending it.
        match apply_tag_refs_to_task(ctx, self.id.as_str(), &self.tags, TagApply::Remove).await {
            Ok(value) => ExecutionResult::Success { value },
            Err(error) => ExecutionResult::Failed { error },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::InitBoard;
    use crate::task::{AddTask, GetTask, TagTask};
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

    /// `untag task` returns exactly the thin ack; the tag's removal is
    /// asserted via `get task` (stored state, not response echo).
    #[tokio::test]
    async fn test_untag_task_returns_thin_ack() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Untag me")
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

        let result = UntagTask::new(task_id, "bug")
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
            !task["tags"].as_array().unwrap().contains(&json!("bug")),
            "tag must be removed from the stored task, got: {}",
            task["tags"]
        );
    }

    /// Untagging by the tag's ULID resolves to that tag's name and removes it —
    /// the same resolver `tag task` uses.
    #[tokio::test]
    async fn test_untag_task_by_tag_ulid_removes_the_named_tag() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Untag by id")
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

        let tag_id = crate::tag::ListTags::default()
            .execute(&ctx)
            .await
            .into_result()
            .unwrap()["tags"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == json!("bug"))
            .expect("the tag entity was auto-created")["id"]
            .as_str()
            .unwrap()
            .to_string();

        UntagTask::new(task_id, &tag_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        let task = GetTask::new(task_id)
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        assert!(
            task["tags"].as_array().unwrap().is_empty(),
            "untag by ULID must remove the resolved tag, got: {}",
            task["tags"]
        );
    }

    /// Untagging a tag that isn't present is idempotent and still acks.
    #[tokio::test]
    async fn test_untag_task_absent_tag_returns_thin_ack() {
        let (_temp, ctx) = setup().await;

        let add_result = AddTask::new("Nothing to untag")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();
        let task_id = add_result["id"].as_str().unwrap();

        let result = UntagTask::new(task_id, "ghost")
            .execute(&ctx)
            .await
            .into_result()
            .unwrap();

        assert_task_mutation_ack(&result, task_id);
    }
}
