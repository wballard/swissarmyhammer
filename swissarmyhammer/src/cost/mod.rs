//! Cost tracking system for API call monitoring
//!
//! This module provides comprehensive cost tracking for Claude Code API interactions
//! during issue workflow execution. It tracks API calls, token usage, and provides
//! cost calculation capabilities throughout issue processing.

pub mod calculator;
pub mod tracker;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod test_utils;

pub use calculator::{
    CostCalculation, CostCalculator, MaxPlanConfig, PaidPlanConfig, PricingModel, PricingRates,
};
pub use tracker::{
    ApiCall, ApiCallId, ApiCallStatus, CostError, CostSession, CostSessionId, CostSessionStatus,
    CostTracker, IssueId,
};
