use crate::error::{Result, SwissArmyHammerError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;
use tracing::debug;

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
    /// Mutex to ensure thread-safe issue creation and prevent race conditions
    /// when multiple threads attempt to create issues simultaneously
    creation_lock: Mutex<()>,
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
            creation_lock: Mutex::new(()),
        })
    }

    /// Create a new FileSystemIssueStorage instance with default directory
    ///
    /// Uses current working directory joined with "issues" as the default location
    pub fn new_default() -> Result<Self> {
        let current_dir = std::env::current_dir().unwrap_or_else(|e| {
            tracing::debug!("Failed to get current directory: {e}, using fallback");
            std::path::PathBuf::from(".")
        });
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

        // Parse filename - supports both numbered and non-numbered formats
        let (parsed_number, name) = parse_any_issue_filename(filename)?;

        let number = match parsed_number {
            Some(num) => {
                // Validate numbered files against the configured maximum
                if num > Config::global().max_issue_number {
                    return Err(SwissArmyHammerError::InvalidIssueNumber(format!(
                        "Issue number {} exceeds maximum ({})",
                        num,
                        Config::global().max_issue_number
                    )));
                }
                num
            }
            None => {
                // For non-numbered files, generate a virtual number based on filename hash
                // This ensures consistent ordering while keeping non-numbered files separate
                // from numbered ones. Use a high base (500,000) to avoid conflicts.
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                filename.hash(&mut hasher);
                let hash = hasher.finish();

                // Map hash to virtual number range to avoid conflicts with numbered files
                let config = Config::global();
                config.virtual_issue_number_base
                    + ((hash % config.virtual_issue_number_range as u64) as u32)
            }
        };

        // Read file content
        let content = fs::read_to_string(path).map_err(SwissArmyHammerError::Io)?;

        // Determine if completed based on path - only mark as completed if directly in the
        // specific completed directory, not just any directory named "complete"
        let completed = path
            .parent()
            .map(|parent| parent == self.state.completed_dir)
            .unwrap_or(false);

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
            } else if path.is_dir() {
                // Recursively scan subdirectories
                match self.list_issues_in_dir(&path) {
                    Ok(sub_issues) => issues.extend(sub_issues),
                    Err(e) => {
                        debug!("Failed to scan subdirectory {}: {}", path.display(), e);
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
        // Only consider explicitly numbered files (< virtual_base) for next number calculation
        // Non-numbered files use virtual numbers >= virtual_base and should not affect numbering
        let config = Config::global();
        let max_number = pending_issues
            .iter()
            .chain(completed_issues.iter())
            .filter(|issue| issue.number.value() < config.virtual_issue_number_base) // Exclude virtual numbers
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
        // Since list_issues_in_dir is now recursive, we only need to scan the root issues directory
        // This will automatically find issues in both pending and completed directories
        let all_issues = self.list_issues_in_dir(&self.state.issues_dir)?;

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
        // Lock to ensure atomic issue creation (prevents race conditions)
        let _lock = self.creation_lock.lock().await;

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
/// - The number exceeds the maximum allowed value (999_999)
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
/// assert_eq!(number, 999_999);
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
/// parse_issue_filename("1_000_000_test").unwrap();
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

/// Parse any issue filename format (numbered or non-numbered)
///
/// This function provides flexible parsing of issue filenames, supporting both the
/// traditional numbered format used for backward compatibility and the newer flexible
/// format that accepts any valid filename. This is the primary parsing function used
/// by the issue system to handle mixed file formats in issue directories.
///
/// ## Parsing Logic
///
/// 1. **First Attempt**: Try to parse as numbered format (`NNNNNN_name`)
/// 2. **Fallback**: If numbered parsing fails, treat entire filename as issue name
/// 3. **Validation**: Ensure the resulting name is not empty
///
/// ## Supported Formats
///
/// ### Traditional Numbered Format
/// - Pattern: `NNNNNN_name` where N is a digit (0-9)
/// - Example: `000123_bug_fix` → `(Some(123), "bug_fix")`
/// - Zero-padding is required for consistency
/// - Maximum number is limited by `Config::global().max_issue_number`
///
/// ### Flexible Format  
/// - Pattern: Any non-empty string
/// - Example: `feature-request` → `(None, "feature-request")`
/// - Assigned virtual numbers during issue creation
/// - Supports dashes, underscores, and other safe characters
///
/// ## Virtual Number Assignment
///
/// Non-numbered files (where this function returns `None` for the number) are assigned
/// virtual numbers in the range [`Config::global().virtual_issue_number_base`..] based
/// on a hash of the filename. This ensures consistent issue numbering while maintaining
/// flexibility in naming.
///
/// # Arguments
///
/// * `filename` - The filename without extension to parse (e.g., "000123_bug_fix" or "README")
///
/// # Returns
///
/// Returns `Ok((Option<u32>, String))` where:
/// - First element is `Some(number)` for successfully parsed numbered files, `None` for non-numbered
/// - Second element is the issue name (extracted name for numbered, full filename for non-numbered)
///
/// # Errors
///
/// Returns `SwissArmyHammerError::Other` if:
/// - The filename is empty
/// - Internal parsing errors occur (rare)
///
/// # Examples
///
/// ## Traditional Numbered Format Examples
///
/// ```rust
/// # use swissarmyhammer::issues::filesystem::parse_any_issue_filename;
/// // Standard numbered format
/// let (number, name) = parse_any_issue_filename("000123_bug_fix").unwrap();
/// assert_eq!(number, Some(123));
/// assert_eq!(name, "bug_fix");
///
/// // Leading zeros are handled correctly
/// let (number, name) = parse_any_issue_filename("000001_first_issue").unwrap();
/// assert_eq!(number, Some(1));
/// assert_eq!(name, "first_issue");
///
/// // Complex names with underscores
/// let (number, name) = parse_any_issue_filename("000456_user_auth_fix").unwrap();
/// assert_eq!(number, Some(456));
/// assert_eq!(name, "user_auth_fix");
/// ```
///
/// ## Flexible Format Examples
///
/// ```rust
/// # use swissarmyhammer::issues::filesystem::parse_any_issue_filename;
/// // Simple non-numbered format
/// let (number, name) = parse_any_issue_filename("README").unwrap();
/// assert_eq!(number, None);
/// assert_eq!(name, "README");
///
/// // Hyphenated names
/// let (number, name) = parse_any_issue_filename("feature-request").unwrap();
/// assert_eq!(number, None);
/// assert_eq!(name, "feature-request");
///
/// // Project documentation
/// let (number, name) = parse_any_issue_filename("project-notes").unwrap();
/// assert_eq!(number, None);
/// assert_eq!(name, "project-notes");
///
/// // Mixed characters
/// let (number, name) = parse_any_issue_filename("bug_report_2024").unwrap();
/// assert_eq!(number, None);
/// assert_eq!(name, "bug_report_2024");
/// ```
///
/// ## Error Cases
///
/// ```rust
/// # use swissarmyhammer::issues::filesystem::parse_any_issue_filename;
/// // Empty filename
/// assert!(parse_any_issue_filename("").is_err());
/// ```
///
/// ## Integration with Virtual Numbers
///
/// ```rust
/// # use swissarmyhammer::issues::filesystem::parse_any_issue_filename;
/// // These would get virtual numbers when converted to issues:
/// let (number, name) = parse_any_issue_filename("TODO").unwrap();
/// assert_eq!(number, None); // Will be assigned virtual number (500000+)
/// assert_eq!(name, "TODO");
///
/// let (number, name) = parse_any_issue_filename("meeting-notes").unwrap();
/// assert_eq!(number, None); // Will be assigned virtual number (500000+)
/// assert_eq!(name, "meeting-notes");
/// ```
pub fn parse_any_issue_filename(filename: &str) -> Result<(Option<u32>, String)> {
    // First try to parse as numbered format
    if let Ok((number, name)) = parse_issue_filename(filename) {
        return Ok((Some(number), name));
    }

    // If that fails, treat the entire filename as the issue name
    // Validate that the filename is not empty and is safe
    if filename.is_empty() {
        return Err(SwissArmyHammerError::Other(
            "Issue filename cannot be empty".to_string(),
        ));
    }

    Ok((None, filename.to_string()))
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
    if name.contains("../") || name.contains("..\\") || name.contains("./") || name.contains(".\\")
    {
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

/// Check if a file path represents a valid issue file
///
/// Determines whether a given file path is a valid issue file that can be processed
/// by the issue system. This function supports both the traditional numbered format
/// and the newer flexible format that allows any markdown file.
///
/// ## Supported Formats
///
/// ### Traditional Numbered Format (Legacy)
/// Files with 6-digit zero-padded numbers followed by an underscore and name:
/// - `000001_my_first_issue.md` - Valid numbered issue
/// - `000123_bug_fix.md` - Valid numbered issue
/// - `999999_last_issue.md` - Valid numbered issue (up to max_issue_number)
///
/// ### Flexible Format (New)
/// Any markdown file with a non-empty filename:
/// - `README.md` - Valid issue (gets virtual number)
/// - `feature-request.md` - Valid issue (gets virtual number)
/// - `bug-report.md` - Valid issue (gets virtual number)
/// - `project-notes.md` - Valid issue (gets virtual number)
///
/// ## Requirements
///
/// 1. **File Extension**: Must have `.md` extension (case-sensitive)
/// 2. **Non-Empty Name**: The filename (without extension) must not be empty
/// 3. **UTF-8 Compatible**: Filename must be valid UTF-8
///
/// ## Virtual Numbering
///
/// Non-numbered files are assigned virtual issue numbers in the range
/// [`Config::global().virtual_issue_number_base`..`Config::global().virtual_issue_number_base + Config::global().virtual_issue_number_range`]
/// based on a hash of the filename. This ensures consistent numbering while avoiding
/// conflicts with traditionally numbered issues.
///
/// ## Examples
///
/// ```rust
/// use std::path::Path;
/// use swissarmyhammer::issues::filesystem::is_issue_file;
///
/// // Traditional numbered formats
/// assert!(is_issue_file(Path::new("000001_first_issue.md")));
/// assert!(is_issue_file(Path::new("000123_bug_fix.md")));
///
/// // Flexible formats
/// assert!(is_issue_file(Path::new("README.md")));
/// assert!(is_issue_file(Path::new("project-notes.md")));
/// assert!(is_issue_file(Path::new("feature-request.md")));
///
/// // Invalid files
/// assert!(!is_issue_file(Path::new("document.txt"))); // Wrong extension
/// assert!(!is_issue_file(Path::new(".md")));          // Empty name
/// assert!(!is_issue_file(Path::new("notes")));        // No extension
/// ```
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

    // Any non-empty filename is valid (supports both numbered and non-numbered formats)
    !filename.is_empty()
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
            100_000,
            Config::global().max_issue_number,
        ];
        for num in valid_numbers {
            assert!(
                num <= Config::global().max_issue_number,
                "Issue number {num} should be valid"
            );
        }

        // Invalid numbers (too large)
        let invalid_numbers = vec![1_000_000, 9_999_999];
        for num in invalid_numbers {
            assert!(
                num > Config::global().max_issue_number,
                "Issue number {num} should be invalid"
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
    fn test_parse_issue_non_numbered_filename() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with non-numbered filename (now valid)
        let test_file = issues_dir.join("invalid_filename.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.name, "invalid_filename");
        assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        // Virtual number for non-numbered files
    }

    #[test]
    fn test_parse_issue_non_standard_format() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with non-standard format (now valid as non-numbered)
        let test_file = issues_dir.join("abc123_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.name, "abc123_test");
        assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        // Virtual number for non-numbered files
    }

    #[test]
    fn test_parse_issue_large_number_as_non_numbered() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create test file with number too large for numbered format (now valid as non-numbered)
        let test_file = issues_dir.join("1000000_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.name, "1000000_test");
        assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        // Virtual number for non-numbered files
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
        fs::write(issues_dir.join("000002_test.txt"), "content").unwrap(); // Should be ignored (not .md)
        fs::write(issues_dir.join("README.md"), "content").unwrap(); // Now valid as non-numbered issue
        fs::write(issues_dir.join("000003_valid.md"), "content").unwrap();

        let issues = storage.list_issues_in_dir(&issues_dir).unwrap();
        assert_eq!(issues.len(), 3); // All .md files are valid issues now

        // Sort by number to make assertions predictable
        let mut sorted_issues = issues;
        sorted_issues.sort_by_key(|issue| issue.number);

        assert_eq!(sorted_issues[0].number, IssueNumber::from(1));
        assert_eq!(sorted_issues[1].number, IssueNumber::from(3));
        // README.md gets a virtual number in virtual range
        assert!(sorted_issues[2].number.value() >= Config::global().virtual_issue_number_base);
        assert_eq!(sorted_issues[2].name, "README");
    }

    #[test]
    fn test_parse_issue_filename_no_underscore() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file with no underscore (now valid as non-numbered)
        let test_file = issues_dir.join("000123test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.name, "000123test");
        assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        // Virtual number for non-numbered files
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
    fn test_parse_issue_filename_starting_with_underscore() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create file starting with underscore (now valid as non-numbered)
        let test_file = issues_dir.join("_test.md");
        fs::write(&test_file, "content").unwrap();

        let result = storage.parse_issue_from_file(&test_file);
        assert!(result.is_ok());
        let issue = result.unwrap();
        assert_eq!(issue.name, "_test");
        assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        // Virtual number for non-numbered files
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

        // Create valid numbered file
        fs::write(issues_dir.join("000001_valid.md"), "content").unwrap();

        // Create files that were previously considered "corrupted" but are now valid as non-numbered issues
        fs::write(issues_dir.join("invalid_format.md"), "content").unwrap(); // Now valid as non-numbered
        fs::write(issues_dir.join("abc123_invalid_number.md"), "content").unwrap(); // Now valid as non-numbered
        fs::write(issues_dir.join("1000000_too_large.md"), "content").unwrap(); // Now valid as non-numbered

        let issues = storage.list_issues_in_dir(&issues_dir).unwrap();
        // All .md files are now valid issues (1 numbered + 3 non-numbered)
        assert_eq!(issues.len(), 4);

        // Sort to make assertions predictable
        let mut sorted_issues = issues;
        sorted_issues.sort_by_key(|issue| issue.number);

        // First issue should be the numbered one
        assert_eq!(sorted_issues[0].number, IssueNumber::from(1));
        assert_eq!(sorted_issues[0].name, "valid");

        // The other 3 should be non-numbered with virtual numbers >= virtual_base
        for issue in sorted_issues.iter().skip(1).take(3) {
            assert!(issue.number.value() >= Config::global().virtual_issue_number_base);
        }
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
                    .create_issue(format!("issue_{i}"), format!("Content {i}"))
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
            assert!(result.is_ok(), "Failed to create issue with name: '{name}'");
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
        assert_eq!(format_issue_number(999_999), "999999");
        assert_eq!(format_issue_number(0), "000000");
    }

    #[test]
    fn test_parse_issue_number_valid() {
        assert_eq!(parse_issue_number("000001").unwrap(), 1);
        assert_eq!(parse_issue_number("000123").unwrap(), 123);
        assert_eq!(parse_issue_number("999999").unwrap(), 999_999);
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
        assert!(parse_issue_number("1_000_000").is_err());
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
    fn test_parse_any_issue_filename_numbered() {
        // Test numbered format (existing behavior)
        let (number, name) = parse_any_issue_filename("000123_test_issue").unwrap();
        assert_eq!(number, Some(123));
        assert_eq!(name, "test_issue");

        let (number, name) = parse_any_issue_filename("000001_simple").unwrap();
        assert_eq!(number, Some(1));
        assert_eq!(name, "simple");

        let (number, name) = parse_any_issue_filename("000456_name_with_underscores").unwrap();
        assert_eq!(number, Some(456));
        assert_eq!(name, "name_with_underscores");
    }

    #[test]
    fn test_parse_any_issue_filename_non_numbered() {
        // Test non-numbered format (new behavior)
        let (number, name) = parse_any_issue_filename("my-issue").unwrap();
        assert_eq!(number, None);
        assert_eq!(name, "my-issue");

        let (number, name) = parse_any_issue_filename("bug-report").unwrap();
        assert_eq!(number, None);
        assert_eq!(name, "bug-report");

        let (number, name) = parse_any_issue_filename("feature_request").unwrap();
        assert_eq!(number, None);
        assert_eq!(name, "feature_request");

        let (number, name) = parse_any_issue_filename("simple").unwrap();
        assert_eq!(number, None);
        assert_eq!(name, "simple");
    }

    #[test]
    fn test_parse_any_issue_filename_edge_cases() {
        // Test empty filename
        assert!(parse_any_issue_filename("").is_err());

        // Test filename that looks like numbered but isn't
        let (number, name) = parse_any_issue_filename("123_not_6_digits").unwrap();
        assert_eq!(number, None); // Should be treated as non-numbered
        assert_eq!(name, "123_not_6_digits");

        // Test filename with underscores but not numbered format
        let (number, name) = parse_any_issue_filename("not_numbered_format").unwrap();
        assert_eq!(number, None);
        assert_eq!(name, "not_numbered_format");
    }

    #[test]
    fn test_virtual_number_generation() {
        // Test that virtual numbers are assigned consistently for non-numbered files
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create non-numbered files directly on disk (not via create_issue API)
        let issue_content = "# Test Issue\nThis is a test issue.";
        let readme_path = issues_dir.join("README.md");
        let notes_path = issues_dir.join("NOTES.md");
        let todo_path = issues_dir.join("TODO.md");

        std::fs::write(&readme_path, issue_content).unwrap();
        std::fs::write(&notes_path, issue_content).unwrap();
        std::fs::write(&todo_path, issue_content).unwrap();

        // Parse the files to get issues with virtual numbers
        let issue1 = storage.parse_issue_from_file(&readme_path).unwrap();
        let issue2 = storage.parse_issue_from_file(&notes_path).unwrap();
        let issue3 = storage.parse_issue_from_file(&todo_path).unwrap();

        let config = Config::global();

        // All should have virtual numbers >= virtual_issue_number_base
        assert!(issue1.number.value() >= config.virtual_issue_number_base);
        assert!(issue2.number.value() >= config.virtual_issue_number_base);
        assert!(issue3.number.value() >= config.virtual_issue_number_base);

        // Virtual numbers should be within the expected range
        assert!(
            issue1.number.value()
                < config.virtual_issue_number_base + config.virtual_issue_number_range
        );
        assert!(
            issue2.number.value()
                < config.virtual_issue_number_base + config.virtual_issue_number_range
        );
        assert!(
            issue3.number.value()
                < config.virtual_issue_number_base + config.virtual_issue_number_range
        );

        // Same filename should always generate the same virtual number (deterministic hashing)
        let issue1_again = storage.parse_issue_from_file(&readme_path).unwrap();
        assert_eq!(issue1.number, issue1_again.number);

        // Different filenames should have different virtual numbers
        assert_ne!(issue1.number, issue2.number);
        assert_ne!(issue2.number, issue3.number);
        assert_ne!(issue1.number, issue3.number);
    }

    #[test]
    fn test_hash_collision_handling() {
        // Test behavior when different filenames generate virtual numbers
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create multiple non-numbered files with different names
        let test_names = vec![
            "test1",
            "test2",
            "test3",
            "test4",
            "test5",
            "README",
            "NOTES",
            "TODO",
            "CHANGELOG",
            "DESIGN",
            "bug-report",
            "feature-request",
            "meeting-notes",
            "project-spec",
            "user-guide",
        ];

        let mut created_issues = Vec::new();
        for name in &test_names {
            let file_path = issues_dir.join(format!("{name}.md"));
            std::fs::write(&file_path, "Test content").unwrap();
            let issue = storage.parse_issue_from_file(&file_path).unwrap();
            created_issues.push(issue);
        }

        // Virtual numbers should be properly distributed and unique
        let mut virtual_numbers: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for issue in &created_issues {
            let config = Config::global();
            // All should be virtual numbers
            assert!(issue.number.value() >= config.virtual_issue_number_base);

            // Each should be unique (hash algorithm should distribute properly)
            assert!(
                virtual_numbers.insert(issue.number.value()),
                "Duplicate virtual number found: {}",
                issue.number.value()
            );
        }

        // Test that we actually created the expected number of issues
        assert_eq!(created_issues.len(), test_names.len());

        // Test that filename-to-virtual-number mapping is consistent
        for (i, name) in test_names.iter().enumerate() {
            let file_path = issues_dir.join(format!("{name}.md"));
            let re_parsed = storage.parse_issue_from_file(&file_path).unwrap();
            assert_eq!(
                re_parsed.number, created_issues[i].number,
                "Virtual number changed for {name} on re-parsing"
            );
        }
    }

    #[tokio::test]
    async fn test_mixed_numbered_and_non_numbered_sorting() {
        // Test that numbered and non-numbered issues are sorted correctly together
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Create mix of numbered and non-numbered issues
        let content = "Test issue content";

        // Create some traditional numbered issues via API
        let _numbered1 = storage
            .create_issue("first".to_string(), content.to_string())
            .await
            .unwrap(); // Should get number 1
        let _numbered2 = storage
            .create_issue("second".to_string(), content.to_string())
            .await
            .unwrap(); // Should get number 2

        // Create some non-numbered issues directly on disk
        let readme_path = issues_dir.join("README.md");
        let notes_path = issues_dir.join("NOTES.md");
        std::fs::write(&readme_path, content).unwrap();
        std::fs::write(&notes_path, content).unwrap();
        let _non_numbered1 = storage.parse_issue_from_file(&readme_path).unwrap();
        let _non_numbered2 = storage.parse_issue_from_file(&notes_path).unwrap();

        // Create another numbered issue
        let _numbered3 = storage
            .create_issue("third".to_string(), content.to_string())
            .await
            .unwrap(); // Should get number 3

        // List all issues
        let all_issues = storage.list_issues().await.unwrap();

        // Should have all 5 issues
        assert_eq!(all_issues.len(), 5);

        // Issues should be sorted by number (numbered first, then virtual)
        let config = Config::global();
        let mut prev_number = 0;
        let mut seen_virtual = false;

        for issue in &all_issues {
            if issue.number.value() < config.virtual_issue_number_base {
                // This is a numbered issue
                assert!(
                    !seen_virtual,
                    "Numbered issue found after virtual issue in sort order"
                );
                assert!(
                    issue.number.value() > prev_number,
                    "Numbered issues should be in ascending order"
                );
                prev_number = issue.number.value();
            } else {
                // This is a virtual number issue
                seen_virtual = true;
                assert!(
                    issue.number.value() >= config.virtual_issue_number_base,
                    "Virtual issue number should be >= base"
                );
            }
        }

        // Verify we have the expected numbered issues in order
        assert_eq!(all_issues[0].number.value(), 1);
        assert_eq!(all_issues[0].name, "first");
        assert_eq!(all_issues[1].number.value(), 2);
        assert_eq!(all_issues[1].name, "second");
        assert_eq!(all_issues[2].number.value(), 3);
        assert_eq!(all_issues[2].name, "third");

        // The remaining two should be virtual numbered issues
        assert!(all_issues[3].number.value() >= config.virtual_issue_number_base);
        assert!(all_issues[4].number.value() >= config.virtual_issue_number_base);
    }

    #[test]
    fn test_virtual_number_boundary_conditions() {
        // Test edge cases around virtual number boundaries
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let config = Config::global();

        // Test the virtual number calculation directly
        let test_filenames = [
            "", // Edge case - empty (should be handled elsewhere)
            "a", // Single character
            "very_long_filename_that_might_cause_issues_with_hashing_and_boundary_calculations_test",
            "filename_with_unicode_字符", // Unicode characters
            "UPPER_CASE_FILENAME",
            "mixed_Case_FileName",
            "filename-with-hyphens",
            "filename.with.dots",
            "filename with spaces", // Spaces (if allowed)
        ];

        for filename in test_filenames.iter().filter(|f| !f.is_empty()) {
            let mut hasher = DefaultHasher::new();
            filename.hash(&mut hasher);
            let hash = hasher.finish();

            let virtual_number = config.virtual_issue_number_base
                + ((hash % config.virtual_issue_number_range as u64) as u32);

            // Verify virtual number is within expected bounds
            assert!(
                virtual_number >= config.virtual_issue_number_base,
                "Virtual number {} for '{}' should be >= {}",
                virtual_number,
                filename,
                config.virtual_issue_number_base
            );
            assert!(
                virtual_number
                    < config.virtual_issue_number_base + config.virtual_issue_number_range,
                "Virtual number {} for '{}' should be < {}",
                virtual_number,
                filename,
                config.virtual_issue_number_base + config.virtual_issue_number_range
            );
        }
    }

    #[test]
    fn test_invalid_input_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let issues_dir = temp_dir.path().to_path_buf();
        let storage = FileSystemIssueStorage::new(issues_dir.clone()).unwrap();

        // Test filename that results in empty string after parsing
        // Note: ".md" is actually valid - file_stem() returns ".md", so we test a different edge case
        let result = parse_any_issue_filename("");
        assert!(result.is_err(), "Empty string should fail to parse");

        // Test filename with only dots (edge case)
        let dots_path = issues_dir.join("....md");
        std::fs::write(&dots_path, "content").unwrap();
        let result = storage.parse_issue_from_file(&dots_path);
        // This should actually work - filename is "..." which is non-empty
        assert!(
            result.is_ok(),
            "Filename with dots should parse as non-numbered: {:?}",
            result
        );
        if let Ok(issue) = result {
            assert_eq!(issue.name, "...");
            let config = Config::global();
            assert!(issue.number.value() >= config.virtual_issue_number_base);
        }

        // Test file with no extension (parse_issue_from_file doesn't check extension - that's done by is_issue_file)
        let no_ext_path = issues_dir.join("no_extension");
        std::fs::write(&no_ext_path, "content").unwrap();
        let result = storage.parse_issue_from_file(&no_ext_path);
        // parse_issue_from_file doesn't enforce .md extension - that's handled by directory scanning
        assert!(
            result.is_ok(),
            "parse_issue_from_file should work on any file regardless of extension"
        );

        // But is_issue_file should reject non-.md files
        assert!(
            !is_issue_file(&no_ext_path),
            "is_issue_file should reject non-.md files"
        );

        // Test moderately long filename (avoid filesystem limits)
        let long_name = "a".repeat(200); // Use 200 chars to avoid filesystem limits
        let long_path = issues_dir.join(format!("{long_name}.md"));
        if std::fs::write(&long_path, "content").is_ok() {
            let result = storage.parse_issue_from_file(&long_path);
            // This should work - long filenames are valid if filesystem supports them
            assert!(result.is_ok(), "Long valid filename should parse correctly");
        } else {
            // If filesystem rejects the filename, that's a system limitation, not our parsing issue
            println!("Filesystem rejected long filename - this is a system limitation");
        }

        // Test filename with only special characters (should get virtual number)
        let special_path = issues_dir.join("!@#$%^&*()_+.md");
        std::fs::write(&special_path, "content").unwrap();
        let result = storage.parse_issue_from_file(&special_path);
        assert!(
            result.is_ok(),
            "Special characters should be valid in non-numbered format"
        );
        let issue = result.unwrap();
        let config = Config::global();
        assert!(issue.number.value() >= config.virtual_issue_number_base);

        // Test nonexistent file
        let nonexistent = issues_dir.join("does_not_exist.md");
        let result = storage.parse_issue_from_file(&nonexistent);
        assert!(result.is_err(), "Nonexistent file should fail");

        // Test file with invalid UTF-8 in content (might fail content reading)
        let utf8_path = issues_dir.join("utf8_test.md");
        std::fs::write(&utf8_path, [0xFF, 0xFE, 0xFD, 0xFC]).unwrap(); // Invalid UTF-8
        let result = storage.parse_issue_from_file(&utf8_path);
        // Invalid UTF-8 content might cause parsing to fail, which is expected behavior
        match result {
            Ok(_) => {
                // If it succeeds, the system handled invalid UTF-8 gracefully
            }
            Err(_) => {
                // If it fails, that's expected due to invalid UTF-8 content
            }
        }
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
        // Valid issue files - numbered format (traditional)
        assert!(is_issue_file(Path::new("000123_test.md")));
        assert!(is_issue_file(Path::new("000001_simple.md")));
        assert!(is_issue_file(Path::new("999999_max.md")));
        assert!(is_issue_file(Path::new("000000_zero.md")));
        assert!(is_issue_file(Path::new("000456_name_with_underscores.md")));

        // Valid issue files - non-numbered format (new)
        assert!(is_issue_file(Path::new("123_test.md"))); // Now valid: any .md file
        assert!(is_issue_file(Path::new("000123test.md"))); // Now valid: any .md file
        assert!(is_issue_file(Path::new("abc123_test.md"))); // Now valid: any .md file
        assert!(is_issue_file(Path::new("README.md"))); // Now valid: any .md file
        assert!(is_issue_file(Path::new("bug-report.md"))); // Valid: non-numbered
        assert!(is_issue_file(Path::new("my-feature.md"))); // Valid: non-numbered

        // Invalid files - wrong extension or no filename
        assert!(!is_issue_file(Path::new("000123_test.txt"))); // Wrong extension
        assert!(!is_issue_file(Path::new("000123_test"))); // No extension
        assert!(!is_issue_file(Path::new(".md"))); // Empty filename

        // Valid but edge cases
        assert!(is_issue_file(Path::new("000123_.md"))); // Valid: empty name part in numbered format

        // Path with directory should work
        assert!(is_issue_file(Path::new("./issues/000123_test.md")));
        assert!(is_issue_file(Path::new("/path/to/README.md")));
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
        assert_eq!(format_issue_number(999_999), "999999");
        assert_eq!(format_issue_number(42), "000042");
    }

    #[test]
    fn test_parse_issue_number_comprehensive() {
        assert_eq!(parse_issue_number("000001").unwrap(), 1);
        assert_eq!(parse_issue_number("999999").unwrap(), 999_999);
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
        let issue1 = storage
            .create_issue("issue_1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("issue_2".to_string(), "Content 2".to_string())
            .await
            .unwrap();
        let issue3 = storage
            .create_issue("issue_3".to_string(), "Content 3".to_string())
            .await
            .unwrap();

        let numbers = vec![
            issue1.number.value(),
            issue2.number.value(),
            issue3.number.value(),
        ];
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
        let issue1 = storage
            .create_issue("issue_1".to_string(), "Original 1".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("issue_2".to_string(), "Original 2".to_string())
            .await
            .unwrap();
        let issue3 = storage
            .create_issue("issue_3".to_string(), "Original 3".to_string())
            .await
            .unwrap();

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
        let issue1 = storage
            .create_issue("issue_1".to_string(), "Content 1".to_string())
            .await
            .unwrap();
        let issue2 = storage
            .create_issue("issue_2".to_string(), "Content 2".to_string())
            .await
            .unwrap();
        let issue3 = storage
            .create_issue("issue_3".to_string(), "Content 3".to_string())
            .await
            .unwrap();

        let numbers = vec![
            issue1.number.value(),
            issue2.number.value(),
            issue3.number.value(),
        ];
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
            .map(|i| (format!("issue_{i}"), format!("Content {i}")))
            .collect();

        let created_issues = storage.create_issues_batch(batch_data).await.unwrap();
        assert_eq!(created_issues.len(), batch_size);

        // Get all issues in batch
        let numbers: Vec<u32> = created_issues.iter().map(|i| i.number.value()).collect();
        let retrieved_issues = storage.get_issues_batch(numbers.clone()).await.unwrap();
        assert_eq!(retrieved_issues.len(), batch_size);

        // Update all issues in batch
        let updates: Vec<(u32, String)> = created_issues
            .iter()
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
        let issue = storage
            .create_issue("existing".to_string(), "Content".to_string())
            .await
            .unwrap();

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
