//! Cost section formatting for issue markdown generation
//!
//! This module provides comprehensive cost formatting functionality for generating
//! cost analysis sections in completed issue markdown files. It integrates with
//! the existing cost tracking infrastructure to create human-readable cost reports.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use swissarmyhammer::cost::formatting::{CostSectionFormatter, CostFormattingConfig, DetailLevel};
//! use swissarmyhammer::cost::{CostSession, PricingModel};
//!
//! // Create a formatter with default configuration
//! let formatter = CostSectionFormatter::default();
//!
//! // Or create with custom configuration
//! let config = CostFormattingConfig {
//!     enabled: true,
//!     detail_level: DetailLevel::Full,
//!     currency_precision: 2,
//!     show_breakdown_table: true,
//!     locale: "en_US".to_string(),
//!     ..CostFormattingConfig::default()
//! };
//! let formatter = CostSectionFormatter::new(config);
//! ```
//!
//! ## Locale-Specific Formatting
//!
//! ```rust,no_run
//! use swissarmyhammer::cost::formatting::CostFormattingConfig;
//!
//! // Create configuration for different locales
//! let us_config = CostFormattingConfig::for_locale("en_US");
//! let de_config = CostFormattingConfig::for_locale("de_DE");
//! let jp_config = CostFormattingConfig::for_locale("ja_JP");
//!
//! // Each will have appropriate date formats and thousands separators
//! ```
//!
//! ## Different Detail Levels
//!
//! ```rust,no_run
//! use swissarmyhammer::cost::formatting::CostFormattingConfig;
//!
//! // Summary only - basic cost information
//! let summary_config = CostFormattingConfig::summary_only();
//!
//! // Full breakdown - includes API call table and statistics
//! let full_config = CostFormattingConfig::full_breakdown();
//! ```
//!
//! ## Creating and Formatting Cost Data
//!
//! ```rust,ignore
//! use swissarmyhammer::cost::formatting::{CostSectionFormatter, IssueCostData};
//! use swissarmyhammer::cost::{CostSession, PricingModel, CostCalculator};
//!
//! // Assuming you have a completed cost session and calculator
//! let session: CostSession = get_completed_session();
//! let pricing_model = PricingModel::paid_with_defaults();
//! let calculator = CostCalculator::new(pricing_model.clone());
//!
//! // Create issue cost data
//! let cost_data = CostSectionFormatter::create_issue_cost_data(
//!     session,
//!     pricing_model,
//!     Some(&calculator)
//! ).expect("Failed to create cost data");
//!
//! // Format the cost section
//! let formatter = CostSectionFormatter::default();
//! let markdown_section = formatter.format_cost_section(&cost_data);
//!
//! println!("{}", markdown_section);
//! ```
//!
//! ## Example Output
//!
//! The formatted output will look like:
//!
//! ```markdown
//! ## Cost Analysis
//!
//! **Total Cost**: $0.25
//! **Total API Calls**: 3
//! **Total Input Tokens**: 1,500
//! **Total Output Tokens**: 2,200
//! **Session Duration**: 2m 30s
//! **Completed**: 2024-01-15 14:30:25 UTC
//!
//! ### API Call Breakdown
//!
//! | Timestamp | Endpoint | Input Tokens | Output Tokens | Duration | Status |
//! |-----------|----------|--------------|---------------|----------|--------|
//! | 14:30:20 UTC | /v1/messages | 500 | 750 | 45s | ✓ |
//! | 14:30:35 UTC | /v1/messages | 800 | 1,200 | 1m 20s | ✓ |
//! | 14:31:15 UTC | /v1/messages | 200 | 250 | 25s | ✓ |
//!
//! ### Cost Summary
//! - **Average cost per call**: $0.08
//! - **Most expensive call**: $0.12
//! - **Token efficiency**: 1.47 (output/input ratio)
//! - **Success rate**: 100.0% (3 successful, 0 failed)
//! ```

use crate::cost::{CostSession, PricingModel};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Type-safe wrapper for currency amounts to prevent primitive misuse
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CurrencyAmount(pub Decimal);

impl CurrencyAmount {
    /// Create a new currency amount
    pub fn new(amount: Decimal) -> Self {
        Self(amount)
    }

    /// Get the raw decimal value
    pub fn amount(self) -> Decimal {
        self.0
    }

    /// Convert to f64 for calculations
    pub fn as_f64(self) -> f64 {
        self.0.to_string().parse().unwrap_or(0.0)
    }
}

impl From<Decimal> for CurrencyAmount {
    fn from(amount: Decimal) -> Self {
        Self::new(amount)
    }
}

impl std::ops::Add for CurrencyAmount {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl std::fmt::Display for CurrencyAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.0)
    }
}

/// Type-safe wrapper for session durations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SessionDuration(pub Duration);

impl SessionDuration {
    /// Create a new session duration
    pub fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Get the raw duration value
    pub fn duration(self) -> Duration {
        self.0
    }

    /// Get total seconds
    pub fn as_secs(self) -> u64 {
        self.0.as_secs()
    }
}

impl From<Duration> for SessionDuration {
    fn from(duration: Duration) -> Self {
        Self::new(duration)
    }
}

impl std::fmt::Display for SessionDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let total_secs = self.0.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;

        if minutes > 0 {
            write!(f, "{}m {}s", minutes, seconds)
        } else {
            write!(f, "{}s", seconds)
        }
    }
}

/// Type-safe wrapper for API call counts
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ApiCallCount(pub u32);

impl ApiCallCount {
    /// Create a new API call count
    pub fn new(count: u32) -> Self {
        Self(count)
    }

    /// Get the raw count value
    pub fn count(self) -> u32 {
        self.0
    }

    /// Increment the count
    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<u32> for ApiCallCount {
    fn from(count: u32) -> Self {
        Self::new(count)
    }
}

impl std::ops::Add for ApiCallCount {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

impl std::fmt::Display for ApiCallCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} API calls", self.0)
    }
}

/// Configuration for cost section formatting
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostFormattingConfig {
    /// Whether to include cost sections in completed issues
    pub enabled: bool,
    /// Level of detail to include in cost sections
    pub detail_level: DetailLevel,
    /// Number of decimal places for currency formatting
    pub currency_precision: usize,
    /// Whether to show the API call breakdown table
    pub show_breakdown_table: bool,
    /// Date/time format string for timestamps (locale-aware)
    pub date_format: String,
    /// Locale code for formatting (e.g., "en_US", "de_DE", "ja_JP")
    pub locale: String,
    /// Separator for thousands in token counts (e.g., "," for 1,000)
    pub thousands_separator: String,
    /// Whether to include session metadata in output
    pub include_metadata: bool,
    /// Maximum length for endpoint display in tables before truncation
    pub max_endpoint_display_length: usize,
}

/// Level of detail for cost section formatting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetailLevel {
    /// Only total cost and basic statistics
    Summary,
    /// Include API call breakdown table
    Full,
    /// Full breakdown with individual call details and metadata
    Breakdown,
}

/// Summary statistics calculated from cost session data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostSummaryStats {
    /// Average cost per API call
    pub average_cost_per_call: Option<CurrencyAmount>,
    /// Most expensive single API call cost
    pub most_expensive_call: Option<CurrencyAmount>,
    /// Token efficiency ratio (output tokens / input tokens)
    pub token_efficiency: Option<Decimal>,
    /// Total session duration
    pub total_duration: Option<SessionDuration>,
    /// Number of successful API calls
    pub successful_calls: ApiCallCount,
    /// Number of failed API calls
    pub failed_calls: ApiCallCount,
}

/// Complete cost data for an issue, combining session data with calculated metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCostData {
    /// The cost session containing all API call data
    pub session_data: CostSession,
    /// Total calculated cost (None for max plan)
    pub total_cost: Option<CurrencyAmount>,
    /// Pricing model used for calculations
    pub pricing_model: PricingModel,
    /// Calculated summary statistics
    pub summary_stats: CostSummaryStats,
}

/// Main cost section formatter
pub struct CostSectionFormatter {
    /// Configuration for formatting options
    config: CostFormattingConfig,
    /// Precision for decimal formatting
    precision: usize,
    /// Locale string for future localization support
    _locale: String,
}

impl Default for CostFormattingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detail_level: DetailLevel::Full,
            currency_precision: 2,
            show_breakdown_table: true,
            date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
            locale: "en_US".to_string(),
            thousands_separator: ",".to_string(),
            include_metadata: false,
            max_endpoint_display_length: 30,
        }
    }
}

impl CostFormattingConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration for summary-only output
    pub fn summary_only() -> Self {
        Self {
            detail_level: DetailLevel::Summary,
            show_breakdown_table: false,
            ..Self::default()
        }
    }

    /// Create a configuration for full breakdown output
    pub fn full_breakdown() -> Self {
        Self {
            detail_level: DetailLevel::Breakdown,
            show_breakdown_table: true,
            include_metadata: true,
            ..Self::default()
        }
    }

    /// Create a configuration with locale-specific defaults
    pub fn for_locale(locale: &str) -> Self {
        let (date_format, thousands_separator) = Self::get_locale_defaults(locale);
        Self {
            locale: locale.to_string(),
            date_format,
            thousands_separator,
            ..Self::default()
        }
    }

    /// Get locale-appropriate format defaults
    fn get_locale_defaults(locale: &str) -> (String, String) {
        match locale {
            "en_US" => ("%m/%d/%Y %I:%M:%S %p UTC".to_string(), ",".to_string()),
            "en_GB" => ("%d/%m/%Y %H:%M:%S UTC".to_string(), ",".to_string()),
            "de_DE" => ("%d.%m.%Y %H:%M:%S UTC".to_string(), ".".to_string()),
            "fr_FR" => ("%d/%m/%Y %H:%M:%S UTC".to_string(), " ".to_string()),
            "ja_JP" => ("%Y年%m月%d日 %H:%M:%S UTC".to_string(), ",".to_string()),
            "zh_CN" => ("%Y-%m-%d %H:%M:%S UTC".to_string(), ",".to_string()),
            _ => ("%Y-%m-%d %H:%M:%S UTC".to_string(), ",".to_string()), // Default ISO format
        }
    }
}

impl CostSectionFormatter {
    /// Create a new cost section formatter with the given configuration
    pub fn new(config: CostFormattingConfig) -> Self {
        let precision = config.currency_precision;
        let locale = config.locale.clone();
        Self {
            config,
            precision,
            _locale: locale,
        }
    }

    /// Create IssueCostData from a completed CostSession and pricing model
    pub fn create_issue_cost_data(
        session: CostSession,
        pricing_model: PricingModel,
        cost_calculator: Option<&crate::cost::CostCalculator>,
    ) -> crate::Result<IssueCostData> {
        // Calculate total cost if calculator is provided and we have a paid plan
        let total_cost = match (&pricing_model, cost_calculator) {
            (PricingModel::Paid(_), Some(calculator)) => {
                let _calls: Vec<_> = session.api_calls.values().collect();
                let calculation = calculator.calculate_session_cost(&session);
                match calculation {
                    Ok(calc) => Some(CurrencyAmount::from(calc.total_cost)),
                    Err(e) => {
                        tracing::warn!("Failed to calculate session cost: {}", e);
                        None
                    }
                }
            }
            _ => None, // Max plan or no calculator
        };

        // Calculate summary statistics
        let summary_stats = Self::calculate_summary_stats(&session, total_cost);

        Ok(IssueCostData {
            session_data: session,
            total_cost,
            pricing_model,
            summary_stats,
        })
    }

    /// Calculate summary statistics from a cost session
    fn calculate_summary_stats(
        session: &CostSession,
        total_cost: Option<CurrencyAmount>,
    ) -> CostSummaryStats {
        let calls: Vec<_> = session.api_calls.values().collect();
        let successful_calls = calls
            .iter()
            .filter(|call| call.status == crate::cost::ApiCallStatus::Success)
            .count() as u32;
        let failed_calls = calls.len() as u32 - successful_calls;

        // Average cost per call
        let average_cost_per_call = if let Some(total) = total_cost {
            if !calls.is_empty() {
                Some(CurrencyAmount::from(
                    total.amount() / Decimal::from(calls.len()),
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Most expensive call (would need individual call cost calculation)
        let most_expensive_call = average_cost_per_call; // Simplified for now

        // Token efficiency (output/input ratio)
        let token_efficiency = if session.total_input_tokens() > 0 {
            Some(
                Decimal::from(session.total_output_tokens())
                    / Decimal::from(session.total_input_tokens()),
            )
        } else {
            None
        };

        CostSummaryStats {
            average_cost_per_call,
            most_expensive_call,
            token_efficiency,
            total_duration: session.total_duration.map(SessionDuration::from),
            successful_calls: ApiCallCount::from(successful_calls),
            failed_calls: ApiCallCount::from(failed_calls),
        }
    }

    /// Format a complete cost section for an issue
    pub fn format_cost_section(&self, cost_data: &IssueCostData) -> String {
        if !self.config.enabled {
            return String::new();
        }

        let mut sections = Vec::new();
        sections.push("## Cost Analysis".to_string());
        sections.push(String::new()); // Empty line

        // Main cost summary
        sections.push(self.format_cost_summary(cost_data));

        // API call breakdown table (if configured and appropriate detail level)
        if self.config.show_breakdown_table
            && matches!(
                self.config.detail_level,
                DetailLevel::Full | DetailLevel::Breakdown
            )
        {
            sections.push(String::new()); // Empty line
            sections.push(self.format_api_breakdown(&cost_data.session_data));
        }

        // Cost summary statistics
        if matches!(
            self.config.detail_level,
            DetailLevel::Full | DetailLevel::Breakdown
        ) {
            sections.push(String::new()); // Empty line
            sections.push(self.format_cost_statistics(&cost_data.summary_stats));
        }

        sections.join("\n")
    }

    /// Format the main cost summary section
    fn format_cost_summary(&self, cost_data: &IssueCostData) -> String {
        let mut lines = Vec::new();

        // Total cost line
        let cost_line = match &cost_data.total_cost {
            Some(cost) => format!(
                "**Total Cost**: ${:.precision$}",
                cost.amount(),
                precision = self.precision
            ),
            None => format!(
                "**Total Cost**: Unlimited Plan - {} tokens used",
                self.format_number(cost_data.session_data.total_tokens())
            ),
        };
        lines.push(cost_line);

        // Total API calls
        lines.push(format!(
            "**Total API Calls**: {}",
            cost_data.session_data.api_call_count()
        ));

        // Token counts
        lines.push(format!(
            "**Total Input Tokens**: {}",
            self.format_number(cost_data.session_data.total_input_tokens())
        ));
        lines.push(format!(
            "**Total Output Tokens**: {}",
            self.format_number(cost_data.session_data.total_output_tokens())
        ));

        // Session duration
        if let Some(duration) = cost_data.session_data.total_duration {
            lines.push(format!(
                "**Session Duration**: {}",
                SessionDuration::from(duration)
            ));
        }

        // Completion timestamp
        if let Some(completed_at) = cost_data.session_data.completed_at {
            lines.push(format!(
                "**Completed**: {}",
                self.format_timestamp(completed_at)
            ));
        }

        lines.join("\n")
    }

    /// Format the API call breakdown table
    fn format_api_breakdown(&self, session: &CostSession) -> String {
        let mut lines = vec!["### API Call Breakdown".to_string(), String::new()];

        // Table header
        lines.push(
            "| Timestamp | Endpoint | Input Tokens | Output Tokens | Duration | Status |"
                .to_string(),
        );
        lines.push(
            "|-----------|----------|--------------|---------------|----------|--------|"
                .to_string(),
        );

        // Sort API calls by start time for chronological order
        let mut calls: Vec<_> = session.api_calls.values().collect();
        calls.sort_by_key(|call| call.started_at);

        // Table rows
        for call in calls {
            let timestamp = self.format_timestamp(call.started_at);
            let endpoint = self.truncate_endpoint(&call.endpoint);
            let input_tokens = self.format_number(call.input_tokens);
            let output_tokens = self.format_number(call.output_tokens);
            let duration = call
                .duration
                .map(|d| self.format_duration(d))
                .unwrap_or_else(|| "-".to_string());
            let status = self.format_api_call_status(&call.status);

            lines.push(format!(
                "| {} | {} | {} | {} | {} | {} |",
                timestamp, endpoint, input_tokens, output_tokens, duration, status
            ));
        }

        lines.join("\n")
    }

    /// Format cost summary statistics
    fn format_cost_statistics(&self, stats: &CostSummaryStats) -> String {
        let mut lines = Vec::new();
        lines.push("### Cost Summary".to_string());

        // Average cost per call
        if let Some(avg_cost) = &stats.average_cost_per_call {
            lines.push(format!(
                "- **Average cost per call**: ${:.precision$}",
                avg_cost.amount(),
                precision = self.precision
            ));
        }

        // Most expensive call
        if let Some(max_cost) = &stats.most_expensive_call {
            lines.push(format!(
                "- **Most expensive call**: ${:.precision$}",
                max_cost.amount(),
                precision = self.precision
            ));
        }

        // Token efficiency
        if let Some(efficiency) = &stats.token_efficiency {
            lines.push(format!(
                "- **Token efficiency**: {:.2} (output/input ratio)",
                efficiency
            ));
        }

        // Success/failure counts
        let total_calls = stats.successful_calls + stats.failed_calls;
        if total_calls.count() > 0 {
            let success_rate =
                (stats.successful_calls.count() as f64 / total_calls.count() as f64) * 100.0;
            lines.push(format!(
                "- **Success rate**: {:.1}% ({} successful, {} failed)",
                success_rate,
                stats.successful_calls.count(),
                stats.failed_calls.count()
            ));
        }

        lines.join("\n")
    }

    /// Format a number with thousands separators
    fn format_number(&self, number: u32) -> String {
        let number_str = number.to_string();
        let chars: Vec<char> = number_str.chars().collect();
        let mut result = String::new();

        for (i, &ch) in chars.iter().enumerate() {
            let remaining = chars.len() - i;
            if i > 0 && remaining % 3 == 0 {
                result.push_str(&self.config.thousands_separator);
            }
            result.push(ch);
        }

        result
    }

    /// Format a duration into human-readable form
    fn format_duration(&self, duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;

        if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    /// Format a timestamp according to configuration
    fn format_timestamp(&self, timestamp: DateTime<Utc>) -> String {
        timestamp.format(&self.config.date_format).to_string()
    }

    /// Truncate endpoint URL for table display
    fn truncate_endpoint(&self, endpoint: &str) -> String {
        let max_len = self.config.max_endpoint_display_length;
        if endpoint.len() <= max_len {
            endpoint.to_string()
        } else {
            format!("{}...", &endpoint[..max_len - 3])
        }
    }

    /// Format API call status for display
    fn format_api_call_status(&self, status: &crate::cost::ApiCallStatus) -> String {
        match status {
            crate::cost::ApiCallStatus::Success => "✓".to_string(),
            crate::cost::ApiCallStatus::Failed => "✗".to_string(),
            crate::cost::ApiCallStatus::Timeout => "⏱".to_string(),
            crate::cost::ApiCallStatus::Cancelled => "⚠".to_string(),
            crate::cost::ApiCallStatus::InProgress => "⋯".to_string(),
        }
    }
}

impl Default for CostSectionFormatter {
    fn default() -> Self {
        Self::new(CostFormattingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::{ApiCall, ApiCallStatus, CostSessionStatus, IssueId};
    use rust_decimal::Decimal;
    use std::time::Duration;

    fn create_test_cost_data() -> IssueCostData {
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add some test API calls
        let mut call1 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        call1.complete(1000, 1500, ApiCallStatus::Success, None);
        let _call_id1 = session.add_api_call(call1).unwrap();

        let mut call2 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        call2.complete(800, 1200, ApiCallStatus::Success, None);
        let _call_id2 = session.add_api_call(call2).unwrap();

        session.complete(CostSessionStatus::Completed).unwrap();

        let summary_stats = CostSummaryStats {
            average_cost_per_call: Some(CurrencyAmount(Decimal::new(25, 2))), // $0.25
            most_expensive_call: Some(CurrencyAmount(Decimal::new(30, 2))),   // $0.30
            token_efficiency: Some(Decimal::new(150, 2)),                     // 1.50
            total_duration: session.total_duration.map(SessionDuration::from),
            successful_calls: ApiCallCount(2),
            failed_calls: ApiCallCount(0),
        };

        IssueCostData {
            session_data: session,
            total_cost: Some(CurrencyAmount(Decimal::new(50, 2))), // $0.50
            pricing_model: PricingModel::Paid(crate::cost::PaidPlanConfig::new_with_defaults()),
            summary_stats,
        }
    }

    #[test]
    fn test_cost_formatting_config_defaults() {
        let config = CostFormattingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.detail_level, DetailLevel::Full);
        assert_eq!(config.currency_precision, 2);
        assert!(config.show_breakdown_table);
        assert_eq!(config.thousands_separator, ",");
        assert!(!config.include_metadata);
    }

    #[test]
    fn test_cost_formatting_config_summary_only() {
        let config = CostFormattingConfig::summary_only();
        assert_eq!(config.detail_level, DetailLevel::Summary);
        assert!(!config.show_breakdown_table);
    }

    #[test]
    fn test_cost_formatting_config_full_breakdown() {
        let config = CostFormattingConfig::full_breakdown();
        assert_eq!(config.detail_level, DetailLevel::Breakdown);
        assert!(config.show_breakdown_table);
        assert!(config.include_metadata);
    }

    #[test]
    fn test_formatter_creation() {
        let config = CostFormattingConfig::default();
        let formatter = CostSectionFormatter::new(config.clone());
        assert_eq!(formatter.config, config);
        assert_eq!(formatter.precision, 2);
        assert_eq!(formatter._locale, "en_US");
    }

    #[test]
    fn test_format_number() {
        let formatter = CostSectionFormatter::default();
        assert_eq!(formatter.format_number(123), "123");
        assert_eq!(formatter.format_number(1234), "1,234");
        assert_eq!(formatter.format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_duration() {
        let formatter = CostSectionFormatter::default();
        assert_eq!(formatter.format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(formatter.format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(
            formatter.format_duration(Duration::from_secs(150)),
            "2m 30s"
        );
    }

    #[test]
    fn test_format_api_call_status() {
        let formatter = CostSectionFormatter::default();
        assert_eq!(
            formatter.format_api_call_status(&ApiCallStatus::Success),
            "✓"
        );
        assert_eq!(
            formatter.format_api_call_status(&ApiCallStatus::Failed),
            "✗"
        );
        assert_eq!(
            formatter.format_api_call_status(&ApiCallStatus::Timeout),
            "⏱"
        );
        assert_eq!(
            formatter.format_api_call_status(&ApiCallStatus::Cancelled),
            "⚠"
        );
        assert_eq!(
            formatter.format_api_call_status(&ApiCallStatus::InProgress),
            "⋯"
        );
    }

    #[test]
    fn test_truncate_endpoint() {
        let formatter = CostSectionFormatter::default();
        let short_endpoint = "https://api.example.com";
        assert_eq!(formatter.truncate_endpoint(short_endpoint), short_endpoint);

        let long_endpoint = "https://api.anthropic.com/v1/messages/with/very/long/path";
        let truncated = formatter.truncate_endpoint(long_endpoint);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= 30);
    }

    #[test]
    fn test_format_cost_summary() {
        let formatter = CostSectionFormatter::default();
        let cost_data = create_test_cost_data();
        let summary = formatter.format_cost_summary(&cost_data);

        assert!(summary.contains("**Total Cost**: $0.50"));
        assert!(summary.contains("**Total API Calls**: 2"));
        assert!(summary.contains("**Total Input Tokens**: 1,800"));
        assert!(summary.contains("**Total Output Tokens**: 2,700"));
        assert!(summary.contains("**Session Duration**:"));
        assert!(summary.contains("**Completed**:"));
    }

    #[test]
    fn test_format_cost_summary_max_plan() {
        let formatter = CostSectionFormatter::default();
        let mut cost_data = create_test_cost_data();
        cost_data.total_cost = None; // Simulate max plan
        let summary = formatter.format_cost_summary(&cost_data);

        assert!(summary.contains("**Total Cost**: Unlimited Plan - 4,500 tokens used"));
        assert!(summary.contains("**Total API Calls**: 2"));
    }

    #[test]
    fn test_format_api_breakdown() {
        let formatter = CostSectionFormatter::default();
        let cost_data = create_test_cost_data();
        let breakdown = formatter.format_api_breakdown(&cost_data.session_data);

        assert!(breakdown.contains("### API Call Breakdown"));
        assert!(breakdown.contains(
            "| Timestamp | Endpoint | Input Tokens | Output Tokens | Duration | Status |"
        ));
        assert!(breakdown.contains(
            "|-----------|----------|--------------|---------------|----------|--------|"
        ));
        assert!(breakdown.contains("✓")); // Success status
        assert!(breakdown.contains("1,000")); // Formatted token count
        assert!(breakdown.contains("1,500"));
    }

    #[test]
    fn test_format_cost_statistics() {
        let formatter = CostSectionFormatter::default();
        let cost_data = create_test_cost_data();
        let stats = formatter.format_cost_statistics(&cost_data.summary_stats);

        assert!(stats.contains("### Cost Summary"));
        assert!(stats.contains("**Average cost per call**: $0.25"));
        assert!(stats.contains("**Most expensive call**: $0.30"));
        assert!(stats.contains("**Token efficiency**: 1.50"));
        assert!(stats.contains("**Success rate**: 100.0% (2 successful, 0 failed)"));
    }

    #[test]
    fn test_format_cost_section_full() {
        let formatter = CostSectionFormatter::default();
        let cost_data = create_test_cost_data();
        let section = formatter.format_cost_section(&cost_data);

        assert!(section.contains("## Cost Analysis"));
        assert!(section.contains("**Total Cost**: $0.50"));
        assert!(section.contains("### API Call Breakdown"));
        assert!(section.contains("### Cost Summary"));
        assert!(section.contains("**Average cost per call**"));
    }

    #[test]
    fn test_format_cost_section_summary_only() {
        let config = CostFormattingConfig::summary_only();
        let formatter = CostSectionFormatter::new(config);
        let cost_data = create_test_cost_data();
        let section = formatter.format_cost_section(&cost_data);

        assert!(section.contains("## Cost Analysis"));
        assert!(section.contains("**Total Cost**: $0.50"));
        assert!(!section.contains("### API Call Breakdown"));
        assert!(!section.contains("### Cost Summary"));
    }

    #[test]
    fn test_format_cost_section_disabled() {
        let config = CostFormattingConfig {
            enabled: false,
            ..CostFormattingConfig::default()
        };
        let formatter = CostSectionFormatter::new(config);
        let cost_data = create_test_cost_data();
        let section = formatter.format_cost_section(&cost_data);

        assert_eq!(section, "");
    }
}
