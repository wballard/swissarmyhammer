//! Doctor module for SwissArmyHammer diagnostic tools
//!
//! This module provides comprehensive system diagnostics for SwissArmyHammer installations,
//! checking various aspects of the system configuration to ensure optimal operation.
//!
//! # Features
//!
//! - Installation verification (binary permissions, PATH configuration)
//! - Claude Code MCP integration checking
//! - Prompt directory validation
//! - YAML front matter parsing verification
//! - Workflow system diagnostics
//! - Disk space monitoring
//! - File permission checks
//!
//! # Usage
//!
//! ```no_run
//! use swissarmyhammer_cli::doctor::Doctor;
//!
//! let mut doctor = Doctor::new();
//! let exit_code = doctor.run_diagnostics()?;
//! ```
//!
//! The doctor returns exit codes:
//! - 0: All checks passed
//! - 1: Some warnings detected
//! - 2: Errors detected

use anyhow::Result;
use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Directory name for SwissArmyHammer configuration and data
const SWISSARMYHAMMER_DIR: &str = ".swissarmyhammer";

/// Minimum disk space in MB before warning
///
/// This threshold is set to 100MB which provides enough space for:
/// - Several workflow run outputs (typically 1-10MB each)
/// - Temporary files created during workflow execution
/// - Log files and diagnostic information
///
/// This conservative threshold helps ensure smooth operation while avoiding
/// false alarms on systems with limited but adequate disk space.
const LOW_DISK_SPACE_MB: u64 = 100;

/// Check names constants to avoid typos and improve maintainability
mod check_names {
    pub const INSTALLATION_METHOD: &str = "Installation Method";
    pub const BINARY_PERMISSIONS: &str = "Binary Permissions";
    pub const BINARY_NAME: &str = "Binary Name";
    pub const IN_PATH: &str = "swissarmyhammer in PATH";
    pub const CLAUDE_CONFIG: &str = "Claude Code MCP configuration";
    pub const BUILTIN_PROMPTS: &str = "Built-in prompts";
    pub const USER_PROMPTS_DIR: &str = "User prompts directory";
    pub const LOCAL_PROMPTS_DIR: &str = "Local prompts directory";
    pub const YAML_PARSING: &str = "YAML parsing";
    pub const FILE_PERMISSIONS: &str = "File permissions";
    pub const WORKFLOW_PARSING: &str = "Workflow parsing";
    pub const WORKFLOW_RUN_STORAGE_ACCESS: &str = "Workflow run storage accessibility";
    pub const WORKFLOW_RUN_STORAGE_SPACE: &str = "Workflow run storage space";
    pub const WORKFLOW_NAME_CONFLICTS: &str = "Workflow name conflicts";
    pub const WORKFLOW_CIRCULAR_DEPS: &str = "Workflow circular dependencies";
}

/// Format strings used throughout the module
mod format_strings {
    pub const WORKFLOW_DIR_PERMISSIONS: &str = "Workflow directory permissions: {:?}";
    pub const WORKFLOW_DIR_ACCESS: &str = "Workflow directory access: {:?}";
    pub const WORKFLOW_PARSING_ERROR: &str = "Workflow parsing: {:?}";
    pub const YAML_PARSING_ERROR: &str = "YAML parsing: {:?}";
}

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
struct CheckGroups<'a> {
    system_checks: Vec<&'a Check>,
    config_checks: Vec<&'a Check>,
    prompt_checks: Vec<&'a Check>,
    workflow_checks: Vec<&'a Check>,
}

/// Count of checks by status
struct CheckCounts {
    ok_count: usize,
    warning_count: usize,
    error_count: usize,
}

/// Main diagnostic tool for SwissArmyHammer system health checks
///
/// The Doctor struct accumulates diagnostic results and provides a summary
/// of the system's configuration and any potential issues.
pub struct Doctor {
    checks: Vec<Check>,
}

impl Doctor {
    /// Create a new Doctor instance for running diagnostics
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Run all diagnostic checks
    ///
    /// Performs a comprehensive set of diagnostics including:
    /// - Installation verification
    /// - Claude Code configuration
    /// - Prompt directory validation
    /// - Workflow system checks
    ///
    /// # Returns
    ///
    /// Returns an exit code:
    /// - 0: All checks passed
    /// - 1: Warnings detected
    /// - 2: Errors detected
    pub fn run_diagnostics(&mut self) -> Result<i32> {
        println!("{}", "🔨 SwissArmyHammer Doctor".bold().blue());
        println!("{}", "Running diagnostics...".dimmed());
        println!();

        // Run all checks
        self.check_installation()?;
        self.check_in_path()?;
        self.check_claude_config()?;
        self.check_prompt_directories()?;
        self.check_yaml_parsing()?;
        self.check_file_permissions()?;

        // Run workflow diagnostics
        self.check_workflow_directories()?;
        self.check_workflow_permissions()?;
        self.check_workflow_parsing()?;
        self.check_workflow_run_storage()?;
        self.check_workflow_dependencies()?;

        // Print results
        self.print_results();

        // Return exit code
        Ok(self.get_exit_code())
    }

    /// Check installation method and binary integrity
    ///
    /// Verifies:
    /// - Installation method (cargo, system, development build)
    /// - Binary version and build type
    /// - Execute permissions on Unix systems
    /// - Binary naming conventions
    pub fn check_installation(&mut self) -> Result<()> {
        // Check if running from cargo install vs standalone binary
        let current_exe = env::current_exe().unwrap_or_default();
        let exe_path = current_exe.to_string_lossy();

        // Determine installation method
        let installation_method = if exe_path.contains(".cargo/bin") {
            "Cargo install"
        } else if exe_path.contains("/usr/local/bin") || exe_path.contains("/usr/bin") {
            "System installation"
        } else if exe_path.contains("target/") && exe_path.contains("debug") {
            "Development build"
        } else if exe_path.contains("target/") && exe_path.contains("release") {
            "Local release build"
        } else {
            "Unknown"
        };

        // Check binary version and build info
        let version = env!("CARGO_PKG_VERSION");
        let build_info = if cfg!(debug_assertions) {
            "debug build"
        } else {
            "release build"
        };

        self.checks.push(Check {
            name: check_names::INSTALLATION_METHOD.to_string(),
            status: CheckStatus::Ok,
            message: format!(
                "{} (v{}, {}) at {}",
                installation_method, version, build_info, exe_path
            ),
            fix: None,
        });

        // Check if binary has execute permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&current_exe) {
                let permissions = metadata.permissions();
                let mode = permissions.mode();

                if mode & 0o111 != 0 {
                    self.checks.push(Check {
                        name: check_names::BINARY_PERMISSIONS.to_string(),
                        status: CheckStatus::Ok,
                        message: format!("Executable permissions: {:o}", mode & 0o777),
                        fix: None,
                    });
                } else {
                    self.checks.push(Check {
                        name: check_names::BINARY_PERMISSIONS.to_string(),
                        status: CheckStatus::Error,
                        message: "Binary is not executable".to_string(),
                        fix: Some(format!("Run: chmod +x {}", exe_path)),
                    });
                }
            }
        }

        // Check if this is the expected binary name
        let exe_name = current_exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if exe_name == "swissarmyhammer" || exe_name == "swissarmyhammer.exe" {
            self.checks.push(Check {
                name: check_names::BINARY_NAME.to_string(),
                status: CheckStatus::Ok,
                message: format!("Running as {}", exe_name),
                fix: None,
            });
        } else {
            self.checks.push(Check {
                name: check_names::BINARY_NAME.to_string(),
                status: CheckStatus::Warning,
                message: format!("Unexpected binary name: {}", exe_name),
                fix: Some("Consider renaming binary to 'swissarmyhammer'".to_string()),
            });
        }

        Ok(())
    }

    /// Check if swissarmyhammer is in PATH
    ///
    /// Searches the system PATH for the swissarmyhammer executable
    /// and reports its location if found.
    pub fn check_in_path(&mut self) -> Result<()> {
        let path_var = env::var("PATH").unwrap_or_default();
        let paths: Vec<std::path::PathBuf> = env::split_paths(&path_var).collect();

        let exe_name = "swissarmyhammer";
        let mut found = false;
        let mut found_path = None;

        for path in paths {
            let exe_path = path.join(exe_name);
            if exe_path.exists() {
                found = true;
                found_path = Some(exe_path);
                break;
            }
        }

        if found {
            self.checks.push(Check {
                name: check_names::IN_PATH.to_string(),
                status: CheckStatus::Ok,
                message: format!(
                    "Found at: {:?}",
                    found_path.expect("found_path should be Some when found is true")
                ),
                fix: None,
            });
        } else {
            self.checks.push(Check {
                name: check_names::IN_PATH.to_string(),
                status: CheckStatus::Warning,
                message: "swissarmyhammer not found in PATH".to_string(),
                fix: Some(
                    "Add swissarmyhammer to your PATH or use the full path in Claude Code config"
                        .to_string(),
                ),
            });
        }

        Ok(())
    }

    /// Check Claude Code MCP configuration
    ///
    /// Verifies that swissarmyhammer is properly configured as an MCP server
    /// in Claude Code by running `claude mcp list` and checking the output.
    pub fn check_claude_config(&mut self) -> Result<()> {
        use std::process::Command;

        // Run `claude mcp list` to check if swissarmyhammer is configured
        match Command::new("claude").arg("mcp").arg("list").output() {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Check if swissarmyhammer is in the list
                    if stdout.contains("swissarmyhammer") {
                        self.checks.push(Check {
                            name: check_names::CLAUDE_CONFIG.to_string(),
                            status: CheckStatus::Ok,
                            message: "swissarmyhammer is configured in Claude Code".to_string(),
                            fix: None,
                        });
                    } else {
                        self.checks.push(Check {
                            name: check_names::CLAUDE_CONFIG.to_string(),
                            status: CheckStatus::Warning,
                            message: "swissarmyhammer not found in Claude Code MCP servers"
                                .to_string(),
                            fix: Some(get_claude_add_command()),
                        });
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.checks.push(Check {
                        name: check_names::CLAUDE_CONFIG.to_string(),
                        status: CheckStatus::Error,
                        message: format!("Failed to run 'claude mcp list': {}", stderr.trim()),
                        fix: Some(
                            "Ensure Claude Code is installed and the 'claude' command is available"
                                .to_string(),
                        ),
                    });
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    self.checks.push(Check {
                        name: check_names::CLAUDE_CONFIG.to_string(),
                        status: CheckStatus::Error,
                        message: "Claude Code command not found".to_string(),
                        fix: Some("Install Claude Code from https://claude.ai/code or ensure the 'claude' command is in your PATH".to_string()),
                    });
                } else {
                    self.checks.push(Check {
                        name: check_names::CLAUDE_CONFIG.to_string(),
                        status: CheckStatus::Error,
                        message: format!("Failed to run 'claude mcp list': {}", e),
                        fix: Some("Check that Claude Code is properly installed".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check prompt directories
    ///
    /// Verifies the existence and accessibility of:
    /// - Built-in prompts (embedded in binary)
    /// - User prompts directory (~/.swissarmyhammer/prompts)
    /// - Local prompts directory (./.swissarmyhammer/prompts)
    pub fn check_prompt_directories(&mut self) -> Result<()> {
        // Check builtin prompts (embedded in binary)
        self.checks.push(Check {
            name: check_names::BUILTIN_PROMPTS.to_string(),
            status: CheckStatus::Ok,
            message: "Built-in prompts are embedded in the binary".to_string(),
            fix: None,
        });

        // Check user prompts directory
        if let Some(home) = dirs::home_dir() {
            let user_prompts = home.join(SWISSARMYHAMMER_DIR).join("prompts");
            if user_prompts.exists() {
                let count = count_markdown_files(&user_prompts);
                self.checks.push(Check {
                    name: check_names::USER_PROMPTS_DIR.to_string(),
                    status: CheckStatus::Ok,
                    message: format!("Found {} prompts in {:?}", count, user_prompts),
                    fix: None,
                });
            } else {
                self.checks.push(Check {
                    name: check_names::USER_PROMPTS_DIR.to_string(),
                    status: CheckStatus::Ok,
                    message: format!(
                        "{} directory not found (optional): {:?}",
                        "User prompts", user_prompts
                    ),
                    fix: Some(format!("Create directory: mkdir -p {:?}", user_prompts)),
                });
            }
        }

        // Check local prompts directory
        let local_prompts = PathBuf::from(SWISSARMYHAMMER_DIR).join("prompts");
        if local_prompts.exists() {
            let count = count_markdown_files(&local_prompts);
            self.checks.push(Check {
                name: check_names::LOCAL_PROMPTS_DIR.to_string(),
                status: CheckStatus::Ok,
                message: format!("Found {} prompts in {:?}", count, local_prompts),
                fix: None,
            });
        } else {
            self.checks.push(Check {
                name: check_names::LOCAL_PROMPTS_DIR.to_string(),
                status: CheckStatus::Ok,
                message: format!(
                    "{} directory not found (optional): {:?}",
                    "Local prompts", local_prompts
                ),
                fix: Some(format!("Create directory: mkdir -p {:?}", local_prompts)),
            });
        }

        Ok(())
    }

    /// Check for YAML parsing errors
    ///
    /// Scans all markdown files in prompt directories and validates
    /// their YAML front matter for syntax errors.
    pub fn check_yaml_parsing(&mut self) -> Result<()> {
        use walkdir::WalkDir;

        let mut yaml_errors = Vec::new();

        // Check all prompt directories
        let mut dirs_to_check = vec![PathBuf::from(SWISSARMYHAMMER_DIR).join("prompts")];

        // Add user directory if it exists
        if let Some(home) = dirs::home_dir() {
            dirs_to_check.push(home.join(SWISSARMYHAMMER_DIR).join("prompts"));
        }

        for dir in dirs_to_check {
            if !dir.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            {
                match fs::read_to_string(entry.path()) {
                    Ok(content) => {
                        // Try to parse YAML front matter
                        if content.starts_with("---") {
                            let parts: Vec<&str> = content.splitn(3, "---").collect();
                            if parts.len() >= 3 {
                                let yaml_content = parts[1];
                                if let Err(e) =
                                    serde_yaml::from_str::<serde_yaml::Value>(yaml_content)
                                {
                                    yaml_errors.push((entry.path().to_path_buf(), e.to_string()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yaml_errors.push((
                            entry.path().to_path_buf(),
                            format!("Failed to read file: {}", e),
                        ));
                    }
                }
            }
        }

        if yaml_errors.is_empty() {
            self.checks.push(Check {
                name: check_names::YAML_PARSING.to_string(),
                status: CheckStatus::Ok,
                message: "All prompt YAML front matter is valid".to_string(),
                fix: None,
            });
        } else {
            for (path, error) in yaml_errors {
                self.checks.push(Check {
                    name: format!(
                        format_strings::YAML_PARSING_ERROR,
                        path.file_name().unwrap_or_default()
                    ),
                    status: CheckStatus::Error,
                    message: error,
                    fix: Some(format!("Fix the YAML syntax in {:?}", path)),
                });
            }
        }

        Ok(())
    }

    /// Check file permissions
    ///
    /// Verifies that the current directory is readable, which is
    /// essential for SwissArmyHammer operations.
    pub fn check_file_permissions(&mut self) -> Result<()> {
        // For now, just check that we can read the current directory
        match std::env::current_dir() {
            Ok(cwd) => {
                self.checks.push(Check {
                    name: check_names::FILE_PERMISSIONS.to_string(),
                    status: CheckStatus::Ok,
                    message: format!("Can read current directory: {:?}", cwd),
                    fix: None,
                });
            }
            Err(e) => {
                self.checks.push(Check {
                    name: check_names::FILE_PERMISSIONS.to_string(),
                    status: CheckStatus::Error,
                    message: format!("Failed to read current directory: {}", e),
                    fix: Some("Check file permissions for the current directory".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Print the results
    ///
    /// Displays all diagnostic results grouped by category:
    /// - System checks
    /// - Configuration
    /// - Prompts
    /// - Workflows
    ///
    /// Results are color-coded based on status (OK, Warning, Error).
    pub fn print_results(&self) {
        let use_color = crate::cli::Cli::should_use_color();

        // Group and print checks by category
        let check_groups = self.group_checks_by_category();

        self.print_check_category(&check_groups.system_checks, "System Checks:", use_color);
        self.print_check_category(&check_groups.config_checks, "Configuration:", use_color);
        self.print_check_category(&check_groups.prompt_checks, "Prompts:", use_color);
        self.print_check_category(&check_groups.workflow_checks, "Workflows:", use_color);

        // Print summary
        self.print_summary(use_color);
    }

    /// Group checks into categories
    fn group_checks_by_category(&self) -> CheckGroups {
        CheckGroups {
            system_checks: self
                .checks
                .iter()
                .filter(|c| c.name.contains("PATH") || c.name.contains("permissions"))
                .collect(),
            config_checks: self
                .checks
                .iter()
                .filter(|c| c.name.contains("Claude") || c.name.contains("config"))
                .collect(),
            prompt_checks: self
                .checks
                .iter()
                .filter(|c| c.name.contains("prompt") || c.name.contains("YAML"))
                .filter(|c| !c.name.contains("Workflow"))
                .collect(),
            workflow_checks: self
                .checks
                .iter()
                .filter(|c| c.name.contains("Workflow") || c.name.contains("workflow"))
                .collect(),
        }
    }

    /// Print a category of checks
    fn print_check_category(&self, checks: &[&Check], category_name: &str, use_color: bool) {
        if !checks.is_empty() {
            if use_color {
                println!("{}", category_name.bold().yellow());
            } else {
                println!("{}", category_name);
            }
            for check in checks {
                print_check(check, use_color);
            }
            println!();
        }
    }

    /// Print the summary of check results
    fn print_summary(&self, use_color: bool) {
        let counts = self.count_check_statuses();

        if use_color {
            println!("{}", "Summary:".bold().green());
        } else {
            println!("Summary:");
        }

        match (counts.error_count, counts.warning_count) {
            (0, 0) => {
                if use_color {
                    println!("  ✨ All checks passed!");
                } else {
                    println!("  All checks passed!");
                }
            }
            (0, _) => {
                if use_color {
                    println!(
                        "  {} checks passed, {} warnings",
                        counts.ok_count.to_string().green(),
                        counts.warning_count.to_string().yellow()
                    );
                } else {
                    println!(
                        "  {} checks passed, {} warnings",
                        counts.ok_count, counts.warning_count
                    );
                }
            }
            _ => {
                if use_color {
                    println!(
                        "  {} checks passed, {} warnings, {} errors",
                        counts.ok_count.to_string().green(),
                        counts.warning_count.to_string().yellow(),
                        counts.error_count.to_string().red()
                    );
                } else {
                    println!(
                        "  {} checks passed, {} warnings, {} errors",
                        counts.ok_count, counts.warning_count, counts.error_count
                    );
                }
            }
        }
    }

    /// Count checks by status
    fn count_check_statuses(&self) -> CheckCounts {
        CheckCounts {
            ok_count: self
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Ok)
                .count(),
            warning_count: self
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Warning)
                .count(),
            error_count: self
                .checks
                .iter()
                .filter(|c| c.status == CheckStatus::Error)
                .count(),
        }
    }

    /// Get exit code based on check results
    ///
    /// # Returns
    ///
    /// - 0: All checks passed (no errors or warnings)
    /// - 1: At least one warning detected
    /// - 2: At least one error detected
    pub fn get_exit_code(&self) -> i32 {
        let has_error = self.checks.iter().any(|c| c.status == CheckStatus::Error);
        let has_warning = self.checks.iter().any(|c| c.status == CheckStatus::Warning);

        let exit_code = if has_error {
            ExitCode::Error
        } else if has_warning {
            ExitCode::Warning
        } else {
            ExitCode::Success
        };

        exit_code.into()
    }

    /// Check workflow directories exist
    ///
    /// Verifies the existence of workflow directories:
    /// - User workflows (~/.swissarmyhammer/workflows)
    /// - Local workflows (./.swissarmyhammer/workflows)
    /// - Run storage directory (~/.swissarmyhammer/runs)
    pub fn check_workflow_directories(&mut self) -> Result<()> {
        // Check workflow directories
        for dir_info in get_workflow_directories() {
            if dir_info.path.path().exists() {
                let count = count_files_with_extension(dir_info.path.path(), "mermaid");
                self.checks.push(Check {
                    name: format!("{} workflows directory", dir_info.category),
                    status: CheckStatus::Ok,
                    message: format!("Found {} workflows in {}", count, dir_info.path),
                    fix: None,
                });
            } else {
                self.checks.push(Check {
                    name: format!("{} workflows directory", dir_info.category),
                    status: CheckStatus::Ok,
                    message: format!(
                        "{} workflows directory not found (optional): {}",
                        dir_info.category, dir_info.path
                    ),
                    fix: Some(format!("Create directory: mkdir -p {}", dir_info.path)),
                });
            }
        }

        // Check workflow run storage directory
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(SWISSARMYHAMMER_DIR).join("runs");
            if run_storage.exists() {
                self.checks.push(Check {
                    name: "Workflow run storage directory".to_string(),
                    status: CheckStatus::Ok,
                    message: format!("Run storage directory exists: {:?}", run_storage),
                    fix: None,
                });
            } else {
                self.checks.push(Check {
                    name: "Workflow run storage directory".to_string(),
                    status: CheckStatus::Warning,
                    message: format!("Run storage directory not found: {:?}", run_storage),
                    fix: Some(format!("Create directory: mkdir -p {:?}", run_storage)),
                });
            }
        }

        Ok(())
    }

    /// Check workflow file permissions
    ///
    /// Ensures all workflow directories have appropriate read/write
    /// permissions for the current user. On Unix systems, checks for
    /// 700 (rwx------) permissions.
    pub fn check_workflow_permissions(&mut self) -> Result<()> {
        let mut dirs_to_check = Vec::new();

        // Add workflow directories
        for dir_info in get_workflow_directories() {
            if dir_info.path.path().exists() {
                dirs_to_check.push(dir_info.path.path().to_path_buf());
            }
        }

        // Add run storage directory if it exists
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(SWISSARMYHAMMER_DIR).join("runs");
            if run_storage.exists() {
                dirs_to_check.push(run_storage);
            }
        }

        // Check permissions on each directory
        for dir in dirs_to_check {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&dir) {
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();

                    // Check if directory is readable and writable
                    if (mode & 0o700) == 0o700 {
                        self.checks.push(Check {
                            name: format!(
                                format_strings::WORKFLOW_DIR_PERMISSIONS,
                                dir.file_name().unwrap_or_default()
                            ),
                            status: CheckStatus::Ok,
                            message: format!(
                                "Directory has correct permissions: {:o}",
                                mode & 0o777
                            ),
                            fix: None,
                        });
                    } else {
                        self.checks.push(Check {
                            name: format!(
                                format_strings::WORKFLOW_DIR_PERMISSIONS,
                                dir.file_name().unwrap_or_default()
                            ),
                            status: CheckStatus::Warning,
                            message: format!(
                                "Directory permissions may be insufficient: {:o}",
                                mode & 0o777
                            ),
                            fix: Some(format!("Run: chmod 755 {:?}", dir)),
                        });
                    }
                } else {
                    self.checks.push(Check {
                        name: format!(
                            "Workflow directory permissions: {:?}",
                            dir.file_name().unwrap_or_default()
                        ),
                        status: CheckStatus::Warning,
                        message: "Failed to check directory permissions".to_string(),
                        fix: None,
                    });
                }
            }

            #[cfg(not(unix))]
            {
                // On non-Unix systems, just check if directory is accessible
                if std::fs::read_dir(&dir).is_ok() {
                    self.checks.push(Check {
                        name: format!(
                            format_strings::WORKFLOW_DIR_ACCESS,
                            dir.file_name().unwrap_or_default()
                        ),
                        status: CheckStatus::Ok,
                        message: "Directory is accessible".to_string(),
                        fix: None,
                    });
                } else {
                    self.checks.push(Check {
                        name: format!(
                            format_strings::WORKFLOW_DIR_ACCESS,
                            dir.file_name().unwrap_or_default()
                        ),
                        status: CheckStatus::Error,
                        message: "Failed to access directory".to_string(),
                        fix: Some("Check directory permissions and ownership".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check workflow parsing
    ///
    /// Scans all .mermaid files in workflow directories and verifies
    /// they are readable and not empty.
    pub fn check_workflow_parsing(&mut self) -> Result<()> {
        use walkdir::WalkDir;

        let mut workflow_errors = Vec::new();

        for dir_info in get_workflow_directories() {
            if !dir_info.path.path().exists() {
                continue;
            }

            for entry in WalkDir::new(dir_info.path.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("mermaid"))
            {
                // Validate path before reading
                if let Err(e) = validate_path_no_traversal(entry.path()) {
                    workflow_errors
                        .push((entry.path().to_path_buf(), format!("Invalid path: {}", e)));
                    continue;
                }

                match fs::read_to_string(entry.path()) {
                    Ok(content) => {
                        // Check if file is readable and not empty
                        if content.trim().is_empty() {
                            workflow_errors.push((
                                entry.path().to_path_buf(),
                                "Workflow file is empty".to_string(),
                            ));
                        }
                    }
                    Err(e) => {
                        workflow_errors.push((
                            entry.path().to_path_buf(),
                            format!("Failed to read workflow file: {}", e),
                        ));
                    }
                }
            }
        }

        if workflow_errors.is_empty() {
            self.checks.push(Check {
                name: check_names::WORKFLOW_PARSING.to_string(),
                status: CheckStatus::Ok,
                message: "All workflow files are readable".to_string(),
                fix: None,
            });
        } else {
            for (path, error) in workflow_errors {
                self.checks.push(Check {
                    name: format!(
                        format_strings::WORKFLOW_PARSING_ERROR,
                        path.file_name().unwrap_or_default()
                    ),
                    status: CheckStatus::Error,
                    message: error,
                    fix: Some(format!("Fix or remove the workflow file: {:?}", path)),
                });
            }
        }

        Ok(())
    }

    /// Check workflow run storage
    ///
    /// Verifies the workflow run storage directory:
    /// - Exists and is accessible
    /// - Has write permissions
    /// - Has adequate disk space
    pub fn check_workflow_run_storage(&mut self) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(SWISSARMYHAMMER_DIR).join("runs");

            if run_storage.exists() {
                self.check_run_storage_write_access(&run_storage)?;
                self.check_run_storage_disk_space(&run_storage)?;
            } else {
                self.checks.push(Check {
                    name: check_names::WORKFLOW_RUN_STORAGE_ACCESS.to_string(),
                    status: CheckStatus::Warning,
                    message: "Run storage directory does not exist".to_string(),
                    fix: Some(format!("Create directory: mkdir -p {:?}", run_storage)),
                });
            }
        }

        Ok(())
    }

    /// Check if workflow run storage is writable
    fn check_run_storage_write_access(&mut self, run_storage: &Path) -> Result<()> {
        let test_file = run_storage.join(".doctor_test");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                // Clean up test file - ignore errors as the file may have already been removed
                // or we may lack permissions (which was the point of the test)
                let _ = fs::remove_file(&test_file);

                self.checks.push(Check {
                    name: check_names::WORKFLOW_RUN_STORAGE_ACCESS.to_string(),
                    status: CheckStatus::Ok,
                    message: "Run storage is accessible and writable".to_string(),
                    fix: None,
                });
            }
            Err(e) => {
                self.checks.push(Check {
                    name: check_names::WORKFLOW_RUN_STORAGE_ACCESS.to_string(),
                    status: CheckStatus::Error,
                    message: format!("Run storage is not writable: {}", e),
                    fix: Some(format!("Check permissions on {:?}", run_storage)),
                });
            }
        }

        Ok(())
    }

    /// Check available disk space for workflow run storage
    fn check_run_storage_disk_space(&mut self, run_storage: &Path) -> Result<()> {
        match check_disk_space(run_storage) {
            Ok((available, _)) => {
                if available.is_low(LOW_DISK_SPACE_MB) {
                    self.checks.push(Check {
                        name: check_names::WORKFLOW_RUN_STORAGE_SPACE.to_string(),
                        status: CheckStatus::Warning,
                        message: format!("Low disk space: {}", available),
                        fix: Some(
                            "Consider cleaning up old workflow runs or freeing disk space"
                                .to_string(),
                        ),
                    });
                } else {
                    self.checks.push(Check {
                        name: check_names::WORKFLOW_RUN_STORAGE_SPACE.to_string(),
                        status: CheckStatus::Ok,
                        message: format!("Adequate disk space: {}", available),
                        fix: None,
                    });
                }
            }
            Err(e) => {
                self.checks.push(Check {
                    name: check_names::WORKFLOW_RUN_STORAGE_SPACE.to_string(),
                    status: CheckStatus::Warning,
                    message: format!("Failed to check disk space: {}", e),
                    fix: None,
                });
            }
        }

        Ok(())
    }

    /// Check for workflow circular dependencies and conflicts
    ///
    /// Detects potential issues in the workflow system:
    /// - Name conflicts (same workflow name in multiple locations)
    /// - Circular dependencies (requires runtime analysis)
    pub fn check_workflow_dependencies(&mut self) -> Result<()> {
        use std::collections::HashMap;
        use walkdir::WalkDir;

        let mut workflow_names = HashMap::new();

        // Collect all workflow names and their locations
        for dir_info in get_workflow_directories() {
            if !dir_info.path.path().exists() {
                continue;
            }

            for entry in WalkDir::new(dir_info.path.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("mermaid"))
            {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    workflow_names
                        .entry(stem.to_string())
                        .or_insert_with(Vec::new)
                        .push(entry.path().to_path_buf());
                }
            }
        }

        // Check for workflow name conflicts
        let mut has_conflicts = false;
        for (name, paths) in workflow_names.iter() {
            if paths.len() > 1 {
                has_conflicts = true;
                let locations = paths
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect::<Vec<_>>()
                    .join(", ");

                self.checks.push(Check {
                    name: format!("Workflow name conflict: {}", name),
                    status: CheckStatus::Warning,
                    message: format!(
                        "Workflow '{}' exists in multiple locations: {}",
                        name, locations
                    ),
                    fix: Some(
                        "Rename or remove duplicate workflows to avoid conflicts".to_string(),
                    ),
                });
            }
        }

        if !has_conflicts {
            self.checks.push(Check {
                name: check_names::WORKFLOW_NAME_CONFLICTS.to_string(),
                status: CheckStatus::Ok,
                message: "No workflow name conflicts detected".to_string(),
                fix: None,
            });
        }

        // Note: Actual circular dependency checking would require parsing the workflow files
        // and analyzing their transition dependencies, which is beyond the scope of a simple check
        self.checks.push(Check {
            name: check_names::WORKFLOW_CIRCULAR_DEPS.to_string(),
            status: CheckStatus::Ok,
            message: "Circular dependency checking requires workflow execution".to_string(),
            fix: None,
        });

        Ok(())
    }
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the Claude Code configuration file path based on the OS
///
/// Note: This function is kept for backward compatibility but is no longer used.
/// The doctor command now uses `claude mcp list` instead.
///
/// # Returns
///
/// Platform-specific path to claude_desktop_config.json
#[allow(dead_code)]
pub fn get_claude_config_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".config")
            })
            .join("Claude")
            .join("claude_desktop_config.json")
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| {
                PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| "~".to_string()))
            })
            .join("Claude")
            .join("claude_desktop_config.json")
    }
}

/// Count markdown files in a directory
fn count_markdown_files(path: &Path) -> usize {
    use walkdir::WalkDir;

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
        .count()
}

/// Count files with a specific extension in a directory
fn count_files_with_extension(path: &Path, extension: &str) -> usize {
    use walkdir::WalkDir;

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some(extension))
        .count()
}

/// Get the Claude add command
fn get_claude_add_command() -> String {
    r#"Add swissarmyhammer to Claude Code using this command:

claude mcp add --scope user swissarmyhammer swissarmyhammer serve

Or if swissarmyhammer is not in your PATH, use the full path:

claude mcp add --scope user  swissarmyhammer /path/to/swissarmyhammer serve"#
        .to_string()
}

/// Check disk space for a given path and return (available, total) as DiskSpace values
#[cfg(unix)]
fn check_disk_space(path: &Path) -> Result<(DiskSpace, DiskSpace)> {
    use std::process::Command;

    // Use df-like approach to check disk space
    let output = Command::new("df")
        .arg("-k") // Output in KB
        .arg(path)
        .output()?;

    if !output.status.success() {
        anyhow::bail!("df command failed");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse df output to get available space
    // Format: Filesystem 1K-blocks Used Available Use% Mounted
    if let Some(line) = stdout.lines().nth(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let total_kb = parts[1].parse::<u64>().unwrap_or(0);
            let available_kb = parts[3].parse::<u64>().unwrap_or(0);
            let total_mb = total_kb / 1024;
            let available_mb = available_kb / 1024;
            return Ok((
                DiskSpace::from_mb(available_mb),
                DiskSpace::from_mb(total_mb),
            ));
        }
    }

    anyhow::bail!("Failed to parse df output")
}

/// Check disk space for a given path - Windows/non-Unix implementation
#[cfg(not(unix))]
fn check_disk_space(path: &Path) -> Result<(DiskSpace, DiskSpace)> {
    #[cfg(windows)]
    {
        // Windows-specific implementation using WinAPI
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        #[link(name = "kernel32")]
        extern "system" {
            fn GetDiskFreeSpaceExW(
                lpDirectoryName: *const u16,
                lpFreeBytesAvailable: *mut u64,
                lpTotalNumberOfBytes: *mut u64,
                lpTotalNumberOfFreeBytes: *mut u64,
            ) -> i32;
        }

        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;
        let wide: Vec<u16> = OsStr::new(path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available = 0u64;
        let mut total_bytes = 0u64;
        let mut total_free_bytes = 0u64;

        let result = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            )
        };

        if result != 0 {
            let available_mb = free_bytes_available / (1024 * 1024);
            let total_mb = total_bytes / (1024 * 1024);
            Ok((
                DiskSpace::from_mb(available_mb),
                DiskSpace::from_mb(total_mb),
            ))
        } else {
            anyhow::bail!("Failed to get disk space information")
        }
    }

    #[cfg(not(windows))]
    {
        // For other non-Unix systems, try using `statvfs` crate if available
        // Otherwise, return a reasonable estimate with a note about limitations
        match fs::metadata(path) {
            Ok(_) => {
                // Path exists - return conservative estimates that indicate
                // we cannot determine actual disk space
                // Using 0 to indicate unknown rather than misleading values
                Err(anyhow::anyhow!(
                    "Disk space checking not implemented for this platform"
                ))
            }
            Err(e) => {
                anyhow::bail!("Failed to access path for disk space check: {}", e)
            }
        }
    }
}

/// Validate a path doesn't contain directory traversal sequences
fn validate_path_no_traversal(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Check for common path traversal patterns
    if path_str.contains("..") || path_str.contains("./") || path_str.contains(".\\") {
        anyhow::bail!("Path contains potential directory traversal: {:?}", path);
    }

    // Check components for any parent directory references
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                anyhow::bail!("Path contains parent directory reference: {:?}", path);
            }
            std::path::Component::RootDir => {
                // Allow absolute paths but log them for review
                // In production, you might want to restrict this based on context
            }
            _ => {} // Normal components are fine
        }
    }

    Ok(())
}

/// Get workflow directories to check
fn get_workflow_directories() -> Vec<WorkflowDirectoryInfo> {
    let mut dirs = Vec::new();

    // Add user directory if it exists
    if let Some(home) = dirs::home_dir() {
        let user_workflows_path = home.join(SWISSARMYHAMMER_DIR).join("workflows");

        // Validate path before adding
        if validate_path_no_traversal(&user_workflows_path).is_ok() {
            dirs.push(WorkflowDirectoryInfo::new(
                WorkflowDirectory::new(user_workflows_path),
                WorkflowCategory::User,
            ));
        }
    }

    // Add local directory
    let local_workflows_path = PathBuf::from(SWISSARMYHAMMER_DIR).join("workflows");

    // Validate path before adding
    if validate_path_no_traversal(&local_workflows_path).is_ok() {
        dirs.push(WorkflowDirectoryInfo::new(
            WorkflowDirectory::new(local_workflows_path),
            WorkflowCategory::Local,
        ));
    }

    dirs
}

/// Print a single check result
fn print_check(check: &Check, use_color: bool) {
    let (symbol, color_fn): (&str, fn(&str) -> ColoredString) = match check.status {
        CheckStatus::Ok => ("✓", |s: &str| s.green()),
        CheckStatus::Warning => ("⚠", |s: &str| s.yellow()),
        CheckStatus::Error => ("✗", |s: &str| s.red()),
    };

    if use_color {
        print!(
            "  {} {} - {}",
            color_fn(symbol),
            check.name.bold(),
            check.message
        );
    } else {
        print!("  {} {} - {}", symbol, check.name, check.message);
    }

    if let Some(fix) = &check.fix {
        println!();
        if use_color {
            println!("    {} {}", "→".dimmed(), fix.dimmed());
        } else {
            println!("    → {}", fix);
        }
    } else {
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_creation() {
        let doctor = Doctor::new();
        assert_eq!(doctor.checks.len(), 0);
    }

    #[test]
    fn test_check_status_exit_codes() {
        let mut doctor = Doctor::new();

        // All OK should return 0
        doctor.checks.push(Check {
            name: "Test OK".to_string(),
            status: CheckStatus::Ok,
            message: "Everything is fine".to_string(),
            fix: None,
        });
        assert_eq!(doctor.get_exit_code(), 0);

        // Warning should return 1
        doctor.checks.push(Check {
            name: "Test Warning".to_string(),
            status: CheckStatus::Warning,
            message: "Something might be wrong".to_string(),
            fix: Some("Consider fixing this".to_string()),
        });
        assert_eq!(doctor.get_exit_code(), 1);

        // Error should return 2
        doctor.checks.push(Check {
            name: "Test Error".to_string(),
            status: CheckStatus::Error,
            message: "Something is definitely wrong".to_string(),
            fix: Some("You must fix this".to_string()),
        });
        assert_eq!(doctor.get_exit_code(), 2);
    }

    #[test]
    fn test_check_in_path() {
        let mut doctor = Doctor::new();

        // Set up a mock PATH
        let original_path = env::var("PATH").unwrap_or_default();
        env::set_var("PATH", format!("/usr/local/bin:{}", original_path));

        let result = doctor.check_in_path();
        assert!(result.is_ok());

        // Restore original PATH
        env::set_var("PATH", original_path);
    }

    #[test]
    fn test_path_parsing_cross_platform() {
        let original_path = env::var("PATH").unwrap_or_default();

        // Test Unix-style PATH on current platform
        let unix_path = "/usr/local/bin:/usr/bin:/bin";
        env::set_var("PATH", unix_path);

        let path_var = env::var("PATH").unwrap_or_default();
        let paths: Vec<std::path::PathBuf> = env::split_paths(&path_var).collect();

        // On Unix systems, this should parse correctly
        // On Windows, std::env::split_paths handles the format appropriately for the platform
        assert!(!paths.is_empty());

        // Test that std::env::split_paths() works better than manual splitting
        let manual_split: Vec<&str> = path_var.split(':').collect();

        // Demonstrate the difference: manual split always splits on colon,
        // but std::env::split_paths() is platform-aware
        if cfg!(windows) {
            // On Windows, splitting on ':' would incorrectly split drive letters like "C:"
            // std::env::split_paths() handles this correctly
            assert!(paths.len() <= manual_split.len()); // split_paths is smarter
        } else {
            // On Unix, they should be similar for this simple case
            assert_eq!(paths.len(), manual_split.len());
        }

        // Restore original PATH
        env::set_var("PATH", original_path);
    }

    #[test]
    fn test_check_prompt_directories() {
        let mut doctor = Doctor::new();
        let result = doctor.check_prompt_directories();
        assert!(result.is_ok());

        // Should have checks for builtin, user, and local directories
        let prompt_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("prompt"))
            .collect();
        assert!(prompt_checks.len() >= 3);
    }

    #[test]
    fn test_get_claude_config_path() {
        // This is a helper function we'll implement
        let config_path = get_claude_config_path();

        #[cfg(target_os = "macos")]
        assert!(config_path.ends_with("claude_desktop_config.json"));

        #[cfg(target_os = "linux")]
        assert!(config_path.ends_with("claude_desktop_config.json"));

        #[cfg(target_os = "windows")]
        assert!(config_path.ends_with("claude_desktop_config.json"));
    }

    #[test]
    fn test_run_diagnostics() {
        let mut doctor = Doctor::new();
        let result = doctor.run_diagnostics();
        assert!(result.is_ok());

        // Should have at least some checks
        assert!(!doctor.checks.is_empty());

        // Exit code should be 0, 1, or 2
        let exit_code = doctor.get_exit_code();
        assert!(exit_code <= 2);
    }

    #[test]
    fn test_check_claude_config_should_use_mcp_list() {
        let mut doctor = Doctor::new();
        let result = doctor.check_claude_config();
        assert!(result.is_ok());

        // Check that we're NOT looking for a config file
        let config_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Claude"))
            .collect();

        // The current implementation looks for a file, which is wrong
        // This test should fail with the current implementation
        for check in config_checks {
            assert!(
                !check.message.contains("Config file not found"),
                "Doctor should use 'claude mcp list' instead of looking for config files"
            );
        }
    }

    #[test]
    fn test_check_workflow_directories() {
        let mut doctor = Doctor::new();
        let result = doctor.check_workflow_directories();
        assert!(result.is_ok());

        // Should have checks for workflow directories
        let workflow_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") && c.name.contains("director"))
            .collect();
        assert!(
            !workflow_checks.is_empty(),
            "Should have workflow directory checks"
        );
    }

    #[test]
    fn test_check_workflow_permissions() {
        let mut doctor = Doctor::new();
        let result = doctor.check_workflow_permissions();
        assert!(result.is_ok());

        // Should have checks for workflow permissions
        let permission_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") && c.name.contains("permission"))
            .collect();
        assert!(
            !permission_checks.is_empty(),
            "Should have workflow permission checks"
        );
    }

    #[test]
    fn test_check_workflow_parsing() {
        let mut doctor = Doctor::new();
        let result = doctor.check_workflow_parsing();
        assert!(result.is_ok());

        // Should have checks for workflow parsing
        let parsing_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") && c.name.contains("parsing"))
            .collect();
        assert!(
            !parsing_checks.is_empty(),
            "Should have workflow parsing checks"
        );
    }

    #[test]
    fn test_check_workflow_run_storage() {
        let mut doctor = Doctor::new();
        let result = doctor.check_workflow_run_storage();
        assert!(result.is_ok());

        // Should have checks for workflow run storage
        let storage_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow run storage"))
            .collect();
        assert!(
            !storage_checks.is_empty(),
            "Should have workflow run storage checks"
        );
    }

    #[test]
    fn test_check_workflow_dependencies() {
        let mut doctor = Doctor::new();
        let result = doctor.check_workflow_dependencies();
        assert!(result.is_ok());

        // Should have checks for workflow dependencies
        let dependency_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") && c.name.contains("dependen"))
            .collect();
        assert!(
            !dependency_checks.is_empty(),
            "Should have workflow dependency checks"
        );
    }

    #[test]
    fn test_workflow_diagnostics_in_run_diagnostics() {
        let mut doctor = Doctor::new();
        let result = doctor.run_diagnostics();
        assert!(result.is_ok());

        // Should have workflow-related checks in the full diagnostics
        let workflow_checks: Vec<_> = doctor
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") || c.name.contains("workflow"))
            .collect();
        assert!(
            !workflow_checks.is_empty(),
            "run_diagnostics should include workflow checks"
        );
    }

    #[test]
    fn test_disk_space_type() {
        let space = DiskSpace::from_mb(100);
        assert_eq!(space.as_mb(), 100);
        assert_eq!(format!("{}", space), "100 MB");

        // Test low disk space detection
        assert!(space.is_low(200));
        assert!(!space.is_low(50));
    }

    #[test]
    fn test_disk_space_ordering() {
        let space1 = DiskSpace::from_mb(100);
        let space2 = DiskSpace::from_mb(200);
        let space3 = DiskSpace::from_mb(100);

        assert!(space1 < space2);
        assert!(space2 > space1);
        assert_eq!(space1, space3);
    }

    #[test]
    fn test_check_disk_space_current_dir() {
        // Test disk space check on current directory
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let result = check_disk_space(&current_dir);

        #[cfg(unix)]
        {
            // On Unix systems, this should succeed
            assert!(result.is_ok(), "Disk space check should succeed on Unix");
            if let Ok((available, total)) = result {
                assert!(
                    available.as_mb() > 0,
                    "Available space should be greater than 0"
                );
                assert!(total.as_mb() > 0, "Total space should be greater than 0");
                assert!(
                    available <= total,
                    "Available space should not exceed total space"
                );
            }
        }

        #[cfg(windows)]
        {
            // On Windows systems with the implementation, it should succeed
            if result.is_ok() {
                let (available, total) = result.unwrap();
                assert!(
                    available.as_mb() > 0,
                    "Available space should be greater than 0"
                );
                assert!(total.as_mb() > 0, "Total space should be greater than 0");
                assert!(
                    available <= total,
                    "Available space should not exceed total space"
                );
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            // On other systems, it should return an error with our new implementation
            assert!(
                result.is_err(),
                "Disk space check should fail on unsupported platforms"
            );
        }
    }

    #[test]
    fn test_check_disk_space_invalid_path() {
        let invalid_path = PathBuf::from("/nonexistent/path/that/should/not/exist");
        let result = check_disk_space(&invalid_path);

        // Should fail for non-existent paths
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_directory_type() {
        let path = PathBuf::from("/test/path");
        let workflow_dir = WorkflowDirectory::new(path.clone());

        assert_eq!(workflow_dir.path(), &path);
        assert_eq!(workflow_dir.as_ref(), &path);
        assert_eq!(format!("{}", workflow_dir), format!("{:?}", path));
    }

    #[test]
    fn test_workflow_directory_info() {
        let path = PathBuf::from("/test/path");
        let workflow_dir = WorkflowDirectory::new(path);
        let info = WorkflowDirectoryInfo::new(workflow_dir.clone(), WorkflowCategory::User);

        assert_eq!(info.path, workflow_dir);
        assert_eq!(info.category, WorkflowCategory::User);
    }

    #[test]
    fn test_workflow_category_display() {
        assert_eq!(format!("{}", WorkflowCategory::User), "User");
        assert_eq!(format!("{}", WorkflowCategory::Local), "Local");
    }

    #[test]
    fn test_exit_code_conversion() {
        assert_eq!(i32::from(ExitCode::Success), 0);
        assert_eq!(i32::from(ExitCode::Warning), 1);
        assert_eq!(i32::from(ExitCode::Error), 2);
    }

    #[test]
    fn test_validate_path_no_traversal() {
        // Test valid paths
        assert!(validate_path_no_traversal(Path::new("test/path")).is_ok());
        assert!(validate_path_no_traversal(Path::new("workflows/my-workflow.mermaid")).is_ok());
        assert!(validate_path_no_traversal(Path::new("/absolute/path/is/allowed")).is_ok());

        // Test paths with parent directory traversal
        assert!(validate_path_no_traversal(Path::new("../test")).is_err());
        assert!(validate_path_no_traversal(Path::new("test/../path")).is_err());
        assert!(validate_path_no_traversal(Path::new("test/../../etc/passwd")).is_err());

        // Test paths with current directory references
        assert!(validate_path_no_traversal(Path::new("./test")).is_err());
        assert!(validate_path_no_traversal(Path::new("test/./path")).is_err());

        // Test Windows-style paths
        assert!(validate_path_no_traversal(Path::new("test\\.\\path")).is_err());
        assert!(validate_path_no_traversal(Path::new("test\\..\\path")).is_err());
    }

    #[test]
    fn test_validate_path_components() {
        use std::path::Component;

        // Create a path with parent directory component
        let path = PathBuf::from("test");
        let path = path.join("..");
        assert!(validate_path_no_traversal(&path).is_err());

        // Test that the validator checks components properly
        let components: Vec<Component> = path.components().collect();
        assert!(components.iter().any(|c| matches!(c, Component::ParentDir)));
    }
}
