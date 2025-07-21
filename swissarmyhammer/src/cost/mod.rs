//! Cost tracking system for API call monitoring
//!
//! This module provides comprehensive cost tracking for Claude Code API interactions
//! during issue workflow execution. It tracks API calls, token usage, and provides
//! cost calculation capabilities throughout issue processing.

pub mod calculator;
#[cfg(feature = "database")]
pub mod database;
pub mod formatting;
pub mod token_counter;
pub mod token_estimation;
pub mod tracker;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod token_integration_tests;

#[cfg(test)]
pub mod test_utils;

pub use calculator::{
    CostCalculation, CostCalculator, MaxPlanConfig, PaidPlanConfig, PricingModel, PricingRates,
};
pub use formatting::{
    CostFormattingConfig, CostSectionFormatter, CostSummaryStats, DetailLevel, IssueCostData,
};
pub use token_counter::{
    ApiTokenExtractor, ConfidenceLevel, TokenCounter, TokenSource, TokenUsage, TokenValidator,
    ValidationResult, ValidationStats,
};
pub use token_estimation::{ContentType, EstimationConfig, Language, TextAnalyzer, TokenEstimator};
pub use tracker::{
    ApiCall, ApiCallId, ApiCallStatus, CostError, CostSession, CostSessionId, CostSessionStatus,
    CostTracker, IssueId,
};

#[cfg(feature = "database")]
pub use database::{
    CostAnalytics, CostDatabase, DatabaseConfig, DatabaseConfigError, DatabaseError,
    Migration, MigrationError, MigrationRunner,
};

#[cfg(feature = "database")]
pub use database::queries::{
    CostTrend, IssueCostSummary, ModelUsage, QueryError, TimePeriod, TrendQuery,
};
