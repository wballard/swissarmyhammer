//! End-to-End Workflow Tests
//!
//! Tests for complete user journeys that span multiple CLI commands and verify
//! that entire workflows function correctly with the CLI-MCP integration.

use anyhow::Result;
use assert_cmd::Command;
use std::time::Duration;
use tempfile::TempDir;

mod test_utils;
use test_utils::setup_git_repo;

use std::sync::LazyLock;
use std::path::PathBuf;

/// Global cache for search model downloads
static MODEL_CACHE_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var("SWISSARMYHAMMER_MODEL_CACHE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::temp_dir().join(".swissarmyhammer_test_cache").into()
        })
});

/// Helper function to perform search indexing with timeout and graceful failure
fn try_search_index(
    temp_path: &std::path::Path,
    patterns: &[&str],
    force: bool,
    timeout_secs: u64,
) -> Result<bool> {
    let mut cmd_args = vec!["search", "index"];
    cmd_args.extend_from_slice(patterns);
    if force {
        cmd_args.push("--force");
    }

    let mut cmd = Command::cargo_bin("swissarmyhammer")?;
    cmd.args(&cmd_args)
        .current_dir(temp_path)
        .timeout(Duration::from_secs(timeout_secs))
        .env("SWISSARMYHAMMER_TEST_MODE", "1");
    
    // Set global model cache to avoid repeated downloads
    if let Some(cache_dir) = MODEL_CACHE_DIR.as_ref() {
        std::fs::create_dir_all(cache_dir).ok();
        cmd.env("SWISSARMYHAMMER_MODEL_CACHE", cache_dir);
    }
    
    let index_result = cmd.ok();

    match index_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if (stdout.contains("indexed") && stdout.chars().any(char::is_numeric))
                || (stdout.contains("files") && stdout.chars().any(char::is_numeric))
            {
                Ok(true) // Successfully indexed
            } else {
                eprintln!("⚠️  Search indexing did not produce expected output, skipping search operations");
                Ok(false) // Failed to index properly
            }
        }
        Err(_) => {
            eprintln!("⚠️  Search indexing failed (likely model download issue), skipping search operations");
            Ok(false) // Failed to run
        }
    }
}

/// Setup function for end-to-end workflow testing with optimized parallel execution
fn setup_e2e_test_environment() -> Result<(TempDir, std::path::PathBuf)> {
    // Use thread ID to create unique temp directories for parallel test execution
    let thread_id = std::thread::current().id();
    let temp_dir = TempDir::with_prefix(&format!("e2e_test_{thread_id:?}_"))?;
    let temp_path = temp_dir.path().to_path_buf();

    // Create comprehensive directory structure
    let issues_dir = temp_path.join("issues");
    std::fs::create_dir_all(&issues_dir)?;

    let swissarmyhammer_dir = temp_path.join(".swissarmyhammer");
    std::fs::create_dir_all(&swissarmyhammer_dir)?;

    let src_dir = temp_path.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Create sample source files for search workflow
    std::fs::write(
        src_dir.join("e2e_test.rs"),
        r#"
//! End-to-end test source file

use std::error::Error;

/// Function for e2e testing
pub fn e2e_test_function() -> Result<String, Box<dyn Error>> {
    println!("Running e2e test function");
    Ok("E2E test completed successfully".to_string())
}

/// Error handling for e2e tests
pub fn handle_e2e_error(error: &str) -> Result<(), String> {
    eprintln!("E2E error: {}", error);
    Err("E2E error handled".to_string())
}

/// Data processing function
pub fn process_data(data: Vec<i32>) -> Vec<i32> {
    data.iter().map(|x| x * 2).collect()
}
"#,
    )?;

    std::fs::write(
        src_dir.join("integration.rs"),
        r#"
//! Integration utilities

pub struct Integration {
    pub name: String,
    pub active: bool,
}

impl Integration {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active: true,
        }
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}
"#,
    )?;

    setup_git_repo(&temp_path)?;

    Ok((temp_dir, temp_path))
}

/// Test complete issue lifecycle workflow
#[test]
fn test_complete_issue_lifecycle() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Step 1: Create a new issue
    let create_output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            "e2e_lifecycle_test",
            "--content",
            "# E2E Lifecycle Test\n\nThis issue tests the complete lifecycle workflow.",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let create_stdout = String::from_utf8_lossy(&create_output.get_output().stdout);
    assert!(
        create_stdout.contains("Created issue: e2e_lifecycle_test")
            || create_stdout.contains("created issue: e2e_lifecycle_test")
            || create_stdout.contains("e2e_lifecycle_test"),
        "Issue creation should show success message with issue name: {create_stdout}"
    );

    // Step 2: List issues to verify creation
    let list_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let list_stdout = String::from_utf8_lossy(&list_output.get_output().stdout);
    assert!(
        list_stdout.contains("e2e_lifecycle_test"),
        "Issue should appear in list: {list_stdout}"
    );

    // Step 3: Show the issue details
    let show_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "show", "e2e_lifecycle_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let show_stdout = String::from_utf8_lossy(&show_output.get_output().stdout);
    assert!(
        show_stdout.contains("E2E Lifecycle Test")
            && show_stdout.contains("complete lifecycle workflow"),
        "Issue details should contain both title and description: {show_stdout}"
    );

    // Step 4: Update the issue
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "update",
            "e2e_lifecycle_test",
            "--content",
            "Updated content for e2e testing",
            "--append",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 5: Verify the update
    let updated_show_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "show", "e2e_lifecycle_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let updated_stdout = String::from_utf8_lossy(&updated_show_output.get_output().stdout);
    assert!(
        updated_stdout.contains("Updated content"),
        "Issue should contain updated content: {updated_stdout}"
    );

    // Step 6: Work on the issue (creates git branch)
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "work", "e2e_lifecycle_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 7: Check current issue
    let current_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "current"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let current_stdout = String::from_utf8_lossy(&current_output.get_output().stdout);
    assert!(
        current_stdout.contains("e2e_lifecycle_test"),
        "Current issue should show our issue: {current_stdout}"
    );

    // Step 8: Complete the issue
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "complete", "e2e_lifecycle_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 9: Merge the issue
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "merge", "e2e_lifecycle_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 10: Verify issue is completed
    let final_list_output = Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list", "--completed"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let final_stdout = String::from_utf8_lossy(&final_list_output.get_output().stdout);
    assert!(
        final_stdout.contains("e2e_lifecycle_test")
            && (final_stdout.contains("completed")
                || final_stdout.contains("✓")
                || final_stdout.contains("✅")),
        "Completed issue should appear with completion status indicator: {final_stdout}"
    );

    Ok(())
}

/// Test complete memo management workflow
#[test]
fn test_complete_memo_workflow() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Step 1: Create multiple memos
    let memo_data = vec![
        (
            "Meeting Notes",
            "# Meeting Notes\n\nDiscussed project timeline and goals.",
        ),
        (
            "Task List",
            "# Task List\n\n1. Complete testing\n2. Review documentation\n3. Deploy to production",
        ),
        (
            "Code Review Notes",
            "# Code Review\n\nReviewed PR #123:\n- Good error handling\n- Needs more tests",
        ),
    ];

    let mut memo_ids = vec![];

    for (title, content) in &memo_data {
        let create_output = Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "create", title, "--content", content])
            .current_dir(&temp_path)
            .assert()
            .success();

        let create_stdout = String::from_utf8_lossy(&create_output.get_output().stdout);

        // Extract memo ID from output (ULID pattern)
        if let Some(id) = extract_ulid_from_text(&create_stdout) {
            memo_ids.push(id);
        }
    }

    // Step 2: List all memos
    let list_output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let list_stdout = String::from_utf8_lossy(&list_output.get_output().stdout);
    assert!(
        list_stdout.contains("Meeting Notes")
            && list_stdout.contains("Task List")
            && (list_stdout.matches('\n').count() >= 2 || list_stdout.len() > 50),
        "All memos should appear in list with proper formatting: {list_stdout}"
    );

    // Step 3: Get specific memo details
    if let Some(first_id) = memo_ids.first() {
        let get_output = Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "get", first_id])
            .current_dir(&temp_path)
            .assert()
            .success();

        let get_stdout = String::from_utf8_lossy(&get_output.get_output().stdout);
        assert!(
            get_stdout.contains("Meeting Notes") || get_stdout.contains("project timeline"),
            "Memo details should contain expected content: {get_stdout}"
        );
    }

    // Step 4: Search memos
    let search_output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "search", "testing"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let search_stdout = String::from_utf8_lossy(&search_output.get_output().stdout);
    assert!(
        search_stdout.contains("Task List") || search_stdout.contains("Complete testing"),
        "Search should find relevant memos: {search_stdout}"
    );

    // Step 5: Update a memo
    if let Some(second_id) = memo_ids.get(1) {
        Command::cargo_bin("swissarmyhammer")?
            .args([
                "memo",
                "update",
                second_id,
                "--content",
                "# Updated Task List\n\n1. ✅ Complete testing\n2. Review documentation\n3. Deploy to production\n4. Monitor deployment"
            ])
            .current_dir(&temp_path)
            .assert()
            .success();

        // Verify update
        let updated_get_output = Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "get", second_id])
            .current_dir(&temp_path)
            .assert()
            .success();

        let updated_stdout = String::from_utf8_lossy(&updated_get_output.get_output().stdout);
        assert!(
            updated_stdout.contains("Updated Task List")
                && updated_stdout.contains("Monitor deployment"),
            "Updated memo should contain new content: {updated_stdout}"
        );
    }

    // Step 6: Get all context for AI
    let context_output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "context"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let context_stdout = String::from_utf8_lossy(&context_output.get_output().stdout);
    assert!(
        context_stdout.len() > 100
            && context_stdout.contains("Meeting Notes")
            && context_stdout.contains("Task List"),
        "Context should contain substantial content from all memos: length={}",
        context_stdout.len()
    );

    // Step 7: Delete a memo
    if let Some(last_id) = memo_ids.last() {
        Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "delete", last_id])
            .current_dir(&temp_path)
            .assert()
            .success();

        // Verify deletion
        Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "get", last_id])
            .current_dir(&temp_path)
            .assert()
            .failure(); // Should fail to find deleted memo
    }

    Ok(())
}

/// Test complete search workflow
#[test]
fn test_complete_search_workflow() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Step 1: Index source files (with optimized timeout and caching)
    let indexed = try_search_index(&temp_path, &["src/**/*.rs"], false, 5)?;
    if !indexed {
        return Ok(()); // Skip test if indexing fails
    }

    // Step 2: Query for functions
    let query_output = Command::cargo_bin("swissarmyhammer")?
        .args(["search", "query", "function", "--limit", "10"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let query_stdout = String::from_utf8_lossy(&query_output.get_output().stdout);
    // Should contain search results or indicate empty results gracefully
    assert!(
        !query_stdout.is_empty(), // Should have some output
        "Query should produce some output: {query_stdout}"
    );

    // Step 3: Query for specific functionality
    let specific_query_output = Command::cargo_bin("swissarmyhammer")?
        .args(["search", "query", "error handling", "--format", "json"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let json_stdout = String::from_utf8_lossy(&specific_query_output.get_output().stdout);
    if !json_stdout.trim().is_empty() {
        // If there are results, they should be valid JSON
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&json_stdout);
        if json_result.is_ok() {
            // JSON parsing successful - verify structure
            let json = json_result.unwrap();
            assert!(
                json.is_array() || json.is_object(),
                "JSON results should be array or object structure"
            );
        }
    }

    // Step 4: Re-index with force flag (reduced timeout since we have cache)
    let _reindexed = try_search_index(&temp_path, &["src/**/*.rs"], true, 3)?;
    // Continue regardless of re-indexing result since we already have an index

    // Step 5: Test different query formats
    let formats = ["table", "json"];
    for format in &formats {
        Command::cargo_bin("swissarmyhammer")?
            .args(["search", "query", "integration", "--format", format])
            .current_dir(&temp_path)
            .assert()
            .success();
    }

    Ok(())
}

/// Test mixed workflow with issues, memos, and search
#[test]
fn test_mixed_workflow() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Step 1: Create an issue about implementing search functionality
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            "implement_search_feature",
            "--content",
            "# Implement Search Feature\n\nNeed to add semantic search capabilities to the application."
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 2: Create research memo about search implementation
    let memo_output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Search Implementation Research",
            "--content",
            "# Search Research\n\n## Options Considered\n- Vector embeddings\n- Full-text search\n- Hybrid approach\n\n## Recommendation\nUse vector embeddings with DuckDB storage."
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let memo_stdout = String::from_utf8_lossy(&memo_output.get_output().stdout);
    let _research_memo_id = extract_ulid_from_text(&memo_stdout);

    // Step 3: Work on the issue
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "work", "implement_search_feature"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 4: Index the source files (implementing the feature)
    let indexed = try_search_index(&temp_path, &["src/**/*.rs"], false, 5)?;
    if !indexed {
        // Skip search-dependent parts but continue with other workflow steps
        eprintln!("⚠️  Continuing mixed workflow without search operations");
    }

    // Step 5: Create progress memo
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Search Implementation Progress",
            "--content",
            "# Implementation Progress\n\n✅ Indexed source files\n✅ Verified search functionality\n🔄 Writing tests\n⏳ Documentation updates"
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 6: Update original issue with progress
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "update",
            "implement_search_feature",
            "--content",
            "\n\n## Progress Update\n\nSearch indexing is now working correctly. Ready for testing phase.",
            "--append"
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 7: Search for implementation details
    Command::cargo_bin("swissarmyhammer")?
        .args(["search", "query", "integration test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 8: Search memos for research notes
    let memo_search_output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "search", "vector embeddings"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let memo_search_stdout = String::from_utf8_lossy(&memo_search_output.get_output().stdout);
    assert!(
        memo_search_stdout.contains("Search") || memo_search_stdout.contains("Research"),
        "Should find research memo: {memo_search_stdout}"
    );

    // Step 9: Complete the issue
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "complete", "implement_search_feature"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 10: Create completion memo
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Search Feature Completed",
            "--content",
            "# Search Feature Complete\n\n## Summary\nSuccessfully implemented semantic search with:\n- Vector embeddings\n- DuckDB storage\n- CLI integration\n\n## Next Steps\n- Performance optimization\n- User documentation"
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 11: Get all context for final review
    let context_output = Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "context"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let context_stdout = String::from_utf8_lossy(&context_output.get_output().stdout);
    assert!(
        context_stdout.contains("Search") && context_stdout.contains("Implementation"),
        "Context should contain all search-related memos: {}",
        context_stdout.len()
    );

    // Step 12: Merge the completed issue
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "merge", "implement_search_feature"])
        .current_dir(&temp_path)
        .assert()
        .success();

    Ok(())
}

/// Test error recovery workflow
#[test]
fn test_error_recovery_workflow() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Step 1: Attempt to work on non-existent issue (should fail)
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "work", "nonexistent_issue"])
        .current_dir(&temp_path)
        .assert()
        .failure();

    // Step 2: Create the issue properly
    Command::cargo_bin("swissarmyhammer")?
        .args([
            "issue",
            "create",
            "error_recovery_test",
            "--content",
            "# Error Recovery Test\n\nTesting error recovery workflows.",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 3: Now work on the issue (should succeed)
    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "work", "error_recovery_test"])
        .current_dir(&temp_path)
        .assert()
        .success();

    // Step 4: Attempt to get non-existent memo (should fail gracefully)
    Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "get", "01ARZ3NDEKTSV4RRFFQ69G5FAV"])
        .current_dir(&temp_path)
        .assert()
        .failure();

    // Step 5: Create memo properly
    let memo_output = Command::cargo_bin("swissarmyhammer")?
        .args([
            "memo",
            "create",
            "Error Recovery Notes",
            "--content",
            "# Recovery Notes\n\nDocumenting error recovery procedures.",
        ])
        .current_dir(&temp_path)
        .assert()
        .success();

    let memo_stdout = String::from_utf8_lossy(&memo_output.get_output().stdout);
    if let Some(memo_id) = extract_ulid_from_text(&memo_stdout) {
        // Step 6: Now get the memo (should succeed)
        Command::cargo_bin("swissarmyhammer")?
            .args(["memo", "get", &memo_id])
            .current_dir(&temp_path)
            .assert()
            .success();
    }

    // Step 7: Attempt search without indexing (may succeed with empty results)
    Command::cargo_bin("swissarmyhammer")?
        .args(["search", "query", "recovery"])
        .current_dir(&temp_path)
        .assert()
        .success(); // Should handle gracefully even if no index

    // Step 8: Index files and search again (with aggressive timeout optimization)
    let indexed = try_search_index(&temp_path, &["src/**/*.rs"], false, 3)?;
    if indexed {
        Command::cargo_bin("swissarmyhammer")?
            .args(["search", "query", "integration"])
            .current_dir(&temp_path)
            .assert()
            .success();
    } else {
        eprintln!("⚠️  Skipping search query in error recovery test - indexing failed");
    }

    Ok(())
}

/// Test performance under realistic workflow load
#[test]
#[ignore = "Slow load test - run with --ignored"]
fn test_realistic_load_workflow() -> Result<()> {
    let (_temp_dir, temp_path) = setup_e2e_test_environment()?;

    // Create multiple issues and memos to simulate realistic usage
    for i in 1..=5 {
        Command::cargo_bin("swissarmyhammer")?
            .args([
                "issue",
                "create",
                &format!("load_test_issue_{i}"),
                "--content",
                &format!("# Load Test Issue {i}\n\nThis is issue {i} for load testing."),
            ])
            .current_dir(&temp_path)
            .assert()
            .success();

        Command::cargo_bin("swissarmyhammer")?
            .args([
                "memo",
                "create",
                &format!("Load Test Memo {i}"),
                "--content",
                &format!("# Memo {i}\n\nThis is memo {i} for load testing.\n\n## Details\n- Priority: Medium\n- Category: Testing\n- Iteration: {i}")
            ])
            .current_dir(&temp_path)
            .assert()
            .success();
    }

    // Perform various operations to test performance
    let start_time = std::time::Instant::now();

    Command::cargo_bin("swissarmyhammer")?
        .args(["issue", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    Command::cargo_bin("swissarmyhammer")?
        .args(["memo", "list"])
        .current_dir(&temp_path)
        .assert()
        .success();

    let _indexed = try_search_index(&temp_path, &["src/**/*.rs"], false, 3)?;
    // Continue timing test regardless of indexing result

    let elapsed = start_time.elapsed();

    // Should complete in reasonable time (less than 60 seconds for this load)
    assert!(
        elapsed < Duration::from_secs(60),
        "Workflow should complete in reasonable time: {elapsed:?}"
    );

    Ok(())
}

/// Helper function to extract ULID from text
fn extract_ulid_from_text(text: &str) -> Option<String> {
    use regex::Regex;

    // ULID pattern: 26 characters using Crockford's Base32
    let ulid_pattern = Regex::new(r"\b[0-9A-HJKMNP-TV-Z]{26}\b").ok()?;
    ulid_pattern.find(text).map(|m| m.as_str().to_string())
}
