use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use swissarmyhammer::mcp::tool_handlers::ToolHandlers;
use swissarmyhammer::mcp::types::AllCompleteRequest;
use swissarmyhammer::memoranda::{FileSystemMemoStorage, MemoStorage};
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test helper to create a test environment with potential edge cases
struct EdgeCaseTestEnvironment {
    temp_dir: TempDir,
    issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    tool_handlers: ToolHandlers,
}

impl EdgeCaseTestEnvironment {
    async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let issues_dir = temp_dir.path().join("issues");

        let issue_storage = Box::new(
            FileSystemIssueStorage::new(issues_dir).expect("Failed to create issue storage"),
        );
        let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

        let git_ops = Arc::new(tokio::sync::Mutex::new(None::<swissarmyhammer::git::GitOperations>));
        let memo_storage = Box::new(FileSystemMemoStorage::new_default().expect("Failed to create memo storage"));
        let memo_storage = Arc::new(RwLock::new(memo_storage as Box<dyn MemoStorage>));
        let tool_handlers = ToolHandlers::new(issue_storage.clone(), git_ops, memo_storage);

        Self {
            temp_dir,
            issue_storage,
            tool_handlers,
        }
    }
}

#[tokio::test]
async fn test_filesystem_permission_issues() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create an active issue
    let _issue = env
        .issue_storage
        .write()
        .await
        .create_issue("test_permission".to_string(), "Test content".to_string())
        .await
        .unwrap();

    // Make the issues directory read-only to simulate permission problems
    let issues_dir = env.temp_dir.path().join("issues");
    let mut perms = fs::metadata(&issues_dir).unwrap().permissions();
    perms.set_mode(0o444); // Read-only
    fs::set_permissions(&issues_dir, perms).unwrap();

    // The all_complete check should still work with read permissions
    // but might behave differently if there are write operations involved
    let request = AllCompleteRequest {};
    let result = env.tool_handlers.handle_issue_all_complete(request).await;

    // Restore permissions for cleanup
    let mut perms = fs::metadata(&issues_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&issues_dir, perms).unwrap();

    // The result should still be successful, reporting the active issue
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_corrupted_issue_files() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create a valid issue first
    let issue = env
        .issue_storage
        .write()
        .await
        .create_issue("valid_issue".to_string(), "Valid content".to_string())
        .await
        .unwrap();

    // Create a corrupted file in the issues directory that might confuse the parser
    let issues_dir = env.temp_dir.path().join("issues");
    let corrupted_file = issues_dir.join("000999_corrupted.md");
    let mut file = File::create(&corrupted_file).unwrap();
    file.write_all(b"\xFF\xFE\x00\x00corrupted binary data\x00\x00")
        .unwrap();

    // Create a file with any .md name (now valid according to new requirement)
    let non_numbered_filename = issues_dir.join("invalid_format.md");
    let mut file = File::create(&non_numbered_filename).unwrap();
    file.write_all(b"Valid markdown content in non-numbered file")
        .unwrap();

    // Check all complete - should handle corrupted files gracefully
    let request = AllCompleteRequest {};
    let result = env
        .tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    // Should report both valid issues as active (original numbered + new non-numbered)
    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        // Should have 2 active issues: the original numbered one + the non-numbered one
        assert!(text.text.contains("Active: 2"));
        // Should contain the original numbered issue
        assert!(text.text.contains(&issue.name.to_string()));
        // Should contain the non-numbered issue (with auto-assigned virtual number)
        assert!(text.text.contains("invalid_format"));
    } else {
        panic!("Expected text response");
    }
}

#[tokio::test]
async fn test_empty_directories() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create empty complete directory (this might cause issues if not handled properly)
    let complete_dir = env.temp_dir.path().join("issues").join("complete");
    fs::create_dir_all(&complete_dir).unwrap();

    // Check all complete with only empty directories
    let request = AllCompleteRequest {};
    let result = env
        .tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    // Should report no issues found
    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        assert!(text.text.contains("📋 No issues found in the project"));
    } else {
        panic!("Expected text response");
    }
}

#[tokio::test]
async fn test_concurrent_file_operations() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create an initial issue
    let _issue = env
        .issue_storage
        .write()
        .await
        .create_issue("concurrent_test".to_string(), "Test content".to_string())
        .await
        .unwrap();

    // Simulate concurrent operations: one thread checking all_complete while another modifies files
    let storage_clone = env.issue_storage.clone();
    let handlers_clone = Arc::new(env.tool_handlers);

    let check_task = {
        let handlers = handlers_clone.clone();
        tokio::spawn(async move {
            for _i in 0..10 {
                let request = AllCompleteRequest {};
                let _result = handlers.handle_issue_all_complete(request).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        })
    };

    let modify_task = {
        tokio::spawn(async move {
            for i in 0..5 {
                let _create_result = storage_clone
                    .write()
                    .await
                    .create_issue(format!("concurrent_issue_{i}"), "Content".to_string())
                    .await;
                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
            }
        })
    };

    // Wait for both tasks to complete
    let (_check_result, _modify_result) = tokio::try_join!(check_task, modify_task).unwrap();

    // Final check should show multiple active issues
    let request = AllCompleteRequest {};
    let result = handlers_clone
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        // Should show multiple active issues
        assert!(text.text.contains("⏳ Project has active issues"));
        assert!(text.text.contains("Active:"));
    }
}

#[tokio::test]
async fn test_symlink_handling() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create a valid issue
    let issue = env
        .issue_storage
        .write()
        .await
        .create_issue("symlink_test".to_string(), "Test content".to_string())
        .await
        .unwrap();

    let issues_dir = env.temp_dir.path().join("issues");
    let complete_dir = issues_dir.join("complete");
    fs::create_dir_all(&complete_dir).unwrap();

    // Create a symlink pointing to the active issue from the complete directory
    // This could potentially confuse the completion detection logic
    let original_file = issues_dir.join(format!("{}.md", issue.name));
    let symlink_file = complete_dir.join(format!("{}_symlink.md", issue.name));

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&original_file, &symlink_file).unwrap();

        // Check all complete - should handle symlinks correctly
        let request = AllCompleteRequest {};
        let result = env
            .tool_handlers
            .handle_issue_all_complete(request)
            .await
            .unwrap();

        // Should still report the original issue as active (not completed via symlink)
        assert!(!result.is_error.unwrap_or(false));
        if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
            assert!(text.text.contains("⏳ Project has active issues"));
            assert!(text.text.contains("Active: 1"));
        }
    }
}

#[tokio::test]
async fn test_directory_structure_edge_cases() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create nested "complete" directories to test path-based completion detection
    let issues_dir = env.temp_dir.path().join("issues");
    let nested_complete = issues_dir.join("some").join("complete").join("nested");
    fs::create_dir_all(&nested_complete).unwrap();

    // Create an issue file in the nested complete directory
    let nested_issue_file = nested_complete.join("000123_nested_issue.md");
    fs::write(
        &nested_issue_file,
        "# Nested Issue\n\nThis is in a nested complete directory",
    )
    .unwrap();

    // Also create a regular active issue
    let _active_issue = env
        .issue_storage
        .write()
        .await
        .create_issue(
            "regular_active".to_string(),
            "Regular active issue".to_string(),
        )
        .await
        .unwrap();

    // Check all complete
    let request = AllCompleteRequest {};
    let result = env
        .tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        // Should show that not all issues are complete (there's one active issue)
        assert!(text.text.contains("⏳ Project has active issues"));
        // The nested complete file might or might not be detected depending on implementation
        // but there should definitely be at least one active issue
        assert!(text.text.contains("regular_active"));
    }
}

#[tokio::test]
async fn test_cache_invalidation_bug() {
    let env = EdgeCaseTestEnvironment::new().await;

    // Create and complete an issue
    let issue = env
        .issue_storage
        .write()
        .await
        .create_issue("cache_test".to_string(), "Test caching".to_string())
        .await
        .unwrap();

    // Check that it shows as active
    let request = AllCompleteRequest {};
    let result1 = env
        .tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    // Complete the issue
    let _completed = env
        .issue_storage
        .write()
        .await
        .mark_complete(&issue.name)
        .await
        .unwrap();

    // Immediately check again - if there's a caching bug, this might show the old state
    let request = AllCompleteRequest {};
    let result2 = env
        .tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    // Result2 should show all issues complete, not the cached "active issues" result
    assert!(!result1.is_error.unwrap_or(false));
    assert!(!result2.is_error.unwrap_or(false));

    if let rmcp::model::RawContent::Text(text1) = &result1.content[0].raw {
        assert!(text1.text.contains("⏳ Project has active issues"));
    }

    if let rmcp::model::RawContent::Text(text2) = &result2.content[0].raw {
        // This should now show all complete, not cached active state
        assert!(text2.text.contains("🎉 All issues are complete!"));
    }
}
