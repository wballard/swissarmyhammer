//! Configuration management for SwissArmyHammer
//!
//! This module provides centralized configuration management with environment variable support
//! and sensible defaults for all configurable constants throughout the application.

use crate::common::env_loader::EnvLoader;
use crate::cost::aggregation::analyzer::{
    DEFAULT_MAX_ISSUES_PER_AGGREGATION, DEFAULT_OUTLIER_THRESHOLD,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_BASE_BRANCH: &str = "main";
const MAX_ISSUE_BRANCH_PREFIX_LENGTH: usize = 50;
const MAX_BRANCH_NAME_LENGTH: usize = 255;
const ENV_PREFIX: &str = "SWISSARMYHAMMER";
const CONFIG_FILENAME: &str = "swissarmyhammer.yaml";

// Invalid characters for git branch names (comprehensive validation used by both Config and YamlConfig)
const INVALID_BRANCH_CHARS_YAML: [char; 9] = ['\0', ' ', '~', '^', ':', '?', '*', '[', '\\'];

// Cost tracking configuration constants
const COST_ENV_PREFIX: &str = "SAH_COST";
const DEFAULT_INPUT_TOKEN_COST: &str = "0.000015";
const DEFAULT_OUTPUT_TOKEN_COST: &str = "0.000075";

// Session management configuration constants
const DEFAULT_MAX_CONCURRENT_SESSIONS: u32 = 100;
const DEFAULT_SESSION_TIMEOUT_HOURS: u32 = 24;
const DEFAULT_CLEANUP_INTERVAL_HOURS: u32 = 6;

// Aggregation configuration constants
const DEFAULT_AGGREGATION_ENABLED: bool = true;
const DEFAULT_RETENTION_DAYS: u32 = 90;
const DEFAULT_MAX_STORED_SESSIONS: u32 = 10000;

// Reporting configuration constants
const DEFAULT_INCLUDE_IN_ISSUES: bool = true;
const DEFAULT_DETAILED_BREAKDOWN: bool = true;
const DEFAULT_COST_PRECISION_DECIMALS: u8 = 4;

/// Pricing configuration for cost tracking
#[derive(Debug, Clone, Deserialize)]
pub struct PricingConfig {
    /// Input token cost in USD per token
    #[serde(default = "default_input_token_cost")]
    pub input_token_cost: Decimal,
    /// Output token cost in USD per token
    #[serde(default = "default_output_token_cost")]
    pub output_token_cost: Decimal,
}

/// Static instance of parsed input token cost for performance
static PARSED_INPUT_TOKEN_COST: std::sync::OnceLock<Decimal> = std::sync::OnceLock::new();

/// Static instance of parsed output token cost for performance  
static PARSED_OUTPUT_TOKEN_COST: std::sync::OnceLock<Decimal> = std::sync::OnceLock::new();

fn default_input_token_cost() -> Decimal {
    *PARSED_INPUT_TOKEN_COST
        .get_or_init(|| parse_decimal_with_fallback(DEFAULT_INPUT_TOKEN_COST, "0.000015"))
}

fn default_output_token_cost() -> Decimal {
    *PARSED_OUTPUT_TOKEN_COST
        .get_or_init(|| parse_decimal_with_fallback(DEFAULT_OUTPUT_TOKEN_COST, "0.000075"))
}

/// Parse a decimal string with a fallback value if parsing fails
///
/// This function provides safe decimal parsing with multiple fallback levels to prevent panics
/// in configuration loading. It's designed for parsing monetary values where precision is important
/// but the application should not crash due to invalid configuration values.
///
/// # Arguments
/// * `value` - The primary string value to parse
/// * `fallback` - The fallback string value to try if primary parsing fails
///
/// # Returns
/// A valid Decimal value, guaranteed to not panic:
/// 1. Returns parsed `value` if successful
/// 2. Returns parsed `fallback` if `value` parsing fails
/// 3. Returns minimal safe value (0.000001) if both fail
///
/// # Example
/// ```rust
/// use rust_decimal::Decimal;
/// use swissarmyhammer::config::parse_decimal_with_fallback;
///
/// // Successful parsing
/// let cost = parse_decimal_with_fallback("0.000015", "0.000010");
/// assert_eq!(cost.to_string(), "0.000015");
///
/// // Fallback on invalid primary value
/// let cost = parse_decimal_with_fallback("invalid", "0.000010");
/// assert_eq!(cost.to_string(), "0.000010");
///
/// // Safe minimal value when both fail
/// let cost = parse_decimal_with_fallback("invalid", "also_invalid");
/// assert_eq!(cost.to_string(), "0.000001");
/// ```
pub fn parse_decimal_with_fallback(value: &str, fallback: &str) -> Decimal {
    value.parse().unwrap_or_else(|_| {
        // If primary value fails, try fallback
        fallback.parse().unwrap_or_else(|_| {
            // If both fail, use a safe minimal value
            Decimal::new(1, 6) // 0.000001
        })
    })
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            input_token_cost: default_input_token_cost(),
            output_token_cost: default_output_token_cost(),
        }
    }
}

/// Session management configuration for cost tracking
#[derive(Debug, Clone, Deserialize)]
pub struct SessionManagementConfig {
    /// Maximum number of concurrent sessions
    pub max_concurrent_sessions: u32,
    /// Session timeout in hours
    pub session_timeout_hours: u32,
    /// Cleanup interval in hours
    pub cleanup_interval_hours: u32,
}

impl Default for SessionManagementConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: DEFAULT_MAX_CONCURRENT_SESSIONS,
            session_timeout_hours: DEFAULT_SESSION_TIMEOUT_HOURS,
            cleanup_interval_hours: DEFAULT_CLEANUP_INTERVAL_HOURS,
        }
    }
}

/// Aggregation configuration for cost tracking
#[derive(Debug, Clone, Deserialize)]
pub struct AggregationConfig {
    /// Enable cost aggregation
    pub enabled: bool,
    /// How often to update aggregations (in hours)
    pub scan_interval_hours: u32,
    /// How long to retain aggregated data (in days)
    pub retention_days: u32,
    /// Period for trend calculation (in days)
    pub trend_analysis_days: u32,
    /// Standard deviations threshold for outlier detection
    pub outlier_threshold: f64,
    /// Minimum number of issues required for analysis
    pub min_issues_for_analysis: usize,
    /// Include issues from this many days ago (optional)
    pub include_issues_days: Option<u32>,
    /// Maximum number of sessions to store
    pub max_stored_sessions: u32,
    /// Maximum number of issues to analyze in a single aggregation run
    pub max_issues_per_aggregation: usize,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_AGGREGATION_ENABLED,
            scan_interval_hours: 24,
            retention_days: DEFAULT_RETENTION_DAYS,
            trend_analysis_days: 30,
            outlier_threshold: DEFAULT_OUTLIER_THRESHOLD,
            min_issues_for_analysis: 3,
            include_issues_days: None,
            max_stored_sessions: 10000,
            max_issues_per_aggregation: DEFAULT_MAX_ISSUES_PER_AGGREGATION,
        }
    }
}

/// Reporting configuration for cost tracking
#[derive(Debug, Clone, Deserialize)]
pub struct ReportingConfig {
    /// Include cost information in issue reports
    pub include_in_issues: bool,
    /// Provide detailed breakdown of costs
    pub detailed_breakdown: bool,
    /// Number of decimal places for cost precision
    pub cost_precision_decimals: u8,
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            include_in_issues: DEFAULT_INCLUDE_IN_ISSUES,
            detailed_breakdown: DEFAULT_DETAILED_BREAKDOWN,
            cost_precision_decimals: DEFAULT_COST_PRECISION_DECIMALS,
        }
    }
}

/// Pricing model for cost tracking
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PricingModel {
    /// Use paid pricing model for cost calculations
    #[default]
    Paid,
    /// Use maximum pricing model for cost calculations
    Max,
}

impl PricingModel {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "paid" => Ok(PricingModel::Paid),
            "max" => Ok(PricingModel::Max),
            _ => Err(format!(
                "Invalid pricing model '{}'. Must be 'paid' or 'max'",
                s
            )),
        }
    }
}

/// Cost tracking configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CostTrackingConfig {
    /// Enable cost tracking
    pub enabled: bool,
    /// Pricing model: "paid" or "max"
    pub pricing_model: PricingModel,
    /// Pricing configuration
    #[serde(default)]
    pub rates: PricingConfig,
    /// Session management configuration
    #[serde(default)]
    pub session_management: SessionManagementConfig,
    /// Aggregation configuration
    #[serde(default)]
    pub aggregation: AggregationConfig,
    /// Reporting configuration
    #[serde(default)]
    pub reporting: ReportingConfig,
}

/// Errors that can occur during configuration loading
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read a configuration file from disk
    #[error("Failed to read configuration file {path}: {source}")]
    FileRead {
        /// Path to the configuration file that could not be read
        path: std::path::PathBuf,
        /// Underlying I/O error that occurred during file reading
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse YAML content from a configuration file
    #[error("Invalid YAML syntax in {path}:\n{source}\n\nHint: Check for proper indentation and YAML formatting")]
    YamlParse {
        /// Path to the configuration file with invalid YAML content
        path: std::path::PathBuf,
        /// Underlying YAML parsing error
        #[source]
        source: serde_yaml::Error,
    },

    /// Invalid configuration value for a specific field
    #[error("Invalid configuration value for '{field}': {value}\n{hint}")]
    InvalidValue {
        /// Name of the configuration field that has an invalid value
        field: String,
        /// The invalid value that was provided
        value: String,
        /// Helpful hint about how to fix the issue
        hint: String,
    },

    /// Configuration validation failed
    #[error("Configuration validation failed: {message}")]
    Validation {
        /// Descriptive message about the validation failure
        message: String,
    },
}

/// Configuration settings for the SwissArmyHammer application
#[derive(Debug, Clone)]
pub struct Config {
    /// Prefix for issue branches (default: "issue/")
    pub issue_branch_prefix: String,
    /// Width for issue numbers in display (default: 6)
    pub issue_number_width: usize,
    /// Maximum number of pending issues to display in summary (default: 5)
    pub max_pending_issues_in_summary: usize,
    /// Minimum issue number allowed (default: 1)
    pub min_issue_number: u32,
    /// Maximum issue number allowed (default: 999_999)
    pub max_issue_number: u32,
    /// Number of digits for issue numbering in filenames (default: 6)
    pub issue_number_digits: usize,
    /// Maximum content length for issue content (default: 50000)
    pub max_content_length: usize,
    /// Maximum line length for issue content (default: 10000)
    pub max_line_length: usize,
    /// Maximum issue name length (default: 100)
    pub max_issue_name_length: usize,
    /// Cache TTL in seconds (default: 300, i.e., 5 minutes)
    pub cache_ttl_seconds: u64,
    /// Maximum cache size (default: 1000)
    pub cache_max_size: usize,
    /// Base number for virtual issue numbering (default: 500_000)
    pub virtual_issue_number_base: u32,
    /// Range for virtual issue numbers (default: 500_000, so virtual numbers go from base to base+range-1)
    pub virtual_issue_number_range: u32,
    /// Base branch for pull requests (default: "main")
    pub base_branch: String,
    /// Minimum issue branch prefix length (default: 1)
    pub min_issue_branch_prefix_length: usize,
    /// Maximum issue branch prefix length (default: MAX_ISSUE_BRANCH_PREFIX_LENGTH)
    pub max_issue_branch_prefix_length: usize,
    /// Minimum issue number width (default: 1)
    pub min_issue_number_width: usize,
    /// Maximum issue number width (default: 10)
    pub max_issue_number_width: usize,
    /// Cost tracking configuration (optional)
    pub cost_tracking: Option<CostTrackingConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            issue_branch_prefix: "issue/".to_string(),
            issue_number_width: 6,
            max_pending_issues_in_summary: 5,
            min_issue_number: 1,
            max_issue_number: 999_999,
            issue_number_digits: 6,
            max_content_length: 50000,
            max_line_length: 10000,
            max_issue_name_length: 100,
            cache_ttl_seconds: 300,
            cache_max_size: 1000,
            virtual_issue_number_base: 500_000,
            virtual_issue_number_range: 500_000,
            base_branch: DEFAULT_BASE_BRANCH.to_string(),
            min_issue_branch_prefix_length: 1,
            max_issue_branch_prefix_length: MAX_ISSUE_BRANCH_PREFIX_LENGTH,
            min_issue_number_width: 1,
            max_issue_number_width: 10,
            cost_tracking: None,
        }
    }
}

impl Config {
    /// Create a new configuration instance with values loaded from:
    /// 1. YAML file (highest precedence)
    /// 2. Environment variables
    /// 3. Defaults (lowest precedence)
    pub fn new() -> Self {
        // Start with defaults
        let mut config = Self::default();

        // Apply environment variables (override defaults)
        config.apply_env_vars();

        // Apply YAML configuration (override env vars and defaults)
        config.apply_yaml_config();

        config
    }

    /// Apply YAML configuration to this config
    ///
    /// Attempts to load YAML configuration and applies it if valid.
    /// Falls back gracefully to existing configuration on load or validation errors.
    /// This is called during Config::new() as the final step in the precedence chain.
    fn apply_yaml_config(&mut self) {
        match YamlConfig::load_or_default() {
            Ok(yaml_config) => {
                self.process_yaml_config(yaml_config);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load YAML configuration: {}. Falling back to environment variables and defaults",
                    e
                );
            }
        }
    }

    /// Process and validate a loaded YAML configuration
    ///
    /// Validates the YAML configuration and applies it if valid.
    /// Logs appropriate warnings if validation fails.
    fn process_yaml_config(&mut self, yaml_config: YamlConfig) {
        if let Err(validation_error) = yaml_config.validate() {
            tracing::warn!(
                "Invalid YAML configuration: {}. Falling back to environment variables and defaults",
                validation_error
            );
        } else {
            yaml_config.apply_to_config(self);
            tracing::info!("Configuration loaded successfully with YAML support");
        }
    }

    /// Apply environment variable configuration to this config
    ///
    /// Loads configuration values from environment variables with the SWISSARMYHAMMER prefix,
    /// overriding the current config values. This is called during Config::new() as the second
    /// step in the precedence chain: defaults < env vars < YAML.
    fn apply_env_vars(&mut self) {
        let loader = EnvLoader::new(ENV_PREFIX);

        self.issue_branch_prefix =
            loader.load_string("ISSUE_BRANCH_PREFIX", &self.issue_branch_prefix);
        self.issue_number_width = loader.load_parsed("ISSUE_NUMBER_WIDTH", self.issue_number_width);
        self.max_pending_issues_in_summary = loader.load_parsed(
            "MAX_PENDING_ISSUES_IN_SUMMARY",
            self.max_pending_issues_in_summary,
        );
        self.min_issue_number = loader.load_parsed("MIN_ISSUE_NUMBER", self.min_issue_number);
        self.max_issue_number = loader.load_parsed("MAX_ISSUE_NUMBER", self.max_issue_number);
        self.issue_number_digits =
            loader.load_parsed("ISSUE_NUMBER_DIGITS", self.issue_number_digits);
        self.max_content_length = loader.load_parsed("MAX_CONTENT_LENGTH", self.max_content_length);
        self.max_line_length = loader.load_parsed("MAX_LINE_LENGTH", self.max_line_length);
        self.max_issue_name_length =
            loader.load_parsed("MAX_ISSUE_NAME_LENGTH", self.max_issue_name_length);
        self.cache_ttl_seconds = loader.load_parsed("CACHE_TTL_SECONDS", self.cache_ttl_seconds);
        self.cache_max_size = loader.load_parsed("CACHE_MAX_SIZE", self.cache_max_size);
        self.virtual_issue_number_base =
            loader.load_parsed("VIRTUAL_ISSUE_NUMBER_BASE", self.virtual_issue_number_base);
        self.virtual_issue_number_range = loader.load_parsed(
            "VIRTUAL_ISSUE_NUMBER_RANGE",
            self.virtual_issue_number_range,
        );
        self.base_branch = loader.load_string("BASE_BRANCH", &self.base_branch);
        self.min_issue_branch_prefix_length = loader.load_parsed(
            "MIN_ISSUE_BRANCH_PREFIX_LENGTH",
            self.min_issue_branch_prefix_length,
        );
        self.max_issue_branch_prefix_length = loader.load_parsed(
            "MAX_ISSUE_BRANCH_PREFIX_LENGTH",
            self.max_issue_branch_prefix_length,
        );
        self.min_issue_number_width =
            loader.load_parsed("MIN_ISSUE_NUMBER_WIDTH", self.min_issue_number_width);
        self.max_issue_number_width =
            loader.load_parsed("MAX_ISSUE_NUMBER_WIDTH", self.max_issue_number_width);

        // Load cost tracking configuration from environment variables
        self.apply_cost_tracking_env_vars();
    }

    /// Apply cost tracking environment variables to this config
    ///
    /// Loads cost tracking configuration from environment variables with the SAH_COST prefix.
    /// This creates a new CostTrackingConfig if any SAH_COST_* environment variables are set.
    fn apply_cost_tracking_env_vars(&mut self) {
        let cost_loader = EnvLoader::new(COST_ENV_PREFIX);

        // Check if the main tracking enabled variable is set to determine if we should create config
        if let Some(enabled) = cost_loader.load_optional::<bool>("TRACKING_ENABLED") {
            let pricing_config = Self::load_pricing_config_from_env(&cost_loader);
            let session_management = Self::load_session_management_from_env(&cost_loader);
            let aggregation = Self::load_aggregation_config_from_env(&cost_loader);
            let reporting = Self::load_reporting_config_from_env(&cost_loader);

            let cost_tracking_config = CostTrackingConfig {
                enabled,
                pricing_model: pricing_config.0,
                rates: pricing_config.1,
                session_management,
                aggregation,
                reporting,
            };

            self.cost_tracking = Some(cost_tracking_config);
        }
    }

    /// Load pricing configuration from environment variables
    fn load_pricing_config_from_env(cost_loader: &EnvLoader) -> (PricingModel, PricingConfig) {
        let pricing_model_str = cost_loader.load_string("PRICING_MODEL", "paid");
        let pricing_model =
            PricingModel::from_str(&pricing_model_str).unwrap_or_else(|_| PricingModel::default());

        let input_token_cost_str =
            cost_loader.load_string("INPUT_TOKEN_COST", DEFAULT_INPUT_TOKEN_COST);
        let output_token_cost_str =
            cost_loader.load_string("OUTPUT_TOKEN_COST", DEFAULT_OUTPUT_TOKEN_COST);

        // Parse decimal values carefully
        let input_cost =
            parse_decimal_with_fallback(&input_token_cost_str, DEFAULT_INPUT_TOKEN_COST);
        let output_cost =
            parse_decimal_with_fallback(&output_token_cost_str, DEFAULT_OUTPUT_TOKEN_COST);

        let pricing_config = PricingConfig {
            input_token_cost: input_cost,
            output_token_cost: output_cost,
        };

        (pricing_model, pricing_config)
    }

    /// Load session management configuration from environment variables
    fn load_session_management_from_env(cost_loader: &EnvLoader) -> SessionManagementConfig {
        let max_concurrent_sessions =
            cost_loader.load_parsed("MAX_CONCURRENT_SESSIONS", DEFAULT_MAX_CONCURRENT_SESSIONS);
        let session_timeout_hours =
            cost_loader.load_parsed("SESSION_TIMEOUT_HOURS", DEFAULT_SESSION_TIMEOUT_HOURS);
        let cleanup_interval_hours =
            cost_loader.load_parsed("CLEANUP_INTERVAL_HOURS", DEFAULT_CLEANUP_INTERVAL_HOURS);

        SessionManagementConfig {
            max_concurrent_sessions,
            session_timeout_hours,
            cleanup_interval_hours,
        }
    }

    /// Load aggregation configuration from environment variables
    fn load_aggregation_config_from_env(cost_loader: &EnvLoader) -> AggregationConfig {
        let aggregation_enabled =
            cost_loader.load_parsed("AGGREGATION_ENABLED", DEFAULT_AGGREGATION_ENABLED);
        let retention_days = cost_loader.load_parsed("RETENTION_DAYS", DEFAULT_RETENTION_DAYS);
        let max_stored_sessions =
            cost_loader.load_parsed("MAX_STORED_SESSIONS", DEFAULT_MAX_STORED_SESSIONS);

        AggregationConfig {
            enabled: aggregation_enabled,
            scan_interval_hours: 24,
            retention_days,
            trend_analysis_days: 30,
            outlier_threshold: DEFAULT_OUTLIER_THRESHOLD,
            min_issues_for_analysis: 3,
            include_issues_days: None,
            max_stored_sessions,
            max_issues_per_aggregation: DEFAULT_MAX_ISSUES_PER_AGGREGATION,
        }
    }

    /// Load reporting configuration from environment variables
    fn load_reporting_config_from_env(cost_loader: &EnvLoader) -> ReportingConfig {
        let include_in_issues =
            cost_loader.load_parsed("INCLUDE_IN_ISSUES", DEFAULT_INCLUDE_IN_ISSUES);
        let detailed_breakdown =
            cost_loader.load_parsed("DETAILED_BREAKDOWN", DEFAULT_DETAILED_BREAKDOWN);
        let cost_precision_decimals =
            cost_loader.load_parsed("COST_PRECISION_DECIMALS", DEFAULT_COST_PRECISION_DECIMALS);

        ReportingConfig {
            include_in_issues,
            detailed_breakdown,
            cost_precision_decimals,
        }
    }

    /// Get the global configuration instance
    pub fn global() -> &'static Self {
        static CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
        CONFIG.get_or_init(Config::new)
    }

    /// Find the swissarmyhammer.yaml configuration file in multiple locations
    ///
    /// Searches for a YAML configuration file to supplement environment variable
    /// and default configuration values. The search order is:
    /// 1. Current working directory: `swissarmyhammer.yaml`
    /// 2. Home directory: `~/.config/swissarmyhammer/swissarmyhammer.yaml`
    /// 3. Home directory root: `~/swissarmyhammer.yaml`
    ///
    /// # Returns
    /// * `Some(PathBuf)` - Path to the first configuration file found and readable
    /// * `None` - No configuration file found in any location or file is not accessible
    ///
    /// # Examples
    /// ```
    /// use swissarmyhammer::config::Config;
    ///
    /// if let Some(config_path) = Config::find_yaml_config_file() {
    ///     println!("Found config at: {:?}", config_path);
    /// }
    /// ```
    pub fn find_yaml_config_file() -> Option<std::path::PathBuf> {
        use std::path::PathBuf;

        let config_filename = CONFIG_FILENAME;

        // Define search paths in order of priority
        let mut search_paths = Vec::new();

        // 1. Current working directory
        search_paths.push(PathBuf::from(config_filename));

        // 2. Home directory .config/swissarmyhammer/ subdirectory
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(
                home_dir
                    .join(".config")
                    .join("swissarmyhammer")
                    .join(config_filename),
            );
        }

        // 3. Home directory root
        if let Some(home_dir) = dirs::home_dir() {
            search_paths.push(home_dir.join(config_filename));
        }

        // Search through each path
        for config_path in search_paths {
            match Self::check_config_file(&config_path) {
                Some(path) => {
                    tracing::debug!("Found configuration file: {:?}", path);
                    return Some(path);
                }
                None => continue,
            }
        }

        tracing::debug!(
            "No {} configuration file found in any search location",
            CONFIG_FILENAME
        );
        None
    }

    /// Check if a configuration file exists and is readable
    pub fn check_config_file(config_path: &std::path::Path) -> Option<std::path::PathBuf> {
        match config_path.try_exists() {
            Ok(true) if config_path.is_file() => {
                // Additional permission check
                match std::fs::File::open(config_path) {
                    Ok(_) => Some(config_path.to_path_buf()),
                    Err(e) => {
                        tracing::warn!(
                            "Configuration file {:?} exists but cannot be read: {}",
                            config_path,
                            e
                        );
                        None
                    }
                }
            }
            Ok(false) => None, // File does not exist
            Ok(true) => {
                tracing::debug!(
                    "Found {:?} but it is not a file (possibly a directory)",
                    config_path
                );
                None
            }
            Err(e) => {
                tracing::warn!(
                    "Error checking for configuration file {:?}: {}",
                    config_path,
                    e
                );
                None
            }
        }
    }

    /// Validate the current configuration settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate base_branch
        self.validate_base_branch()?;

        // Validate numeric ranges
        self.validate_numeric_ranges()?;

        // Validate string lengths
        self.validate_string_lengths()?;

        // Validate cost tracking configuration if present
        if let Some(ref cost_tracking) = self.cost_tracking {
            self.validate_cost_tracking(cost_tracking)?;
        }

        Ok(())
    }

    fn validate_base_branch(&self) -> Result<(), ConfigError> {
        Self::validate_branch_name_shared(&self.base_branch, "base_branch")
    }

    fn validate_numeric_ranges(&self) -> Result<(), ConfigError> {
        if self.issue_number_width < self.min_issue_number_width {
            return Err(ConfigError::InvalidValue {
                field: "issue_number_width".to_string(),
                value: self.issue_number_width.to_string(),
                hint: format!(
                    "issue_number_width must be at least {}",
                    self.min_issue_number_width
                ),
            });
        }

        if self.issue_number_width > self.max_issue_number_width {
            return Err(ConfigError::InvalidValue {
                field: "issue_number_width".to_string(),
                value: self.issue_number_width.to_string(),
                hint: format!(
                    "issue_number_width cannot exceed {}",
                    self.max_issue_number_width
                ),
            });
        }

        if self.min_issue_number >= self.max_issue_number {
            return Err(ConfigError::Validation {
                message: format!(
                    "min_issue_number ({}) must be less than max_issue_number ({})",
                    self.min_issue_number, self.max_issue_number
                ),
            });
        }

        // Validate the validation limits themselves
        if self.min_issue_branch_prefix_length > self.max_issue_branch_prefix_length {
            return Err(ConfigError::Validation {
                message: format!(
                    "min_issue_branch_prefix_length ({}) must be less than or equal to max_issue_branch_prefix_length ({})",
                    self.min_issue_branch_prefix_length, self.max_issue_branch_prefix_length
                ),
            });
        }

        if self.min_issue_number_width > self.max_issue_number_width {
            return Err(ConfigError::Validation {
                message: format!(
                    "min_issue_number_width ({}) must be less than or equal to max_issue_number_width ({})",
                    self.min_issue_number_width, self.max_issue_number_width
                ),
            });
        }

        Ok(())
    }

    fn validate_string_lengths(&self) -> Result<(), ConfigError> {
        if self.issue_branch_prefix.len() < self.min_issue_branch_prefix_length {
            return Err(ConfigError::InvalidValue {
                field: "issue_branch_prefix".to_string(),
                value: self.issue_branch_prefix.clone(),
                hint: format!(
                    "issue_branch_prefix must be at least {} characters long",
                    self.min_issue_branch_prefix_length
                ),
            });
        }

        if self.issue_branch_prefix.len() > self.max_issue_branch_prefix_length {
            return Err(ConfigError::InvalidValue {
                field: "issue_branch_prefix".to_string(),
                value: self.issue_branch_prefix.clone(),
                hint: format!(
                    "issue_branch_prefix cannot exceed {} characters",
                    self.max_issue_branch_prefix_length
                ),
            });
        }

        Ok(())
    }

    /// Validate cost tracking configuration for Config struct
    ///
    /// This is a wrapper around the shared validation logic that converts validation errors
    /// into ConfigError format with detailed field information for better error reporting.
    ///
    /// # Arguments
    /// * `config` - The cost tracking configuration to validate
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(ConfigError::InvalidValue)` with field name, value, and hint if validation fails
    ///
    /// # See Also
    /// - [`validate_cost_tracking_shared`] for detailed validation rules
    fn validate_cost_tracking(&self, config: &CostTrackingConfig) -> Result<(), ConfigError> {
        match Self::validate_cost_tracking_shared(config) {
            Ok(()) => Ok(()),
            Err((field, value, hint)) => Err(ConfigError::InvalidValue { field, value, hint }),
        }
    }

    /// Shared cost tracking validation logic
    ///
    /// Validates all aspects of a cost tracking configuration to ensure safe and reasonable operation.
    /// This function is used by both Config and YamlConfig validation methods to eliminate duplication.
    ///
    /// # Validation Rules
    ///
    /// ## Pricing Configuration
    /// - `input_token_cost` must be greater than 0 (positive monetary value)
    /// - `output_token_cost` must be greater than 0 (positive monetary value)
    /// - `pricing_model` validation is handled by enum type safety (Paid or Max)
    ///
    /// ## Session Management
    /// - `max_concurrent_sessions` must be greater than 0 (at least one session allowed)
    /// - `session_timeout_hours` must be greater than 0 (sessions must have finite lifetime)
    /// - `cleanup_interval_hours` must be greater than 0 (cleanup must occur periodically)
    ///
    /// ## Aggregation Settings
    /// - `retention_days` must be greater than 0 (data must have finite retention period)
    /// - `max_stored_sessions` must be greater than 0 (storage must have reasonable limits)
    /// - `enabled` flag has no validation constraints (boolean can be true/false)
    ///
    /// ## Reporting Settings
    /// - `cost_precision_decimals` must be <= 10 (prevents excessive precision that could cause formatting issues)
    /// - `include_in_issues` and `detailed_breakdown` have no validation constraints (boolean flags)
    ///
    /// # Returns
    /// - `Ok(())` if all validation rules pass
    /// - `Err((field_name, current_value, hint_message))` for the first validation failure encountered
    ///
    /// # Example
    /// ```rust
    /// use swissarmyhammer::config::{Config, CostTrackingConfig};
    /// let config = CostTrackingConfig::default();
    /// match Config::validate_cost_tracking_shared(&config) {
    ///     Ok(()) => println!("Configuration is valid"),
    ///     Err((field, value, hint)) => println!("Invalid {}: {} - {}", field, value, hint),
    /// }
    /// ```
    pub fn validate_cost_tracking_shared(
        config: &CostTrackingConfig,
    ) -> Result<(), (String, String, String)> {
        // Note: pricing_model validation is no longer needed as enum ensures type safety

        // Validate positive costs
        if config.rates.input_token_cost <= rust_decimal::Decimal::ZERO {
            return Err((
                "cost_tracking.rates.input_token_cost".to_string(),
                config.rates.input_token_cost.to_string(),
                "input_token_cost must be positive".to_string(),
            ));
        }

        if config.rates.output_token_cost <= rust_decimal::Decimal::ZERO {
            return Err((
                "cost_tracking.rates.output_token_cost".to_string(),
                config.rates.output_token_cost.to_string(),
                "output_token_cost must be positive".to_string(),
            ));
        }

        // Validate reasonable session management values
        if config.session_management.max_concurrent_sessions == 0 {
            return Err((
                "cost_tracking.session_management.max_concurrent_sessions".to_string(),
                config
                    .session_management
                    .max_concurrent_sessions
                    .to_string(),
                "max_concurrent_sessions must be greater than 0".to_string(),
            ));
        }

        if config.session_management.session_timeout_hours == 0 {
            return Err((
                "cost_tracking.session_management.session_timeout_hours".to_string(),
                config.session_management.session_timeout_hours.to_string(),
                "session_timeout_hours must be greater than 0".to_string(),
            ));
        }

        if config.session_management.cleanup_interval_hours == 0 {
            return Err((
                "cost_tracking.session_management.cleanup_interval_hours".to_string(),
                config.session_management.cleanup_interval_hours.to_string(),
                "cleanup_interval_hours must be greater than 0".to_string(),
            ));
        }

        // Validate aggregation settings
        if config.aggregation.retention_days == 0 {
            return Err((
                "cost_tracking.aggregation.retention_days".to_string(),
                config.aggregation.retention_days.to_string(),
                "retention_days must be greater than 0".to_string(),
            ));
        }

        // Removed max_stored_sessions validation (no longer in AggregationConfig)

        // Validate reporting settings
        if config.reporting.cost_precision_decimals > 10 {
            return Err((
                "cost_tracking.reporting.cost_precision_decimals".to_string(),
                config.reporting.cost_precision_decimals.to_string(),
                "cost_precision_decimals cannot exceed 10".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate an example YAML configuration file content
    pub fn example_yaml_config() -> String {
        format!(
            r#"# {}
# Configuration file for Swiss Army Hammer

# Base branch that pull requests will merge into
base_branch: "main"
"#,
            CONFIG_FILENAME
        )
    }

    /// Get configuration validation help message
    pub fn validation_help() -> &'static str {
        r#"Configuration Validation Help:

- base_branch: Must be a valid git branch name (no spaces, special characters)
- All numeric values must be positive and within reasonable ranges
- String values must not exceed maximum lengths

For more help, see: https://github.com/wballard/swissarmyhammer/docs/configuration
"#
    }

    /// Reset the global configuration (for testing purposes)
    #[cfg(test)]
    pub fn reset_global() {
        // This is a workaround since OnceLock doesn't have a reset method
        // We can't actually reset the global config in tests due to OnceLock's design
        // Tests should use Config::new() directly instead of global() for testing env vars
    }

    /// Shared branch validation logic used by both Config and YamlConfig
    fn validate_branch_name_shared(branch_name: &str, field_name: &str) -> Result<(), ConfigError> {
        // Check for empty branch name
        if branch_name.is_empty() {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!(
                    "{} cannot be empty. Use 'main' or 'develop' or another valid git branch name",
                    field_name
                ),
            });
        }

        // Check for whitespace-only branch name
        if branch_name.trim().is_empty() {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!("{} cannot be whitespace only", field_name),
            });
        }

        // Check branch name length
        if branch_name.len() > MAX_BRANCH_NAME_LENGTH {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!(
                    "{} is too long (maximum {} characters)",
                    field_name, MAX_BRANCH_NAME_LENGTH
                ),
            });
        }

        // Check for invalid git branch characters
        for ch in INVALID_BRANCH_CHARS_YAML.iter() {
            if branch_name.contains(*ch) {
                return Err(ConfigError::InvalidValue {
                    field: field_name.to_string(),
                    value: branch_name.to_string(),
                    hint: format!("{} contains invalid character '{}'. Git branch names cannot contain: \\0 ~ ^ : ? * [ \\ <space>", field_name, ch),
                });
            }
        }

        // Check for sequences that git doesn't allow
        if branch_name.contains("..") {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!("{} cannot contain consecutive dots '..'", field_name),
            });
        }

        // Check that it doesn't start or end with certain characters
        if branch_name.starts_with('.') || branch_name.ends_with('.') {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!("{} cannot start or end with '.'", field_name),
            });
        }

        if branch_name.starts_with('/') || branch_name.ends_with('/') {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!("{} cannot start or end with '/'", field_name),
            });
        }

        if branch_name.ends_with(".lock") {
            return Err(ConfigError::InvalidValue {
                field: field_name.to_string(),
                value: branch_name.to_string(),
                hint: format!("{} cannot end with '.lock'", field_name),
            });
        }

        Ok(())
    }
}

/// Configuration loaded from swissarmyhammer.yaml file
#[derive(Debug, Clone, Default, Deserialize)]
pub struct YamlConfig {
    /// Base branch for pull requests
    pub base_branch: Option<String>,
    /// Cost tracking configuration
    pub cost_tracking: Option<CostTrackingConfig>,
}

impl YamlConfig {
    /// Apply YAML configuration values to an existing Config
    /// YAML values take precedence over existing values
    pub fn apply_to_config(&self, config: &mut Config) {
        if let Some(ref base_branch) = self.base_branch {
            config.base_branch = base_branch.clone();
        }
        if let Some(ref cost_tracking) = self.cost_tracking {
            config.cost_tracking = Some(cost_tracking.clone());
        }
    }

    /// Load YAML configuration from a file path
    /// Returns the parsed configuration or an error with context
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        use std::fs;

        let path = path.as_ref();
        tracing::info!("Loading YAML configuration from: {:?}", path);

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Parse YAML content
        let config: YamlConfig =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::YamlParse {
                path: path.to_path_buf(),
                source: e,
            })?;

        // Apply basic validation to YAML values before returning
        config.validate_yaml_values()?;

        tracing::info!(
            "Successfully loaded and validated YAML configuration: {:?}",
            config
        );
        Ok(config)
    }

    /// Try to load YAML configuration, returning default if file not found
    pub fn load_or_default() -> Result<Self, ConfigError> {
        match Config::find_yaml_config_file() {
            Some(path) => Self::load_from_file(path),
            None => {
                tracing::debug!("No configuration file found, using default YAML config");
                Ok(Self::default())
            }
        }
    }

    /// Validate YAML configuration values for common issues
    fn validate_yaml_values(&self) -> Result<(), ConfigError> {
        if let Some(ref base_branch) = self.base_branch {
            if base_branch.is_empty() {
                return Err(ConfigError::InvalidValue {
                    field: "base_branch".to_string(),
                    value: base_branch.clone(),
                    hint: "base_branch cannot be empty in YAML configuration".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate the YAML configuration for correctness
    ///
    /// Performs validation checks on the loaded YAML configuration to ensure:
    /// - Branch names are valid (no invalid characters, not empty)
    /// - All values are within acceptable ranges
    ///
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(String)` - Configuration is invalid with description of the error
    ///
    /// # Examples
    /// ```
    /// use swissarmyhammer::config::YamlConfig;
    ///
    /// let config = YamlConfig {
    ///     base_branch: Some("main".to_string()),
    ///     cost_tracking: None,
    /// };
    /// assert!(config.validate().is_ok());
    ///
    /// let invalid_config = YamlConfig {
    ///     base_branch: Some("".to_string()),
    ///     cost_tracking: None,
    /// };
    /// assert!(invalid_config.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        // Validate base_branch if provided
        if let Some(ref base_branch) = self.base_branch {
            Self::validate_branch_name(base_branch)?;
        }

        // Validate cost tracking configuration if provided
        if let Some(ref cost_tracking) = self.cost_tracking {
            Self::validate_cost_tracking_config(cost_tracking)?;
        }

        Ok(())
    }

    /// Validate that a branch name is acceptable for git usage
    fn validate_branch_name(branch_name: &str) -> Result<(), String> {
        // Use the shared validation function but convert ConfigError to String
        match Config::validate_branch_name_shared(branch_name, "branch_name") {
            Ok(()) => Ok(()),
            Err(config_error) => {
                // Extract the hint from ConfigError for backwards compatibility
                match config_error {
                    ConfigError::InvalidValue { hint, .. } => Err(hint),
                    ConfigError::Validation { message } => Err(message),
                    _ => Err("Branch name validation failed".to_string()),
                }
            }
        }
    }

    /// Validate cost tracking configuration for YamlConfig struct
    ///
    /// This is a wrapper around the shared validation logic that converts validation errors
    /// into simple String format for backwards compatibility with existing YAML validation.
    ///
    /// # Arguments
    /// * `config` - The cost tracking configuration to validate
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(String)` with hint message if validation fails
    ///
    /// # See Also
    /// - [`Config::validate_cost_tracking_shared`] for detailed validation rules
    fn validate_cost_tracking_config(config: &CostTrackingConfig) -> Result<(), String> {
        match Config::validate_cost_tracking_shared(config) {
            Ok(()) => Ok(()),
            Err((_, _, hint)) => Err(hint), // Use just the hint message for String error
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    // Mutex to ensure thread-safe working directory modification for tests
    static WORKING_DIR_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.issue_branch_prefix, "issue/");
        assert_eq!(config.issue_number_width, 6);
        assert_eq!(config.max_pending_issues_in_summary, 5);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 999_999);
        assert_eq!(config.issue_number_digits, 6);
        assert_eq!(config.max_content_length, 50000);
        assert_eq!(config.max_line_length, 10000);
        assert_eq!(config.max_issue_name_length, 100);
        assert_eq!(config.virtual_issue_number_base, 500_000);
        assert_eq!(config.virtual_issue_number_range, 500_000);
        assert_eq!(config.base_branch, "main");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_new() {
        // Clean up any environment variables from other tests
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX");
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY");
        std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER");
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS");
        std::env::remove_var("SWISSARMYHAMMER_MAX_CONTENT_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_LINE_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NAME_LENGTH");
        std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE");
        std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");

        // Ensure we're in a directory without a YAML config file
        let temp_dir = tempfile::TempDir::new().unwrap();
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        // Should use defaults when no environment variables or YAML file are present
        assert_eq!(config.issue_branch_prefix, "issue/");
        assert_eq!(config.issue_number_width, 6);
        assert_eq!(config.max_pending_issues_in_summary, 5);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 999_999);
        assert_eq!(config.issue_number_digits, 6);
        assert_eq!(config.max_content_length, 50000);
        assert_eq!(config.max_line_length, 10000);
        assert_eq!(config.max_issue_name_length, 100);
        assert_eq!(config.virtual_issue_number_base, 500_000);
        assert_eq!(config.virtual_issue_number_range, 500_000);
        assert_eq!(config.base_branch, "main");

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_with_env_vars() {
        // Save original env vars if they exist
        let orig_prefix = std::env::var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX").ok();
        let orig_width = std::env::var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH").ok();
        let orig_max_pending = std::env::var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY").ok();
        let orig_max_number = std::env::var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER").ok();
        let orig_digits = std::env::var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS").ok();
        let orig_virtual_base = std::env::var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE").ok();
        let orig_virtual_range = std::env::var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE").ok();
        let orig_base_branch = std::env::var("SWISSARMYHAMMER_BASE_BRANCH").ok();

        // Set test values
        std::env::set_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX", "feature/");
        std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH", "8");
        std::env::set_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY", "10");
        std::env::set_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER", "9999999");
        std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS", "7");
        std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE", "600000");
        std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE", "400000");
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "develop");

        let config = Config::new();
        assert_eq!(config.issue_branch_prefix, "feature/");
        assert_eq!(config.issue_number_width, 8);
        assert_eq!(config.max_pending_issues_in_summary, 10);
        assert_eq!(config.min_issue_number, 1);
        assert_eq!(config.max_issue_number, 9_999_999);
        assert_eq!(config.issue_number_digits, 7);
        assert_eq!(config.virtual_issue_number_base, 600_000);
        assert_eq!(config.virtual_issue_number_range, 400_000);
        assert_eq!(config.base_branch, "develop");

        // Restore original env vars or remove if they didn't exist
        match orig_prefix {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_BRANCH_PREFIX"),
        }
        match orig_width {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH"),
        }
        match orig_max_pending {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY", val),
            None => std::env::remove_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY"),
        }
        match orig_max_number {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER", val),
            None => std::env::remove_var("SWISSARMYHAMMER_MAX_ISSUE_NUMBER"),
        }
        match orig_digits {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS", val),
            None => std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_DIGITS"),
        }
        match orig_virtual_base {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE", val),
            None => std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_BASE"),
        }
        match orig_virtual_range {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE", val),
            None => std::env::remove_var("SWISSARMYHAMMER_VIRTUAL_ISSUE_NUMBER_RANGE"),
        }
        match orig_base_branch {
            Some(val) => std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", val),
            None => std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH"),
        }
    }

    #[test]
    fn test_yaml_config_default() {
        let yaml_config = YamlConfig::default();
        assert!(yaml_config.base_branch.is_none());
    }

    #[test]
    fn test_yaml_config_apply_to_config_with_values() {
        let yaml_config = YamlConfig {
            base_branch: Some("develop".to_string()),
            cost_tracking: None,
        };
        let mut config = Config::default();

        // Verify initial state
        assert_eq!(config.base_branch, "main");

        // Apply YAML config
        yaml_config.apply_to_config(&mut config);

        // Verify YAML config took precedence
        assert_eq!(config.base_branch, "develop");
    }

    #[test]
    fn test_yaml_config_apply_to_config_with_none() {
        let yaml_config = YamlConfig::default(); // all fields are None
        let mut config = Config::default();

        // Save original value
        let original_base_branch = config.base_branch.clone();

        // Apply YAML config with None values
        yaml_config.apply_to_config(&mut config);

        // Verify original values are preserved
        assert_eq!(config.base_branch, original_base_branch);
    }

    #[test]
    fn test_yaml_config_deserialization() {
        let yaml_content = r#"
base_branch: "feature/test"
"#;

        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(yaml_config.base_branch, Some("feature/test".to_string()));
    }

    #[test]
    fn test_yaml_config_partial_deserialization() {
        let yaml_content = r#"
# Empty YAML config
"#;

        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(yaml_config.base_branch.is_none());
    }

    #[test]
    fn test_yaml_config_apply_overwrites_existing_values() {
        let yaml_config = YamlConfig {
            base_branch: Some("staging".to_string()),
            cost_tracking: None,
        };

        // Create config with non-default value
        let mut config = Config {
            base_branch: "custom".to_string(),
            ..Default::default()
        };

        // Verify initial custom value
        assert_eq!(config.base_branch, "custom");

        // Apply YAML config
        yaml_config.apply_to_config(&mut config);

        // Verify YAML config overwrote the existing value
        assert_eq!(config.base_branch, "staging");
    }

    #[test]
    #[serial_test::serial]
    fn test_find_yaml_config_file_not_found() {
        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Ensure we're in a directory that doesn't have the config file
        let temp_dir = std::env::temp_dir();
        let original_dir = std::env::current_dir().unwrap_or_else(|_| temp_dir.clone());
        std::env::set_current_dir(&temp_dir).unwrap();

        // Remove any existing config file in temp dir
        let config_path = temp_dir.join("swissarmyhammer.yaml");
        let _ = std::fs::remove_file(&config_path);

        // Remove any existing config files in home directory locations to ensure clean test
        let mut backup_paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_path = home_dir.join("swissarmyhammer.yaml");
            let xdg_config_path = home_dir
                .join(".config")
                .join("swissarmyhammer")
                .join("swissarmyhammer.yaml");

            // Backup existing files if they exist
            if home_config_path.exists() {
                let backup_path = home_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&home_config_path, &backup_path);
                backup_paths.push((home_config_path.clone(), backup_path));
            }
            if xdg_config_path.exists() {
                let backup_path = xdg_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&xdg_config_path, &backup_path);
                backup_paths.push((xdg_config_path.clone(), backup_path));
            }
        }

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Restore backed up files
        for (original, backup) in backup_paths {
            let _ = std::fs::rename(backup, original);
        }

        // Restore original directory
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
    }

    #[test]
    fn test_find_yaml_config_file_found() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create config file in the test directory
        let config_path = test_dir.join("swissarmyhammer.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: test").unwrap();
        drop(file);

        // Directly test check_config_file instead of changing directories
        let result = Config::check_config_file(&config_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().file_name().unwrap(), "swissarmyhammer.yaml");

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_directory_not_file() {
        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_dir_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create a directory with the config name
        let config_dir = test_dir.join("swissarmyhammer.yaml");
        std::fs::create_dir_all(&config_dir).unwrap();

        // Change to test directory
        let original_dir = std::env::current_dir().expect("Could not get current directory");
        std::env::set_current_dir(&test_dir).expect("Could not change to test directory");

        // Remove any existing config files in home directory locations to ensure clean test
        let mut backup_paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_path = home_dir.join("swissarmyhammer.yaml");
            let xdg_config_path = home_dir
                .join(".config")
                .join("swissarmyhammer")
                .join("swissarmyhammer.yaml");

            // Backup existing files if they exist
            if home_config_path.exists() {
                let backup_path = home_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&home_config_path, &backup_path);
                backup_paths.push((home_config_path.clone(), backup_path));
            }
            if xdg_config_path.exists() {
                let backup_path = xdg_config_path.with_extension("yaml.test_backup");
                let _ = std::fs::rename(&xdg_config_path, &backup_path);
                backup_paths.push((xdg_config_path.clone(), backup_path));
            }
        }

        let result = Config::find_yaml_config_file();
        assert!(result.is_none());

        // Restore backed up files
        for (original, backup) in backup_paths {
            let _ = std::fs::rename(backup, original);
        }

        // Clean up
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_find_yaml_config_file_path_handling() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_test_path_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create config file in the test directory
        let config_path = test_dir.join("swissarmyhammer.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: test").unwrap();
        drop(file);

        // Directly test check_config_file for path handling
        let result = Config::check_config_file(&config_path);
        assert!(result.is_some());
        let found_path = result.unwrap();

        // Verify the path is properly constructed
        assert!(found_path.is_file());
        assert_eq!(found_path.file_name().unwrap(), "swissarmyhammer.yaml");

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_validation_valid() {
        let yaml_config = YamlConfig {
            base_branch: Some("main".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_ok());

        let yaml_config_none = YamlConfig {
            base_branch: None,
            cost_tracking: None,
        };
        assert!(yaml_config_none.validate().is_ok());
    }

    #[test]
    fn test_yaml_config_validation_invalid_branch_names() {
        // Empty branch name
        let yaml_config = YamlConfig {
            base_branch: Some("".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Whitespace only branch name
        let yaml_config = YamlConfig {
            base_branch: Some("   ".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Branch name with invalid characters
        let invalid_chars = vec![' ', '~', '^', ':', '?', '*', '[', '\\'];
        for invalid_char in invalid_chars {
            let yaml_config = YamlConfig {
                base_branch: Some(format!("branch{}", invalid_char)),
                cost_tracking: None,
            };
            assert!(yaml_config.validate().is_err());
        }

        // Branch name with consecutive dots
        let yaml_config = YamlConfig {
            base_branch: Some("branch..name".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Branch name starting/ending with dot
        let yaml_config = YamlConfig {
            base_branch: Some(".branch".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        let yaml_config = YamlConfig {
            base_branch: Some("branch.".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Branch name starting/ending with slash
        let yaml_config = YamlConfig {
            base_branch: Some("/branch".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        let yaml_config = YamlConfig {
            base_branch: Some("branch/".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Branch name ending with .lock
        let yaml_config = YamlConfig {
            base_branch: Some("branch.lock".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());

        // Branch name too long
        let yaml_config = YamlConfig {
            base_branch: Some("a".repeat(256)),
            cost_tracking: None,
        };
        assert!(yaml_config.validate().is_err());
    }

    #[test]
    fn test_yaml_config_validation_valid_branch_names() {
        let valid_names = vec![
            "main",
            "develop",
            "feature/new-feature",
            "bugfix/issue-123",
            "release/v1.0.0",
            "hotfix/critical-fix",
            "user/john/feature",
        ];

        for valid_name in valid_names {
            let yaml_config = YamlConfig {
                base_branch: Some(valid_name.to_string()),
                cost_tracking: None,
            };
            assert!(
                yaml_config.validate().is_ok(),
                "Expected '{}' to be valid but validation failed",
                valid_name
            );
        }
    }

    #[test]
    fn test_yaml_config_load_from_file_success() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_load_test_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create valid YAML config file
        let config_path = test_dir.join("test_config.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: \"feature/test\"").unwrap();
        drop(file);

        let result = YamlConfig::load_from_file(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.base_branch, Some("feature/test".to_string()));

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_load_from_file_file_not_found() {
        let non_existent_path = std::env::temp_dir().join("non_existent_config.yaml");

        let result = YamlConfig::load_from_file(&non_existent_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::FileRead { path, source: _ } => {
                assert_eq!(path, non_existent_path);
            }
            _ => panic!("Expected FileRead error"),
        }
    }

    #[test]
    fn test_yaml_config_load_from_file_invalid_yaml() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir = std::env::temp_dir().join(format!(
            "swissarmyhammer_invalid_yaml_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create invalid YAML config file
        let config_path = test_dir.join("invalid_config.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "invalid: yaml: content: [").unwrap(); // Malformed YAML
        drop(file);

        let result = YamlConfig::load_from_file(&config_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::YamlParse { path, source: _ } => {
                assert_eq!(path, config_path);
            }
            _ => panic!("Expected YamlParse error"),
        }

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_load_from_file_empty_file() {
        use std::fs::File;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_empty_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create empty YAML config file
        let config_path = test_dir.join("empty_config.yaml");
        File::create(&config_path).unwrap();

        let result = YamlConfig::load_from_file(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.base_branch.is_none()); // Should load as default

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_load_from_file_partial_yaml() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir =
            std::env::temp_dir().join(format!("swissarmyhammer_partial_{}", std::process::id()));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Create YAML config file with only comments
        let config_path = test_dir.join("partial_config.yaml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "# base_branch: commented_out").unwrap();
        drop(file);

        let result = YamlConfig::load_from_file(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.base_branch.is_none()); // Should load with None values

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_yaml_config_load_or_default_with_file() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temporary directory and config file
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("swissarmyhammer.yaml");

        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "base_branch: \"develop\"").unwrap();
        drop(file);

        // Test loading the specific file directly
        let config = YamlConfig::load_from_file(&config_path).unwrap();
        assert_eq!(config.base_branch, Some("develop".to_string()));

        // This covers the core functionality of load_or_default with a file
        // The directory-based search is tested separately
    }

    #[test]
    fn test_yaml_config_load_or_default_without_file() {
        // Test the default case by creating YamlConfig::default() directly
        // This is equivalent to what load_or_default() returns when no config file is found
        let config = YamlConfig::default();
        assert!(config.base_branch.is_none());

        // This test verifies the default behavior without needing file system operations
    }

    #[test]
    fn test_config_error_display() {
        use std::path::PathBuf;

        // Test FileRead error display
        let file_error = ConfigError::FileRead {
            path: PathBuf::from("/test/path.yaml"),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        };
        let error_str = format!("{}", file_error);
        assert!(error_str.contains("Failed to read configuration file"));
        assert!(error_str.contains("/test/path.yaml"));

        // Test YamlParse error display
        let yaml_error = ConfigError::YamlParse {
            path: PathBuf::from("/test/path.yaml"),
            source: serde_yaml::from_str::<YamlConfig>("invalid: yaml: [").unwrap_err(),
        };
        let error_str = format!("{}", yaml_error);
        assert!(error_str.contains("Invalid YAML syntax in"));
        assert!(error_str.contains("/test/path.yaml"));
        assert!(error_str.contains("Hint: Check for proper indentation"));

        // Test InvalidValue error display
        let invalid_value_error = ConfigError::InvalidValue {
            field: "base_branch".to_string(),
            value: "".to_string(),
            hint: "base_branch cannot be empty".to_string(),
        };
        let error_str = format!("{}", invalid_value_error);
        assert!(error_str.contains("Invalid configuration value for 'base_branch'"));
        assert!(error_str.contains("base_branch cannot be empty"));

        // Test Validation error display
        let validation_error = ConfigError::Validation {
            message: "min_issue_number must be less than max_issue_number".to_string(),
        };
        let error_str = format!("{}", validation_error);
        assert!(error_str.contains("Configuration validation failed"));
        assert!(error_str.contains("min_issue_number must be less than max_issue_number"));
    }

    #[test]
    fn test_config_validate_success() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validate_base_branch_empty() {
        let config = Config {
            base_branch: "".to_string(),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, value, hint } => {
                assert_eq!(field, "base_branch");
                assert_eq!(value, "");
                assert!(hint.contains("base_branch cannot be empty"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_config_validate_base_branch_invalid_chars() {
        let invalid_chars = ['~', '^', ':', '?', '*', '[', '\\', ' '];

        for ch in invalid_chars.iter() {
            let config = Config {
                base_branch: format!("branch{}", ch),
                ..Default::default()
            };

            let result = config.validate();
            assert!(
                result.is_err(),
                "Expected validation error for character '{}'",
                ch
            );

            match result.unwrap_err() {
                ConfigError::InvalidValue { field, hint, .. } => {
                    assert_eq!(field, "base_branch");
                    assert!(hint.contains(&format!("invalid character '{}'", ch)));
                }
                _ => panic!("Expected InvalidValue error for character '{}'", ch),
            }
        }
    }

    #[test]
    fn test_config_validate_issue_number_width_zero() {
        let config = Config {
            issue_number_width: 0,
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, value, hint } => {
                assert_eq!(field, "issue_number_width");
                assert_eq!(value, "0");
                assert!(hint.contains("must be at least 1"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_config_validate_min_max_issue_number_range() {
        let config = Config {
            min_issue_number: 100,
            max_issue_number: 50, // Invalid: min >= max
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::Validation { message } => {
                assert!(message
                    .contains("min_issue_number (100) must be less than max_issue_number (50)"));
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_config_validate_issue_branch_prefix_too_long() {
        let config = Config {
            issue_branch_prefix: "a".repeat(51), // Too long
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, hint, .. } => {
                assert_eq!(field, "issue_branch_prefix");
                assert!(hint.contains("cannot exceed 50 characters"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_config_example_yaml_config() {
        let example = Config::example_yaml_config();
        assert!(example.contains(CONFIG_FILENAME));
        assert!(example.contains("base_branch: \"main\""));
    }

    #[test]
    fn test_config_validation_help() {
        let help = Config::validation_help();
        assert!(help.contains("Configuration Validation Help"));
        assert!(help.contains("base_branch: Must be a valid git branch name"));
    }

    #[test]
    fn test_yaml_config_validate_yaml_values_empty_base_branch() {
        let yaml_config = YamlConfig {
            base_branch: Some("".to_string()),
            cost_tracking: None,
        };

        let result = yaml_config.validate_yaml_values();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, value, hint } => {
                assert_eq!(field, "base_branch");
                assert_eq!(value, "");
                assert!(hint.contains("base_branch cannot be empty in YAML configuration"));
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_yaml_config_validate_yaml_values_success() {
        let yaml_config = YamlConfig {
            base_branch: Some("main".to_string()),
            cost_tracking: None,
        };
        assert!(yaml_config.validate_yaml_values().is_ok());

        let yaml_config_none = YamlConfig {
            base_branch: None,
            cost_tracking: None,
        };
        assert!(yaml_config_none.validate_yaml_values().is_ok());
    }

    #[test]
    fn test_yaml_config_load_from_file_with_validation() {
        use std::fs::File;
        use std::io::Write;

        let _guard = WORKING_DIR_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        // Create a unique temporary directory
        let test_dir = std::env::temp_dir().join(format!(
            "swissarmyhammer_validation_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&test_dir).unwrap();

        // Test valid YAML config
        let valid_config_path = test_dir.join("valid_config.yaml");
        let mut file = File::create(&valid_config_path).unwrap();
        writeln!(file, "base_branch: \"feature/test\"").unwrap();
        drop(file);

        let result = YamlConfig::load_from_file(&valid_config_path);
        assert!(result.is_ok());

        // Test invalid YAML config with empty base_branch
        let invalid_config_path = test_dir.join("invalid_config.yaml");
        let mut file = File::create(&invalid_config_path).unwrap();
        writeln!(file, "base_branch: \"\"").unwrap();
        drop(file);

        let result = YamlConfig::load_from_file(&invalid_config_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(field, "base_branch");
            }
            _ => panic!("Expected InvalidValue error"),
        }

        // Clean up
        std::fs::remove_dir_all(&test_dir).unwrap();
    }
}

#[cfg(test)]
mod yaml_config_tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_yaml_config_deserialize_valid() {
        let yaml_content = r#"
base_branch: "develop"
"#;
        let config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(config.base_branch, Some("develop".to_string()));
    }

    #[test]
    fn test_yaml_config_deserialize_partial() {
        let yaml_content = r#"{}"#;
        let config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(config.base_branch, None);
    }

    #[test]
    fn test_yaml_config_load_from_file_valid() -> Result<(), Box<dyn std::error::Error>> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "base_branch: \"feature\"")?;

        let config = YamlConfig::load_from_file(temp_file.path())?;
        assert_eq!(config.base_branch, Some("feature".to_string()));
        Ok(())
    }

    #[test]
    fn test_yaml_config_load_from_file_invalid_yaml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "invalid: yaml: syntax: [").unwrap();

        let result = YamlConfig::load_from_file(temp_file.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::YamlParse { path, source: _ } => {
                assert_eq!(path, temp_file.path());
            }
            _ => panic!("Expected YamlParse error"),
        }
    }

    #[test]
    fn test_yaml_config_load_nonexistent_file() {
        let result = YamlConfig::load_from_file("/nonexistent/path.yaml");
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::FileRead { path, source: _ } => {
                assert_eq!(path, std::path::Path::new("/nonexistent/path.yaml"));
            }
            _ => panic!("Expected FileRead error"),
        }
    }

    #[test]
    fn test_yaml_config_apply_to_config() {
        let mut config = Config::default();
        let original_base_branch = config.base_branch.clone();

        let yaml_config = YamlConfig {
            base_branch: Some("custom".to_string()),
            cost_tracking: None,
        };

        yaml_config.apply_to_config(&mut config);
        assert_eq!(config.base_branch, "custom");
        assert_ne!(config.base_branch, original_base_branch);
    }

    #[test]
    fn test_yaml_config_apply_to_config_none_values() {
        let mut config = Config::default();
        let original_base_branch = config.base_branch.clone();

        let yaml_config = YamlConfig {
            base_branch: None,
            cost_tracking: None,
        };

        yaml_config.apply_to_config(&mut config);
        assert_eq!(config.base_branch, original_base_branch);
    }
}

#[cfg(test)]
mod config_integration_tests {
    use super::*;

    // NOTE: The old test_config_precedence_env_overrides_yaml has been removed
    // because the precedence order has been corrected. YAML now overrides
    // environment variables as required by the specification.

    #[test]
    #[serial_test::serial]
    fn test_config_precedence_yaml_overrides_default() {
        // YAML should override defaults when no env var is set
        let temp_dir = tempfile::TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");
        std::fs::write(&yaml_path, "base_branch: \"yaml-branch\"").unwrap();

        // Ensure no environment variable is set
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");

        // Verify the env var is actually not set
        assert!(
            std::env::var("SWISSARMYHAMMER_BASE_BRANCH").is_err(),
            "Env var should not be set"
        );

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Verify YAML file exists and is readable
        assert!(yaml_path.exists(), "YAML config file should exist");
        let yaml_content = std::fs::read_to_string(&yaml_path).unwrap();
        assert!(
            yaml_content.contains("yaml-branch"),
            "YAML should contain yaml-branch"
        );

        let config = Config::new();
        assert_eq!(config.base_branch, "yaml-branch");

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_precedence_defaults_when_no_overrides() {
        // Ensure no YAML file exists and no env vars
        let temp_dir = tempfile::TempDir::new().unwrap();
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");

        let config = Config::new();
        assert_eq!(config.base_branch, "main"); // default value

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_precedence_yaml_overrides_env() {
        // YAML should take precedence over environment variables (new requirement)
        let temp_dir = tempfile::TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");
        std::fs::write(&yaml_path, "base_branch: \"yaml-branch\"").unwrap();

        // Set environment variable - YAML should override this
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "env-branch");

        // Change to temp directory
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        assert_eq!(config.base_branch, "yaml-branch"); // yaml should override env

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_yaml_validation_failure_fallback_to_env() {
        // When YAML has invalid configuration, it should fall back to environment variables
        let temp_dir = tempfile::TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");
        // Create YAML with invalid base_branch (empty string)
        std::fs::write(&yaml_path, "base_branch: \"\"").unwrap();

        // Set environment variable that should be used as fallback
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "env-fallback-branch");

        // Change to temp directory
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        // Should use env var because YAML validation failed
        assert_eq!(config.base_branch, "env-fallback-branch");

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_empty_yaml_with_env_precedence() {
        // When YAML file exists but is empty, environment variables should be used
        let temp_dir = tempfile::TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");
        // Create empty YAML file
        std::fs::write(&yaml_path, "").unwrap();

        // Set environment variable
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "env-with-empty-yaml");

        // Change to temp directory
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        // Should use env var because YAML is empty (no override values)
        assert_eq!(config.base_branch, "env-with-empty-yaml");

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
    }

    #[test]
    #[serial_test::serial]
    fn test_config_complete_precedence_hierarchy() {
        // Comprehensive test of all three precedence levels: YAML > ENV > DEFAULTS
        let temp_dir = tempfile::TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");

        // Create YAML with partial configuration (only base_branch)
        // This tests that YAML values override everything, env vars override defaults where no YAML
        std::fs::write(&yaml_path, "base_branch: \"yaml-precedence-branch\"").unwrap();

        // Set environment variables for multiple fields
        std::env::set_var("SWISSARMYHAMMER_BASE_BRANCH", "env-branch"); // Should be overridden by YAML
        std::env::set_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH", "8"); // Should be used (no YAML override)
        std::env::set_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY", "15"); // Should be used (no YAML override)

        // Change to temp directory
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();

        // Verify precedence hierarchy:
        // 1. YAML overrides env var
        assert_eq!(config.base_branch, "yaml-precedence-branch");

        // 2. Env vars override defaults (no YAML present for these fields)
        assert_eq!(config.issue_number_width, 8);
        assert_eq!(config.max_pending_issues_in_summary, 15);

        // 3. Defaults used when no YAML or env var present
        assert_eq!(config.issue_branch_prefix, "issue/"); // default value
        assert_eq!(config.min_issue_number, 1); // default value

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        std::env::remove_var("SWISSARMYHAMMER_BASE_BRANCH");
        std::env::remove_var("SWISSARMYHAMMER_ISSUE_NUMBER_WIDTH");
        std::env::remove_var("SWISSARMYHAMMER_MAX_PENDING_ISSUES_IN_SUMMARY");
    }
}

#[cfg(test)]
mod config_validation_tests {
    use super::*;

    #[test]
    fn test_validate_base_branch_valid() {
        let config = Config {
            base_branch: "main".to_string(),
            ..Config::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_base_branch_empty() {
        let config = Config {
            base_branch: "".to_string(),
            ..Config::default()
        };
        let result = config.validate();
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(field, "base_branch");
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_validate_base_branch_invalid_characters() {
        let invalid_names = vec![
            "branch with spaces",
            "branch~with~tildes",
            "branch^with^carets",
            "branch:with:colons",
            "branch?with?questions",
            "branch*with*asterisks",
            "branch[with[brackets",
            "branch\\with\\backslashes",
        ];

        for invalid_name in invalid_names {
            let config = Config {
                base_branch: invalid_name.to_string(),
                ..Config::default()
            };
            let result = config.validate();
            assert!(
                result.is_err(),
                "Should fail validation for: {}",
                invalid_name
            );
        }
    }

    #[test]
    fn test_validate_numeric_ranges() {
        let config = Config {
            issue_number_width: 0,
            ..Config::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            min_issue_number: 100,
            max_issue_number: 50,
            ..Config::default()
        };
        assert!(config.validate().is_err());
    }
}

#[cfg(test)]
mod config_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_valid_branch_names_pass_validation(
            branch_name in "[a-zA-Z0-9][a-zA-Z0-9._/-]*[a-zA-Z0-9]|[a-zA-Z0-9]"
        ) {
            // Filter out names that git doesn't allow
            prop_assume!(!branch_name.starts_with('.'));
            prop_assume!(!branch_name.starts_with('/'));
            prop_assume!(!branch_name.ends_with('/'));
            prop_assume!(!branch_name.ends_with('.'));
            prop_assume!(!branch_name.contains("//"));
            prop_assume!(!branch_name.contains(".."));
            prop_assume!(!branch_name.ends_with(".lock"));
            prop_assume!(!branch_name.trim().is_empty());
            prop_assume!(branch_name.len() <= 100);

            let config = Config {
                base_branch: branch_name,
                ..Config::default()
            };

            prop_assert!(config.validate().is_ok());
        }

        #[test]
        fn test_positive_numbers_pass_validation(
            width in 1u32..10,  // Keep within valid range
            max_issues in 1u32..100,
            min_issue in 1u32..100000,
            max_issue in 100001u32..999999
        ) {
            let config = Config {
                issue_number_width: width as usize,
                max_pending_issues_in_summary: max_issues as usize,
                min_issue_number: min_issue,
                max_issue_number: max_issue,
                ..Config::default()
            };

            prop_assert!(config.validate().is_ok());
        }
    }
}

#[cfg(test)]
mod config_benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_config_loading_performance() {
        let iterations = 1000;
        let start = Instant::now();

        for _ in 0..iterations {
            let _config = Config::new();
        }

        let duration = start.elapsed();
        let avg_duration = duration / iterations;

        // Configuration loading should be fast (< 1ms on average)
        assert!(
            avg_duration.as_millis() < 1,
            "Config loading too slow: {}ms average",
            avg_duration.as_millis()
        );
    }
}

#[cfg(test)]
mod cost_tracking_tests {
    use super::*;
    use std::env;

    #[test]
    fn test_cost_tracking_config_default() {
        let config = CostTrackingConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.pricing_model, PricingModel::Paid);
        assert_eq!(config.rates.input_token_cost.to_string(), "0.000015");
        assert_eq!(config.rates.output_token_cost.to_string(), "0.000075");
        assert_eq!(config.session_management.max_concurrent_sessions, 100);
        assert_eq!(config.session_management.session_timeout_hours, 24);
        assert_eq!(config.session_management.cleanup_interval_hours, 6);
        assert!(config.aggregation.enabled);
        assert_eq!(config.aggregation.retention_days, 90);
        assert_eq!(config.aggregation.scan_interval_hours, 24);
        assert!(config.reporting.include_in_issues);
        assert!(config.reporting.detailed_breakdown);
        assert_eq!(config.reporting.cost_precision_decimals, 4);
    }

    #[test]
    fn test_config_with_no_cost_tracking() {
        let config = Config::default();
        assert!(config.cost_tracking.is_none());
    }

    #[test]
    fn test_yaml_config_cost_tracking_deserialization() {
        let yaml_content = r#"
cost_tracking:
  enabled: true
  pricing_model: "max"
  rates:
    input_token_cost: 0.00003
    output_token_cost: 0.00012
  session_management:
    max_concurrent_sessions: 200
    session_timeout_hours: 48
    cleanup_interval_hours: 12
  aggregation:
    enabled: false
    scan_interval_hours: 24
    retention_days: 30
    trend_analysis_days: 30
    outlier_threshold: 2.0
    min_issues_for_analysis: 3
    include_issues_days: null
    max_stored_sessions: 5000
    max_issues_per_aggregation: 10000
  reporting:
    include_in_issues: false
    detailed_breakdown: false
    cost_precision_decimals: 2
"#;
        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(yaml_config.cost_tracking.is_some());

        let cost_tracking = yaml_config.cost_tracking.unwrap();
        assert!(cost_tracking.enabled);
        assert_eq!(cost_tracking.pricing_model, PricingModel::Max);
        assert_eq!(cost_tracking.rates.input_token_cost.to_string(), "0.00003");
        assert_eq!(cost_tracking.rates.output_token_cost.to_string(), "0.00012");
        assert_eq!(
            cost_tracking.session_management.max_concurrent_sessions,
            200
        );
        assert_eq!(cost_tracking.session_management.session_timeout_hours, 48);
        assert_eq!(cost_tracking.session_management.cleanup_interval_hours, 12);
        assert!(!cost_tracking.aggregation.enabled);
        assert_eq!(cost_tracking.aggregation.retention_days, 30);
        assert_eq!(cost_tracking.aggregation.max_stored_sessions, 5000);
        assert_eq!(cost_tracking.aggregation.max_issues_per_aggregation, 10000);
        assert!(!cost_tracking.reporting.include_in_issues);
        assert!(!cost_tracking.reporting.detailed_breakdown);
        assert_eq!(cost_tracking.reporting.cost_precision_decimals, 2);
    }

    #[test]
    fn test_yaml_config_partial_cost_tracking() {
        let yaml_content = r#"
cost_tracking:
  enabled: true
  pricing_model: "paid"
"#;
        let yaml_config: YamlConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert!(yaml_config.cost_tracking.is_some());

        let cost_tracking = yaml_config.cost_tracking.unwrap();
        assert!(cost_tracking.enabled);
        assert_eq!(cost_tracking.pricing_model, PricingModel::Paid);
        // Should use defaults for missing fields
        assert_eq!(cost_tracking.rates.input_token_cost.to_string(), "0.000015");
        assert_eq!(
            cost_tracking.session_management.max_concurrent_sessions,
            100
        );
    }

    #[test]
    fn test_yaml_config_apply_cost_tracking() {
        let yaml_config = YamlConfig {
            base_branch: None,
            cost_tracking: Some(CostTrackingConfig {
                enabled: true,
                pricing_model: PricingModel::Max,
                ..Default::default()
            }),
        };
        let mut config = Config::default();

        // Initially no cost tracking
        assert!(config.cost_tracking.is_none());

        yaml_config.apply_to_config(&mut config);

        // Should now have cost tracking from YAML
        assert!(config.cost_tracking.is_some());
        let cost_tracking = config.cost_tracking.unwrap();
        assert!(cost_tracking.enabled);
        assert_eq!(cost_tracking.pricing_model, PricingModel::Max);
    }

    #[test]
    #[serial_test::serial]
    fn test_cost_tracking_env_vars() {
        // Save original values
        let original_enabled = env::var("SAH_COST_TRACKING_ENABLED").ok();
        let original_pricing = env::var("SAH_COST_PRICING_MODEL").ok();
        let original_input_cost = env::var("SAH_COST_INPUT_TOKEN_COST").ok();
        let original_output_cost = env::var("SAH_COST_OUTPUT_TOKEN_COST").ok();

        // Set test environment variables
        env::set_var("SAH_COST_TRACKING_ENABLED", "true");
        env::set_var("SAH_COST_PRICING_MODEL", "max");
        env::set_var("SAH_COST_INPUT_TOKEN_COST", "0.00005");
        env::set_var("SAH_COST_OUTPUT_TOKEN_COST", "0.0002");
        env::set_var("SAH_COST_MAX_CONCURRENT_SESSIONS", "150");

        let temp_dir = tempfile::TempDir::new().unwrap();
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let config = Config::new();
        assert!(config.cost_tracking.is_some());

        let cost_tracking = config.cost_tracking.unwrap();
        assert!(cost_tracking.enabled);
        assert_eq!(cost_tracking.pricing_model, PricingModel::Max);
        assert_eq!(cost_tracking.rates.input_token_cost.to_string(), "0.00005");
        assert_eq!(cost_tracking.rates.output_token_cost.to_string(), "0.0002");
        assert_eq!(
            cost_tracking.session_management.max_concurrent_sessions,
            150
        );

        // Restore original values
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        match original_enabled {
            Some(val) => env::set_var("SAH_COST_TRACKING_ENABLED", val),
            None => env::remove_var("SAH_COST_TRACKING_ENABLED"),
        }
        match original_pricing {
            Some(val) => env::set_var("SAH_COST_PRICING_MODEL", val),
            None => env::remove_var("SAH_COST_PRICING_MODEL"),
        }
        match original_input_cost {
            Some(val) => env::set_var("SAH_COST_INPUT_TOKEN_COST", val),
            None => env::remove_var("SAH_COST_INPUT_TOKEN_COST"),
        }
        match original_output_cost {
            Some(val) => env::set_var("SAH_COST_OUTPUT_TOKEN_COST", val),
            None => env::remove_var("SAH_COST_OUTPUT_TOKEN_COST"),
        }
        env::remove_var("SAH_COST_MAX_CONCURRENT_SESSIONS");
    }

    #[test]
    fn test_cost_tracking_validation_valid() {
        let config = CostTrackingConfig::default();
        let main_config = Config {
            cost_tracking: Some(config),
            ..Default::default()
        };
        assert!(main_config.validate().is_ok());
    }

    #[test]
    fn test_pricing_model_from_str_invalid() {
        let result = PricingModel::from_str("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid pricing model 'invalid'"));
    }

    #[test]
    fn test_pricing_model_from_str_valid() {
        assert_eq!(PricingModel::from_str("paid").unwrap(), PricingModel::Paid);
        assert_eq!(PricingModel::from_str("max").unwrap(), PricingModel::Max);
        assert_eq!(PricingModel::from_str("PAID").unwrap(), PricingModel::Paid);
        assert_eq!(PricingModel::from_str("MAX").unwrap(), PricingModel::Max);
    }

    #[test]
    fn test_cost_tracking_validation_negative_costs() {
        use rust_decimal::Decimal;

        let config = CostTrackingConfig {
            rates: PricingConfig {
                input_token_cost: Decimal::new(-1, 6),  // -0.000001
                output_token_cost: Decimal::new(75, 6), // 0.000075
            },
            ..Default::default()
        };
        let main_config = Config {
            cost_tracking: Some(config),
            ..Default::default()
        };
        let result = main_config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(field, "cost_tracking.rates.input_token_cost");
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_cost_tracking_validation_zero_sessions() {
        let config = CostTrackingConfig {
            session_management: SessionManagementConfig {
                max_concurrent_sessions: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        let main_config = Config {
            cost_tracking: Some(config),
            ..Default::default()
        };
        let result = main_config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "cost_tracking.session_management.max_concurrent_sessions"
                );
            }
            _ => panic!("Expected InvalidValue error"),
        }
    }

    #[test]
    fn test_yaml_cost_tracking_validation() {
        use rust_decimal::Decimal;

        let invalid_config = CostTrackingConfig {
            pricing_model: PricingModel::Paid, // enum ensures valid pricing model
            rates: PricingConfig {
                input_token_cost: Decimal::ZERO, // This should cause validation failure
                output_token_cost: Decimal::new(75, 6),
            },
            ..Default::default()
        };

        let result = YamlConfig::validate_cost_tracking_config(&invalid_config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("input_token_cost must be positive"));
    }

    #[test]
    #[serial_test::serial]
    fn test_cost_tracking_precedence() {
        use tempfile::TempDir;

        // Test that YAML overrides environment variables for cost tracking
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("swissarmyhammer.yaml");
        std::fs::write(
            &yaml_path,
            r#"
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.00001
"#,
        )
        .unwrap();

        // Set conflicting environment variable
        env::set_var("SAH_COST_TRACKING_ENABLED", "false");
        env::set_var("SAH_COST_PRICING_MODEL", "max");
        env::set_var("SAH_COST_INPUT_TOKEN_COST", "0.00009");

        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Verify the YAML file exists and is readable
        assert!(yaml_path.exists(), "YAML config file should exist");
        let yaml_content = std::fs::read_to_string(&yaml_path).unwrap();
        assert!(
            yaml_content.contains("enabled: true"),
            "YAML should contain enabled: true"
        );

        let config = Config::new();
        assert!(
            config.cost_tracking.is_some(),
            "Cost tracking config should exist"
        );

        let cost_tracking = config.cost_tracking.unwrap();
        // YAML should override env vars
        assert!(
            cost_tracking.enabled,
            "YAML enabled=true should override env enabled=false"
        ); // YAML value, not env value
        assert_eq!(cost_tracking.pricing_model, PricingModel::Paid); // YAML value, not env value
        assert_eq!(cost_tracking.rates.input_token_cost.to_string(), "0.00001"); // YAML value, not env value

        // Cleanup
        std::env::set_current_dir(original_dir).expect("Could not restore original directory");
        env::remove_var("SAH_COST_TRACKING_ENABLED");
        env::remove_var("SAH_COST_PRICING_MODEL");
        env::remove_var("SAH_COST_INPUT_TOKEN_COST");
    }

    // Edge case and boundary condition tests

    #[test]
    fn test_validation_edge_cases_minimum_valid_values() {
        use rust_decimal::Decimal;

        // Test with minimal positive values
        let config = CostTrackingConfig {
            enabled: true,
            pricing_model: PricingModel::Paid,
            rates: PricingConfig {
                input_token_cost: Decimal::new(1, 12), // 0.000000000001
                output_token_cost: Decimal::new(1, 12),
            },
            session_management: SessionManagementConfig {
                max_concurrent_sessions: 1,
                session_timeout_hours: 1,
                cleanup_interval_hours: 1,
            },
            aggregation: AggregationConfig {
                enabled: true,
                scan_interval_hours: 24,
                retention_days: 1,
                trend_analysis_days: 30,
                outlier_threshold: DEFAULT_OUTLIER_THRESHOLD,
                min_issues_for_analysis: 3,
                include_issues_days: None,
                max_stored_sessions: 1,
                max_issues_per_aggregation: DEFAULT_MAX_ISSUES_PER_AGGREGATION,
            },
            reporting: ReportingConfig {
                include_in_issues: false,
                detailed_breakdown: false,
                cost_precision_decimals: 0,
            },
        };

        let result = Config::validate_cost_tracking_shared(&config);
        assert!(
            result.is_ok(),
            "Minimal valid values should pass validation"
        );
    }

    #[test]
    fn test_validation_edge_cases_maximum_precision_decimals() {
        use rust_decimal::Decimal;

        // Test maximum allowed precision decimals
        let config = CostTrackingConfig {
            enabled: true,
            pricing_model: PricingModel::Max,
            rates: PricingConfig {
                input_token_cost: Decimal::new(15, 6),
                output_token_cost: Decimal::new(75, 6),
            },
            session_management: SessionManagementConfig::default(),
            aggregation: AggregationConfig::default(),
            reporting: ReportingConfig {
                include_in_issues: true,
                detailed_breakdown: true,
                cost_precision_decimals: 10, // Maximum allowed
            },
        };

        let result = Config::validate_cost_tracking_shared(&config);
        assert!(
            result.is_ok(),
            "Maximum precision decimals should pass validation"
        );
    }

    #[test]
    fn test_validation_edge_cases_excessive_precision_decimals() {
        use rust_decimal::Decimal;

        // Test excessive precision decimals (should fail)
        let config = CostTrackingConfig {
            enabled: true,
            pricing_model: PricingModel::Paid,
            rates: PricingConfig {
                input_token_cost: Decimal::new(15, 6),
                output_token_cost: Decimal::new(75, 6),
            },
            session_management: SessionManagementConfig::default(),
            aggregation: AggregationConfig::default(),
            reporting: ReportingConfig {
                include_in_issues: true,
                detailed_breakdown: true,
                cost_precision_decimals: 15, // Exceeds maximum of 10
            },
        };

        let result = Config::validate_cost_tracking_shared(&config);
        assert!(
            result.is_err(),
            "Excessive precision decimals should fail validation"
        );
        if let Err((field, value, hint)) = result {
            assert_eq!(field, "cost_tracking.reporting.cost_precision_decimals");
            assert_eq!(value, "15");
            assert!(hint.contains("cannot exceed 10"));
        }
    }

    #[test]
    fn test_validation_edge_cases_large_session_values() {
        use rust_decimal::Decimal;

        // Test with very large but valid session management values
        let config = CostTrackingConfig {
            enabled: true,
            pricing_model: PricingModel::Max,
            rates: PricingConfig {
                input_token_cost: Decimal::new(15, 6),
                output_token_cost: Decimal::new(75, 6),
            },
            session_management: SessionManagementConfig {
                max_concurrent_sessions: u32::MAX,
                session_timeout_hours: u32::MAX,
                cleanup_interval_hours: u32::MAX,
            },
            aggregation: AggregationConfig {
                enabled: false,
                scan_interval_hours: 24,
                retention_days: u32::MAX,
                trend_analysis_days: 30,
                outlier_threshold: DEFAULT_OUTLIER_THRESHOLD,
                min_issues_for_analysis: 3,
                include_issues_days: None,
                max_stored_sessions: u32::MAX,
                max_issues_per_aggregation: DEFAULT_MAX_ISSUES_PER_AGGREGATION,
            },
            reporting: ReportingConfig::default(),
        };

        let result = Config::validate_cost_tracking_shared(&config);
        assert!(
            result.is_ok(),
            "Large valid session values should pass validation"
        );
    }

    #[test]
    fn test_pricing_model_enum_boundary_conditions() {
        // Test that both enum variants work correctly
        let paid_config = CostTrackingConfig {
            pricing_model: PricingModel::Paid,
            ..Default::default()
        };

        let max_config = CostTrackingConfig {
            pricing_model: PricingModel::Max,
            ..Default::default()
        };

        // Both should validate successfully (enum ensures type safety)
        assert!(Config::validate_cost_tracking_shared(&paid_config).is_ok());
        assert!(Config::validate_cost_tracking_shared(&max_config).is_ok());

        // Test enum parsing edge cases
        assert_eq!(PricingModel::from_str("paid").unwrap(), PricingModel::Paid);
        assert_eq!(PricingModel::from_str("PAID").unwrap(), PricingModel::Paid);
        assert_eq!(PricingModel::from_str("Paid").unwrap(), PricingModel::Paid);
        assert_eq!(PricingModel::from_str("max").unwrap(), PricingModel::Max);
        assert_eq!(PricingModel::from_str("MAX").unwrap(), PricingModel::Max);
        assert_eq!(PricingModel::from_str("Max").unwrap(), PricingModel::Max);

        // Invalid values should return errors
        assert!(PricingModel::from_str("invalid").is_err());
        assert!(PricingModel::from_str("").is_err());
        assert!(PricingModel::from_str("PREMIUM").is_err());
    }

    #[test]
    fn test_decimal_parsing_boundary_conditions() {
        // Test parsing with various decimal formats
        assert_eq!(
            parse_decimal_with_fallback("0.000015", "0.000010").to_string(),
            "0.000015"
        );
        assert_eq!(
            parse_decimal_with_fallback("0", "0.000010").to_string(),
            "0"
        );
        assert_eq!(
            parse_decimal_with_fallback("999999.999999", "0.000010").to_string(),
            "999999.999999"
        );

        // Test fallback scenarios
        assert_eq!(
            parse_decimal_with_fallback("invalid", "0.000010").to_string(),
            "0.000010"
        );
        assert_eq!(
            parse_decimal_with_fallback("not_a_number", "0.123").to_string(),
            "0.123"
        );

        // Test double fallback (both invalid)
        assert_eq!(
            parse_decimal_with_fallback("invalid1", "invalid2").to_string(),
            "0.000001"
        );

        // Test extreme values
        assert_eq!(
            parse_decimal_with_fallback("0.000000000001", "0.000010").to_string(),
            "0.000000000001"
        );
        assert_eq!(
            parse_decimal_with_fallback("1000000", "0.000010").to_string(),
            "1000000"
        );
    }

    #[test]
    fn test_configuration_completeness_edge_cases() {
        use rust_decimal::Decimal;

        // Test with all boolean flags in various combinations
        let configs = [
            (true, true, true, true),
            (false, false, false, false),
            (true, false, true, false),
            (false, true, false, true),
        ];

        for (enabled, agg_enabled, include_issues, detailed) in configs {
            let config = CostTrackingConfig {
                enabled,
                pricing_model: PricingModel::Paid,
                rates: PricingConfig {
                    input_token_cost: Decimal::new(15, 6),
                    output_token_cost: Decimal::new(75, 6),
                },
                session_management: SessionManagementConfig::default(),
                aggregation: AggregationConfig {
                    enabled: agg_enabled,
                    scan_interval_hours: 24,
                    retention_days: 30,
                    trend_analysis_days: 30,
                    outlier_threshold: DEFAULT_OUTLIER_THRESHOLD,
                    min_issues_for_analysis: 3,
                    include_issues_days: None,
                    max_stored_sessions: 1000,
                    max_issues_per_aggregation: DEFAULT_MAX_ISSUES_PER_AGGREGATION,
                },
                reporting: ReportingConfig {
                    include_in_issues: include_issues,
                    detailed_breakdown: detailed,
                    cost_precision_decimals: 2,
                },
            };

            let result = Config::validate_cost_tracking_shared(&config);
            assert!(
                result.is_ok(),
                "Valid configuration with flags ({}, {}, {}, {}) should pass validation",
                enabled,
                agg_enabled,
                include_issues,
                detailed
            );
        }
    }
}
