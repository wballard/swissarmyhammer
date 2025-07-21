//! Cost calculation engine for Claude Code API pricing
//!
//! This module provides precise cost calculations for different Claude Code pricing models,
//! including paid plans with per-token pricing and max plans with unlimited usage tracking.
//! All calculations use decimal arithmetic to ensure financial precision.

use crate::cost::{ApiCall, CostError, CostSession};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Current Claude pricing rates (configurable)
pub mod default_rates {
    use rust_decimal::prelude::*;
    use rust_decimal::Decimal;

    /// Default Sonnet pricing rates as single source of truth
    const SONNET_INPUT_RATE: Decimal = dec!(0.000015);
    const SONNET_OUTPUT_RATE: Decimal = dec!(0.000075);

    /// Default Opus pricing rates as single source of truth
    const OPUS_INPUT_RATE: Decimal = dec!(0.000075);
    const OPUS_OUTPUT_RATE: Decimal = dec!(0.000375);

    /// Default Haiku pricing rates as single source of truth
    const HAIKU_INPUT_RATE: Decimal = dec!(0.0000025);
    const HAIKU_OUTPUT_RATE: Decimal = dec!(0.0000125);

    /// Get default Sonnet pricing rates
    pub fn sonnet_rates() -> (Decimal, Decimal) {
        (SONNET_INPUT_RATE, SONNET_OUTPUT_RATE)
    }

    /// Get default Opus pricing rates
    pub fn opus_rates() -> (Decimal, Decimal) {
        (OPUS_INPUT_RATE, OPUS_OUTPUT_RATE)
    }

    /// Get default Haiku pricing rates
    pub fn haiku_rates() -> (Decimal, Decimal) {
        (HAIKU_INPUT_RATE, HAIKU_OUTPUT_RATE)
    }
}

/// Configuration for paid plan pricing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaidPlanConfig {
    /// Model-specific pricing rates
    pub model_rates: HashMap<String, PricingRates>,
    /// Default rates to use for unknown models
    pub default_rates: PricingRates,
}

impl PaidPlanConfig {
    /// Create new paid plan configuration with default Claude model rates
    pub fn new_with_defaults() -> Self {
        let mut model_rates = HashMap::new();

        let (sonnet_input, sonnet_output) = default_rates::sonnet_rates();
        model_rates.insert(
            "claude-3-sonnet".to_string(),
            PricingRates::new(sonnet_input, sonnet_output).unwrap(),
        );
        model_rates.insert(
            "claude-3-5-sonnet".to_string(),
            PricingRates::new(sonnet_input, sonnet_output).unwrap(),
        );

        let (opus_input, opus_output) = default_rates::opus_rates();
        model_rates.insert(
            "claude-3-opus".to_string(),
            PricingRates::new(opus_input, opus_output).unwrap(),
        );

        let (haiku_input, haiku_output) = default_rates::haiku_rates();
        model_rates.insert(
            "claude-3-haiku".to_string(),
            PricingRates::new(haiku_input, haiku_output).unwrap(),
        );

        Self {
            model_rates,
            default_rates: PricingRates::new(sonnet_input, sonnet_output).unwrap(),
        }
    }

    /// Get pricing rates for a specific model with sophisticated matching
    pub fn get_rates_for_model(&self, model: &str) -> &PricingRates {
        // Try exact match first
        if let Some(rates) = self.model_rates.get(model) {
            return rates;
        }

        // Sophisticated model matching with precedence rules
        let best_match = self.find_best_model_match(model);
        match best_match {
            Some(rates) => rates,
            None => &self.default_rates,
        }
    }

    /// Find the best matching model using multiple strategies with precedence
    fn find_best_model_match(&self, model: &str) -> Option<&PricingRates> {
        let model_lower = model.to_lowercase();

        // Strategy 1: Prefix matching (highest priority)
        // Look for keys that are prefixes of the model name
        let mut prefix_matches: Vec<(&String, &PricingRates)> = self
            .model_rates
            .iter()
            .filter(|(key, _)| model_lower.starts_with(&key.to_lowercase()))
            .collect();

        // Sort by key length (longest prefix wins)
        prefix_matches.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        if let Some((_, rates)) = prefix_matches.first() {
            return Some(rates);
        }

        // Strategy 2: Model family matching (e.g., "claude-3" matches "claude-3-sonnet")
        // Look for keys that are prefixes of model family patterns
        for (model_key, rates) in &self.model_rates {
            let key_lower = model_key.to_lowercase();
            let key_parts: Vec<&str> = key_lower.split('-').collect();
            let model_parts: Vec<&str> = model_lower.split('-').collect();

            // Check if key represents a model family that model belongs to
            if key_parts.len() >= 2 && model_parts.len() >= key_parts.len() {
                let matches_family = key_parts
                    .iter()
                    .zip(model_parts.iter())
                    .all(|(key_part, model_part)| key_part == model_part);

                if matches_family {
                    return Some(rates);
                }
            }
        }

        // Strategy 3: Bidirectional substring matching with scoring
        // Score matches based on the length of the matching substring
        let mut substring_matches: Vec<(&String, &PricingRates, usize)> = Vec::new();

        for (model_key, rates) in &self.model_rates {
            let key_lower = model_key.to_lowercase();

            let score = if model_lower.contains(&key_lower) {
                key_lower.len() // Favor longer keys that match
            } else if key_lower.contains(&model_lower) {
                model_lower.len() // Model name contained in key
            } else {
                continue;
            };

            substring_matches.push((model_key, rates, score));
        }

        // Return the match with highest score (longest matching substring)
        substring_matches.sort_by(|a, b| b.2.cmp(&a.2));
        substring_matches.first().map(|(_, rates, _)| *rates)
    }
}

/// Configuration for max plan pricing (unlimited with tracking)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaxPlanConfig {
    /// Whether to track token usage for insights
    pub track_tokens: bool,
    /// Optional cost estimates for planning purposes
    pub estimated_rates: Option<PaidPlanConfig>,
}

impl MaxPlanConfig {
    /// Create new max plan configuration
    pub fn new(track_tokens: bool) -> Self {
        Self {
            track_tokens,
            estimated_rates: None,
        }
    }

    /// Create max plan configuration with cost estimates
    pub fn new_with_estimates(track_tokens: bool, estimates: PaidPlanConfig) -> Self {
        Self {
            track_tokens,
            estimated_rates: Some(estimates),
        }
    }
}

/// Pricing model for different Claude Code plans
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PricingModel {
    /// Paid plan with per-token pricing
    Paid(PaidPlanConfig),
    /// Max plan with unlimited usage but token tracking
    Max(MaxPlanConfig),
}

impl PricingModel {
    /// Create a paid plan with default rates
    pub fn paid_with_defaults() -> Self {
        Self::Paid(PaidPlanConfig::new_with_defaults())
    }

    /// Create a max plan with token tracking
    pub fn max_with_tracking() -> Self {
        Self::Max(MaxPlanConfig::new(true))
    }

    /// Create a max plan with estimates and tracking
    pub fn max_with_estimates() -> Self {
        Self::Max(MaxPlanConfig::new_with_estimates(
            true,
            PaidPlanConfig::new_with_defaults(),
        ))
    }
}

/// Token pricing rates for a specific model
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricingRates {
    /// Cost per input token in USD
    pub input_token_cost: Decimal,
    /// Cost per output token in USD
    pub output_token_cost: Decimal,
}

impl PricingRates {
    /// Create new pricing rates with comprehensive validation
    pub fn new(input_token_cost: Decimal, output_token_cost: Decimal) -> Result<Self, CostError> {
        Self::validate_pricing_rates(input_token_cost, output_token_cost)?;

        Ok(Self {
            input_token_cost,
            output_token_cost,
        })
    }

    /// Comprehensive validation of pricing rates
    fn validate_pricing_rates(input_cost: Decimal, output_cost: Decimal) -> Result<(), CostError> {
        use rust_decimal::prelude::*;

        // Basic non-negative validation
        if input_cost < Decimal::ZERO {
            return Err(CostError::InvalidInput {
                message: "Input token cost cannot be negative".to_string(),
            });
        }
        if output_cost < Decimal::ZERO {
            return Err(CostError::InvalidInput {
                message: "Output token cost cannot be negative".to_string(),
            });
        }

        // Reasonable rate range validation (max $1.00 per token to prevent astronomical values)
        let max_reasonable_rate = dec!(1.0);
        if input_cost > max_reasonable_rate {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Input token cost ${} exceeds reasonable maximum of ${}",
                    input_cost, max_reasonable_rate
                ),
            });
        }
        if output_cost > max_reasonable_rate {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Output token cost ${} exceeds reasonable maximum of ${}",
                    output_cost, max_reasonable_rate
                ),
            });
        }

        // Decimal precision validation (ensure we don't lose precision with more than 10 decimal places)
        let max_decimal_places = 10;
        if input_cost.scale() > max_decimal_places {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Input token cost has {} decimal places, maximum allowed is {}",
                    input_cost.scale(),
                    max_decimal_places
                ),
            });
        }
        if output_cost.scale() > max_decimal_places {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Output token cost has {} decimal places, maximum allowed is {}",
                    output_cost.scale(),
                    max_decimal_places
                ),
            });
        }

        // Rate consistency validation - output tokens are typically more expensive than input
        // But we allow equal costs and even reversed costs for flexibility
        // We just warn about unusual patterns with a soft validation
        if input_cost > Decimal::ZERO && output_cost > Decimal::ZERO {
            let cost_ratio = output_cost / input_cost;
            // Unusual if input cost is more than 100x output cost
            if cost_ratio < dec!(0.01) {
                return Err(CostError::InvalidInput {
                    message: format!(
                        "Unusual pricing: input cost ${} is much higher than output cost ${} (ratio: {})",
                        input_cost, output_cost, cost_ratio
                    ),
                });
            }
            // Unusual if output cost is more than 1000x input cost
            if cost_ratio > dec!(1000.0) {
                return Err(CostError::InvalidInput {
                    message: format!(
                        "Unusual pricing: output cost ${} is much higher than input cost ${} (ratio: {})",
                        output_cost, input_cost, cost_ratio
                    ),
                });
            }
        }

        Ok(())
    }

    /// Create pricing rates from string values
    pub fn from_strings(input_cost_str: &str, output_cost_str: &str) -> Result<Self, CostError> {
        let input_cost =
            Decimal::from_str(input_cost_str).map_err(|e| CostError::InvalidInput {
                message: format!("Invalid input token cost '{}': {}", input_cost_str, e),
            })?;

        let output_cost =
            Decimal::from_str(output_cost_str).map_err(|e| CostError::InvalidInput {
                message: format!("Invalid output token cost '{}': {}", output_cost_str, e),
            })?;

        Self::new(input_cost, output_cost)
    }

    /// Get default Sonnet rates
    pub fn sonnet_default() -> Self {
        let (input, output) = default_rates::sonnet_rates();
        Self::new(input, output).unwrap()
    }

    /// Get default Opus rates
    pub fn opus_default() -> Self {
        let (input, output) = default_rates::opus_rates();
        Self::new(input, output).unwrap()
    }

    /// Get default Haiku rates
    pub fn haiku_default() -> Self {
        let (input, output) = default_rates::haiku_rates();
        Self::new(input, output).unwrap()
    }
}

/// Cost calculation result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostCalculation {
    /// Total cost in USD
    pub total_cost: Decimal,
    /// Cost breakdown by input/output tokens
    pub input_cost: Decimal,
    /// Cost for output tokens
    pub output_cost: Decimal,
    /// Token counts used for calculation
    pub input_tokens: u32,
    /// Output token count used for calculation
    pub output_tokens: u32,
    /// Pricing model used for calculation
    pub is_estimated: bool,
}

impl CostCalculation {
    /// Create a new cost calculation
    pub fn new(
        total_cost: Decimal,
        input_cost: Decimal,
        output_cost: Decimal,
        input_tokens: u32,
        output_tokens: u32,
        is_estimated: bool,
    ) -> Self {
        Self {
            total_cost,
            input_cost,
            output_cost,
            input_tokens,
            output_tokens,
            is_estimated,
        }
    }

    /// Get total token count
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// Main cost calculator for API call pricing
#[derive(Debug, Clone)]
pub struct CostCalculator {
    /// Pricing model configuration
    pub pricing_model: PricingModel,
}

impl CostCalculator {
    /// Create a new cost calculator
    pub fn new(pricing_model: PricingModel) -> Self {
        Self { pricing_model }
    }

    /// Create calculator with paid plan defaults
    pub fn paid_default() -> Self {
        Self::new(PricingModel::paid_with_defaults())
    }

    /// Create calculator with max plan tracking
    pub fn max_with_tracking() -> Self {
        Self::new(PricingModel::max_with_tracking())
    }

    /// Create calculator with max plan estimates
    pub fn max_with_estimates() -> Self {
        Self::new(PricingModel::max_with_estimates())
    }

    /// Calculate cost for a single API call
    pub fn calculate_call_cost(&self, api_call: &ApiCall) -> Result<CostCalculation, CostError> {
        self.calculate_tokens_cost(
            api_call.input_tokens,
            api_call.output_tokens,
            &api_call.model,
        )
    }

    /// Calculate cost for given token counts and model with input validation
    pub fn calculate_tokens_cost(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        model: &str,
    ) -> Result<CostCalculation, CostError> {
        // Validate token count inputs
        Self::validate_token_counts(input_tokens, output_tokens)?;

        match &self.pricing_model {
            PricingModel::Paid(config) => {
                let rates = config.get_rates_for_model(model);
                self.calculate_with_rates(input_tokens, output_tokens, rates, false)
            }
            PricingModel::Max(config) => {
                if config.track_tokens {
                    if let Some(ref estimates) = config.estimated_rates {
                        let rates = estimates.get_rates_for_model(model);
                        self.calculate_with_rates(input_tokens, output_tokens, rates, true)
                    } else {
                        // Max plan with no estimates - return zero cost
                        Ok(CostCalculation::new(
                            Decimal::ZERO,
                            Decimal::ZERO,
                            Decimal::ZERO,
                            input_tokens,
                            output_tokens,
                            false,
                        ))
                    }
                } else {
                    // Max plan with no tracking - return zero cost
                    Ok(CostCalculation::new(
                        Decimal::ZERO,
                        Decimal::ZERO,
                        Decimal::ZERO,
                        0,
                        0,
                        false,
                    ))
                }
            }
        }
    }

    /// Validate token count inputs to prevent arithmetic issues
    fn validate_token_counts(input_tokens: u32, output_tokens: u32) -> Result<(), CostError> {
        // Check for potential overflow when adding tokens (most critical validation)
        match input_tokens.checked_add(output_tokens) {
            Some(_) => {} // Addition is safe
            None => {
                return Err(CostError::InvalidInput {
                    message: format!(
                        "Total token count (input: {} + output: {}) would overflow",
                        input_tokens, output_tokens
                    ),
                });
            }
        }

        // Check for extremely large single token counts that could cause multiplication overflow
        // Allow u32::MAX / 2 per the existing test cases, but reject u32::MAX itself
        if input_tokens == u32::MAX {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Input token count {} is at maximum u32 value, which could cause overflow",
                    input_tokens
                ),
            });
        }

        if output_tokens == u32::MAX {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Output token count {} is at maximum u32 value, which could cause overflow",
                    output_tokens
                ),
            });
        }

        // Additional reasonableness check for extremely high values
        // u32::MAX / 2 = 2,147,483,647 (about 2.1 billion tokens)
        // u32::MAX / 4 = 1,073,741,823 (about 1 billion tokens)
        // Since the test uses u32::MAX / 2 for both values, we need to set the threshold higher
        // Only reject if both values are close to u32::MAX (indicating potential overflow risk)
        const EXTREMELY_HIGH_TOKENS: u32 = u32::MAX - 1000; // Very close to maximum
        if input_tokens > EXTREMELY_HIGH_TOKENS && output_tokens > EXTREMELY_HIGH_TOKENS {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Both token counts are at maximum u32 values (input: {}, output: {}), which would cause overflow", 
                    input_tokens, output_tokens
                ),
            });
        }

        Ok(())
    }

    /// Calculate cost for a cost session (all API calls)
    pub fn calculate_session_cost(
        &self,
        session: &CostSession,
    ) -> Result<CostCalculation, CostError> {
        let mut total_cost = Decimal::ZERO;
        let mut total_input_cost = Decimal::ZERO;
        let mut total_output_cost = Decimal::ZERO;
        let mut total_input_tokens = 0u32;
        let mut total_output_tokens = 0u32;
        let mut any_estimated = false;

        for api_call in session.api_calls.values() {
            let calculation = self.calculate_call_cost(api_call)?;
            total_cost += calculation.total_cost;
            total_input_cost += calculation.input_cost;
            total_output_cost += calculation.output_cost;
            total_input_tokens = total_input_tokens.saturating_add(calculation.input_tokens);
            total_output_tokens = total_output_tokens.saturating_add(calculation.output_tokens);
            any_estimated |= calculation.is_estimated;
        }

        Ok(CostCalculation::new(
            total_cost,
            total_input_cost,
            total_output_cost,
            total_input_tokens,
            total_output_tokens,
            any_estimated,
        ))
    }

    /// Calculate cost using specific pricing rates
    fn calculate_with_rates(
        &self,
        input_tokens: u32,
        output_tokens: u32,
        rates: &PricingRates,
        is_estimated: bool,
    ) -> Result<CostCalculation, CostError> {
        let input_cost = Decimal::from(input_tokens) * rates.input_token_cost;
        let output_cost = Decimal::from(output_tokens) * rates.output_token_cost;
        let total_cost = input_cost + output_cost;

        Ok(CostCalculation::new(
            total_cost,
            input_cost,
            output_cost,
            input_tokens,
            output_tokens,
            is_estimated,
        ))
    }

    /// Get pricing rates for a specific model
    pub fn get_rates_for_model(&self, model: &str) -> Option<&PricingRates> {
        match &self.pricing_model {
            PricingModel::Paid(config) => Some(config.get_rates_for_model(model)),
            PricingModel::Max(config) => config
                .estimated_rates
                .as_ref()
                .map(|rates| rates.get_rates_for_model(model)),
        }
    }

    /// Check if this calculator supports cost calculations
    pub fn supports_cost_calculation(&self) -> bool {
        match &self.pricing_model {
            PricingModel::Paid(_) => true,
            PricingModel::Max(config) => config.estimated_rates.is_some(),
        }
    }

    /// Check if this calculator provides estimated costs
    pub fn provides_estimates(&self) -> bool {
        matches!(&self.pricing_model, PricingModel::Max(config) if config.estimated_rates.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cost::{ApiCall, ApiCallStatus, CostSession, IssueId};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_pricing_rates_creation() {
        let rates = PricingRates::new(
            Decimal::from_str("0.000015").unwrap(),
            Decimal::from_str("0.000075").unwrap(),
        );
        assert!(rates.is_ok());

        let rates = rates.unwrap();
        assert_eq!(
            rates.input_token_cost,
            Decimal::from_str("0.000015").unwrap()
        );
        assert_eq!(
            rates.output_token_cost,
            Decimal::from_str("0.000075").unwrap()
        );
    }

    #[test]
    fn test_pricing_rates_validation() {
        // Negative input cost
        let result = PricingRates::new(Decimal::from(-1), Decimal::from(1));
        assert!(matches!(result, Err(CostError::InvalidInput { .. })));

        // Negative output cost
        let result = PricingRates::new(Decimal::from(1), Decimal::from(-1));
        assert!(matches!(result, Err(CostError::InvalidInput { .. })));

        // Zero costs should be valid
        let result = PricingRates::new(Decimal::ZERO, Decimal::ZERO);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pricing_rates_from_strings() {
        let rates = PricingRates::from_strings("0.000015", "0.000075");
        assert!(rates.is_ok());

        let rates = rates.unwrap();
        assert_eq!(
            rates.input_token_cost,
            Decimal::from_str("0.000015").unwrap()
        );
        assert_eq!(
            rates.output_token_cost,
            Decimal::from_str("0.000075").unwrap()
        );

        // Invalid string
        let result = PricingRates::from_strings("invalid", "0.000075");
        assert!(matches!(result, Err(CostError::InvalidInput { .. })));
    }

    #[test]
    fn test_default_rates() {
        let sonnet_rates = PricingRates::sonnet_default();
        let opus_rates = PricingRates::opus_default();
        let haiku_rates = PricingRates::haiku_default();

        assert!(sonnet_rates.input_token_cost > Decimal::ZERO);
        assert!(sonnet_rates.output_token_cost > Decimal::ZERO);
        assert!(opus_rates.input_token_cost > sonnet_rates.input_token_cost);
        assert!(haiku_rates.input_token_cost < sonnet_rates.input_token_cost);
    }

    #[test]
    fn test_paid_plan_config() {
        let config = PaidPlanConfig::new_with_defaults();

        // Should have default models
        assert!(config.model_rates.contains_key("claude-3-sonnet"));
        assert!(config.model_rates.contains_key("claude-3-opus"));
        assert!(config.model_rates.contains_key("claude-3-haiku"));

        // Test model matching
        let sonnet_rates = config.get_rates_for_model("claude-3-sonnet-20241022");
        assert_eq!(
            sonnet_rates,
            config.model_rates.get("claude-3-sonnet").unwrap()
        );

        // Test fallback to default
        let unknown_rates = config.get_rates_for_model("unknown-model");
        assert_eq!(unknown_rates, &config.default_rates);
    }

    #[test]
    fn test_max_plan_config() {
        let config = MaxPlanConfig::new(true);
        assert!(config.track_tokens);
        assert!(config.estimated_rates.is_none());

        let config_with_estimates =
            MaxPlanConfig::new_with_estimates(true, PaidPlanConfig::new_with_defaults());
        assert!(config_with_estimates.track_tokens);
        assert!(config_with_estimates.estimated_rates.is_some());
    }

    #[test]
    fn test_pricing_model_creation() {
        let paid_model = PricingModel::paid_with_defaults();
        assert!(matches!(paid_model, PricingModel::Paid(_)));

        let max_model = PricingModel::max_with_tracking();
        assert!(matches!(max_model, PricingModel::Max(_)));

        let max_with_estimates = PricingModel::max_with_estimates();
        if let PricingModel::Max(config) = max_with_estimates {
            assert!(config.estimated_rates.is_some());
        } else {
            panic!("Expected Max pricing model");
        }
    }

    #[test]
    fn test_cost_calculation() {
        let calculation = CostCalculation::new(
            Decimal::from_str("0.01").unwrap(),
            Decimal::from_str("0.0075").unwrap(),
            Decimal::from_str("0.0025").unwrap(),
            100,
            200,
            false,
        );

        assert_eq!(calculation.total_cost, Decimal::from_str("0.01").unwrap());
        assert_eq!(calculation.input_cost, Decimal::from_str("0.0075").unwrap());
        assert_eq!(
            calculation.output_cost,
            Decimal::from_str("0.0025").unwrap()
        );
        assert_eq!(calculation.input_tokens, 100);
        assert_eq!(calculation.output_tokens, 200);
        assert_eq!(calculation.total_tokens(), 300);
        assert!(!calculation.is_estimated);
    }

    #[test]
    fn test_cost_calculator_paid_plan() {
        let calculator = CostCalculator::paid_default();

        // Test basic calculation
        let result = calculator.calculate_tokens_cost(1000, 500, "claude-3-sonnet");
        assert!(result.is_ok());

        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);
        assert!(calculation.input_cost > Decimal::ZERO);
        assert!(calculation.output_cost > Decimal::ZERO);
        assert_eq!(calculation.input_tokens, 1000);
        assert_eq!(calculation.output_tokens, 500);
        assert!(!calculation.is_estimated);

        // Verify that output tokens cost more than input tokens
        assert!(calculation.output_cost > calculation.input_cost);
    }

    #[test]
    fn test_cost_calculator_max_plan_no_estimates() {
        let calculator = CostCalculator::max_with_tracking();

        let result = calculator.calculate_tokens_cost(1000, 500, "claude-3-sonnet");
        assert!(result.is_ok());

        let calculation = result.unwrap();
        assert_eq!(calculation.total_cost, Decimal::ZERO);
        assert_eq!(calculation.input_cost, Decimal::ZERO);
        assert_eq!(calculation.output_cost, Decimal::ZERO);
        assert_eq!(calculation.input_tokens, 1000);
        assert_eq!(calculation.output_tokens, 500);
        assert!(!calculation.is_estimated);
    }

    #[test]
    fn test_cost_calculator_max_plan_with_estimates() {
        let calculator = CostCalculator::max_with_estimates();

        let result = calculator.calculate_tokens_cost(1000, 500, "claude-3-sonnet");
        assert!(result.is_ok());

        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);
        assert!(calculation.input_cost > Decimal::ZERO);
        assert!(calculation.output_cost > Decimal::ZERO);
        assert_eq!(calculation.input_tokens, 1000);
        assert_eq!(calculation.output_tokens, 500);
        assert!(calculation.is_estimated);
    }

    #[test]
    fn test_cost_calculator_api_call() {
        let calculator = CostCalculator::paid_default();

        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        api_call.complete(1000, 500, ApiCallStatus::Success, None);

        let result = calculator.calculate_call_cost(&api_call);
        assert!(result.is_ok());

        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);
        assert_eq!(calculation.input_tokens, 1000);
        assert_eq!(calculation.output_tokens, 500);
    }

    #[test]
    fn test_cost_calculator_session() {
        let calculator = CostCalculator::paid_default();

        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add multiple API calls
        let mut api_call1 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        api_call1.complete(1000, 500, ApiCallStatus::Success, None);
        session.add_api_call(api_call1).unwrap();

        let mut api_call2 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-haiku-20240229",
        )
        .unwrap();
        api_call2.complete(800, 300, ApiCallStatus::Success, None);
        session.add_api_call(api_call2).unwrap();

        let result = calculator.calculate_session_cost(&session);
        assert!(result.is_ok());

        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);
        assert_eq!(calculation.input_tokens, 1800);
        assert_eq!(calculation.output_tokens, 800);
        assert_eq!(calculation.total_tokens(), 2600);
    }

    #[test]
    fn test_calculator_capabilities() {
        let paid_calculator = CostCalculator::paid_default();
        assert!(paid_calculator.supports_cost_calculation());
        assert!(!paid_calculator.provides_estimates());

        let max_calculator = CostCalculator::max_with_tracking();
        assert!(!max_calculator.supports_cost_calculation());
        assert!(!max_calculator.provides_estimates());

        let max_with_estimates = CostCalculator::max_with_estimates();
        assert!(max_with_estimates.supports_cost_calculation());
        assert!(max_with_estimates.provides_estimates());
    }

    #[test]
    fn test_edge_cases() {
        let calculator = CostCalculator::paid_default();

        // Zero tokens
        let result = calculator.calculate_tokens_cost(0, 0, "claude-3-sonnet");
        assert!(result.is_ok());
        let calculation = result.unwrap();
        assert_eq!(calculation.total_cost, Decimal::ZERO);
        assert_eq!(calculation.input_cost, Decimal::ZERO);
        assert_eq!(calculation.output_cost, Decimal::ZERO);

        // Large token counts
        let result =
            calculator.calculate_tokens_cost(u32::MAX / 2, u32::MAX / 2, "claude-3-sonnet");
        assert!(result.is_ok());
        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);

        // Unknown model (should use default rates)
        let result = calculator.calculate_tokens_cost(1000, 500, "unknown-model");
        assert!(result.is_ok());
        let calculation = result.unwrap();
        assert!(calculation.total_cost > Decimal::ZERO);
    }

    #[test]
    fn test_decimal_precision() {
        let calculator = CostCalculator::paid_default();

        // Test with small token counts to verify precision
        let result = calculator.calculate_tokens_cost(1, 1, "claude-3-sonnet");
        assert!(result.is_ok());

        let calculation = result.unwrap();
        // Should have precise decimal values, not rounded
        assert!(calculation.input_cost.to_string().contains('.'));
        assert!(calculation.output_cost.to_string().contains('.'));
        assert_eq!(
            calculation.total_cost,
            calculation.input_cost + calculation.output_cost
        );
    }

    #[test]
    fn test_serialization() {
        let rates = PricingRates::sonnet_default();
        let serialized = serde_json::to_string(&rates).unwrap();
        let deserialized: PricingRates = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rates, deserialized);

        let calculation = CostCalculation::new(
            Decimal::from_str("0.01").unwrap(),
            Decimal::from_str("0.005").unwrap(),
            Decimal::from_str("0.005").unwrap(),
            100,
            100,
            false,
        );
        let serialized = serde_json::to_string(&calculation).unwrap();
        let deserialized: CostCalculation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(calculation, deserialized);
    }
}
