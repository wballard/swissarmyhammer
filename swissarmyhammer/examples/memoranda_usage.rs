//! Memoranda API usage examples for SwissArmyHammer library
//!
//! This example demonstrates how to programmatically interact with the memoranda system
//! for structured note-taking and knowledge management.

use std::collections::HashMap;
use swissarmyhammer::memoranda::{
    storage::MemoryStorage,
    operations::{create_memo, get_memo, update_memo, delete_memo, list_memos, search_memos},
    models::{Memo, CreateMemoRequest, UpdateMemoRequest, SearchRequest},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SwissArmyHammer Memoranda API Examples");
    println!("==========================================\n");

    // Initialize storage (in production, use FileStorage instead)
    let storage = MemoryStorage::new();

    // Example 1: Creating memos
    println!("📝 Example 1: Creating Memos");
    println!("-----------------------------");
    
    let memo1 = create_memo(&storage, CreateMemoRequest {
        title: "API Design Meeting".to_string(),
        content: r#"# API Design Meeting - January 15, 2024

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
- Focus: Review implementation progress"#.to_string(),
    }).await?;

    println!("✅ Created memo: {}", memo1.title);
    println!("🆔 ID: {}", memo1.id);
    println!("📅 Created: {}\n", memo1.created_at);

    let memo2 = create_memo(&storage, CreateMemoRequest {
        title: "Rust Learning Notes".to_string(),
        content: r#"# Rust Learning Progress

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
- Build a CLI tool project"#.to_string(),
    }).await?;

    println!("✅ Created memo: {}", memo2.title);
    println!("🆔 ID: {}\n", memo2.id);

    // Example 2: Listing memos
    println!("📋 Example 2: Listing All Memos");
    println!("--------------------------------");
    
    let memos = list_memos(&storage).await?;
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
    
    if let Some(retrieved_memo) = get_memo(&storage, &memo1.id).await? {
        println!("📝 Retrieved memo: {}", retrieved_memo.title);
        println!("🆔 ID: {}", retrieved_memo.id);
        println!("📅 Created: {}", retrieved_memo.created_at);
        println!("🔄 Updated: {}", retrieved_memo.updated_at);
        println!("📖 Content length: {} characters\n", retrieved_memo.content.len());
    }

    // Example 4: Searching memos
    println!("🔎 Example 4: Searching Memos");
    println!("-----------------------------");
    
    let search_results = search_memos(&storage, SearchRequest {
        query: "API authentication".to_string(),
        limit: Some(10),
    }).await?;

    println!("🔍 Search results for 'API authentication':");
    for result in search_results {
        println!("  📄 {} (ID: {})", result.memo.title, result.memo.id);
        if let Some(score) = result.relevance_score {
            println!("  📊 Relevance: {:.2}", score);
        }
        if !result.highlights.is_empty() {
            println!("  🎯 Highlights: {:?}", result.highlights);
        }
        println!();
    }

    // Example 5: Advanced search patterns
    println!("🔍 Example 5: Advanced Search Patterns");
    println!("--------------------------------------");

    // Search for Rust-related content
    let rust_results = search_memos(&storage, SearchRequest {
        query: "Rust".to_string(),
        limit: Some(5),
    }).await?;

    println!("🦀 Rust-related memos ({} found):", rust_results.len());
    for result in rust_results {
        println!("  📄 {}", result.memo.title);
    }
    println!();

    // Search for action items
    let action_results = search_memos(&storage, SearchRequest {
        query: "action items".to_string(),
        limit: Some(10),
    }).await?;

    println!("✅ Memos with action items ({} found):", action_results.len());
    for result in action_results {
        println!("  📄 {}", result.memo.title);
    }
    println!();

    // Example 6: Updating memos
    println!("📝 Example 6: Updating Memos");
    println!("-----------------------------");
    
    let updated_memo = update_memo(&storage, UpdateMemoRequest {
        id: memo1.id.clone(),
        content: r#"# API Design Meeting - January 15, 2024

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
- Focus: Review Bob's implementation and plan next sprint"#.to_string(),
    }).await?;

    println!("✅ Updated memo: {}", updated_memo.title);
    println!("🔄 New updated_at: {}\n", updated_memo.updated_at);

    // Example 7: Error handling patterns
    println!("⚠️  Example 7: Error Handling Patterns");
    println!("-------------------------------------");

    // Attempt to get a non-existent memo
    match get_memo(&storage, "01INVALID_MEMO_ID_HERE").await {
        Ok(Some(_)) => println!("Found memo (unexpected)"),
        Ok(None) => println!("✅ Correctly handled: Memo not found"),
        Err(e) => println!("❌ Error occurred: {}", e),
    }

    // Attempt to update a non-existent memo
    match update_memo(&storage, UpdateMemoRequest {
        id: "01NONEXISTENT_MEMO_ID_123".to_string(),
        content: "Updated content".to_string(),
    }).await {
        Ok(_) => println!("Updated memo (unexpected)"),
        Err(e) => println!("✅ Correctly handled error: {}", e),
    }
    println!();

    // Example 8: Batch operations
    println!("📦 Example 8: Batch Operations");
    println!("------------------------------");

    // Create multiple memos for demonstration
    let project_memos = vec![
        ("Sprint Planning", "# Sprint 12 Planning\n\n- Goal: User authentication\n- Story points: 34"),
        ("Daily Standup", "# Daily Standup Notes\n\n## Blockers\n- Database migration pending"),
        ("Code Review", "# Code Review Checklist\n\n- [ ] Tests pass\n- [ ] Documentation updated"),
    ];

    let mut created_ids = Vec::new();
    for (title, content) in project_memos {
        let memo = create_memo(&storage, CreateMemoRequest {
            title: title.to_string(),
            content: content.to_string(),
        }).await?;
        created_ids.push(memo.id);
        println!("✅ Created: {}", title);
    }

    println!("\n📊 Final Statistics:");
    let final_memos = list_memos(&storage).await?;
    println!("  📝 Total memos: {}", final_memos.len());
    
    let total_content_length: usize = final_memos.iter()
        .map(|m| m.content.len())
        .sum();
    println!("  📖 Total content: {} characters", total_content_length);
    
    let avg_content_length = if !final_memos.is_empty() {
        total_content_length / final_memos.len()
    } else {
        0
    };
    println!("  📏 Average content: {} characters per memo", avg_content_length);

    // Example 9: Integration patterns
    println!("\n🔗 Example 9: Integration Patterns");
    println!("-----------------------------------");

    // Export all memos for external processing
    let all_memos = list_memos(&storage).await?;
    let mut context_export = String::new();
    
    for memo in &all_memos {
        context_export.push_str(&format!(
            "## {} (ID: {})\n\nCreated: {}\nUpdated: {}\n\n{}\n\n===\n\n",
            memo.title, memo.id, memo.created_at, memo.updated_at, memo.content
        ));
    }
    
    println!("📄 Generated context export ({} chars)", context_export.len());
    println!("💡 This format is perfect for AI assistant integration!\n");

    // Cleanup example (delete some memos)
    println!("🧹 Example 10: Cleanup Operations");
    println!("---------------------------------");

    for id in &created_ids[..2] { // Delete first 2 demo memos
        match delete_memo(&storage, id).await {
            Ok(true) => println!("✅ Deleted memo: {}", id),
            Ok(false) => println!("⚠️  Memo not found: {}", id),
            Err(e) => println!("❌ Error deleting memo: {}", e),
        }
    }

    let remaining_memos = list_memos(&storage).await?;
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

/// Helper function to demonstrate custom memo processing
async fn analyze_memo_content(memo: &Memo) -> HashMap<String, usize> {
    let mut stats = HashMap::new();
    
    stats.insert("lines".to_string(), memo.content.lines().count());
    stats.insert("words".to_string(), memo.content.split_whitespace().count());
    stats.insert("characters".to_string(), memo.content.len());
    
    // Count markdown headers
    let headers = memo.content.lines()
        .filter(|line| line.starts_with('#'))
        .count();
    stats.insert("headers".to_string(), headers);
    
    // Count action items (lines with [ ] or [x])
    let action_items = memo.content.lines()
        .filter(|line| line.contains("[ ]") || line.contains("[x]"))
        .count();
    stats.insert("action_items".to_string(), action_items);
    
    stats
}

/// Example integration with external systems
async fn export_memos_to_json(memos: &[Memo]) -> Result<String, Box<dyn std::error::Error>> {
    use serde_json::json;
    
    let export_data = json!({
        "export_timestamp": chrono::Utc::now(),
        "total_memos": memos.len(),
        "memos": memos.iter().map(|memo| {
            json!({
                "id": memo.id,
                "title": memo.title,
                "content": memo.content,
                "created_at": memo.created_at,
                "updated_at": memo.updated_at,
                "metadata": {
                    "content_length": memo.content.len(),
                    "line_count": memo.content.lines().count(),
                }
            })
        }).collect::<Vec<_>>()
    });
    
    Ok(serde_json::to_string_pretty(&export_data)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memo_operations() {
        let storage = MemoryStorage::new();
        
        // Test creating a memo
        let memo = create_memo(&storage, CreateMemoRequest {
            title: "Test Memo".to_string(),
            content: "Test content".to_string(),
        }).await.unwrap();
        
        assert_eq!(memo.title, "Test Memo");
        assert_eq!(memo.content, "Test content");
        
        // Test retrieving the memo
        let retrieved = get_memo(&storage, &memo.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Memo");
        
        // Test listing memos
        let memos = list_memos(&storage).await.unwrap();
        assert_eq!(memos.len(), 1);
        
        // Test searching
        let results = search_memos(&storage, SearchRequest {
            query: "Test".to_string(),
            limit: Some(10),
        }).await.unwrap();
        assert_eq!(results.len(), 1);
        
        // Test deleting
        let deleted = delete_memo(&storage, &memo.id).await.unwrap();
        assert!(deleted);
        
        let remaining = list_memos(&storage).await.unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[tokio::test]
    async fn test_memo_content_analysis() {
        let memo = Memo {
            id: "01TEST123456789012345678".to_string(),
            title: "Test Memo".to_string(),
            content: "# Header 1\n\nSome content\n\n## Header 2\n\n- [ ] Task 1\n- [x] Task 2".to_string(),
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