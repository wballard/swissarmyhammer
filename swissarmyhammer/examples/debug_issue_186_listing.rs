use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create storage pointing to the actual issues directory
    let issues_dir = PathBuf::from("/Users/wballard/github/swissarmyhammer/issues");
    let storage = FileSystemIssueStorage::new(issues_dir)?;
    
    println!("=== Debugging Issue 186 Listing ===\n");
    
    // List all issues
    let all_issues = storage.list_issues().await?;
    
    println!("Total issues found: {}", all_issues.len());
    
    // Look specifically for issue 000186
    let issue_186 = all_issues.iter().find(|issue| {
        issue.name.contains("186") || 
        issue.file_path.file_stem().unwrap_or_default().to_str().unwrap_or("").contains("186")
    });
    
    if let Some(issue) = issue_186 {
        println!("\n🔍 Found Issue 186:");
        println!("  Name: '{}'", issue.name);
        println!("  File: {}", issue.file_path.display());
        println!("  Completed: {}", issue.completed);
        println!("  Created: {}", issue.created_at);
        println!("  Content preview: {}", 
                 issue.content.lines().take(2).collect::<Vec<_>>().join("\n"));
    } else {
        println!("\n❌ Issue 186 not found!");
    }
    
    // Filter to pending issues only
    let pending_issues: Vec<_> = all_issues.iter()
        .filter(|issue| !issue.completed)
        .collect();
    
    println!("\n📋 Pending issues: {}", pending_issues.len());
    for (i, issue) in pending_issues.iter().enumerate() {
        if i < 10 {  // Show first 10
            println!("  {}. {} ({})", i+1, issue.name, 
                     issue.file_path.file_name().unwrap().to_str().unwrap());
        }
    }
    if pending_issues.len() > 10 {
        println!("  ... and {} more", pending_issues.len() - 10);
    }
    
    // Show completed issues count
    let completed_count = all_issues.iter().filter(|issue| issue.completed).count();
    println!("\n✅ Completed issues: {completed_count}");
    
    // Check the next issue logic specifically
    if !pending_issues.is_empty() {
        let next_issue = pending_issues[0];
        println!("\n🎯 Next issue would be: '{}' ({})", 
                 next_issue.name, 
                 next_issue.file_path.file_name().unwrap().to_str().unwrap());
    } else {
        println!("\n🎯 No next issue - all completed!");
    }
    
    Ok(())
}