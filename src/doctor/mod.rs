use anyhow::Result;
use colored::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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
        Self {
            checks: Vec::new(),
        }
    }

    /// Run all diagnostic checks
    pub fn run_diagnostics(&mut self) -> Result<i32> {
        println!("{}", "🔨 SwissArmyHammer Doctor".bold().blue());
        println!("{}", "Running diagnostics...".dimmed());
        println!();
        
        // Run all checks
        self.check_in_path()?;
        self.check_claude_config()?;
        self.check_prompt_directories()?;
        self.check_yaml_parsing()?;
        self.check_file_permissions()?;
        
        // Print results
        self.print_results();
        
        // Return exit code
        Ok(self.get_exit_code())
    }

    /// Check if swissarmyhammer is in PATH
    pub fn check_in_path(&mut self) -> Result<()> {
        let path_var = env::var("PATH").unwrap_or_default();
        let paths: Vec<&str> = path_var.split(':').collect();
        
        let exe_name = "swissarmyhammer";
        let mut found = false;
        let mut found_path = None;
        
        for path in paths {
            let exe_path = Path::new(path).join(exe_name);
            if exe_path.exists() {
                found = true;
                found_path = Some(exe_path);
                break;
            }
        }
        
        if found {
            self.checks.push(Check {
                name: "swissarmyhammer in PATH".to_string(),
                status: CheckStatus::Ok,
                message: format!("Found at: {:?}", found_path.expect("found_path should be Some when found is true")),
                fix: None,
            });
        } else {
            self.checks.push(Check {
                name: "swissarmyhammer in PATH".to_string(),
                status: CheckStatus::Warning,
                message: "swissarmyhammer not found in PATH".to_string(),
                fix: Some("Add swissarmyhammer to your PATH or use the full path in Claude Code config".to_string()),
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
                            name: "Claude Code MCP configuration".to_string(),
                            status: CheckStatus::Ok,
                            message: "swissarmyhammer is configured in Claude Code".to_string(),
                            fix: None,
                        });
                    } else {
                        self.checks.push(Check {
                            name: "Claude Code MCP configuration".to_string(),
                            status: CheckStatus::Warning,
                            message: "swissarmyhammer not found in Claude Code MCP servers".to_string(),
                            fix: Some(get_claude_add_command()),
                        });
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.checks.push(Check {
                        name: "Claude Code MCP configuration".to_string(),
                        status: CheckStatus::Error,
                        message: format!("Failed to run 'claude mcp list': {}", stderr.trim()),
                        fix: Some("Ensure Claude Code is installed and the 'claude' command is available".to_string()),
                    });
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    self.checks.push(Check {
                        name: "Claude Code MCP configuration".to_string(),
                        status: CheckStatus::Error,
                        message: "Claude Code command not found".to_string(),
                        fix: Some("Install Claude Code from https://claude.ai/code or ensure the 'claude' command is in your PATH".to_string()),
                    });
                } else {
                    self.checks.push(Check {
                        name: "Claude Code MCP configuration".to_string(),
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
            name: "Built-in prompts".to_string(),
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
                    name: "User prompts directory".to_string(),
                    status: CheckStatus::Ok,
                    message: format!("Found {} prompts in {:?}", count, user_prompts),
                    fix: None,
                });
            } else {
                self.checks.push(Check {
                    name: "User prompts directory".to_string(),
                    status: CheckStatus::Ok,
                    message: format!("User prompts directory not found (optional): {:?}", user_prompts),
                    fix: Some(format!("Create directory: mkdir -p {:?}", user_prompts)),
                });
            }
        }
        
        // Check local prompts directory
        let local_prompts = PathBuf::from(".swissarmyhammer").join("prompts");
        if local_prompts.exists() {
            let count = count_markdown_files(&local_prompts);
            self.checks.push(Check {
                name: "Local prompts directory".to_string(),
                status: CheckStatus::Ok,
                message: format!("Found {} prompts in {:?}", count, local_prompts),
                fix: None,
            });
        } else {
            self.checks.push(Check {
                name: "Local prompts directory".to_string(),
                status: CheckStatus::Ok,
                message: format!("Local prompts directory not found (optional): {:?}", local_prompts),
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
        let mut dirs_to_check = vec![
            PathBuf::from(".swissarmyhammer").join("prompts"),
        ];
        
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
                                if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
                                    yaml_errors.push((entry.path().to_path_buf(), e.to_string()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yaml_errors.push((entry.path().to_path_buf(), format!("Failed to read file: {}", e)));
                    }
                }
            }
        }
        
        if yaml_errors.is_empty() {
            self.checks.push(Check {
                name: "YAML parsing".to_string(),
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
                    name: "File permissions".to_string(),
                    status: CheckStatus::Ok,
                    message: format!("Can read current directory: {:?}", cwd),
                    fix: None,
                });
            }
            Err(e) => {
                self.checks.push(Check {
                    name: "File permissions".to_string(),
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
        let system_checks: Vec<_> = self.checks.iter()
            .filter(|c| c.name.contains("PATH") || c.name.contains("permissions"))
            .collect();
            
        let config_checks: Vec<_> = self.checks.iter()
            .filter(|c| c.name.contains("Claude") || c.name.contains("config"))
            .collect();
            
        let prompt_checks: Vec<_> = self.checks.iter()
            .filter(|c| c.name.contains("prompt") || c.name.contains("YAML"))
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
        
        // Print summary
        let ok_count = self.checks.iter().filter(|c| c.status == CheckStatus::Ok).count();
        let warning_count = self.checks.iter().filter(|c| c.status == CheckStatus::Warning).count();
        let error_count = self.checks.iter().filter(|c| c.status == CheckStatus::Error).count();
        
        if use_color {
            println!("{}", "Summary:".bold().green());
        } else {
            println!("Summary:");
        }
        
        if error_count > 0 {
            if use_color {
                println!("  {} checks passed, {} warnings, {} errors", 
                    ok_count.to_string().green(),
                    warning_count.to_string().yellow(),
                    error_count.to_string().red()
                );
            } else {
                println!("  {} checks passed, {} warnings, {} errors", 
                    ok_count,
                    warning_count,
                    error_count
                );
            }
        } else if warning_count > 0 {
            if use_color {
                println!("  {} checks passed, {} warnings", 
                    ok_count.to_string().green(),
                    warning_count.to_string().yellow()
                );
            } else {
                println!("  {} checks passed, {} warnings", 
                    ok_count,
                    warning_count
                );
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
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("~")).join(".config"))
            .join("Claude")
            .join("claude_desktop_config.json")
    }
    
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| "~".to_string())))
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

/// Get the Claude add command
fn get_claude_add_command() -> String {
    r#"Add swissarmyhammer to Claude Code using this command:

claude mcp add swissarmyhammer swissarmyhammer serve

Or if swissarmyhammer is not in your PATH, use the full path:

claude mcp add swissarmyhammer /path/to/swissarmyhammer serve"#
    .to_string()
}

/// Print a single check result
fn print_check(check: &Check, use_color: bool) {
    let (symbol, color_fn): (&str, fn(&str) -> ColoredString) = match check.status {
        CheckStatus::Ok => ("✓", |s: &str| s.green()),
        CheckStatus::Warning => ("⚠", |s: &str| s.yellow()),
        CheckStatus::Error => ("✗", |s: &str| s.red()),
    };
    
    if use_color {
        print!("  {} {} - {}", 
            color_fn(symbol),
            check.name.bold(),
            check.message
        );
    } else {
        print!("  {} {} - {}", 
            symbol,
            check.name,
            check.message
        );
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
    fn test_check_prompt_directories() {
        let mut doctor = Doctor::new();
        let result = doctor.check_prompt_directories();
        assert!(result.is_ok());
        
        // Should have checks for builtin, user, and local directories
        let prompt_checks: Vec<_> = doctor.checks.iter()
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
}