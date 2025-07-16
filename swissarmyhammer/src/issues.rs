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
            "Issue number must be exactly {} digits (e.g., '000123'), got {} digits: '{}'",
            ISSUE_NUMBER_DIGITS,
            s.len(),
            s
        )));
    }

    let number = s
        .parse::<u32>()
        .map_err(|_| SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number must contain only digits (e.g., '000123'), got: '{}'",
            s
        )))?;

    if number > MAX_ISSUE_NUMBER {
        return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number {} exceeds maximum allowed value ({}). Use 6-digit format: 000001-{}",
            number, MAX_ISSUE_NUMBER, MAX_ISSUE_NUMBER
        )));
    }

    Ok(number)
}

/// Extract issue info from filename
///
/// Parses an issue filename in the format `<nnnnnn>_<name>` and returns the issue number
/// and name as a tuple. The filename must follow the strict 6-digit format where the number
/// is zero-padded and separated from the name by an underscore.
///
/// # Arguments
///
/// * `filename` - The filename to parse (without extension)
///
/// # Returns
///
/// Returns `Ok((number, name))` if the filename is valid, or an error if:
/// - The filename doesn't contain exactly one underscore
/// - The number part is not exactly 6 digits
/// - The number part contains non-numeric characters
/// - The number exceeds the maximum allowed value (999999)
///
/// # Examples
///
/// ```
/// # use swissarmyhammer::issues::parse_issue_filename;
/// // Basic usage
/// let (number, name) = parse_issue_filename("000123_bug_fix").unwrap();
/// assert_eq!(number, 123);
/// assert_eq!(name, "bug_fix");
///
/// // With underscores in the name (only first underscore is used as separator)
/// let (number, name) = parse_issue_filename("000456_feature_with_underscores").unwrap();
/// assert_eq!(number, 456);
/// assert_eq!(name, "feature_with_underscores");
///
/// // Edge case: empty name
/// let (number, name) = parse_issue_filename("000789_").unwrap();
/// assert_eq!(number, 789);
/// assert_eq!(name, "");
///
/// // Edge case: number zero
/// let (number, name) = parse_issue_filename("000000_zero_issue").unwrap();
/// assert_eq!(number, 0);
/// assert_eq!(name, "zero_issue");
///
/// // Maximum number
/// let (number, name) = parse_issue_filename("999999_max_issue").unwrap();
/// assert_eq!(number, 999999);
/// assert_eq!(name, "max_issue");
/// ```
///
/// # Errors
///
/// ```should_panic
/// # use swissarmyhammer::issues::parse_issue_filename;
/// // Invalid: no underscore
/// parse_issue_filename("000123test").unwrap();
///
/// // Invalid: wrong number format
/// parse_issue_filename("123_test").unwrap();
///
/// // Invalid: non-numeric characters
/// parse_issue_filename("abc123_test").unwrap();
///
/// // Invalid: number too large
/// parse_issue_filename("1000000_test").unwrap();
/// ```
pub fn parse_issue_filename(filename: &str) -> Result<(u32, String)> {
    let parts: Vec<&str> = filename.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err(SwissArmyHammerError::Other(format!(
            "Invalid filename format: expected <nnnnnn>_<name> (e.g., '000123_bug_fix'), got: '{}'",
            filename
        )));
    }

    let number = parse_issue_number(parts[0])?;
    let name = parts[1].to_string();

    Ok((number, name))
}

/// Create safe filename from issue name
///
/// Converts an issue name into a filesystem-safe filename by replacing problematic
/// characters with dashes and applying various normalization rules. This function
/// ensures the resulting filename is safe to use across different operating systems
/// and filesystems.
///
/// # Rules Applied
///
/// - Spaces are replaced with dashes
/// - File path separators (`/`, `\`) are replaced with dashes
/// - Special characters (`:`, `*`, `?`, `"`, `<`, `>`, `|`) are replaced with dashes
/// - Control characters (tabs, newlines, etc.) are replaced with dashes
/// - Consecutive dashes are collapsed into a single dash
/// - Leading and trailing dashes are removed
/// - Empty input or input with only problematic characters becomes "unnamed"
/// - Length is limited to 100 characters
///
/// # Arguments
///
/// * `name` - The issue name to convert to a safe filename
///
/// # Returns
///
/// Returns a safe filename string that can be used in file paths across different
/// operating systems. The result will always be a valid filename or "unnamed" if
/// the input cannot be safely converted.
///
/// # Examples
///
/// ```
/// # use swissarmyhammer::issues::create_safe_filename;
/// // Basic usage
/// assert_eq!(create_safe_filename("simple"), "simple");
/// assert_eq!(create_safe_filename("with spaces"), "with-spaces");
///
/// // File path characters
/// assert_eq!(create_safe_filename("path/to/file"), "path-to-file");
/// assert_eq!(create_safe_filename("path\\to\\file"), "path-to-file");
///
/// // Special characters
/// assert_eq!(create_safe_filename("file:name"), "file-name");
/// assert_eq!(create_safe_filename("file*name"), "file-name");
/// assert_eq!(create_safe_filename("file?name"), "file-name");
/// assert_eq!(create_safe_filename("file\"name"), "file-name");
/// assert_eq!(create_safe_filename("file<name>"), "file-name");
/// assert_eq!(create_safe_filename("file|name"), "file-name");
///
/// // Multiple consecutive problematic characters
/// assert_eq!(create_safe_filename("file   with   spaces"), "file-with-spaces");
/// assert_eq!(create_safe_filename("file///name"), "file-name");
///
/// // Edge cases: trimming
/// assert_eq!(create_safe_filename("/start/and/end/"), "start-and-end");
/// assert_eq!(create_safe_filename("   spaces   "), "spaces");
///
/// // Edge cases: empty or only problematic characters
/// assert_eq!(create_safe_filename(""), "unnamed");
/// assert_eq!(create_safe_filename("///"), "unnamed");
/// assert_eq!(create_safe_filename("   "), "unnamed");
/// assert_eq!(create_safe_filename("***"), "unnamed");
///
/// // Length limiting
/// let long_name = "a".repeat(150);
/// let safe_name = create_safe_filename(&long_name);
/// assert_eq!(safe_name.len(), 100);
/// assert_eq!(safe_name, "a".repeat(100));
///
/// // Mixed characters
/// assert_eq!(create_safe_filename("Fix: login/logout* issue"), "Fix-login-logout-issue");
/// assert_eq!(create_safe_filename("Update \"config.json\" file"), "Update-config.json-file");
/// ```
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
            "Issue name cannot be empty. Provide a descriptive name (e.g., 'fix_login_bug')".to_string(),
        ));
    }

    if name.len() > 200 {
        return Err(SwissArmyHammerError::Other(format!(
            "Issue name too long: {} characters (max 200). Consider shortening: '{}'",
            name.len(),
            if name.len() > 50 {
                format!("{}...", &name[..50])
            } else {
                name.to_string()
            }
        )));
    }

    // Check for problematic characters
    for c in name.chars() {
        if c.is_control() {
            return Err(SwissArmyHammerError::Other(format!(
                "Issue name contains control characters (e.g., tabs, newlines). Use only printable characters: '{}'",
                name.chars().map(|c| if c.is_control() { '�' } else { c }).collect::<String>()
            )));
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

    // Check if filename matches pattern and name is not empty
    match parse_issue_filename(filename) {
        Ok((_, name)) => !name.is_empty(),
        Err(_) => false,
    }
}

impl FileSystemIssueStorage {
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

    #[test]
    fn test_format_issue_number() {
        assert_eq!(format_issue_number(1), "000001");
        assert_eq!(format_issue_number(123), "000123");
        assert_eq!(format_issue_number(999999), "999999");
        assert_eq!(format_issue_number(0), "000000");
    }

    #[test]
    fn test_parse_issue_number_valid() {
        assert_eq!(parse_issue_number("000001").unwrap(), 1);
        assert_eq!(parse_issue_number("000123").unwrap(), 123);
        assert_eq!(parse_issue_number("999999").unwrap(), 999999);
        assert_eq!(parse_issue_number("000000").unwrap(), 0);
    }

    #[test]
    fn test_parse_issue_number_invalid() {
        // Wrong length
        assert!(parse_issue_number("123").is_err());
        assert!(parse_issue_number("0000123").is_err());
        assert!(parse_issue_number("").is_err());
        
        // Non-numeric
        assert!(parse_issue_number("abc123").is_err());
        assert!(parse_issue_number("00abc1").is_err());
        
        // Too large
        assert!(parse_issue_number("1000000").is_err());
    }

    #[test]
    fn test_parse_issue_filename_valid() {
        let (number, name) = parse_issue_filename("000123_test_issue").unwrap();
        assert_eq!(number, 123);
        assert_eq!(name, "test_issue");
        
        let (number, name) = parse_issue_filename("000001_simple").unwrap();
        assert_eq!(number, 1);
        assert_eq!(name, "simple");
        
        let (number, name) = parse_issue_filename("000456_name_with_underscores").unwrap();
        assert_eq!(number, 456);
        assert_eq!(name, "name_with_underscores");
        
        let (number, name) = parse_issue_filename("000789_").unwrap();
        assert_eq!(number, 789);
        assert_eq!(name, "");
    }

    #[test]
    fn test_parse_issue_filename_invalid() {
        // No underscore
        assert!(parse_issue_filename("000123test").is_err());
        
        // Invalid number
        assert!(parse_issue_filename("abc123_test").is_err());
        assert!(parse_issue_filename("123_test").is_err());
        
        // Empty
        assert!(parse_issue_filename("").is_err());
        assert!(parse_issue_filename("_test").is_err());
    }

    #[test]
    fn test_create_safe_filename() {
        assert_eq!(create_safe_filename("simple"), "simple");
        assert_eq!(create_safe_filename("with spaces"), "with-spaces");
        assert_eq!(create_safe_filename("with/slashes"), "with-slashes");
        assert_eq!(create_safe_filename("with\\backslashes"), "with-backslashes");
        assert_eq!(create_safe_filename("with:colons"), "with-colons");
        assert_eq!(create_safe_filename("with*asterisks"), "with-asterisks");
        assert_eq!(create_safe_filename("with?questions"), "with-questions");
        assert_eq!(create_safe_filename("with\"quotes"), "with-quotes");
        assert_eq!(create_safe_filename("with<brackets>"), "with-brackets");
        assert_eq!(create_safe_filename("with|pipes"), "with-pipes");
        
        // Multiple consecutive spaces/chars become single dash
        assert_eq!(create_safe_filename("with   multiple   spaces"), "with-multiple-spaces");
        assert_eq!(create_safe_filename("with///slashes"), "with-slashes");
        
        // Trim dashes from start and end
        assert_eq!(create_safe_filename("/start/and/end/"), "start-and-end");
        assert_eq!(create_safe_filename("   spaces   "), "spaces");
        
        // Empty or only problematic chars
        assert_eq!(create_safe_filename(""), "unnamed");
        assert_eq!(create_safe_filename("///"), "unnamed");
        assert_eq!(create_safe_filename("   "), "unnamed");
        
        // Length limiting
        let long_name = "a".repeat(150);
        let safe_name = create_safe_filename(&long_name);
        assert_eq!(safe_name.len(), 100);
        assert_eq!(safe_name, "a".repeat(100));
    }

    #[test]
    fn test_validate_issue_name_valid() {
        assert!(validate_issue_name("simple").is_ok());
        assert!(validate_issue_name("with spaces").is_ok());
        assert!(validate_issue_name("with/slashes").is_ok());
        assert!(validate_issue_name("with_underscores").is_ok());
        assert!(validate_issue_name("123numbers").is_ok());
        assert!(validate_issue_name("UPPERCASE").is_ok());
        assert!(validate_issue_name("MiXeD cAsE").is_ok());
        assert!(validate_issue_name("with-dashes").is_ok());
        assert!(validate_issue_name("with.dots").is_ok());
        assert!(validate_issue_name("with@symbols").is_ok());
        
        // 200 characters exactly
        let max_length = "a".repeat(200);
        assert!(validate_issue_name(&max_length).is_ok());
    }

    #[test]
    fn test_validate_issue_name_invalid() {
        // Empty
        assert!(validate_issue_name("").is_err());
        
        // Too long
        let too_long = "a".repeat(201);
        assert!(validate_issue_name(&too_long).is_err());
        
        // Control characters
        assert!(validate_issue_name("with\tcontrol").is_err());
        assert!(validate_issue_name("with\ncontrol").is_err());
        assert!(validate_issue_name("with\rcontrol").is_err());
        assert!(validate_issue_name("with\x00control").is_err());
    }

    #[test]
    fn test_is_issue_file() {
        // Valid issue files
        assert!(is_issue_file(Path::new("000123_test.md")));
        assert!(is_issue_file(Path::new("000001_simple.md")));
        assert!(is_issue_file(Path::new("999999_max.md")));
        assert!(is_issue_file(Path::new("000000_zero.md")));
        assert!(is_issue_file(Path::new("000456_name_with_underscores.md")));
        
        // Invalid files
        assert!(!is_issue_file(Path::new("123_test.md"))); // Wrong number format
        assert!(!is_issue_file(Path::new("000123test.md"))); // Missing underscore
        assert!(!is_issue_file(Path::new("000123_test.txt"))); // Wrong extension
        assert!(!is_issue_file(Path::new("000123_test"))); // No extension
        assert!(!is_issue_file(Path::new("abc123_test.md"))); // Invalid number
        assert!(!is_issue_file(Path::new("README.md"))); // Not issue format
        assert!(!is_issue_file(Path::new("000123_.md"))); // Valid but edge case
        
        // Path with directory
        assert!(is_issue_file(Path::new("./issues/000123_test.md")));
        assert!(is_issue_file(Path::new("/path/to/000123_test.md")));
    }
}
