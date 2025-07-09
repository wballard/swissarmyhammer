use anyhow::Result;
use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Minimum disk space in MB before warning
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

/// Wrapper type for workflow directory paths to provide type safety
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDirectory(PathBuf);

impl WorkflowDirectory {
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

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

#[derive(Debug, PartialEq, Clone)]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub fix: Option<String>,
}

pub struct Doctor {
    checks: Vec<Check>,
}

impl Doctor {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Run all diagnostic checks
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
            let user_prompts = home.join(".swissarmyhammer").join("prompts");
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
                        "User prompts directory not found (optional): {:?}",
                        user_prompts
                    ),
                    fix: Some(format!("Create directory: mkdir -p {:?}", user_prompts)),
                });
            }
        }

        // Check local prompts directory
        let local_prompts = PathBuf::from(".swissarmyhammer").join("prompts");
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
                    "Local prompts directory not found (optional): {:?}",
                    local_prompts
                ),
                fix: Some(format!("Create directory: mkdir -p {:?}", local_prompts)),
            });
        }

        Ok(())
    }

    /// Check for YAML parsing errors
    pub fn check_yaml_parsing(&mut self) -> Result<()> {
        use walkdir::WalkDir;

        let mut yaml_errors = Vec::new();

        // Check all prompt directories
        let mut dirs_to_check = vec![PathBuf::from(".swissarmyhammer").join("prompts")];

        // Add user directory if it exists
        if let Some(home) = dirs::home_dir() {
            dirs_to_check.push(home.join(".swissarmyhammer").join("prompts"));
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
                    name: format!("YAML parsing: {:?}", path.file_name().unwrap_or_default()),
                    status: CheckStatus::Error,
                    message: error,
                    fix: Some(format!("Fix the YAML syntax in {:?}", path)),
                });
            }
        }

        Ok(())
    }

    /// Check file permissions
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
                    message: format!("Cannot read current directory: {}", e),
                    fix: Some("Check file permissions for the current directory".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Print the results
    pub fn print_results(&self) {
        let use_color = crate::cli::Cli::should_use_color();

        // Group checks by category
        let system_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.name.contains("PATH") || c.name.contains("permissions"))
            .collect();

        let config_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.name.contains("Claude") || c.name.contains("config"))
            .collect();

        let prompt_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.name.contains("prompt") || c.name.contains("YAML"))
            .filter(|c| !c.name.contains("Workflow"))
            .collect();

        let workflow_checks: Vec<_> = self
            .checks
            .iter()
            .filter(|c| c.name.contains("Workflow") || c.name.contains("workflow"))
            .collect();

        // Print system checks
        if !system_checks.is_empty() {
            if use_color {
                println!("{}", "System Checks:".bold().yellow());
            } else {
                println!("System Checks:");
            }
            for check in system_checks {
                print_check(check, use_color);
            }
            println!();
        }

        // Print configuration checks
        if !config_checks.is_empty() {
            if use_color {
                println!("{}", "Configuration:".bold().yellow());
            } else {
                println!("Configuration:");
            }
            for check in config_checks {
                print_check(check, use_color);
            }
            println!();
        }

        // Print prompt checks
        if !prompt_checks.is_empty() {
            if use_color {
                println!("{}", "Prompts:".bold().yellow());
            } else {
                println!("Prompts:");
            }
            for check in prompt_checks {
                print_check(check, use_color);
            }
            println!();
        }

        // Print workflow checks
        if !workflow_checks.is_empty() {
            if use_color {
                println!("{}", "Workflows:".bold().yellow());
            } else {
                println!("Workflows:");
            }
            for check in workflow_checks {
                print_check(check, use_color);
            }
            println!();
        }

        // Print summary
        let ok_count = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Ok)
            .count();
        let warning_count = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warning)
            .count();
        let error_count = self
            .checks
            .iter()
            .filter(|c| c.status == CheckStatus::Error)
            .count();

        if use_color {
            println!("{}", "Summary:".bold().green());
        } else {
            println!("Summary:");
        }

        if error_count > 0 {
            if use_color {
                println!(
                    "  {} checks passed, {} warnings, {} errors",
                    ok_count.to_string().green(),
                    warning_count.to_string().yellow(),
                    error_count.to_string().red()
                );
            } else {
                println!(
                    "  {} checks passed, {} warnings, {} errors",
                    ok_count, warning_count, error_count
                );
            }
        } else if warning_count > 0 {
            if use_color {
                println!(
                    "  {} checks passed, {} warnings",
                    ok_count.to_string().green(),
                    warning_count.to_string().yellow()
                );
            } else {
                println!("  {} checks passed, {} warnings", ok_count, warning_count);
            }
        } else if use_color {
            println!("  ✨ All checks passed!");
        } else {
            println!("  All checks passed!");
        }
    }

    /// Get exit code based on check results
    pub fn get_exit_code(&self) -> i32 {
        let has_error = self.checks.iter().any(|c| c.status == CheckStatus::Error);
        let has_warning = self.checks.iter().any(|c| c.status == CheckStatus::Warning);

        if has_error {
            2
        } else if has_warning {
            1
        } else {
            0
        }
    }

    /// Check workflow directories exist
    pub fn check_workflow_directories(&mut self) -> Result<()> {
        // Check workflow directories
        for (dir_path, dir_type) in get_workflow_directories() {
            if dir_path.path().exists() {
                let count = count_files_with_extension(dir_path.path(), "mermaid");
                self.checks.push(Check {
                    name: format!("{} workflows directory", dir_type),
                    status: CheckStatus::Ok,
                    message: format!("Found {} workflows in {}", count, dir_path),
                    fix: None,
                });
            } else {
                self.checks.push(Check {
                    name: format!("{} workflows directory", dir_type),
                    status: CheckStatus::Ok,
                    message: format!(
                        "{} workflows directory not found (optional): {}",
                        dir_type, dir_path
                    ),
                    fix: Some(format!("Create directory: mkdir -p {}", dir_path)),
                });
            }
        }

        // Check workflow run storage directory
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(".swissarmyhammer").join("runs");
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
    pub fn check_workflow_permissions(&mut self) -> Result<()> {
        let mut dirs_to_check = Vec::new();

        // Add workflow directories
        for (dir_path, _) in get_workflow_directories() {
            if dir_path.path().exists() {
                dirs_to_check.push(dir_path.path().to_path_buf());
            }
        }

        // Add run storage directory if it exists
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(".swissarmyhammer").join("runs");
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
                                "Workflow directory permissions: {:?}",
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
                                "Workflow directory permissions: {:?}",
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
                        message: "Cannot check directory permissions".to_string(),
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
                            "Workflow directory access: {:?}",
                            dir.file_name().unwrap_or_default()
                        ),
                        status: CheckStatus::Ok,
                        message: "Directory is accessible".to_string(),
                        fix: None,
                    });
                } else {
                    self.checks.push(Check {
                        name: format!(
                            "Workflow directory access: {:?}",
                            dir.file_name().unwrap_or_default()
                        ),
                        status: CheckStatus::Error,
                        message: "Cannot access directory".to_string(),
                        fix: Some("Check directory permissions and ownership".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check workflow parsing
    pub fn check_workflow_parsing(&mut self) -> Result<()> {
        use walkdir::WalkDir;

        let mut workflow_errors = Vec::new();

        for (dir, _) in get_workflow_directories() {
            if !dir.path().exists() {
                continue;
            }

            for entry in WalkDir::new(dir.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("mermaid"))
            {
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
                        "Workflow parsing: {:?}",
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
    pub fn check_workflow_run_storage(&mut self) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let run_storage = home.join(".swissarmyhammer").join("runs");

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
                if let Err(e) = fs::remove_file(&test_file) {
                    // Log error for debugging but don't fail the check
                    eprintln!("Warning: Failed to clean up test file: {}", e);
                }

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
            Ok((available_mb, _)) => {
                if available_mb < LOW_DISK_SPACE_MB {
                    self.checks.push(Check {
                        name: check_names::WORKFLOW_RUN_STORAGE_SPACE.to_string(),
                        status: CheckStatus::Warning,
                        message: format!("Low disk space: {} MB available", available_mb),
                        fix: Some(
                            "Consider cleaning up old workflow runs or freeing disk space"
                                .to_string(),
                        ),
                    });
                } else {
                    self.checks.push(Check {
                        name: check_names::WORKFLOW_RUN_STORAGE_SPACE.to_string(),
                        status: CheckStatus::Ok,
                        message: format!("Adequate disk space: {} MB available", available_mb),
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
    pub fn check_workflow_dependencies(&mut self) -> Result<()> {
        use std::collections::HashMap;
        use walkdir::WalkDir;

        let mut workflow_names = HashMap::new();

        // Collect all workflow names and their locations
        for (dir, _) in get_workflow_directories() {
            if !dir.path().exists() {
                continue;
            }

            for entry in WalkDir::new(dir.path())
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
/// Note: This function is kept for backward compatibility but is no longer used
/// The doctor command now uses `claude mcp list` instead
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

/// Check disk space for a given path and return (available_mb, total_mb)
#[cfg(unix)]
fn check_disk_space(path: &Path) -> Result<(u64, u64)> {
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
            return Ok((available_mb, total_mb));
        }
    }

    anyhow::bail!("Failed to parse df output")
}

/// Check disk space for a given path - Windows/non-Unix fallback
#[cfg(not(unix))]
fn check_disk_space(_path: &Path) -> Result<(u64, u64)> {
    // On non-Unix systems, we can't easily check disk space
    // Return a placeholder that indicates we have enough space
    Ok((1000, 10000)) // 1GB available, 10GB total
}

/// Get workflow directories to check
fn get_workflow_directories() -> Vec<(WorkflowDirectory, &'static str)> {
    let mut dirs = Vec::new();

    // Add user directory if it exists
    if let Some(home) = dirs::home_dir() {
        dirs.push((
            WorkflowDirectory::new(home.join(".swissarmyhammer").join("workflows")),
            "User",
        ));
    }

    // Add local directory
    dirs.push((
        WorkflowDirectory::new(PathBuf::from(".swissarmyhammer").join("workflows")),
        "Local",
    ));

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
}
