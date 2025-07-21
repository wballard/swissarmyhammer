// THIS IS A SCRATCH FILE
use swissarmyhammer::issues::{FileSystemIssueStorage, IssueStorage};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let issues_dir = current_dir.join("issues");
    
    println!("Issues directory: {:?}", issues_dir);
    println!("Issues directory exists: {}", issues_dir.exists());
    
    let completed_dir = issues_dir.join("complete");
    println!("Complete directory: {:?}", completed_dir);
    println!("Complete directory exists: {}", completed_dir.exists());
    
    let storage = FileSystemIssueStorage::new(issues_dir)?;
    
    println!("\n--- Testing get_next_issue_number ---");
    let next_number = storage.get_next_issue_number()?;
    println!("Next issue number would be: {:06}", next_number);
    
    println!("\n--- Listing all issues ---");
    let all_issues = storage.list_issues().await?;
    
    let mut numbered_issues: Vec<_> = all_issues.iter()
        .filter(|issue| issue.number.value() < 500000) // Only explicitly numbered issues
        .collect();
    numbered_issues.sort_by_key(|issue| issue.number.value());
    
    println!("Found {} numbered issues:", numbered_issues.len());
    for issue in &numbered_issues {
        println!("  {} - {} ({})", 
                 format!("{:06}", issue.number.value()),
                 issue.name.as_str(),
                 issue.file_path.display());
    }
    
    if let Some(highest) = numbered_issues.last() {
        println!("\nHighest numbered issue: {:06}", highest.number.value());
        println!("Next number should be: {:06}", highest.number.value() + 1);
    }
    
    Ok(())
}