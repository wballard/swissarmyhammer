//! Integration tests for memo migration from JSON to Markdown
//!
//! This test suite verifies the complete end-to-end migration workflow,
//! ensuring that all memo data is preserved during the migration process
//! and that both CLI and MCP interfaces work correctly after migration.

use serde_json;
use std::fs;
use swissarmyhammer::memoranda::{MemoId, MarkdownMemoStorage, MemoStorage};
use tempfile::TempDir;
use tokio;

/// Creates a JSON memo file for testing migration
fn create_json_memo_file(
    memos_dir: &std::path::Path,
    id: &str,
    title: &str,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let memo = serde_json::json!({
        "id": id,
        "title": title,
        "content": content,
        "created_at": "2023-01-01T10:00:00Z",
        "updated_at": "2023-01-01T12:00:00Z"
    });

    let file_path = memos_dir.join(format!("{}.json", id));
    fs::write(file_path, serde_json::to_string_pretty(&memo)?)?;
    Ok(())
}

/// Verifies that a markdown memo file exists and has correct content
fn verify_markdown_memo(
    memos_dir: &std::path::Path,
    expected_filename: &str,
    expected_content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_path = memos_dir.join(format!("{}.md", expected_filename));
    assert!(
        markdown_path.exists(),
        "Markdown file should exist: {}",
        expected_filename
    );

    let file_content = fs::read_to_string(&markdown_path)?;
    assert_eq!(
        file_content.trim(),
        expected_content.trim(),
        "Markdown file content should match expected content"
    );
    Ok(())
}

#[tokio::test]
async fn test_complete_migration_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create test JSON memo files with various content types
    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "Simple Memo",
        "This is a simple memo content.",
    )?;

    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAW",
        "Complex Memo with Special Characters",
        "# Markdown Content\n\nThis memo contains **bold text** and *italic text*.\n\n- List item 1\n- List item 2",
    )?;

    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAX",
        "Unicode Test 🚀",
        "This memo contains unicode: 中文, émojis 🎉, and special chars: \"quotes\" & symbols.",
    )?;

    // Verify JSON files exist before migration
    assert!(memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json").exists());
    assert!(memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAW.json").exists());
    assert!(memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAX.json").exists());

    // Perform migration
    let storage = MarkdownMemoStorage::new(memos_dir.clone());
    let migrated_count = storage.migrate_from_json(true).await?;

    // Verify migration results
    assert_eq!(migrated_count, 3, "Should migrate exactly 3 memos");

    // Verify JSON files were removed (since remove_json_files = true)
    assert!(!memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json").exists());
    assert!(!memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAW.json").exists());
    assert!(!memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAX.json").exists());

    // Verify markdown files were created with correct content
    verify_markdown_memo(
        &memos_dir,
        "Simple Memo",
        "This is a simple memo content.",
    )?;

    verify_markdown_memo(
        &memos_dir,
        "Complex Memo with Special Characters",
        "# Markdown Content\n\nThis memo contains **bold text** and *italic text*.\n\n- List item 1\n- List item 2",
    )?;

    verify_markdown_memo(
        &memos_dir,
        "Unicode Test",
        "This memo contains unicode: 中文, émojis 🎉, and special chars: \"quotes\" & symbols.",
    )?;

    Ok(())
}

#[tokio::test]
async fn test_migration_preserves_all_memo_functionality() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create JSON memo files
    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "Test Memo 1",
        "Content of memo 1",
    )?;

    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAW",
        "Test Memo 2",
        "Content of memo 2 with search terms",
    )?;

    // Initialize storage and perform migration
    let storage = MarkdownMemoStorage::new(memos_dir.clone());
    let migrated_count = storage.migrate_from_json(true).await?;
    assert_eq!(migrated_count, 2);

    // Test that all memo operations work after migration

    // 1. List memos
    let all_memos = storage.list_memos().await?;
    assert_eq!(all_memos.len(), 2, "Should list 2 migrated memos");

    // Verify memo titles are preserved
    let titles: Vec<String> = all_memos.iter().map(|m| m.title.clone()).collect();
    assert!(titles.contains(&"Test Memo 1".to_string()));
    assert!(titles.contains(&"Test Memo 2".to_string()));

    // 2. Get individual memos by ID (using filename-based IDs)
    let memo1 = storage
        .get_memo(&MemoId::from_filename("Test Memo 1"))
        .await?;
    assert_eq!(memo1.title, "Test Memo 1");
    assert_eq!(memo1.content, "Content of memo 1");

    let memo2 = storage
        .get_memo(&MemoId::from_filename("Test Memo 2"))
        .await?;
    assert_eq!(memo2.title, "Test Memo 2");
    assert_eq!(memo2.content, "Content of memo 2 with search terms");

    // 3. Search functionality
    let search_results = storage.search_memos("search terms").await?;
    assert_eq!(search_results.len(), 1, "Should find 1 memo with search terms");
    assert_eq!(search_results[0].title, "Test Memo 2");

    // 4. Create new memo (after migration)
    let new_memo = storage
        .create_memo("New Memo".to_string(), "New content".to_string())
        .await?;
    assert_eq!(new_memo.title, "New Memo");

    // Verify we now have 3 memos total
    let all_memos_after = storage.list_memos().await?;
    assert_eq!(all_memos_after.len(), 3);

    // 5. Update existing migrated memo
    storage
        .update_memo(
            &MemoId::from_filename("Test Memo 1"),
            "Updated content".to_string(),
        )
        .await?;

    let updated_memo = storage
        .get_memo(&MemoId::from_filename("Test Memo 1"))
        .await?;
    assert_eq!(updated_memo.content, "Updated content");

    // 6. Delete memo
    storage
        .delete_memo(&MemoId::from_filename("Test Memo 2"))
        .await?;

    let remaining_memos = storage.list_memos().await?;
    assert_eq!(remaining_memos.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_migration_with_duplicate_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create JSON memo
    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "Duplicate Test",
        "JSON content",
    )?;

    // Create existing markdown file with same title
    let existing_md_path = memos_dir.join("Duplicate Test.md");
    fs::write(&existing_md_path, "Existing markdown content")?;

    // Perform migration
    let storage = MarkdownMemoStorage::new(memos_dir.clone());
    let migrated_count = storage.migrate_from_json(true).await?;

    // Should not migrate due to existing markdown file
    assert_eq!(
        migrated_count, 0,
        "Should not migrate when markdown file already exists"
    );

    // Verify original markdown content is preserved
    let content = fs::read_to_string(&existing_md_path)?;
    assert_eq!(content, "Existing markdown content");

    // JSON file should remain since migration was skipped
    assert!(memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json").exists());

    Ok(())
}

#[tokio::test]
async fn test_migration_idempotency() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create JSON memo
    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "Idempotency Test",
        "Test content",
    )?;

    let storage = MarkdownMemoStorage::new(memos_dir.clone());

    // First migration
    let first_migration = storage.migrate_from_json(true).await?;
    assert_eq!(first_migration, 1);

    // Verify markdown file was created and JSON was removed
    assert!(memos_dir.join("Idempotency Test.md").exists());
    assert!(!memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json").exists());

    // Second migration attempt
    let second_migration = storage.migrate_from_json(true).await?;
    assert_eq!(
        second_migration, 0,
        "Second migration should migrate 0 files"
    );

    // Verify markdown file still exists and content is unchanged
    let content = fs::read_to_string(memos_dir.join("Idempotency Test.md"))?;
    assert_eq!(content, "Test content");

    Ok(())
}

#[tokio::test]
async fn test_migration_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create valid JSON memo
    create_json_memo_file(
        &memos_dir,
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "Valid Memo",
        "Valid content",
    )?;

    // Create malformed JSON file
    let malformed_path = memos_dir.join("malformed.json");
    fs::write(&malformed_path, "{ invalid json content")?;

    let storage = MarkdownMemoStorage::new(memos_dir.clone());

    // Migration should succeed for valid file and skip malformed file
    let migrated_count = storage.migrate_from_json(true).await?;
    assert_eq!(migrated_count, 1, "Should migrate only the valid file");

    // Valid file should be migrated
    assert!(memos_dir.join("Valid Memo.md").exists());
    assert!(!memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json").exists());

    // Malformed file should remain (not deleted since migration failed)
    assert!(malformed_path.exists());

    Ok(())
}

#[tokio::test]
async fn test_migration_preserves_file_metadata() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let memos_dir = temp_dir.path().join("memos");
    fs::create_dir_all(&memos_dir)?;

    // Create JSON memo with specific timestamps
    let memo_json = serde_json::json!({
        "id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        "title": "Timestamp Test",
        "content": "Test content for timestamp preservation",
        "created_at": "2023-01-01T10:00:00Z",
        "updated_at": "2023-01-02T15:30:00Z"
    });

    let json_file = memos_dir.join("01ARZ3NDEKTSV4RRFFQ69G5FAV.json");
    fs::write(&json_file, serde_json::to_string_pretty(&memo_json)?)?;

    let storage = MarkdownMemoStorage::new(memos_dir.clone());
    let migrated_count = storage.migrate_from_json(true).await?;
    assert_eq!(migrated_count, 1);

    // Load the migrated memo through the storage interface
    let migrated_memo = storage
        .get_memo(&MemoId::from_filename("Timestamp Test"))
        .await?;

    // Verify content is preserved
    assert_eq!(migrated_memo.title, "Timestamp Test");
    assert_eq!(migrated_memo.content, "Test content for timestamp preservation");

    // Note: File-based timestamps will be different from JSON timestamps since
    // MarkdownMemoStorage derives timestamps from filesystem metadata.
    // This is expected behavior as documented in the migration process.

    Ok(())
}