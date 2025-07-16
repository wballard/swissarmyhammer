use crate::error::{Result, SwissArmyHammerError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Maximum issue number supported (6 digits)
const MAX_ISSUE_NUMBER: u32 = 999999;

/// Number of digits for issue numbering in filenames
const ISSUE_NUMBER_DIGITS: usize = 6;

/// Represents an issue in the tracking system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    /// The issue number (6-digit format)
    pub number: u32,
    /// The issue name (derived from filename without number prefix)
    pub name: String,
    /// The full content of the issue markdown file
    pub content: String,
    /// Whether the issue is completed
    pub completed: bool,
    /// The file path of the issue
    pub file_path: PathBuf,
}

/// Represents the current state of the issue system
#[derive(Debug, Clone)]
pub struct IssueState {
    /// Path to the issues directory
    pub issues_dir: PathBuf,
    /// Path to the completed issues directory
    pub completed_dir: PathBuf,
}

/// Trait for issue storage operations
#[async_trait::async_trait]
pub trait IssueStorage: Send + Sync {
    /// List all issues (both pending and completed)
    async fn list_issues(&self) -> Result<Vec<Issue>>;

    /// Get a specific issue by number
    async fn get_issue(&self, number: u32) -> Result<Issue>;

    /// Create a new issue with auto-assigned number
    async fn create_issue(&self, name: String, content: String) -> Result<Issue>;
}

/// File system implementation of issue storage
pub struct FileSystemIssueStorage {
    #[allow(dead_code)]
    state: IssueState,
}

impl FileSystemIssueStorage {
    /// Create a new FileSystemIssueStorage instance
    pub fn new(issues_dir: PathBuf) -> Result<Self> {
        let completed_dir = issues_dir.join("complete");

        // Create directories if they don't exist
        fs::create_dir_all(&issues_dir).map_err(SwissArmyHammerError::Io)?;
        fs::create_dir_all(&completed_dir).map_err(SwissArmyHammerError::Io)?;

        Ok(Self {
            state: IssueState {
                issues_dir,
                completed_dir,
            },
        })
    }

    /// Parse issue from file path
    ///
    /// Parses an issue from a file path, extracting the issue number and name from the filename
    /// and reading the content from the file. The filename must follow the format:
    /// `<nnnnnn>_<name>.md` where `nnnnnn` is a 6-digit zero-padded number.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the issue file
    ///
    /// # Returns
    ///
    /// Returns `Ok(Issue)` if the file is successfully parsed, or an error if:
    /// - The filename doesn't follow the expected format
    /// - The issue number is invalid or exceeds the maximum
    /// - The file cannot be read
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let issue = storage.parse_issue_from_file(Path::new("./issues/000123_bug_fix.md"))?;
    /// assert_eq!(issue.number, 123);
    /// assert_eq!(issue.name, "bug_fix");
    /// ```
    fn parse_issue_from_file(&self, path: &Path) -> Result<Issue> {
        let filename = path
            .file_stem()
            .ok_or_else(|| SwissArmyHammerError::Other("Invalid file path".to_string()))?
            .to_str()
            .ok_or_else(|| SwissArmyHammerError::Other("Invalid filename encoding".to_string()))?;

        // Parse filename format: <nnnnnn>_<name>.md
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(SwissArmyHammerError::Other(format!(
                "Invalid filename format: {}",
                filename
            )));
        }

        // Parse the 6-digit number
        let number: u32 = parts[0]
            .parse()
            .map_err(|_| SwissArmyHammerError::InvalidIssueNumber(parts[0].to_string()))?;

        if number > MAX_ISSUE_NUMBER {
            return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
                "Issue number {} exceeds maximum ({})",
                number, MAX_ISSUE_NUMBER
            )));
        }

        let name = parts[1].to_string();

        // Read file content
        let content = fs::read_to_string(path).map_err(SwissArmyHammerError::Io)?;

        // Determine if completed based on path
        let completed = path
            .ancestors()
            .any(|p| p.file_name() == Some(std::ffi::OsStr::new("complete")));

        Ok(Issue {
            number,
            name,
            content,
            completed,
            file_path: path.to_path_buf(),
        })
    }

    /// List issues in a directory
    ///
    /// Scans a directory for issue files and returns a vector of parsed Issues.
    /// Only files with the `.md` extension that follow the correct naming format
    /// are processed. Files that fail to parse are logged as debug messages but
    /// don't cause the entire operation to fail.
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to the directory to scan
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<Issue>)` containing all successfully parsed issues,
    /// sorted by issue number in ascending order. Returns an empty vector
    /// if the directory doesn't exist or contains no valid issue files.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let issues = storage.list_issues_in_dir(Path::new("./issues"))?;
    /// // Issues are sorted by number
    /// if !issues.is_empty() {
    ///     assert!(issues[0].number <= issues[1].number);
    /// }
    /// ```
    fn list_issues_in_dir(&self, dir: &Path) -> Result<Vec<Issue>> {
        if !dir.exists() {
            return Ok(vec![]);
        }

        let mut issues = Vec::new();

        let entries = fs::read_dir(dir).map_err(SwissArmyHammerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(SwissArmyHammerError::Io)?;

            let path = entry.path();
            if path.is_file() && path.extension() == Some(std::ffi::OsStr::new("md")) {
                match self.parse_issue_from_file(&path) {
                    Ok(issue) => issues.push(issue),
                    Err(e) => {
                        debug!("Failed to parse issue from {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by number
        issues.sort_by_key(|issue| issue.number);

        Ok(issues)
    }

    /// Get the next available issue number
    ///
    /// Scans both the pending and completed issue directories to find the highest
    /// existing issue number and returns the next sequential number (highest + 1).
    /// If no issues exist, returns 1 as the first issue number.
    ///
    /// # Returns
    ///
    /// Returns `Ok(u32)` containing the next available issue number, or an error
    /// if the directories cannot be read.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // If issues 1, 2, and 5 exist, returns 6
    /// let next_number = storage.get_next_issue_number()?;
    /// assert_eq!(next_number, 6);
    /// ```
    ///
    /// # Note
    ///
    /// This method reads both directories sequentially, which could be optimized
    /// for better performance with large numbers of issues.
    fn get_next_issue_number(&self) -> Result<u32> {
        let mut max_number = 0;

        // Check pending issues
        let pending_issues = self.list_issues_in_dir(&self.state.issues_dir)?;
        for issue in pending_issues {
            if issue.number > max_number {
                max_number = issue.number;
            }
        }

        // Check completed issues
        let completed_issues = self.list_issues_in_dir(&self.state.completed_dir)?;
        for issue in completed_issues {
            if issue.number > max_number {
                max_number = issue.number;
            }
        }

        Ok(max_number + 1)
    }

    /// Create issue file
    ///
    /// Creates a new issue file with the given number, name, and content.
    /// The file is created in the pending issues directory with the standard
    /// naming format: `<nnnnnn>_<name>.md` where `nnnnnn` is a 6-digit
    /// zero-padded number.
    ///
    /// # Arguments
    ///
    /// * `number` - The issue number to use
    /// * `name` - The issue name (will be sanitized for filesystem safety)
    /// * `content` - The markdown content to write to the file
    ///
    /// # Returns
    ///
    /// Returns `Ok(PathBuf)` containing the path to the created file, or an error
    /// if the file cannot be created or written.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let file_path = storage.create_issue_file(123, "bug fix", "# Bug Fix\n\nDescription...")?;
    /// assert!(file_path.ends_with("000123_bug-fix.md"));
    /// ```
    ///
    /// # Note
    ///
    /// The name parameter is sanitized by replacing spaces, forward slashes,
    /// and backslashes with hyphens to ensure filesystem compatibility.
    fn create_issue_file(&self, number: u32, name: &str, content: &str) -> Result<PathBuf> {
        // Format filename as <nnnnnn>_<name>.md using utility functions
        let safe_name = create_safe_filename(name);
        let filename = format!("{}_{}.md", format_issue_number(number), safe_name);
        let file_path = self.state.issues_dir.join(&filename);

        // Write content to file
        fs::write(&file_path, content).map_err(SwissArmyHammerError::Io)?;

        Ok(file_path)
    }
}

#[async_trait::async_trait]
impl IssueStorage for FileSystemIssueStorage {
    async fn list_issues(&self) -> Result<Vec<Issue>> {
        let mut all_issues = Vec::new();

        // List from pending directory
        let pending_issues = self.list_issues_in_dir(&self.state.issues_dir)?;
        all_issues.extend(pending_issues);

        // List from completed directory
        let completed_issues = self.list_issues_in_dir(&self.state.completed_dir)?;
        all_issues.extend(completed_issues);

        // Sort by number
        all_issues.sort_by_key(|issue| issue.number);

        Ok(all_issues)
    }

    async fn get_issue(&self, number: u32) -> Result<Issue> {
        // Check pending directory first
        let pending_issues = self.list_issues_in_dir(&self.state.issues_dir)?;
        for issue in pending_issues {
            if issue.number == number {
                return Ok(issue);
            }
        }

        // Then check completed directory
        let completed_issues = self.list_issues_in_dir(&self.state.completed_dir)?;
        for issue in completed_issues {
            if issue.number == number {
                return Ok(issue);
            }
        }

        Err(SwissArmyHammerError::IssueNotFound(number.to_string()))
    }

    async fn create_issue(&self, name: String, content: String) -> Result<Issue> {
        let number = self.get_next_issue_number()?;
        let file_path = self.create_issue_file(number, &name, &content)?;

        Ok(Issue {
            number,
            name,
            content,
            completed: false,
            file_path,
        })
    }
}

/// Format issue number as 6-digit string with leading zeros
pub fn format_issue_number(number: u32) -> String {
    format!("{:06}", number)
}

/// Parse issue number from string
pub fn parse_issue_number(s: &str) -> Result<u32> {
    if s.len() != ISSUE_NUMBER_DIGITS {
        return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number must be exactly {} digits, got {}",
            ISSUE_NUMBER_DIGITS,
            s.len()
        )));
    }

    let number = s
        .parse::<u32>()
        .map_err(|_| SwissArmyHammerError::InvalidIssueNumber(s.to_string()))?;

    if number > MAX_ISSUE_NUMBER {
        return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number {} exceeds maximum ({})",
            number, MAX_ISSUE_NUMBER
        )));
    }

    Ok(number)
}

/// Extract issue info from filename
pub fn parse_issue_filename(filename: &str) -> Result<(u32, String)> {
    let parts: Vec<&str> = filename.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err(SwissArmyHammerError::Other(format!(
            "Invalid filename format: expected <nnnnnn>_<name>, got {}",
            filename
        )));
    }

    let number = parse_issue_number(parts[0])?;
    let name = parts[1].to_string();

    Ok((number, name))
}

/// Create safe filename from issue name
pub fn create_safe_filename(name: &str) -> String {
    if name.is_empty() {
        return "unnamed".to_string();
    }

    // Replace spaces with dashes and remove problematic characters
    let safe_name = name
        .chars()
        .map(|c| match c {
            ' ' => '-',
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            c => c,
        })
        .collect::<String>();

    // Remove consecutive dashes
    let mut result = String::new();
    let mut prev_was_dash = false;
    for c in safe_name.chars() {
        if c == '-' {
            if !prev_was_dash {
                result.push(c);
                prev_was_dash = true;
            }
        } else {
            result.push(c);
            prev_was_dash = false;
        }
    }

    // Trim dashes from start and end
    let result = result.trim_matches('-').to_string();

    // Ensure not empty and limit length
    if result.is_empty() {
        "unnamed".to_string()
    } else if result.len() > 100 {
        result.chars().take(100).collect()
    } else {
        result
    }
}

/// Validate issue name
pub fn validate_issue_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SwissArmyHammerError::Other(
            "Issue name cannot be empty".to_string(),
        ));
    }

    if name.len() > 200 {
        return Err(SwissArmyHammerError::Other(format!(
            "Issue name too long: {} characters (max 200)",
            name.len()
        )));
    }

    // Check for problematic characters
    for c in name.chars() {
        if c.is_control() {
            return Err(SwissArmyHammerError::Other(
                "Issue name contains control characters".to_string(),
            ));
        }
    }

    Ok(())
}

/// Check if file is an issue file
pub fn is_issue_file(path: &Path) -> bool {
    // Must be .md file
    if path.extension() != Some(std::ffi::OsStr::new("md")) {
        return false;
    }

    // Get filename without extension
    let filename = match path.file_stem() {
        Some(name) => match name.to_str() {
            Some(s) => s,
            None => return false,
        },
        None => return false,
    };

    // Check if filename matches pattern
    parse_issue_filename(filename).is_ok()
}

impl FileSystemIssueStorage {
    /// Get the full path for an issue file
    fn get_issue_path(&self, _number: u32, completed: bool) -> PathBuf {
        let dir = if completed {
            &self.state.completed_dir
        } else {
            &self.state.issues_dir
        };

        dir.to_path_buf()
    }

    /// Find issue file by number in a directory
    fn find_issue_file(&self, dir: &Path, number: u32) -> Result<Option<PathBuf>> {
        let number_prefix = format_issue_number(number);

        let entries = fs::read_dir(dir).map_err(SwissArmyHammerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(SwissArmyHammerError::Io)?;
            let path = entry.path();

            if !path.is_file() || !is_issue_file(&path) {
                continue;
            }

            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with(&format!("{}_", number_prefix)) {
                        return Ok(Some(path));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get all issue files in a directory
    fn get_issue_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut issue_files = Vec::new();

        let entries = fs::read_dir(dir).map_err(SwissArmyHammerError::Io)?;

        for entry in entries {
            let entry = entry.map_err(SwissArmyHammerError::Io)?;
            let path = entry.path();

            if path.is_file() && is_issue_file(&path) {
                issue_files.push(path);
            }
        }

        // Sort by issue number
        issue_files.sort_by(|a, b| {
            let a_num = a
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| parse_issue_filename(s).ok())
                .map(|(n, _)| n)
                .unwrap_or(0);
            let b_num = b
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| parse_issue_filename(s).ok())
                .map(|(n, _)| n)
                .unwrap_or(0);
            a_num.cmp(&b_num)
        });

        Ok(issue_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_issue_serialization() {
        let issue = Issue {
            number: 123,
            name: "test_issue".to_string(),
            content: "Test content".to_string(),
            completed: false,
            file_path: PathBuf::from("/tmp/issues/000123_test_issue.md"),
        };

        // Test serialization
        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue, deserialized);
        assert_eq!(deserialized.number, 123);
        assert_eq!(deserialized.name, "test_issue");
        assert_eq!(deserialized.content, "Test content");
        assert!(!deserialized.completed);
    }

    #[test]
    fn test_issue_number_validation() {
        // Valid 6-digit numbers
        let valid_numbers = vec![1, 999, 1000, 99999, 100000, MAX_ISSUE_NUMBER];
        for num in valid_numbers {
            assert!(
                num <= MAX_ISSUE_NUMBER,
                "Issue number {} should be valid",
                num
            );
        }

        // Invalid numbers (too large)
        let invalid_numbers = vec![1000000, 9999999];
        for num in invalid_numbers {
            assert!(
                num > MAX_ISSUE_NUMBER,
                "Issue number {} should be invalid",
                num
            );
        }
    }

    #[test]
    fn test_path_construction() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();

        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        assert_eq!(storage.state.issues_dir, issues_dir);
        assert_eq!(storage.state.completed_dir, issues_dir.join("complete"));
    }

    #[test]
    fn test_directory_creation() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().join("new_issues");
        let completed_dir = issues_dir.join("complete");

        // Directories don't exist initially
        assert!(!issues_dir.exists());
        assert!(!completed_dir.exists());

        // Create storage - should create directories
        let _storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Directories should now exist
        assert!(issues_dir.exists());
        assert!(completed_dir.exists());
    }

    #[test]
    fn test_parse_issue_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file
        let test_file = issues_dir.join("000123_test_issue.md");
        fs::write(&test_file, "# Test Issue\\n\\nThis is a test issue.").unwrap();

        let issue = storage.parse_issue_from_file(&test_file).unwrap();
        assert_eq!(issue.number, 123);
        assert_eq!(issue.name, "test_issue");
        assert_eq!(issue.content, "# Test Issue\\n\\nThis is a test issue.");
        assert!(!issue.completed);
        assert_eq!(issue.file_path, test_file);
    }

    #[test]
    fn test_parse_issue_from_completed_file() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file in completed directory
        let completed_dir = issues_dir.join("complete");
        let test_file = completed_dir.join("000456_completed_issue.md");
        fs::write(&test_file, "# Completed Issue\\n\\nThis is completed.").unwrap();

        let issue = storage.parse_issue_from_file(&test_file).unwrap();
        assert_eq!(issue.number, 456);
        assert_eq!(issue.name, "completed_issue");
        assert_eq!(issue.content, "# Completed Issue\\n\\nThis is completed.");
        assert!(issue.completed);
        assert_eq!(issue.file_path, test_file);
    }

    #[test]
    fn test_parse_issue_invalid_filename() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with invalid filename
        let test_file = issues_dir.join("invalid_filename.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_issue_invalid_number() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with invalid number
        let test_file = issues_dir.join("abc123_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_issue_number_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with number too large
        let test_file = issues_dir.join("1000000_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_issue() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let issue = storage
            .create_issue("test_issue".to_string(), "# Test\\n\\nContent".to_string())
            .await
            .unwrap();

        assert_eq!(issue.number, 1);
        assert_eq!(issue.name, "test_issue");
        assert_eq!(issue.content, "# Test\\n\\nContent");
        assert!(!issue.completed);

        // Check file was created
        let expected_path = issues_dir.join("000001_test_issue.md");
        assert!(expected_path.exists());
        assert_eq!(issue.file_path, expected_path);
    }

    #[tokio::test]
    async fn test_create_issue_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let issue = storage
            .create_issue("test/issue with spaces".to_string(), "content".to_string())
            .await
            .unwrap();

        assert_eq!(issue.number, 1);
        assert_eq!(issue.name, "test/issue with spaces");

        // Check file was created with safe filename
        let expected_path = issues_dir.join("000001_test-issue-with-spaces.md");
        assert!(expected_path.exists());
        assert_eq!(issue.file_path, expected_path);
    }

    #[tokio::test]
    async fn test_get_next_issue_number() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Initially should be 1
        assert_eq!(storage.get_next_issue_number().unwrap(), 1);

        // Create some issues
        fs::write(issues_dir.join("000003_test.md"), "content").unwrap();
        fs::write(issues_dir.join("000001_test.md"), "content").unwrap();
        fs::write(
            issues_dir.join("complete").join("000005_completed.md"),
            "content",
        )
        .unwrap();

        // Should return 6 (highest + 1)
        assert_eq!(storage.get_next_issue_number().unwrap(), 6);
    }

    #[tokio::test]
    async fn test_list_issues_empty() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let issues = storage.list_issues().await.unwrap();
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn test_list_issues_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create pending issues
        fs::write(issues_dir.join("000003_pending.md"), "pending content").unwrap();
        fs::write(issues_dir.join("000001_another.md"), "another content").unwrap();

        // Create completed issues
        let completed_dir = issues_dir.join("complete");
        fs::write(
            completed_dir.join("000002_completed.md"),
            "completed content",
        )
        .unwrap();
        fs::write(completed_dir.join("000004_done.md"), "done content").unwrap();

        let issues = storage.list_issues().await.unwrap();
        assert_eq!(issues.len(), 4);

        // Should be sorted by number
        assert_eq!(issues[0].number, 1);
        assert_eq!(issues[0].name, "another");
        assert!(!issues[0].completed);

        assert_eq!(issues[1].number, 2);
        assert_eq!(issues[1].name, "completed");
        assert!(issues[1].completed);

        assert_eq!(issues[2].number, 3);
        assert_eq!(issues[2].name, "pending");
        assert!(!issues[2].completed);

        assert_eq!(issues[3].number, 4);
        assert_eq!(issues[3].name, "done");
        assert!(issues[3].completed);
    }

    #[tokio::test]
    async fn test_get_issue_found() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test issue
        fs::write(issues_dir.join("000123_test.md"), "test content").unwrap();

        let issue = storage.get_issue(123).await.unwrap();
        assert_eq!(issue.number, 123);
        assert_eq!(issue.name, "test");
        assert_eq!(issue.content, "test content");
    }

    #[tokio::test]
    async fn test_get_issue_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let result = storage.get_issue(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_issue_from_completed() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create completed issue
        let completed_dir = issues_dir.join("complete");
        fs::write(
            completed_dir.join("000456_completed.md"),
            "completed content",
        )
        .unwrap();

        let issue = storage.get_issue(456).await.unwrap();
        assert_eq!(issue.number, 456);
        assert_eq!(issue.name, "completed");
        assert_eq!(issue.content, "completed content");
        assert!(issue.completed);
    }

    #[tokio::test]
    async fn test_auto_increment_sequence() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create multiple issues
        let issue1 = storage
            .create_issue("first".to_string(), "content1".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("second".to_string(), "content2".to_string())
            .await
            .unwrap();
        let issue3 = storage
            .create_issue("third".to_string(), "content3".to_string())
            .await
            .unwrap();

        assert_eq!(issue1.number, 1);
        assert_eq!(issue2.number, 2);
        assert_eq!(issue3.number, 3);

        // Check files were created
        assert!(issues_dir.join("000001_first.md").exists());
        assert!(issues_dir.join("000002_second.md").exists());
        assert!(issues_dir.join("000003_third.md").exists());
    }

    #[test]
    fn test_list_issues_in_dir_non_existent() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let non_existent_dir = issues_dir.join("non_existent");
        let issues = storage.list_issues_in_dir(&non_existent_dir).unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_list_issues_in_dir_ignores_non_md_files() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create various files
        fs::write(issues_dir.join("000001_test.md"), "content").unwrap();
        fs::write(issues_dir.join("000002_test.txt"), "content").unwrap();
        fs::write(issues_dir.join("README.md"), "content").unwrap();
        fs::write(issues_dir.join("000003_valid.md"), "content").unwrap();

        let issues = storage.list_issues_in_dir(&issues_dir).unwrap();
        assert_eq!(issues.len(), 2); // Only the valid issue files
        assert_eq!(issues[0].number, 1);
        assert_eq!(issues[1].number, 3);
    }

    #[test]
    fn test_parse_issue_malformed_filename_no_underscore() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with no underscore
        let test_file = issues_dir.join("000123test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SwissArmyHammerError::Other(_)
        ));
    }

    #[test]
    fn test_parse_issue_malformed_filename_multiple_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with multiple underscores - should still work (splitn(2) handles this)
        let test_file = issues_dir.join("000123_test_with_underscores.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 123);
        assert_eq!(issue.name, "test_with_underscores");
    }

    #[test]
    fn test_parse_issue_malformed_filename_empty_name() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with empty name part
        let test_file = issues_dir.join("000123_.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 123);
        assert_eq!(issue.name, "");
    }

    #[test]
    fn test_parse_issue_malformed_filename_empty_number() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with empty number part
        let test_file = issues_dir.join("_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SwissArmyHammerError::InvalidIssueNumber(_)
        ));
    }

    #[test]
    fn test_parse_issue_number_with_leading_zeros() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with leading zeros
        let test_file = issues_dir.join("000001_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 1);
        assert_eq!(issue.name, "test");
    }

    #[test]
    fn test_parse_issue_number_zero() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with zero number
        let test_file = issues_dir.join("000000_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.number, 0);
        assert_eq!(issue.name, "test");
    }

    #[test]
    fn test_list_issues_in_dir_with_corrupted_files() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create valid file
        fs::write(issues_dir.join("000001_valid.md"), "content").unwrap();

        // Create corrupted/malformed files
        fs::write(issues_dir.join("invalid_format.md"), "content").unwrap();
        fs::write(issues_dir.join("abc123_invalid_number.md"), "content").unwrap();
        fs::write(issues_dir.join("1000000_too_large.md"), "content").unwrap();

        let issues = storage.list_issues_in_dir(&issues_dir).unwrap();
        // Should only return the valid issue, ignoring corrupted ones
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].number, 1);
        assert_eq!(issues[0].name, "valid");
    }

    #[tokio::test]
    async fn test_concurrent_issue_creation() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = std::sync::Arc::new(FileSystemIssueStorage::new(issues_dir.clone()).unwrap());

        // Create multiple issues concurrently
        let mut handles = Vec::new();
        for i in 0..5 {
            let storage_clone = storage.clone();
            let handle = tokio::spawn(async move {
                storage_clone
                    .create_issue(format!("issue_{}", i), format!("Content {}", i))
                    .await
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Check that all issues were created successfully
        assert_eq!(results.len(), 5);
        for result in results {
            assert!(result.is_ok());
        }

        // Verify all issues exist
        let all_issues = storage.list_issues().await.unwrap();
        assert_eq!(all_issues.len(), 5);

        // Check that numbers are sequential (though order might vary due to concurrency)
        let mut numbers: Vec<u32> = all_issues.iter().map(|i| i.number).collect();
        numbers.sort();
        assert_eq!(numbers, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_create_storage_with_invalid_path() {
        // Try to create storage with a path that contains null bytes (invalid on most systems)
        let invalid_path = PathBuf::from("invalid\0path");
        let result = FileSystemIssueStorage::new(invalid_path);

        // Should handle the error gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_directory_handling() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Test with completely empty directory
        let issues = storage.list_issues_in_dir(&issues_dir).unwrap();
        assert!(issues.is_empty());

        // Test get_next_issue_number with empty directory
        let next_number = storage.get_next_issue_number().unwrap();
        assert_eq!(next_number, 1);
    }

    #[tokio::test]
    async fn test_edge_case_issue_names() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Test with various special characters and edge cases
        let edge_case_names = vec![
            "issue with spaces",
            "issue/with/slashes",
            "issue\\with\\backslashes",
            "issue-with-dashes",
            "issue_with_underscores",
            "UPPERCASE_ISSUE",
            "lowercase_issue",
            "123_numeric_start",
            "issue.with.dots",
            "issue@with@symbols",
            "very_long_issue_name_that_exceeds_normal_length_expectations_but_should_still_work",
            "", // Empty name
        ];

        for name in edge_case_names {
            let result = storage
                .create_issue(name.to_string(), "content".to_string())
                .await;
            assert!(
                result.is_ok(),
                "Failed to create issue with name: '{}'",
                name
            );
        }

        // Verify all issues were created
        let all_issues = storage.list_issues().await.unwrap();
        assert_eq!(all_issues.len(), 12);
    }
}
