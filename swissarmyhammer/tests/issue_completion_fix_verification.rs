use std::fs;
use std::sync::Arc;
use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use swissarmyhammer::mcp::tool_handlers::ToolHandlers;
use swissarmyhammer::mcp::types::AllCompleteRequest;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Test that verifies the completion detection works correctly
#[tokio::test]
async fn test_completion_detection_fix() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let issues_dir = temp_dir.path().join("issues");
    
    let issue_storage = Box::new(
        FileSystemIssueStorage::new(issues_dir.clone()).expect("Failed to create issue storage"),
    );
    let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));
    
    let git_ops = Arc::new(tokio::sync::Mutex::new(None));
    let tool_handlers = ToolHandlers::new(issue_storage.clone(), git_ops);

    // Create some regular active issues
    let issue1 = issue_storage
        .write()
        .await
        .create_issue("active_issue_1".to_string(), "This is an active issue".to_string())
        .await
        .unwrap();
    
    let _issue2 = issue_storage
        .write()
        .await
        .create_issue("active_issue_2".to_string(), "This is another active issue".to_string())
        .await
        .unwrap();
    
    // Complete one of them
    let _completed_issue = issue_storage
        .write()
        .await
        .mark_complete(issue1.number.into())
        .await
        .unwrap();
    
    // Now create a file in a nested directory with "complete" in the path
    // This should NOT be marked as completed with our fix
    let nested_complete_dir = issues_dir.join("archive").join("complete").join("old");
    fs::create_dir_all(&nested_complete_dir).unwrap();
    
    // Manually create a file that follows the issue naming convention
    let nested_issue_file = nested_complete_dir.join("000099_nested_issue.md");
    fs::write(&nested_issue_file, "# Nested Issue\n\nThis should NOT be completed despite being in a path with 'complete'").unwrap();
    
    // List all issues and verify their completion status
    let all_issues = issue_storage.read().await.list_issues().await.unwrap();
    
    println!("Found {} issues:", all_issues.len());
    for issue in &all_issues {
        let issue_num: u32 = issue.number.into();
        println!(
            "  Issue {}: name='{}', completed={}, path='{}'",
            issue_num, issue.name, issue.completed, issue.file_path.display()
        );
    }
    
    // Count completion status
    let completed_count = all_issues.iter().filter(|i| i.completed).count();
    let active_count = all_issues.iter().filter(|i| !i.completed).count();
    
    println!("Completed: {}, Active: {}", completed_count, active_count);
    
    // We should have:
    // - 1 completed issue (issue1 that we explicitly completed)  
    // - 2 active issues (issue2 + the nested issue that should NOT be marked completed)
    assert_eq!(completed_count, 1, "Should have exactly 1 completed issue");
    assert_eq!(active_count, 2, "Should have exactly 2 active issues");
    
    // Verify the nested issue is not marked as completed
    let nested_issue = all_issues.iter()
        .find(|i| i.name == "nested_issue")
        .expect("Should find the nested issue");
    assert!(!nested_issue.completed, "Nested issue should NOT be completed despite 'complete' in path");
    
    // Check all complete functionality
    let request = AllCompleteRequest {};
    let result = tool_handlers.handle_issue_all_complete(request).await.unwrap();
    
    assert!(!result.is_error.unwrap_or(false));
    if let rmcp::model::RawContent::Text(text) = &result.content[0].raw {
        // Should show active issues (not all complete)
        assert!(text.text.contains("⏳ Project has active issues"));
        assert!(text.text.contains("Active: 2"));
        assert!(text.text.contains("Completed: 1"));
    } else {
        panic!("Expected text response");
    }
}

/// Test the exact scenario that would have caused the bug
#[tokio::test]
async fn test_path_ancestor_bug_fix() {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let issues_dir = temp_dir.path().join("issues");
    
    let issue_storage = Box::new(
        FileSystemIssueStorage::new(issues_dir.clone()).expect("Failed to create issue storage"),
    );
    let issue_storage = Arc::new(RwLock::new(issue_storage as Box<dyn IssueStorage>));

    // Create the standard complete directory
    let complete_dir = issues_dir.join("complete");
    fs::create_dir_all(&complete_dir).unwrap();
    
    // Create a legitimately completed issue
    let legitimate_complete = complete_dir.join("000001_legitimate.md");
    fs::write(&legitimate_complete, "# Legitimate Complete\n\nThis should be completed").unwrap();
    
    // Create deeply nested directories with "complete" in the path
    let deep_nested = issues_dir.join("project").join("complete").join("archive").join("backup");
    fs::create_dir_all(&deep_nested).unwrap();
    
    // This file should NOT be marked as completed despite "complete" being an ancestor
    let deep_nested_issue = deep_nested.join("000002_deep_nested.md");
    fs::write(&deep_nested_issue, "# Deep Nested\n\nThis should NOT be completed").unwrap();
    
    // List issues and check completion detection
    let all_issues = issue_storage.read().await.list_issues().await.unwrap();
    
    println!("Path completion test - Found {} issues:", all_issues.len());
    for issue in &all_issues {
        let issue_num: u32 = issue.number.into();
        println!(
            "  Issue {}: name='{}', completed={}, path='{}'",
            issue_num, issue.name, issue.completed, issue.file_path.display()
        );
    }
    
    // Find issues by name and verify completion status
    let legitimate = all_issues.iter().find(|i| i.name == "legitimate").expect("Should find legitimate issue");
    let deep_nested = all_issues.iter().find(|i| i.name == "deep_nested").expect("Should find deep nested issue");
    
    // The issue directly in "complete" should be completed
    assert!(legitimate.completed, "Issue directly in 'complete' directory should be completed");
    
    // The issue in nested path should NOT be completed (this is the bug fix)
    assert!(!deep_nested.completed, "Issue in nested path with 'complete' ancestor should NOT be completed");
    
    println!("✅ Path completion detection fix verified!");
}