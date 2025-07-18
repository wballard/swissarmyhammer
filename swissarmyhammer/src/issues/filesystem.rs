use crate::error::{Result, SwissArmyHammerError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;
use chrono::{DateTime, Utc};

/// Maximum issue number supported (6 digits)
use crate::config::Config;

/// Type-safe wrapper for issue numbers to prevent mixing with other u32 values
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct IssueNumber(u32);

impl IssueNumber {
    /// Create a new issue number with validation
    pub fn new(number: u32) -> Result<Self> {
        if number > Config::global().max_issue_number {
            return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
                "Issue number {} exceeds maximum ({})",
                number,
                Config::global().max_issue_number
            )));
        }
        Ok(Self(number))
    }
    
    /// Get the raw u32 value
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for IssueNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06}", self.0)
    }
}

impl From<u32> for IssueNumber {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<IssueNumber> for u32 {
    fn from(value: IssueNumber) -> Self {
        value.0
    }
}

impl std::ops::Add<u32> for IssueNumber {
    type Output = IssueNumber;
    
    fn add(self, rhs: u32) -> Self::Output {
        IssueNumber(self.0 + rhs)
    }
}

/// Represents an issue in the tracking system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Issue {
    /// The issue number (6-digit format)
    pub number: IssueNumber,
    /// The issue name (derived from filename without number prefix)
    pub name: String,
    /// The full content of the issue markdown file
    pub content: String,
    /// Whether the issue is completed
    pub completed: bool,
    /// The file path of the issue
    pub file_path: PathBuf,
    /// When the issue was created
    pub created_at: DateTime<Utc>,
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

    /// Update an existing issue's content
    async fn update_issue(&self, number: u32, content: String) -> Result<Issue>;

    /// Mark an issue as complete (move to complete directory)
    async fn mark_complete(&self, number: u32) -> Result<Issue>;

    /// Batch operations for better performance
    /// Create multiple issues at once
    async fn create_issues_batch(&self, issues: Vec<(String, String)>) -> Result<Vec<Issue>>;
    
    /// Get multiple issues by their numbers
    async fn get_issues_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>>;
    
    /// Update multiple issues at once
    async fn update_issues_batch(&self, updates: Vec<(u32, String)>) -> Result<Vec<Issue>>;
    
    /// Mark multiple issues as complete
    async fn mark_complete_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>>;
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

    /// Create a new FileSystemIssueStorage instance with default directory
    /// 
    /// Uses current working directory joined with "issues" as the default location
    pub fn new_default() -> Result<Self> {
        let current_dir = std::env::current_dir().map_err(SwissArmyHammerError::Io)?;
        let issues_dir = current_dir.join("issues");
        Self::new(issues_dir)
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
    /// assert_eq!(issue.number, IssueNumber::from(123));
    /// assert_eq!(issue.name, "bug_fix");
    /// ```
    fn parse_issue_from_file(&self, path: &Path) -> Result<Issue> {
        let filename = path
            .file_stem()
            .ok_or_else(|| {
                SwissArmyHammerError::parsing_failed(
                    "file path",
                    &path.display().to_string(),
                    "no file stem",
                )
            })?
            .to_str()
            .ok_or_else(|| {
                SwissArmyHammerError::parsing_failed(
                    "filename",
                    &path.display().to_string(),
                    "invalid UTF-8 encoding",
                )
            })?;

        // Parse filename format: <nnnnnn>_<name>.md
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(SwissArmyHammerError::parsing_failed(
                "filename format",
                filename,
                "expected format: <nnnnnn>_<name>.md",
            ));
        }

        // Parse the 6-digit number
        let number: u32 = parts[0]
            .parse()
            .map_err(|_| SwissArmyHammerError::InvalidIssueNumber(parts[0].to_string()))?;

        if number > Config::global().max_issue_number {
            return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
                "Issue number {} exceeds maximum ({})",
                number,
                Config::global().max_issue_number
            )));
        }

        let name = parts[1].to_string();

        // Read file content
        let content = fs::read_to_string(path).map_err(SwissArmyHammerError::Io)?;

        // Determine if completed based on path
        let completed = path
            .ancestors()
            .any(|p| p.file_name() == Some(std::ffi::OsStr::new("complete")));

        // Get file creation time for created_at
        let created_at = path
            .metadata()
            .and_then(|m| m.created())
            .or_else(|_| path.metadata().and_then(|m| m.modified()))
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());

        Ok(Issue {
            number: IssueNumber::from(number),
            name,
            content,
            completed,
            file_path: path.to_path_buf(),
            created_at,
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
        // Check pending issues
        let pending_issues = self.list_issues_in_dir(&self.state.issues_dir)?;
        // Check completed issues
        let completed_issues = self.list_issues_in_dir(&self.state.completed_dir)?;

        // Combine both iterators and find the maximum issue number
        let max_number = pending_issues
            .iter()
            .chain(completed_issues.iter())
            .max_by_key(|issue| issue.number)
            .map(|issue| issue.number)
            .unwrap_or(IssueNumber::from(0));

        Ok((max_number + 1).into())
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

    /// Update issue content
    async fn update_issue_impl(&self, number: u32, content: String) -> Result<Issue> {
        debug!("Updating issue {}", number);

        // Find the issue file (check both directories)
        let issue = self.get_issue(number).await?;
        let path = &issue.file_path;

        // Atomic write using temp file and rename
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, &content).map_err(SwissArmyHammerError::Io)?;
        std::fs::rename(&temp_path, path).map_err(SwissArmyHammerError::Io)?;

        debug!(
            "Successfully updated issue {} at path {}",
            number,
            path.display()
        );
        Ok(Issue { content, ..issue })
    }

    /// Move issue between directories
    async fn move_issue(&self, number: u32, to_completed: bool) -> Result<Issue> {
        debug!(
            "Moving issue {} to {}",
            number,
            if to_completed { "completed" } else { "pending" }
        );

        // Find current issue
        let mut issue = self.get_issue(number).await?;

        // Check if already in target state
        if issue.completed == to_completed {
            debug!("Issue {} already in target state", number);
            return Ok(issue);
        }

        // Determine source and target paths
        let target_dir = if to_completed {
            &self.state.completed_dir
        } else {
            &self.state.issues_dir
        };

        // Create target path with same filename
        let filename = issue
            .file_path
            .file_name()
            .ok_or_else(|| SwissArmyHammerError::Other("Invalid file path".to_string()))?;
        let target_path = target_dir.join(filename);

        // Move file atomically
        std::fs::rename(&issue.file_path, &target_path).map_err(SwissArmyHammerError::Io)?;

        // Update issue struct
        issue.file_path = target_path.clone();
        issue.completed = to_completed;

        debug!(
            "Successfully moved issue {} to {}",
            number,
            target_path.display()
        );
        Ok(issue)
    }

    /// Check if all issues are completed
    pub async fn all_complete(&self) -> Result<bool> {
        let pending_issues = self.list_issues_in_dir(&self.state.issues_dir)?;
        let pending_count = pending_issues
            .into_iter()
            .filter(|issue| !issue.completed)
            .count();

        Ok(pending_count == 0)
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
        // Use existing list_issues() method to avoid duplicating search logic
        let all_issues = self.list_issues().await?;

        all_issues
            .into_iter()
            .find(|issue| issue.number == IssueNumber::from(number))
            .ok_or_else(|| SwissArmyHammerError::IssueNotFound(number.to_string()))
    }

    async fn create_issue(&self, name: String, content: String) -> Result<Issue> {
        let number = self.get_next_issue_number()?;
        let file_path = self.create_issue_file(number, &name, &content)?;
        let sanitized_name = sanitize_issue_name(&name);
        let created_at = Utc::now();

        Ok(Issue {
            number: IssueNumber::from(number),
            name: sanitized_name,
            content,
            completed: false,
            file_path,
            created_at,
        })
    }

    async fn update_issue(&self, number: u32, content: String) -> Result<Issue> {
        self.update_issue_impl(number, content).await
    }

    async fn mark_complete(&self, number: u32) -> Result<Issue> {
        self.move_issue(number, true).await
    }

    async fn create_issues_batch(&self, issues: Vec<(String, String)>) -> Result<Vec<Issue>> {
        let mut created_issues = Vec::new();
        
        for (name, content) in issues {
            let issue = self.create_issue(name, content).await?;
            created_issues.push(issue);
        }
        
        Ok(created_issues)
    }

    async fn get_issues_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>> {
        // First, verify all issues exist before returning any
        for number in &numbers {
            self.get_issue(*number).await?; // This will fail if issue doesn't exist
        }
        
        let mut issues = Vec::new();
        
        for number in numbers {
            let issue = self.get_issue(number).await?;
            issues.push(issue);
        }
        
        Ok(issues)
    }

    async fn update_issues_batch(&self, updates: Vec<(u32, String)>) -> Result<Vec<Issue>> {
        // First, verify all issues exist before updating any
        for (number, _) in &updates {
            self.get_issue(*number).await?; // This will fail if issue doesn't exist
        }
        
        let mut updated_issues = Vec::new();
        
        for (number, content) in updates {
            let issue = self.update_issue(number, content).await?;
            updated_issues.push(issue);
        }
        
        Ok(updated_issues)
    }

    async fn mark_complete_batch(&self, numbers: Vec<u32>) -> Result<Vec<Issue>> {
        // First, verify all issues exist before marking any complete
        for number in &numbers {
            self.get_issue(*number).await?; // This will fail if issue doesn't exist
        }
        
        let mut completed_issues = Vec::new();
        
        for number in numbers {
            let issue = self.mark_complete(number).await?;
            completed_issues.push(issue);
        }
        
        Ok(completed_issues)
    }
}

/// Format issue number as 6-digit string with leading zeros
pub fn format_issue_number(number: u32) -> String {
    format!("{number:06}")
}

/// Parse issue number from string
pub fn parse_issue_number(s: &str) -> Result<u32> {
    if s.len() != Config::global().issue_number_digits {
        return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number must be exactly {} digits (e.g., '000123'), got {} digits: '{}'",
            Config::global().issue_number_digits,
            s.len(),
            s
        )));
    }

    let number = s.parse::<u32>().map_err(|_| {
        SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number must contain only digits (e.g., '000123'), got: '{s}'"
        ))
    })?;

    if number > Config::global().max_issue_number {
        return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
            "Issue number {} exceeds maximum allowed value ({}). Use 6-digit format: 000001-{}",
            number,
            Config::global().max_issue_number,
            Config::global().max_issue_number
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
            "Invalid filename format: expected <nnnnnn>_<name> (e.g., '000123_bug_fix'), got: '{filename}'"
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

    // Configurable length limit
    let max_filename_length = std::env::var("SWISSARMYHAMMER_MAX_FILENAME_LENGTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    // Check for path traversal attempts
    if name.contains("../") || name.contains("..\\") || name.contains("./") || name.contains(".\\")
    {
        return "path_traversal_attempted".to_string();
    }

    // Replace spaces with dashes and remove problematic characters
    let safe_name = name
        .chars()
        .map(|c| match c {
            ' ' => '-',
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            c if c.is_control() => '-',
            // Additional security: replace null bytes and other dangerous characters
            '\0' | '\x01'..='\x1F' | '\x7F' => '-',
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
    let result = if result.is_empty() {
        "unnamed".to_string()
    } else if result.len() > max_filename_length {
        result.chars().take(max_filename_length).collect()
    } else {
        result
    };

    // Check for reserved filenames on different operating systems
    validate_against_reserved_names(&result)
}

/// Validate filename against reserved names on different operating systems
fn validate_against_reserved_names(name: &str) -> String {
    // Windows reserved names
    let windows_reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    // Unix/Linux reserved or problematic names
    let unix_reserved = [".", "..", "/", "\\"];

    let name_upper = name.to_uppercase();

    // Check Windows reserved names
    if windows_reserved.contains(&name_upper.as_str()) {
        return format!("{name}_file");
    }

    // Check Unix reserved names
    if unix_reserved.contains(&name) {
        return format!("{name}_file");
    }

    // Check for names that start with a dot (hidden files)
    if name.starts_with('.') && name.len() > 1 {
        return format!("hidden_{}", &name[1..]);
    }

    // Check for names that end with a dot (Windows issue)
    if name.ends_with('.') {
        return format!("{}_file", name.trim_end_matches('.'));
    }

    // Check for overly long names that might cause issues
    if name.len() > 255 {
        return name.chars().take(250).collect::<String>() + "_trunc";
    }

    name.to_string()
}

/// Sanitize issue name for security while preserving most names
pub fn sanitize_issue_name(name: &str) -> String {
    // Only sanitize dangerous path traversal attempts
    if name.contains("../") || name.contains("..\\") || name.contains("./") || name.contains(".\\") {
        return "path_traversal_attempted".to_string();
    }
    // Preserve all other names, including empty names
    name.to_string()
}

/// Validate issue name
pub fn validate_issue_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SwissArmyHammerError::Other(
            "Issue name cannot be empty. Provide a descriptive name (e.g., 'fix_login_bug')"
                .to_string(),
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

impl FileSystemIssueStorage {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test issue storage with temporary directory
    fn create_test_storage() -> (FileSystemIssueStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().join("issues");

        let storage = FileSystemIssueStorage::new(issues_dir).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_issue_serialization() {
        let created_at = Utc::now();
        let issue = Issue {
            number: IssueNumber::from(123),
            name: "test_issue".to_string(),
            content: "Test content".to_string(),
            completed: false,
            file_path: PathBuf::from("/tmp/issues/000123_test_issue.md"),
            created_at,
        };

        // Test serialization
        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: Issue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue, deserialized);
        assert_eq!(deserialized.number, IssueNumber::from(123));
        assert_eq!(deserialized.name, "test_issue");
        assert_eq!(deserialized.content, "Test content");
        assert!(!deserialized.completed);
        assert_eq!(deserialized.created_at, created_at);
    }

    #[test]
    fn test_issue_number_validation() {
        // Valid 6-digit numbers
        let valid_numbers = vec![
            1,
            999,
            1000,
            99999,
            100000,
            Config::global().max_issue_number,
        ];
        for num in valid_numbers {
            assert!(
                num <= Config::global().max_issue_number,
                "Issue number {} should be valid",
                num
            );
        }

        // Invalid numbers (too large)
        let invalid_numbers = vec![1000000, 9999999];
        for num in invalid_numbers {
            assert!(
                num > Config::global().max_issue_number,
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
        assert_eq!(issue.number, IssueNumber::from(123));
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
        assert_eq!(issue.number, IssueNumber::from(456));
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

        assert_eq!(issue.number, IssueNumber::from(1));
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

        assert_eq!(issue.number, IssueNumber::from(1));
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
        assert_eq!(issues[0].number, IssueNumber::from(1));
        assert_eq!(issues[0].name, "another");
        assert!(!issues[0].completed);

        assert_eq!(issues[1].number, IssueNumber::from(2));
        assert_eq!(issues[1].name, "completed");
        assert!(issues[1].completed);

        assert_eq!(issues[2].number, IssueNumber::from(3));
        assert_eq!(issues[2].name, "pending");
        assert!(!issues[2].completed);

        assert_eq!(issues[3].number, IssueNumber::from(4));
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
        assert_eq!(issue.number, IssueNumber::from(123));
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
        assert_eq!(issue.number, IssueNumber::from(456));
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

        assert_eq!(issue1.number, IssueNumber::from(1));
        assert_eq!(issue2.number, IssueNumber::from(2));
        assert_eq!(issue3.number, IssueNumber::from(3));

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
        assert_eq!(issues[0].number, IssueNumber::from(1));
        assert_eq!(issues[1].number, IssueNumber::from(3));
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
        assert_eq!(issue.number, IssueNumber::from(123));
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
        assert_eq!(issue.number, IssueNumber::from(123));
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
        assert_eq!(issue.number, IssueNumber::from(1));
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
        assert_eq!(issue.number, IssueNumber::from(0));
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
        assert_eq!(issues[0].number, IssueNumber::from(1));
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
        let mut numbers: Vec<u32> = all_issues.iter().map(|i| i.number.into()).collect();
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

    #[tokio::test]
    async fn test_update_issue() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create initial issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Original content".to_string())
            .await
            .unwrap();

        // Update the issue
        let updated_content = "Updated content with new information";
        let updated_issue = storage
            .update_issue(issue.number.into(), updated_content.to_string())
            .await
            .unwrap();

        assert_eq!(updated_issue.number, issue.number);
        assert_eq!(updated_issue.name, issue.name);
        assert_eq!(updated_issue.content, updated_content);
        assert_eq!(updated_issue.file_path, issue.file_path);
        assert_eq!(updated_issue.completed, issue.completed);

        // Verify file was updated
        let file_content = std::fs::read_to_string(&updated_issue.file_path).unwrap();
        assert_eq!(file_content, updated_content);
    }

    #[tokio::test]
    async fn test_update_issue_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let result = storage.update_issue(999, "New content".to_string()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mark_complete() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create initial issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Test content".to_string())
            .await
            .unwrap();

        assert!(!issue.completed);

        // Mark as complete
        let completed_issue = storage.mark_complete(issue.number.into()).await.unwrap();

        assert_eq!(completed_issue.number, issue.number);
        assert_eq!(completed_issue.name, issue.name);
        assert_eq!(completed_issue.content, issue.content);
        assert!(completed_issue.completed);

        // Verify file was moved to completed directory
        let expected_path = issues_dir.join("complete").join("000001_test_issue.md");
        assert_eq!(completed_issue.file_path, expected_path);
        assert!(expected_path.exists());
        assert!(!issue.file_path.exists());
    }

    #[tokio::test]
    async fn test_mark_complete_already_completed() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create and complete an issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Test content".to_string())
            .await
            .unwrap();

        let completed_issue = storage.mark_complete(issue.number.into()).await.unwrap();

        // Try to mark as complete again - should be no-op
        let completed_again = storage.mark_complete(issue.number.into()).await.unwrap();

        assert_eq!(completed_issue.file_path, completed_again.file_path);
        assert!(completed_again.completed);
    }

    #[tokio::test]
    async fn test_mark_complete_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let result = storage.mark_complete(999).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_all_complete_empty() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        let result = storage.all_complete().await.unwrap();
        assert!(result); // No issues means all are complete
    }

    #[tokio::test]
    async fn test_all_complete_with_pending() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create some issues
        storage
            .create_issue("issue1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        storage
            .create_issue("issue2".to_string(), "Content 2".to_string())
            .await
            .unwrap();

        let result = storage.all_complete().await.unwrap();
        assert!(!result); // Has pending issues
    }

    #[tokio::test]
    async fn test_all_complete_all_completed() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create and complete all issues
        let issue1 = storage
            .create_issue("issue1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("issue2".to_string(), "Content 2".to_string())
            .await
            .unwrap();

        storage.mark_complete(issue1.number.into()).await.unwrap();
        storage.mark_complete(issue2.number.into()).await.unwrap();

        let result = storage.all_complete().await.unwrap();
        assert!(result); // All issues are complete
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
        assert_eq!(
            create_safe_filename("with\\backslashes"),
            "with-backslashes"
        );
        assert_eq!(create_safe_filename("with:colons"), "with-colons");
        assert_eq!(create_safe_filename("with*asterisks"), "with-asterisks");
        assert_eq!(create_safe_filename("with?questions"), "with-questions");
        assert_eq!(create_safe_filename("with\"quotes"), "with-quotes");
        assert_eq!(create_safe_filename("with<brackets>"), "with-brackets");
        assert_eq!(create_safe_filename("with|pipes"), "with-pipes");

        // Multiple consecutive spaces/chars become single dash
        assert_eq!(
            create_safe_filename("with   multiple   spaces"),
            "with-multiple-spaces"
        );
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

    // Comprehensive tests for issue operations as specified in the issue
    #[tokio::test]
    async fn test_create_issue_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Create first issue
        let issue1 = storage
            .create_issue("test_issue".to_string(), "Test content".to_string())
            .await
            .unwrap();

        assert_eq!(issue1.number, IssueNumber::from(1));
        assert_eq!(issue1.name, "test_issue");
        assert_eq!(issue1.content, "Test content");
        assert!(!issue1.completed);

        // Create second issue - should auto-increment
        let issue2 = storage
            .create_issue("another_issue".to_string(), "More content".to_string())
            .await
            .unwrap();

        assert_eq!(issue2.number, IssueNumber::from(2));
    }

    #[tokio::test]
    async fn test_list_issues_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Initially empty
        let issues = storage.list_issues().await.unwrap();
        assert!(issues.is_empty());

        // Create some issues
        storage
            .create_issue("issue1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        storage
            .create_issue("issue2".to_string(), "Content 2".to_string())
            .await
            .unwrap();

        let issues = storage.list_issues().await.unwrap();
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].number, IssueNumber::from(1));
        assert_eq!(issues[1].number, IssueNumber::from(2));
    }

    #[tokio::test]
    async fn test_get_issue_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Create an issue
        let created = storage
            .create_issue("test_issue".to_string(), "Test content".to_string())
            .await
            .unwrap();

        // Get it back
        let retrieved = storage.get_issue(created.number.into()).await.unwrap();
        assert_eq!(retrieved.number, created.number);
        assert_eq!(retrieved.name, created.name);
        assert_eq!(retrieved.content, created.content);

        // Try to get non-existent issue
        let result = storage.get_issue(999).await;
        assert!(matches!(
            result,
            Err(SwissArmyHammerError::IssueNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_update_issue_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Create an issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Original content".to_string())
            .await
            .unwrap();

        // Update it
        let updated = storage
            .update_issue(issue.number.into(), "Updated content".to_string())
            .await
            .unwrap();

        assert_eq!(updated.number, issue.number);
        assert_eq!(updated.content, "Updated content");

        // Verify it's persisted
        let retrieved = storage.get_issue(issue.number.into()).await.unwrap();
        assert_eq!(retrieved.content, "Updated content");
    }

    #[tokio::test]
    async fn test_update_nonexistent_issue_comprehensive() {
        let (storage, _temp) = create_test_storage();

        let result = storage.update_issue(999, "Content".to_string()).await;
        assert!(matches!(
            result,
            Err(SwissArmyHammerError::IssueNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_mark_complete_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Create an issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Content".to_string())
            .await
            .unwrap();

        assert!(!issue.completed);

        // Mark it complete
        let completed = storage.mark_complete(issue.number.into()).await.unwrap();
        assert!(completed.completed);

        // Verify file was moved
        assert!(completed.file_path.to_string_lossy().contains("complete"));

        // Verify it appears in completed list
        let all_issues = storage.list_issues().await.unwrap();
        let completed_issues: Vec<_> = all_issues.iter().filter(|i| i.completed).collect();
        assert_eq!(completed_issues.len(), 1);
    }

    #[tokio::test]
    async fn test_mark_complete_idempotent_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Create and complete an issue
        let issue = storage
            .create_issue("test_issue".to_string(), "Content".to_string())
            .await
            .unwrap();

        storage.mark_complete(issue.number.into()).await.unwrap();

        // Mark complete again - should be idempotent
        let result = storage.mark_complete(issue.number.into()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().completed);
    }

    #[tokio::test]
    async fn test_all_complete_comprehensive() {
        let (storage, _temp) = create_test_storage();

        // Initially true (no issues)
        assert!(storage.all_complete().await.unwrap());

        // Create issues
        let issue1 = storage
            .create_issue("issue1".to_string(), "Content".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("issue2".to_string(), "Content".to_string())
            .await
            .unwrap();

        // Now false
        assert!(!storage.all_complete().await.unwrap());

        // Complete one
        storage.mark_complete(issue1.number.into()).await.unwrap();
        assert!(!storage.all_complete().await.unwrap());

        // Complete both
        storage.mark_complete(issue2.number.into()).await.unwrap();
        assert!(storage.all_complete().await.unwrap());
    }

    #[test]
    fn test_format_issue_number_comprehensive() {
        assert_eq!(format_issue_number(1), "000001");
        assert_eq!(format_issue_number(999999), "999999");
        assert_eq!(format_issue_number(42), "000042");
    }

    #[test]
    fn test_parse_issue_number_comprehensive() {
        assert_eq!(parse_issue_number("000001").unwrap(), 1);
        assert_eq!(parse_issue_number("999999").unwrap(), 999999);
        assert_eq!(parse_issue_number("000042").unwrap(), 42);

        // Invalid cases
        assert!(parse_issue_number("").is_err());
        assert!(parse_issue_number("abc").is_err());
        assert!(parse_issue_number("12345").is_err()); // Not 6 digits
    }

    #[test]
    fn test_parse_issue_filename_comprehensive() {
        let (num, name) = parse_issue_filename("000001_test_issue").unwrap();
        assert_eq!(num, 1);
        assert_eq!(name, "test_issue");

        let (num, name) = parse_issue_filename("000042_complex_name_with_underscores").unwrap();
        assert_eq!(num, 42);
        assert_eq!(name, "complex_name_with_underscores");

        // Invalid cases
        assert!(parse_issue_filename("no_number").is_err());
        assert!(parse_issue_filename("123_short").is_err());
    }

    #[test]
    fn test_create_safe_filename_comprehensive() {
        assert_eq!(create_safe_filename("simple"), "simple");
        assert_eq!(create_safe_filename("with spaces"), "with-spaces");
        assert_eq!(
            create_safe_filename("special/chars*removed"),
            "special-chars-removed"
        );
        assert_eq!(create_safe_filename("   trimmed   "), "trimmed");

        // Long names should be truncated
        let long_name = "a".repeat(200);
        let safe_name = create_safe_filename(&long_name);
        assert!(safe_name.len() <= 100);
    }

    #[test]
    fn test_create_safe_filename_security() {
        // Test path traversal protection
        assert_eq!(
            create_safe_filename("../etc/passwd"),
            "path_traversal_attempted"
        );
        assert_eq!(create_safe_filename("./config"), "path_traversal_attempted");
        assert_eq!(
            create_safe_filename("..\\windows\\system32"),
            "path_traversal_attempted"
        );

        // Test Windows reserved names
        assert_eq!(create_safe_filename("CON"), "CON_file");
        assert_eq!(create_safe_filename("PRN"), "PRN_file");
        assert_eq!(create_safe_filename("AUX"), "AUX_file");
        assert_eq!(create_safe_filename("NUL"), "NUL_file");
        assert_eq!(create_safe_filename("COM1"), "COM1_file");
        assert_eq!(create_safe_filename("LPT1"), "LPT1_file");

        // Test case insensitive Windows reserved names
        assert_eq!(create_safe_filename("con"), "con_file");
        assert_eq!(create_safe_filename("Com1"), "Com1_file");

        // Test Unix reserved names (when used as standalone names)
        assert_eq!(create_safe_filename("."), "._file");
        assert_eq!(create_safe_filename(".."), ".._file");

        // Test hidden files (starting with dot)
        assert_eq!(create_safe_filename(".hidden"), "hidden_hidden");
        assert_eq!(create_safe_filename(".gitignore"), "hidden_gitignore");

        // Test names ending with dot (Windows issue)
        assert_eq!(create_safe_filename("filename."), "filename_file");
        assert_eq!(create_safe_filename("test..."), "test_file");

        // Test null bytes and control characters
        assert_eq!(create_safe_filename("test\0null"), "test-null");
        assert_eq!(create_safe_filename("test\x01control"), "test-control");
        assert_eq!(create_safe_filename("test\x7Fdelete"), "test-delete");

        // Test very long names - gets truncated to max_filename_length (default 100)
        let very_long_name = "a".repeat(300);
        let safe_name = create_safe_filename(&very_long_name);
        assert_eq!(safe_name.len(), 100);
        assert_eq!(safe_name, "a".repeat(100));
    }

    #[tokio::test]
    async fn test_create_issues_batch() {
        let (storage, _temp) = create_test_storage();
        
        let batch_data = vec![
            ("issue_1".to_string(), "Content 1".to_string()),
            ("issue_2".to_string(), "Content 2".to_string()),
            ("issue_3".to_string(), "Content 3".to_string()),
        ];
        
        let issues = storage.create_issues_batch(batch_data).await.unwrap();
        
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].name, "issue_1");
        assert_eq!(issues[0].content, "Content 1");
        assert_eq!(issues[1].name, "issue_2");
        assert_eq!(issues[1].content, "Content 2");
        assert_eq!(issues[2].name, "issue_3");
        assert_eq!(issues[2].content, "Content 3");
        
        // Verify issues were actually created
        let all_issues = storage.list_issues().await.unwrap();
        assert_eq!(all_issues.len(), 3);
    }

    #[tokio::test]
    async fn test_create_issues_batch_empty() {
        let (storage, _temp) = create_test_storage();
        
        let batch_data = vec![];
        let issues = storage.create_issues_batch(batch_data).await.unwrap();
        
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_get_issues_batch() {
        let (storage, _temp) = create_test_storage();
        
        // Create some issues first
        let issue1 = storage.create_issue("issue_1".to_string(), "Content 1".to_string()).await.unwrap();
        let issue2 = storage.create_issue("issue_2".to_string(), "Content 2".to_string()).await.unwrap();
        let issue3 = storage.create_issue("issue_3".to_string(), "Content 3".to_string()).await.unwrap();
        
        let numbers = vec![issue1.number.value(), issue2.number.value(), issue3.number.value()];
        let retrieved_issues = storage.get_issues_batch(numbers).await.unwrap();
        
        assert_eq!(retrieved_issues.len(), 3);
        assert_eq!(retrieved_issues[0].number, issue1.number);
        assert_eq!(retrieved_issues[1].number, issue2.number);
        assert_eq!(retrieved_issues[2].number, issue3.number);
    }

    #[tokio::test]
    async fn test_get_issues_batch_empty() {
        let (storage, _temp) = create_test_storage();
        
        let numbers = vec![];
        let issues = storage.get_issues_batch(numbers).await.unwrap();
        
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_get_issues_batch_nonexistent() {
        let (storage, _temp) = create_test_storage();
        
        let numbers = vec![999, 1000, 1001];
        let result = storage.get_issues_batch(numbers).await;
        
        // Should fail because the issues don't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_issues_batch() {
        let (storage, _temp) = create_test_storage();
        
        // Create some issues first
        let issue1 = storage.create_issue("issue_1".to_string(), "Original 1".to_string()).await.unwrap();
        let issue2 = storage.create_issue("issue_2".to_string(), "Original 2".to_string()).await.unwrap();
        let issue3 = storage.create_issue("issue_3".to_string(), "Original 3".to_string()).await.unwrap();
        
        let updates = vec![
            (issue1.number.value(), "Updated 1".to_string()),
            (issue2.number.value(), "Updated 2".to_string()),
            (issue3.number.value(), "Updated 3".to_string()),
        ];
        
        let updated_issues = storage.update_issues_batch(updates).await.unwrap();
        
        assert_eq!(updated_issues.len(), 3);
        assert_eq!(updated_issues[0].content, "Updated 1");
        assert_eq!(updated_issues[1].content, "Updated 2");
        assert_eq!(updated_issues[2].content, "Updated 3");
        
        // Verify updates were persisted
        let retrieved_issue1 = storage.get_issue(issue1.number.value()).await.unwrap();
        assert_eq!(retrieved_issue1.content, "Updated 1");
    }

    #[tokio::test]
    async fn test_update_issues_batch_empty() {
        let (storage, _temp) = create_test_storage();
        
        let updates = vec![];
        let issues = storage.update_issues_batch(updates).await.unwrap();
        
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_update_issues_batch_nonexistent() {
        let (storage, _temp) = create_test_storage();
        
        let updates = vec![
            (999, "Updated 1".to_string()),
            (1000, "Updated 2".to_string()),
        ];
        
        let result = storage.update_issues_batch(updates).await;
        
        // Should fail because the issues don't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mark_complete_batch() {
        let (storage, _temp) = create_test_storage();
        
        // Create some issues first
        let issue1 = storage.create_issue("issue_1".to_string(), "Content 1".to_string()).await.unwrap();
        let issue2 = storage.create_issue("issue_2".to_string(), "Content 2".to_string()).await.unwrap();
        let issue3 = storage.create_issue("issue_3".to_string(), "Content 3".to_string()).await.unwrap();
        
        let numbers = vec![issue1.number.value(), issue2.number.value(), issue3.number.value()];
        let completed_issues = storage.mark_complete_batch(numbers).await.unwrap();
        
        assert_eq!(completed_issues.len(), 3);
        assert!(completed_issues[0].completed);
        assert!(completed_issues[1].completed);
        assert!(completed_issues[2].completed);
        
        // Verify issues were marked complete
        let retrieved_issue1 = storage.get_issue(issue1.number.value()).await.unwrap();
        assert!(retrieved_issue1.completed);
    }

    #[tokio::test]
    async fn test_mark_complete_batch_empty() {
        let (storage, _temp) = create_test_storage();
        
        let numbers = vec![];
        let issues = storage.mark_complete_batch(numbers).await.unwrap();
        
        assert_eq!(issues.len(), 0);
    }

    #[tokio::test]
    async fn test_mark_complete_batch_nonexistent() {
        let (storage, _temp) = create_test_storage();
        
        let numbers = vec![999, 1000, 1001];
        let result = storage.mark_complete_batch(numbers).await;
        
        // Should fail because the issues don't exist
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_operations_preserve_order() {
        let (storage, _temp) = create_test_storage();
        
        // Create issues in a specific order
        let batch_data = vec![
            ("alpha".to_string(), "First".to_string()),
            ("beta".to_string(), "Second".to_string()),
            ("gamma".to_string(), "Third".to_string()),
        ];
        
        let created_issues = storage.create_issues_batch(batch_data).await.unwrap();
        
        // Verify order is preserved
        assert_eq!(created_issues[0].name, "alpha");
        assert_eq!(created_issues[1].name, "beta");
        assert_eq!(created_issues[2].name, "gamma");
        
        // Get issues in different order
        let numbers = vec![
            created_issues[2].number.value(),
            created_issues[0].number.value(),
            created_issues[1].number.value(),
        ];
        
        let retrieved_issues = storage.get_issues_batch(numbers).await.unwrap();
        
        // Should preserve requested order
        assert_eq!(retrieved_issues[0].name, "gamma");
        assert_eq!(retrieved_issues[1].name, "alpha");
        assert_eq!(retrieved_issues[2].name, "beta");
    }

    #[tokio::test]
    async fn test_batch_operations_with_large_batches() {
        let (storage, _temp) = create_test_storage();
        
        // Create a large batch
        let batch_size = 100;
        let batch_data: Vec<(String, String)> = (1..=batch_size)
            .map(|i| (format!("issue_{}", i), format!("Content {}", i)))
            .collect();
        
        let created_issues = storage.create_issues_batch(batch_data).await.unwrap();
        assert_eq!(created_issues.len(), batch_size);
        
        // Get all issues in batch
        let numbers: Vec<u32> = created_issues.iter().map(|i| i.number.value()).collect();
        let retrieved_issues = storage.get_issues_batch(numbers.clone()).await.unwrap();
        assert_eq!(retrieved_issues.len(), batch_size);
        
        // Update all issues in batch
        let updates: Vec<(u32, String)> = created_issues.iter()
            .map(|i| (i.number.value(), format!("Updated {}", i.number.value())))
            .collect();
        let updated_issues = storage.update_issues_batch(updates).await.unwrap();
        assert_eq!(updated_issues.len(), batch_size);
        
        // Mark half complete in batch
        let half_numbers: Vec<u32> = numbers.iter().take(batch_size / 2).cloned().collect();
        let completed_issues = storage.mark_complete_batch(half_numbers).await.unwrap();
        assert_eq!(completed_issues.len(), batch_size / 2);
        
        // Verify final state
        let all_issues = storage.list_issues().await.unwrap();
        assert_eq!(all_issues.len(), batch_size);
        
        let completed_count = all_issues.iter().filter(|i| i.completed).count();
        assert_eq!(completed_count, batch_size / 2);
    }

    #[tokio::test]
    async fn test_batch_operations_partial_failure_behavior() {
        let (storage, _temp) = create_test_storage();
        
        // Create one issue
        let issue = storage.create_issue("existing".to_string(), "Content".to_string()).await.unwrap();
        
        // Try to get batch with mix of existing and non-existing issues
        let numbers = vec![issue.number.value(), 999, 1000];
        let result = storage.get_issues_batch(numbers).await;
        
        // Should fail entirely, not return partial results
        assert!(result.is_err());
        
        // Try to update batch with mix of existing and non-existing issues
        let updates = vec![
            (issue.number.value(), "Updated".to_string()),
            (999, "Should fail".to_string()),
        ];
        let result = storage.update_issues_batch(updates).await;
        
        // Should fail entirely
        assert!(result.is_err());
        
        // Verify original issue was not updated
        let retrieved_issue = storage.get_issue(issue.number.value()).await.unwrap();
        assert_eq!(retrieved_issue.content, "Content");
    }
}
