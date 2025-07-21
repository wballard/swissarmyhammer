// THIS IS A SCRATCH FILE
// Test for the real scenario with existing issue files

#[cfg(test)]
mod test_real_issue_scenario {
    use super::*;
    use crate::issues::{FileSystemIssueStorage, IssueStorage};
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_real_issue_scenario() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Simulate the real scenario:
        // Main directory: 000185, 000186, 000187, 000188
        fs::write(issues_dir.join("000185.md"), "issue 185").unwrap();
        fs::write(issues_dir.join("000186.md"), "issue 186").unwrap();
        fs::write(issues_dir.join("000187.md"), "issue 187").unwrap();
        fs::write(issues_dir.join("000188.md"), "issue 188").unwrap();

        // Complete directory: up to 000184
        let complete_dir = issues_dir.join("complete");
        for i in 1..=184 {
            fs::write(
                complete_dir.join(format!("{:06}.md", i)),
                format!("completed issue {}", i),
            )
            .unwrap();
        }

        // Test what the next issue number would be
        let next_number = storage.get_next_issue_number().unwrap();
        println!("Next issue number: {}", next_number);

        // Should be 189 (188 + 1)
        assert_eq!(next_number, 189, "Expected next issue number to be 189");

        // Test creating a new issue
        let new_issue = storage
            .create_issue("test_new_issue".to_string(), "New issue content".to_string())
            .await
            .unwrap();

        assert_eq!(new_issue.number.value(), 189);
        assert!(new_issue.file_path.ends_with("000189_test_new_issue.md"));
    }
}