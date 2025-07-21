use std::fs;
use std::sync::Arc;
use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use swissarmyhammer::mcp::tool_handlers::ToolHandlers;
use swissarmyhammer::mcp::types::AllCompleteRequest;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test the precision of path-based completion detection
/// This test verifies that only files directly in "complete" directory are marked as completed
#[tokio::test]
async fn test_precise_completion_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let _issues_dir = temp_dir.path().join("issues");

    // Create multiple issue storage instances to test different directory structures
    let scenarios = vec![
        ("standard_issues", "issues"),
        ("complete_suffix", "issues_complete"), // Should NOT be detected as completed
        ("complete_prefix", "complete_issues"), // Should NOT be detected as completed
        ("complete_middle", "issues_complete_old"), // Should NOT be detected as completed
    ];

    for (test_name, dir_name) in scenarios {
        let test_dir = temp_dir.path().join(test_name);
        let specific_issues_dir = test_dir.join(dir_name);

        // Create the test environment
        let issue_storage = Box::new(
            FileSystemIssueStorage::new(specific_issues_dir.clone())
                .expect("Failed to create issue storage"),
        );
        let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

        let git_ops = Arc::new(tokio::sync::Mutex::new(None));
        let tool_handlers = ToolHandlers::new(issue_storage.clone(), git_ops);

        // Create a standard active issue
        let _active_issue = issue_storage
            .write()
            .await
            .create_issue(
                format!("{test_name}_active"),
                "This is an active issue".to_string(),
            )
            .await
            .unwrap();

        // Create the complete directory
        let complete_dir = specific_issues_dir.join("complete");
        fs::create_dir_all(&complete_dir).unwrap();

        // Create a properly completed issue
        let completed_issue_file = complete_dir.join(format!("000099_{test_name}_completed.md"));
        fs::write(
            &completed_issue_file,
            format!("# Completed {test_name}\n\nThis is completed"),
        )
        .unwrap();

        // List issues and analyze completion detection
        let all_issues = issue_storage.read().await.list_issues().await.unwrap();

        println!("\n=== Test: {test_name} (directory: {dir_name}) ===");
        for issue in &all_issues {
            println!(
                "  Issue: name='{}', completed={}, path='{}'",
                issue.name,
                issue.completed,
                issue.file_path.display()
            );
        }

        // Count completion status
        let completed_count = all_issues.iter().filter(|i| i.completed).count();
        let active_count = all_issues.iter().filter(|i| !i.completed).count();

        println!("  Completed: {completed_count}, Active: {active_count}");

        // Verify correct detection regardless of directory name containing "complete"
        // The key insight: completion should be based on immediate parent directory name,
        // not whether "complete" appears anywhere in the path
        assert_eq!(
            completed_count, 1,
            "Should have exactly 1 completed issue for test {test_name}"
        );
        assert_eq!(
            active_count, 1,
            "Should have exactly 1 active issue for test {test_name}"
        );

        // Verify specific issues have correct completion status
        let active = all_issues
            .iter()
            .find(|i| i.name.as_str().contains("active"))
            .expect("Should find active issue");
        let completed = all_issues
            .iter()
            .find(|i| i.name.as_str().contains("completed"))
            .expect("Should find completed issue");

        assert!(
            !active.completed,
            "Active issue should not be completed for test {test_name}"
        );
        assert!(
            completed.completed,
            "Issue in 'complete' directory should be completed for test {test_name}"
        );

        // Test all_complete functionality
        let request = AllCompleteRequest {};
        let result = tool_handlers
            .handle_issue_all_complete(request)
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
            // Should show active issues (not all complete) for all scenarios
            assert!(
                text.text.contains("⏳ Project has active issues"),
                "Should show active issues for test {test_name}"
            );
            assert!(
                text.text.contains("Active: 1"),
                "Should show 1 active issue for test {test_name}"
            );
            assert!(
                text.text.contains("Completed: 1"),
                "Should show 1 completed issue for test {test_name}"
            );
        } else {
            panic!("Expected text response for test {test_name}");
        }

        println!("  ✅ Test {test_name} passed");
    }

    println!("\n🎉 All path completion precision tests passed!");
}

/// Test that demonstrates the fix for overly broad ancestor checking
#[tokio::test]
async fn test_ancestor_vs_parent_completion_detection() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");

    // This test demonstrates why checking ancestors vs immediate parent matters
    // Though in practice, the file scanning won't find deeply nested files anyway

    // Create a main issues directory
    let issues_dir = temp_dir.path().join("issues");
    let issue_storage = Box::new(
        FileSystemIssueStorage::new(issues_dir.clone()).expect("Failed to create issue storage"),
    );
    let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

    // Create proper active and completed issues
    let _active = issue_storage
        .write()
        .await
        .create_issue("main_active".to_string(), "Main active issue".to_string())
        .await
        .unwrap();

    let completed = issue_storage
        .write()
        .await
        .create_issue(
            "to_be_completed".to_string(),
            "Will be completed".to_string(),
        )
        .await
        .unwrap();

    // Complete the issue properly
    let _completed_issue = issue_storage
        .write()
        .await
        .mark_complete(&completed.name)
        .await
        .unwrap();

    // List issues and verify the fix works
    let all_issues = issue_storage.read().await.list_issues().await.unwrap();

    println!("\nAncestor vs Parent test results:");
    for issue in &all_issues {
        println!(
            "  Issue: name='{}', completed={}, path='{}'",
            issue.name,
            issue.completed,
            issue.file_path.display()
        );
    }

    let completed_count = all_issues.iter().filter(|i| i.completed).count();
    let active_count = all_issues.iter().filter(|i| !i.completed).count();

    // Should have 1 completed and 1 active
    assert_eq!(completed_count, 1, "Should have exactly 1 completed issue");
    assert_eq!(active_count, 1, "Should have exactly 1 active issue");

    // Verify completion detection logic
    for issue in &all_issues {
        if issue.name.as_str() == "to_be_completed" {
            assert!(
                issue.completed,
                "Properly completed issue should be marked completed"
            );
            // Verify it's in the complete directory
            assert!(
                issue.file_path.to_string_lossy().contains("/complete/"),
                "Completed issue should be in complete directory"
            );
        } else {
            assert!(!issue.completed, "Active issue should not be completed");
        }
    }

    println!("  ✅ Ancestor vs Parent completion detection test passed");
}
