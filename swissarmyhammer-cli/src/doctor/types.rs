//! Type definitions for the doctor module

use std::path::{Path, PathBuf};

/// Wrapper type for workflow directory paths to provide type safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDirectory(PathBuf);

impl WorkflowDirectory {
    /// Create a new WorkflowDirectory from a PathBuf
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the workflow directory
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use swissarmyhammer_cli::doctor::WorkflowDirectory;
    ///
    /// let dir = WorkflowDirectory::new(PathBuf::from("/home/user/.swissarmyhammer/workflows"));
    /// ```
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Get the underlying path
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use swissarmyhammer_cli::doctor::WorkflowDirectory;
    ///
    /// let dir = WorkflowDirectory::new(PathBuf::from("/test"));
    /// assert_eq!(dir.path(), Path::new("/test"));
    /// ```
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for WorkflowDirectory {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Display for WorkflowDirectory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Type-safe wrapper for disk space measurements in megabytes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DiskSpace {
    mb: u64,
}

impl DiskSpace {
    /// Create a new DiskSpace value from megabytes
    pub fn from_mb(mb: u64) -> Self {
        Self { mb }
    }

    /// Get the value in megabytes
    #[allow(dead_code)]
    pub fn as_mb(&self) -> u64 {
        self.mb
    }

    /// Check if disk space is below a certain threshold
    pub fn is_low(&self, threshold_mb: u64) -> bool {
        self.mb < threshold_mb
    }
}

impl std::fmt::Display for DiskSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} MB", self.mb)
    }
}

/// Information about a workflow directory including its path and category
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDirectoryInfo {
    pub path: WorkflowDirectory,
    pub category: WorkflowCategory,
}

impl WorkflowDirectoryInfo {
    /// Create a new WorkflowDirectoryInfo
    ///
    /// # Arguments
    ///
    /// * `path` - The workflow directory path
    /// * `category` - The category of the workflow directory (User or Local)
    ///
    /// # Example
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use swissarmyhammer_cli::doctor::{WorkflowDirectory, WorkflowDirectoryInfo, WorkflowCategory};
    ///
    /// let dir = WorkflowDirectory::new(PathBuf::from("/home/user/.swissarmyhammer/workflows"));
    /// let info = WorkflowDirectoryInfo::new(dir, WorkflowCategory::User);
    /// ```
    pub fn new(path: WorkflowDirectory, category: WorkflowCategory) -> Self {
        Self { path, category }
    }
}

/// Category of workflow directory (User or Local)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowCategory {
    User,
    Local,
}

impl std::fmt::Display for WorkflowCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkflowCategory::User => write!(f, "User"),
            WorkflowCategory::Local => write!(f, "Local"),
        }
    }
}

/// Status of a diagnostic check
#[derive(Debug, PartialEq, Clone)]
pub enum CheckStatus {
    /// Check passed without issues
    Ok,
    /// Check passed but with potential issues
    Warning,
    /// Check failed with errors
    Error,
}

/// Exit codes for the doctor command
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// All checks passed
    Success = 0,
    /// Warnings detected
    Warning = 1,
    /// Errors detected
    Error = 2,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code as i32
    }
}

/// Result of a single diagnostic check
#[derive(Debug, Clone)]
pub struct Check {
    /// Name of the check performed
    pub name: String,
    /// Status of the check (Ok, Warning, Error)
    pub status: CheckStatus,
    /// Descriptive message about the check result
    pub message: String,
    /// Optional fix suggestion for warnings or errors
    pub fix: Option<String>,
}

impl Check {
    /// Create a new Check with builder pattern
    ///
    /// # Example
    ///
    /// ```
    /// use swissarmyhammer_cli::doctor::{Check, CheckStatus};
    ///
    /// let check = Check::new("Test Check", CheckStatus::Ok)
    ///     .with_message("Everything is working")
    ///     .with_fix("No fix needed")
    ///     .build();
    /// ```
    pub fn new(name: impl Into<String>, status: CheckStatus) -> CheckBuilder {
        CheckBuilder {
            name: name.into(),
            status,
            message: String::new(),
            fix: None,
        }
    }
}

/// Builder for creating Check instances
pub struct CheckBuilder {
    name: String,
    status: CheckStatus,
    message: String,
    fix: Option<String>,
}

impl CheckBuilder {
    /// Set the message for this check
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set the fix suggestion for this check
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// Build the Check instance
    pub fn build(self) -> Check {
        Check {
            name: self.name,
            status: self.status,
            message: self.message,
            fix: self.fix,
        }
    }
}

/// Groups of checks organized by category
pub(crate) struct CheckGroups<'a> {
    pub system_checks: Vec<&'a Check>,
    pub config_checks: Vec<&'a Check>,
    pub prompt_checks: Vec<&'a Check>,
    pub workflow_checks: Vec<&'a Check>,
}

/// Count of checks by status
pub(crate) struct CheckCounts {
    pub ok_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}
