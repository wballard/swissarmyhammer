use std::fs;
use std::sync::Arc;
use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use swissarmyhammer::mcp::tool_handlers::ToolHandlers;
use swissarmyhammer::mcp::types::AllCompleteRequest;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test specifically for the path-based completion detection bug
#[tokio::test]
async fn test_nested_complete_directory_bug() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let issues_dir = temp_dir.path().join("issues");

    let issue_storage = Box::new(
        FileSystemIssueStorage::new(issues_dir.clone()).expect("Failed to create issue storage"),
    );
    let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

    let git_ops = Arc::new(tokio::sync::Mutex::new(None));
    let tool_handlers = ToolHandlers::new(issue_storage.clone(), git_ops);

    // Create the standard directory structure
    let complete_dir = issues_dir.join("complete");
    fs::create_dir_all(&complete_dir).unwrap();

    // Create a legitimate completed issue
    let completed_issue_file = complete_dir.join("000001_legitimate_completed.md");
    fs::write(
        &completed_issue_file,
        "# Completed Issue\n\nThis issue is completed",
    )
    .unwrap();

    // Create a nested directory structure that has "complete" in the path but should NOT be marked as completed
    let nested_complete_dir = issues_dir.join("archive").join("complete").join("old");
    fs::create_dir_all(&nested_complete_dir).unwrap();

    // Create an issue file in this nested structure that should NOT be marked as completed
    let nested_issue_file = nested_complete_dir.join("000002_nested_not_completed.md");
    fs::write(&nested_issue_file, "# Nested Issue\n\nThis issue is in a nested directory with 'complete' in path but should NOT be completed").unwrap();

    // Create a regular active issue
    let _active_issue = issue_storage
        .write()
        .await
        .create_issue(
            "regular_active".to_string(),
            "This is a regular active issue".to_string(),
        )
        .await
        .unwrap();

    // List all issues and check their completion status
    let all_issues = issue_storage.read().await.list_issues().await.unwrap();

    // Verify the completion status is correct
    for issue in &all_issues {
        let issue_num: u32 = issue.number.into();
        match issue_num {
            1 => {
                // The legitimate completed issue should be marked as completed
                assert!(
                    issue.completed,
                    "Issue 000001 should be completed (in complete directory)"
                );
                assert_eq!(issue.name, "legitimate_completed");
            }
            2 => {
                // The nested issue should NOT be completed despite having "complete" in its path
                assert!(
                    !issue.completed,
                    "Issue 000002 should NOT be completed (nested directory)"
                );
                assert_eq!(issue.name, "nested_not_completed");
            }
            _ => {
                // The regular active issue should not be completed
                assert!(
                    !issue.completed,
                    "Regular active issue should not be completed"
                );
                assert_eq!(issue.name, "regular_active");
            }
        }
    }

    // Check all complete functionality
    let request = AllCompleteRequest {};
    let result = tool_handlers
        .handle_issue_all_complete(request)
        .await
        .unwrap();

    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        // Should show active issues (the nested file and regular active issue)
        assert!(text.text.contains("⏳ Project has active issues"));
        assert!(text.text.contains("Active: 2")); // nested issue + regular active
        assert!(text.text.contains("Completed: 1")); // only the legitimate completed issue

        // Verify specific issues are listed correctly
        assert!(text.text.contains("nested_not_completed"));
        assert!(text.text.contains("regular_active"));
        assert!(text.text.contains("legitimate_completed"));
    } else {
        panic!("Expected text response");
    }
}

/// Test the old buggy behavior vs new correct behavior
#[tokio::test]
async fn test_path_completion_detection_precision() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let issues_dir = temp_dir.path().join("issues");

    let issue_storage = Box::new(
        FileSystemIssueStorage::new(issues_dir.clone()).expect("Failed to create issue storage"),
    );
    let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

    // Create various directory structures to test edge cases
    let scenarios = vec![
        ("complete/000001_direct_complete.md", true), // Should be completed
        ("complete/sub/000002_sub_complete.md", false), // Should NOT be completed (sub-directory)
        ("other/complete/000003_nested_complete.md", false), // Should NOT be completed (nested)
        ("complete_backup/000004_backup.md", false),  // Should NOT be completed (different name)
        ("archive/old_complete/000005_archive.md", false), // Should NOT be completed (nested)
    ];

    for (path, _expected_completed) in scenarios {
        let full_path = issues_dir.join(path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(&full_path, format!("# Issue\n\nTest issue in {}", path)).unwrap();
    }

    // List all issues and verify completion detection
    let all_issues = issue_storage.read().await.list_issues().await.unwrap();

    for issue in &all_issues {
        let issue_num: u32 = issue.number.into();
        let expected = match issue_num {
            1 => true,  // direct_complete should be completed
            _ => false, // all others should NOT be completed
        };

        assert_eq!(
            issue.completed, expected,
            "Issue {} ({}) completion status should be {} but was {}",
            issue_num, issue.name, expected, issue.completed
        );
    }

    // Verify counts
    let completed_count = all_issues.iter().filter(|i| i.completed).count();
    let active_count = all_issues.iter().filter(|i| !i.completed).count();

    assert_eq!(completed_count, 1, "Should have exactly 1 completed issue");
    assert_eq!(active_count, 4, "Should have exactly 4 active issues");
}
