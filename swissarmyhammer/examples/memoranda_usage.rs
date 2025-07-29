//! Memoranda API usage examples for SwissArmyHammer library
//!
//! This example demonstrates how to programmatically interact with the memoranda system
//! for structured note-taking and knowledge management.

use swissarmyhammer::memoranda::{MarkdownMemoStorage, MemoStorage};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SwissArmyHammer Memoranda API Examples");
    println!("==========================================\n");

    // Initialize temporary storage for this example
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage = MarkdownMemoStorage::new(temp_dir.path().join("memos"));

    // Example 1: Creating memos
    println!("📝 Example 1: Creating Memos");
    println!("-----------------------------");

    let memo1 = storage
        .create_memo(
            "API Design Meeting".to_string(),
            r#"# API Design Meeting - January 15, 2024

## Attendees
- Alice (Backend Engineer)
- Bob (Frontend Engineer)
- Carol (Product Manager)

## Decisions Made
1. Use REST API with GraphQL for complex queries
2. Implement OAuth2 for authentication
3. Add rate limiting: 1000 requests/hour per user

## Action Items
- [ ] Alice: Implement OAuth endpoints (by Friday)
- [ ] Bob: Update frontend auth flow (by next Monday)
- [ ] Carol: Update API documentation (by Wednesday)

## Next Meeting
- Date: January 22, 2024
- Focus: Review implementation progress"#
                .to_string(),
        )
        .await?;

    println!("✅ Created memo: {}", memo1.title);
    println!("🆔 ID: {}", memo1.id);
    println!("📅 Created: {}\n", memo1.created_at);

    let memo2 = storage
        .create_memo(
            "Rust Learning Notes".to_string(),
            r#"# Rust Learning Progress

## Completed Topics
- ✅ Ownership and borrowing
- ✅ Pattern matching
- ✅ Error handling with Result<T, E>
- ✅ Async/await basics

## Current Focus: Advanced Patterns
### Iterator Combinators
```rust
let numbers: Vec<i32> = vec![1, 2, 3, 4, 5]
    .iter()
    .filter(|&&x| x > 2)
    .map(|&x| x * 2)
    .collect();
```

### Error Propagation
```rust
fn read_file() -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string("file.txt")?;
    Ok(content.trim().to_string())
}
```

## Next Steps
- Learn about lifetimes in depth
- Practice with smart pointers
- Build a CLI tool project"#
                .to_string(),
        )
        .await?;

    println!("✅ Created memo: {}", memo2.title);
    println!("🆔 ID: {}\n", memo2.id);

    // Example 2: Listing memos
    println!("📋 Example 2: Listing All Memos");
    println!("--------------------------------");

    let memos = storage.list_memos().await?;
    println!("📝 Found {} memos:", memos.len());

    for memo in &memos {
        println!("  🆔 {}", memo.id);
        println!("  📄 {}", memo.title);
        println!("  📅 {}", memo.created_at);
        let preview = if memo.content.len() > 100 {
            format!("{}...", &memo.content[..97])
        } else {
            memo.content.clone()
        };
        println!("  💬 {}\n", preview.replace('\n', "\\n"));
    }

    // Example 3: Retrieving specific memos
    println!("🔍 Example 3: Retrieving Specific Memos");
    println!("---------------------------------------");

    let retrieved_memo = storage.get_memo(&memo1.id).await?;
    {
        println!("📝 Retrieved memo: {}", retrieved_memo.title);
        println!("🆔 ID: {}", retrieved_memo.id);
        println!("📅 Created: {}", retrieved_memo.created_at);
        println!("🔄 Updated: {}", retrieved_memo.updated_at);
        println!(
            "📖 Content length: {} characters\n",
            retrieved_memo.content.len()
        );
    }

    // Example 4: Searching memos
    println!("🔎 Example 4: Searching Memos");
    println!("-----------------------------");

    let search_results = storage.search_memos("API authentication").await?;

    println!("🔍 Search results for 'API authentication':");
    for memo in search_results {
        println!("  📄 {} (ID: {})", memo.title, memo.id);
        println!();
    }

    // Example 5: Advanced search patterns
    println!("🔍 Example 5: Advanced Search Patterns");
    println!("--------------------------------------");

    // Search for Rust-related content
    let rust_results = storage.search_memos("Rust").await?;

    println!("🦀 Rust-related memos ({} found):", rust_results.len());
    for memo in rust_results {
        println!("  📄 {}", memo.title);
    }
    println!();

    // Search for action items
    let action_results = storage.search_memos("action items").await?;

    println!(
        "✅ Memos with action items ({} found):",
        action_results.len()
    );
    for memo in action_results {
        println!("  📄 {}", memo.title);
    }
    println!();

    // Example 6: Updating memos
    println!("📝 Example 6: Updating Memos");
    println!("-----------------------------");

    let updated_memo = storage
        .update_memo(
            &memo1.id,
            r#"# API Design Meeting - January 15, 2024

## Attendees
- Alice (Backend Engineer)
- Bob (Frontend Engineer)
- Carol (Product Manager)

## Decisions Made
1. Use REST API with GraphQL for complex queries
2. Implement OAuth2 for authentication
3. Add rate limiting: 1000 requests/hour per user

## Action Items Progress (UPDATED)
- [x] Alice: Implement OAuth endpoints (✅ Completed ahead of schedule!)
- [ ] Bob: Update frontend auth flow (in progress, on track)
- [x] Carol: Update API documentation (✅ Completed)

## Follow-up Notes
- Alice's OAuth implementation is excellent
- Need to review Bob's auth flow before Monday
- Documentation is comprehensive and clear

## Next Meeting
- Date: January 22, 2024
- Focus: Review Bob's implementation and plan next sprint"#
                .to_string(),
        )
        .await?;

    println!("✅ Updated memo: {}", updated_memo.title);
    println!("🔄 New updated_at: {}\n", updated_memo.updated_at);

    // Example 7: Error handling patterns
    println!("⚠️  Example 7: Error Handling Patterns");
    println!("-------------------------------------");

    // Attempt to get a non-existent memo
    match swissarmyhammer::memoranda::MemoId::from_string("01INVALID_MEMO_ID_HERE".to_string()) {
        Ok(id) => match storage.get_memo(&id).await {
            Ok(_) => println!("Found memo (unexpected)"),
            Err(e) => println!("✅ Correctly handled: {e}"),
        },
        Err(e) => println!("✅ Correctly handled: Invalid memo ID format: {e}"),
    }

    // Attempt to update a non-existent memo
    match storage
        .update_memo(
            &swissarmyhammer::memoranda::MemoId::from_string(
                "01NONEXISTENT_MEMO_ID_123".to_string(),
            )
            .unwrap_or_else(|_| swissarmyhammer::memoranda::MemoId::new()),
            "Updated content".to_string(),
        )
        .await
    {
        Ok(_) => println!("Updated memo (unexpected)"),
        Err(e) => println!("✅ Correctly handled error: {e}"),
    }
    println!();

    // Example 8: Batch operations
    println!("📦 Example 8: Batch Operations");
    println!("------------------------------");

    // Create multiple memos for demonstration
    let project_memos = vec![
        (
            "Sprint Planning",
            "# Sprint 12 Planning\n\n- Goal: User authentication\n- Story points: 34",
        ),
        (
            "Daily Standup",
            "# Daily Standup Notes\n\n## Blockers\n- Database migration pending",
        ),
        (
            "Code Review",
            "# Code Review Checklist\n\n- [ ] Tests pass\n- [ ] Documentation updated",
        ),
    ];

    let mut created_ids = Vec::new();
    for (title, content) in project_memos {
        let memo = storage
            .create_memo(title.to_string(), content.to_string())
            .await?;
        created_ids.push(memo.id);
        println!("✅ Created: {title}");
    }

    println!("\n📊 Final Statistics:");
    let final_memos = storage.list_memos().await?;
    println!("  📝 Total memos: {}", final_memos.len());

    let total_content_length: usize = final_memos.iter().map(|m| m.content.len()).sum();
    println!("  📖 Total content: {total_content_length} characters");

    let avg_content_length = if !final_memos.is_empty() {
        total_content_length / final_memos.len()
    } else {
        0
    };
    println!("  📏 Average content: {avg_content_length} characters per memo");

    // Example 9: Integration patterns
    println!("\n🔗 Example 9: Integration Patterns");
    println!("-----------------------------------");

    // Export all memos for external processing
    let all_memos = storage.list_memos().await?;
    let mut context_export = String::new();

    for memo in &all_memos {
        context_export.push_str(&format!(
            "## {} (ID: {})\n\nCreated: {}\nUpdated: {}\n\n{}\n\n===\n\n",
            memo.title, memo.id, memo.created_at, memo.updated_at, memo.content
        ));
    }

    println!(
        "📄 Generated context export ({} chars)",
        context_export.len()
    );
    println!("💡 This format is perfect for AI assistant integration!\n");

    // Cleanup example (delete some memos)
    println!("🧹 Example 10: Cleanup Operations");
    println!("---------------------------------");

    for id in &created_ids[..2] {
        // Delete first 2 demo memos
        match storage.delete_memo(id).await {
            Ok(_) => println!("✅ Deleted memo: {id}"),
            Err(e) => {
                if e.to_string().contains("not found") {
                    println!("⚠️  Memo not found: {id}");
                } else {
                    println!("❌ Error deleting memo: {e}");
                }
            }
        }
    }

    let remaining_memos = storage.list_memos().await?;
    println!("📊 Remaining memos: {}\n", remaining_memos.len());

    // Final summary
    println!("🎉 API Examples Completed!");
    println!("==========================");
    println!("✅ Demonstrated memo creation and management");
    println!("✅ Showed search and retrieval patterns");
    println!("✅ Illustrated error handling best practices");
    println!("✅ Provided integration and automation examples");
    println!();
    println!("💡 Next steps:");
    println!("   - Integrate memoranda into your application");
    println!("   - Use FileStorage for persistent data");
    println!("   - Implement custom search algorithms");
    println!("   - Build MCP tools for AI assistant integration");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memo_operations() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let storage = MarkdownMemoStorage::new(temp_dir.path().join("memos"));

        // Test creating a memo
        let memo = storage
            .create_memo("Test Memo".to_string(), "Test content".to_string())
            .await
            .unwrap();

        assert_eq!(memo.title, "Test Memo");
        assert_eq!(memo.content, "Test content");

        // Test retrieving the memo
        let retrieved = storage.get_memo(&memo.id).await.unwrap();
        assert_eq!(retrieved.title, "Test Memo");

        // Test listing memos
        let memos = storage.list_memos().await.unwrap();
        assert_eq!(memos.len(), 1);

        // Test searching
        let results = storage.search_memos("Test").await.unwrap();
        assert_eq!(results.len(), 1);

        // Test deleting
        storage.delete_memo(&memo.id).await.unwrap();

        let remaining = storage.list_memos().await.unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[tokio::test]
    async fn test_memo_content_analysis() {
        let memo = Memo {
            id: MemoId::from_string("01TEST123456789012345678".to_string()).unwrap(),
            title: "Test Memo".to_string(),
            content: "# Header 1\n\nSome content\n\n## Header 2\n\n- [ ] Task 1\n- [x] Task 2"
                .to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let stats = analyze_memo_content(&memo).await;

        assert_eq!(stats["headers"], 2);
        assert_eq!(stats["action_items"], 2);
        assert!(stats["words"] > 0);
        assert!(stats["lines"] > 0);
    }
}
