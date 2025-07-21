//! Token counting and validation system for Claude Code API interactions
//!
//! This module provides comprehensive token counting capabilities including:
//! - Extraction from Claude API responses
//! - Fallback token estimation
//! - Validation between API and estimated counts
//! - Support for different Claude models and tokenization approaches

use crate::cost::CostError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Default discrepancy threshold for validation (10%)
pub const DEFAULT_DISCREPANCY_THRESHOLD: f32 = 0.10;

/// Maximum number of validation records to keep in memory
pub const MAX_VALIDATION_RECORDS: usize = 1000;

/// Source of token count information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenSource {
    /// Token count extracted from API response
    ApiResponse,
    /// Token count estimated from text
    Estimated,
    /// Mixed source (partial API data with estimation)
    Mixed,
}

/// Confidence level in token count accuracy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Low confidence (> 20% uncertainty)
    Low,
    /// Medium confidence (5-20% uncertainty)
    Medium,
    /// High confidence (< 5% uncertainty)
    High,
    /// Exact count from API
    Exact,
}

/// Token usage information with source and confidence
///
/// ## Design Rationale: Data + Behavior Combination
///
/// This struct combines data fields with closely related behavior methods, which is
/// appropriate for the following reasons:
///
/// ### Why the current design is optimal:
///
/// 1. **Cohesion**: The methods are directly related to the data and don't perform
///    complex business logic - they are simple constructors and property queries
///
/// 2. **Rust conventions**: This follows common Rust patterns where simple behavioral
///    methods are implemented directly on data structures (similar to `Option::is_some()`)
///
/// 3. **Simplicity**: The methods are thin wrappers that don't warrant separate abstractions
///
/// 4. **Performance**: No additional indirection or trait objects needed for simple queries
///
/// 5. **Usability**: Having constructors and queries on the same type improves API ergonomics
///
/// ### Alternative architectures considered:
///
/// - **Separate trait for queries**: Would add complexity without clear benefits
/// - **Builder pattern**: Overkill for a simple data structure with few fields
/// - **Factory functions**: Less discoverable than associated methods
///
/// ### When to separate data from behavior:
///
/// Future separation would be warranted if:
/// - Methods become complex with significant business logic
/// - Multiple implementations of behavior are needed (polymorphism)
/// - Cross-cutting concerns emerge (logging, caching, etc.)
/// - The struct grows to have many unrelated methods
///
/// The current design strikes the right balance between simplicity and functionality.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens
    pub input_tokens: u32,
    /// Number of output tokens
    pub output_tokens: u32,
    /// Total tokens (input + output)
    pub total_tokens: u32,
    /// Source of the token count
    pub source: TokenSource,
    /// Confidence level in the accuracy
    pub confidence: ConfidenceLevel,
}

impl TokenUsage {
    /// Create new token usage from counts
    pub fn new(
        input_tokens: u32,
        output_tokens: u32,
        source: TokenSource,
        confidence: ConfidenceLevel,
    ) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
            source,
            confidence,
        }
    }

    /// Create token usage from API response data
    pub fn from_api(input_tokens: u32, output_tokens: u32) -> Self {
        Self::new(
            input_tokens,
            output_tokens,
            TokenSource::ApiResponse,
            ConfidenceLevel::Exact,
        )
    }

    /// Create token usage from estimation
    pub fn from_estimation(
        input_tokens: u32,
        output_tokens: u32,
        confidence: ConfidenceLevel,
    ) -> Self {
        Self::new(
            input_tokens,
            output_tokens,
            TokenSource::Estimated,
            confidence,
        )
    }

    /// Check if this usage is from API response
    pub fn is_from_api(&self) -> bool {
        matches!(self.source, TokenSource::ApiResponse)
    }

    /// Check if this usage is estimated
    pub fn is_estimated(&self) -> bool {
        matches!(self.source, TokenSource::Estimated | TokenSource::Mixed)
    }
}

/// Validation result comparing API and estimated token counts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// API-reported token usage
    pub api_usage: TokenUsage,
    /// Estimated token usage
    pub estimated_usage: TokenUsage,
    /// Percentage difference between counts
    pub discrepancy_percentage: f32,
    /// Whether the discrepancy exceeds the threshold
    pub exceeds_threshold: bool,
    /// Model used for the API call
    pub model: String,
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(
        api_usage: TokenUsage,
        estimated_usage: TokenUsage,
        model: String,
        threshold: f32,
    ) -> Self {
        let discrepancy_percentage = Self::calculate_discrepancy(&api_usage, &estimated_usage);
        let exceeds_threshold = discrepancy_percentage > threshold;

        Self {
            api_usage,
            estimated_usage,
            discrepancy_percentage,
            exceeds_threshold,
            model,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Calculate discrepancy percentage between two token usages
    fn calculate_discrepancy(api_usage: &TokenUsage, estimated_usage: &TokenUsage) -> f32 {
        if api_usage.total_tokens == 0 {
            if estimated_usage.total_tokens == 0 {
                0.0 // Both zero, perfect match
            } else {
                100.0 // API has zero but estimate has tokens
            }
        } else {
            let api_total = api_usage.total_tokens as f32;
            let estimated_total = estimated_usage.total_tokens as f32;
            ((api_total - estimated_total).abs() / api_total) * 100.0
        }
    }
}

/// API token extractor for parsing different response formats
pub struct ApiTokenExtractor;

impl ApiTokenExtractor {
    /// Extract token usage from Claude API response JSON
    pub fn extract_from_response(&self, response_body: &str) -> Result<TokenUsage, CostError> {
        let json: Value =
            serde_json::from_str(response_body).map_err(|e| CostError::InvalidInput {
                message: format!("Invalid JSON response: {}", e),
            })?;

        // Try standard Claude API format
        if let Some(usage) = json.get("usage") {
            // Validate that usage is an object
            if !usage.is_object() {
                return Err(CostError::InvalidInput {
                    message: "Invalid API response: 'usage' field is not an object".to_string(),
                });
            }

            // Extract and validate token values
            match (
                usage.get("input_tokens").and_then(|v| v.as_u64()),
                usage.get("output_tokens").and_then(|v| v.as_u64()),
            ) {
                (Some(input), Some(output)) => {
                    // Validate token values are within reasonable bounds
                    Self::validate_token_values(input, output)?;

                    debug!(
                        input_tokens = input,
                        output_tokens = output,
                        "Extracted token usage from Claude API response"
                    );
                    return Ok(TokenUsage::from_api(input as u32, output as u32));
                }
                (None, Some(_)) => {
                    return Err(CostError::InvalidInput {
                        message: "Invalid API response: 'input_tokens' field is missing or not a valid number".to_string(),
                    });
                }
                (Some(_), None) => {
                    return Err(CostError::InvalidInput {
                        message: "Invalid API response: 'output_tokens' field is missing or not a valid number".to_string(),
                    });
                }
                (None, None) => {
                    return Err(CostError::InvalidInput {
                        message: "Invalid API response: both 'input_tokens' and 'output_tokens' fields are missing or invalid".to_string(),
                    });
                }
            }
        }

        // Try alternative format
        match (
            json.get("input_token_count").and_then(|v| v.as_u64()),
            json.get("output_token_count").and_then(|v| v.as_u64()),
        ) {
            (Some(input), Some(output)) => {
                // Validate token values are within reasonable bounds
                Self::validate_token_values(input, output)?;

                debug!(
                    input_tokens = input,
                    output_tokens = output,
                    "Extracted token usage from alternative API response format"
                );
                return Ok(TokenUsage::from_api(input as u32, output as u32));
            }
            (None, Some(_)) => {
                return Err(CostError::InvalidInput {
                    message: "Invalid API response: 'input_token_count' field is missing or not a valid number".to_string(),
                });
            }
            (Some(_), None) => {
                return Err(CostError::InvalidInput {
                    message: "Invalid API response: 'output_token_count' field is missing or not a valid number".to_string(),
                });
            }
            (None, None) => {
                // Continue to final error - no token usage found
            }
        }

        Err(CostError::InvalidInput {
            message: "No valid token usage found in API response. Expected 'usage' object with 'input_tokens'/'output_tokens' or 'input_token_count'/'output_token_count' fields.".to_string(),
        })
    }

    /// Validate that token values are within reasonable bounds
    fn validate_token_values(input_tokens: u64, output_tokens: u64) -> Result<(), CostError> {
        const MAX_REASONABLE_TOKENS: u64 = 10_000_000; // 10 million tokens as upper bound

        if input_tokens > MAX_REASONABLE_TOKENS {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Input token count {} exceeds maximum reasonable value {}",
                    input_tokens, MAX_REASONABLE_TOKENS
                ),
            });
        }

        if output_tokens > MAX_REASONABLE_TOKENS {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Output token count {} exceeds maximum reasonable value {}",
                    output_tokens, MAX_REASONABLE_TOKENS
                ),
            });
        }

        // Check that values fit in u32 (since TokenUsage uses u32)
        if input_tokens > u32::MAX as u64 {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Input token count {} exceeds u32 maximum value {}",
                    input_tokens,
                    u32::MAX
                ),
            });
        }

        if output_tokens > u32::MAX as u64 {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Output token count {} exceeds u32 maximum value {}",
                    output_tokens,
                    u32::MAX
                ),
            });
        }

        Ok(())
    }

    /// Extract token usage from response headers
    pub fn extract_from_headers(&self, headers: &HashMap<String, String>) -> Option<TokenUsage> {
        // Check for Anthropic-style headers
        let input_tokens = headers
            .get("anthropic-input-tokens")
            .or_else(|| headers.get("x-input-tokens"))
            .and_then(|v| v.parse::<u32>().ok());

        let output_tokens = headers
            .get("anthropic-output-tokens")
            .or_else(|| headers.get("x-output-tokens"))
            .and_then(|v| v.parse::<u32>().ok());

        if let (Some(input), Some(output)) = (input_tokens, output_tokens) {
            debug!(
                input_tokens = input,
                output_tokens = output,
                "Extracted token usage from response headers"
            );
            Some(TokenUsage::from_api(input, output))
        } else {
            None
        }
    }
}

/// Token validator for comparing API and estimated counts
pub struct TokenValidator {
    /// Discrepancy threshold for flagging validation issues
    pub discrepancy_threshold: f32,
    /// Historical validation results
    validation_history: Vec<ValidationResult>,
}

impl TokenValidator {
    /// Create a new token validator
    pub fn new(discrepancy_threshold: f32) -> Self {
        Self {
            discrepancy_threshold,
            validation_history: Vec::new(),
        }
    }

    /// Validate token usage against estimation
    pub fn validate(
        &mut self,
        api_usage: TokenUsage,
        estimated_usage: TokenUsage,
        model: &str,
    ) -> ValidationResult {
        let result = ValidationResult::new(
            api_usage,
            estimated_usage,
            model.to_string(),
            self.discrepancy_threshold,
        );

        if result.exceeds_threshold {
            warn!(
                model = model,
                api_total = result.api_usage.total_tokens,
                estimated_total = result.estimated_usage.total_tokens,
                discrepancy = result.discrepancy_percentage,
                "Token count discrepancy exceeds threshold"
            );
        }

        // Store validation result (with memory limit)
        self.validation_history.push(result.clone());
        if self.validation_history.len() > MAX_VALIDATION_RECORDS {
            self.validation_history.remove(0);
        }

        result
    }

    /// Get validation accuracy statistics
    pub fn get_accuracy_stats(&self) -> ValidationStats {
        if self.validation_history.is_empty() {
            return ValidationStats::default();
        }

        let total_validations = self.validation_history.len();
        let failed_validations = self
            .validation_history
            .iter()
            .filter(|r| r.exceeds_threshold)
            .count();

        let accuracy_percentage =
            ((total_validations - failed_validations) as f32 / total_validations as f32) * 100.0;

        let avg_discrepancy = self
            .validation_history
            .iter()
            .map(|r| r.discrepancy_percentage)
            .sum::<f32>()
            / total_validations as f32;

        ValidationStats {
            total_validations,
            failed_validations,
            accuracy_percentage,
            average_discrepancy: avg_discrepancy,
        }
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new(DEFAULT_DISCREPANCY_THRESHOLD)
    }
}

/// Validation accuracy statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Total number of validations performed
    pub total_validations: usize,
    /// Number of validations that failed threshold
    pub failed_validations: usize,
    /// Accuracy percentage (successful validations / total)
    pub accuracy_percentage: f32,
    /// Average discrepancy percentage
    pub average_discrepancy: f32,
}

impl Default for ValidationStats {
    fn default() -> Self {
        Self {
            total_validations: 0,
            failed_validations: 0,
            accuracy_percentage: 100.0,
            average_discrepancy: 0.0,
        }
    }
}

/// Main token counter interface
pub struct TokenCounter {
    /// API token extractor
    pub api_extractor: ApiTokenExtractor,
    /// Token validator
    pub validator: TokenValidator,
}

impl TokenCounter {
    /// Create a new token counter
    pub fn new(discrepancy_threshold: f32) -> Self {
        Self {
            api_extractor: ApiTokenExtractor,
            validator: TokenValidator::new(discrepancy_threshold),
        }
    }

    /// Count tokens from API response with optional validation
    pub fn count_from_response(
        &mut self,
        response_body: &str,
        estimated_usage: Option<TokenUsage>,
        model: &str,
    ) -> Result<TokenUsage, CostError> {
        let api_usage = self.api_extractor.extract_from_response(response_body)?;

        // Perform validation if estimation is provided
        if let Some(estimated) = estimated_usage {
            let _validation = self.validator.validate(api_usage.clone(), estimated, model);
        }

        info!(
            model = model,
            input_tokens = api_usage.input_tokens,
            output_tokens = api_usage.output_tokens,
            total_tokens = api_usage.total_tokens,
            source = ?api_usage.source,
            "Token count extracted from API response"
        );

        Ok(api_usage)
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> ValidationStats {
        self.validator.get_accuracy_stats()
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new(DEFAULT_DISCREPANCY_THRESHOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_creation() {
        let usage = TokenUsage::new(100, 200, TokenSource::ApiResponse, ConfidenceLevel::Exact);

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 200);
        assert_eq!(usage.total_tokens, 300);
        assert_eq!(usage.source, TokenSource::ApiResponse);
        assert_eq!(usage.confidence, ConfidenceLevel::Exact);
        assert!(usage.is_from_api());
        assert!(!usage.is_estimated());
    }

    #[test]
    fn test_token_usage_from_api() {
        let usage = TokenUsage::from_api(50, 75);

        assert_eq!(usage.input_tokens, 50);
        assert_eq!(usage.output_tokens, 75);
        assert_eq!(usage.total_tokens, 125);
        assert_eq!(usage.source, TokenSource::ApiResponse);
        assert_eq!(usage.confidence, ConfidenceLevel::Exact);
    }

    #[test]
    fn test_token_usage_from_estimation() {
        let usage = TokenUsage::from_estimation(60, 90, ConfidenceLevel::Medium);

        assert_eq!(usage.input_tokens, 60);
        assert_eq!(usage.output_tokens, 90);
        assert_eq!(usage.total_tokens, 150);
        assert_eq!(usage.source, TokenSource::Estimated);
        assert_eq!(usage.confidence, ConfidenceLevel::Medium);
        assert!(usage.is_estimated());
        assert!(!usage.is_from_api());
    }

    #[test]
    fn test_api_token_extractor_claude_format() {
        let response_json = r#"{
            "id": "msg_123",
            "content": [{"text": "Hello world"}],
            "usage": {
                "input_tokens": 150,
                "output_tokens": 25
            }
        }"#;

        let extractor = ApiTokenExtractor;
        let usage = extractor.extract_from_response(response_json).unwrap();
        assert_eq!(usage.input_tokens, 150);
        assert_eq!(usage.output_tokens, 25);
        assert_eq!(usage.total_tokens, 175);
        assert_eq!(usage.source, TokenSource::ApiResponse);
        assert_eq!(usage.confidence, ConfidenceLevel::Exact);
    }

    #[test]
    fn test_api_token_extractor_alternative_format() {
        let response_json = r#"{
            "input_token_count": 100,
            "output_token_count": 50,
            "response": "Test response"
        }"#;

        let extractor = ApiTokenExtractor;
        let usage = extractor.extract_from_response(response_json).unwrap();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_api_token_extractor_no_usage() {
        let response_json = r#"{
            "id": "msg_123",
            "content": [{"text": "Hello world"}]
        }"#;

        let extractor = ApiTokenExtractor;
        let result = extractor.extract_from_response(response_json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CostError::InvalidInput { .. }
        ));
    }

    #[test]
    fn test_api_token_extractor_headers() {
        let mut headers = HashMap::new();
        headers.insert("anthropic-input-tokens".to_string(), "200".to_string());
        headers.insert("anthropic-output-tokens".to_string(), "75".to_string());

        let extractor = ApiTokenExtractor;
        let usage = extractor.extract_from_headers(&headers).unwrap();
        assert_eq!(usage.input_tokens, 200);
        assert_eq!(usage.output_tokens, 75);
        assert_eq!(usage.total_tokens, 275);
    }

    #[test]
    fn test_api_token_extractor_headers_alternative() {
        let mut headers = HashMap::new();
        headers.insert("x-input-tokens".to_string(), "120".to_string());
        headers.insert("x-output-tokens".to_string(), "80".to_string());

        let extractor = ApiTokenExtractor;
        let usage = extractor.extract_from_headers(&headers).unwrap();
        assert_eq!(usage.input_tokens, 120);
        assert_eq!(usage.output_tokens, 80);
    }

    #[test]
    fn test_validation_result_discrepancy_calculation() {
        let api_usage = TokenUsage::from_api(100, 100);
        let estimated_usage = TokenUsage::from_estimation(110, 90, ConfidenceLevel::Medium);

        let result = ValidationResult::new(
            api_usage,
            estimated_usage,
            "claude-3-sonnet".to_string(),
            0.1,
        );

        // Expected: |200 - 200| / 200 * 100 = 0%
        assert_eq!(result.discrepancy_percentage, 0.0);
        assert!(!result.exceeds_threshold);
    }

    #[test]
    fn test_validation_result_high_discrepancy() {
        let api_usage = TokenUsage::from_api(100, 100);
        let estimated_usage = TokenUsage::from_estimation(150, 150, ConfidenceLevel::Low);

        let result = ValidationResult::new(
            api_usage,
            estimated_usage,
            "claude-3-sonnet".to_string(),
            0.1, // 10% threshold
        );

        // Expected: |200 - 300| / 200 * 100 = 50%
        assert_eq!(result.discrepancy_percentage, 50.0);
        assert!(result.exceeds_threshold);
    }

    #[test]
    fn test_token_validator_basic() {
        let mut validator = TokenValidator::new(0.1);

        let api_usage = TokenUsage::from_api(100, 50);
        let estimated_usage = TokenUsage::from_estimation(95, 55, ConfidenceLevel::High);

        let result = validator.validate(api_usage, estimated_usage, "claude-3-sonnet");

        // Discrepancy should be small
        assert!(result.discrepancy_percentage < 10.0);
        assert!(!result.exceeds_threshold);
    }

    #[test]
    fn test_token_validator_accuracy_stats() {
        let mut validator = TokenValidator::new(0.1);

        // Add some good validations
        for i in 0..8 {
            let api_usage = TokenUsage::from_api(100, 100);
            let estimated_usage =
                TokenUsage::from_estimation(95 + i, 105 - i, ConfidenceLevel::High);
            validator.validate(api_usage, estimated_usage, "claude-3-sonnet");
        }

        // Add some bad validations
        for i in 0..2 {
            let api_usage = TokenUsage::from_api(100, 100);
            let estimated_usage =
                TokenUsage::from_estimation(150 + i * 10, 150 + i * 10, ConfidenceLevel::Low);
            validator.validate(api_usage, estimated_usage, "claude-3-sonnet");
        }

        let stats = validator.get_accuracy_stats();
        assert_eq!(stats.total_validations, 10);
        assert_eq!(stats.failed_validations, 2);
        assert_eq!(stats.accuracy_percentage, 80.0);
        assert!(stats.average_discrepancy > 0.0);
    }

    #[test]
    fn test_token_counter_basic() {
        let mut counter = TokenCounter::default();

        let response_json = r#"{
            "usage": {
                "input_tokens": 150,
                "output_tokens": 75
            }
        }"#;

        let usage = counter
            .count_from_response(response_json, None, "claude-3-sonnet")
            .unwrap();

        assert_eq!(usage.input_tokens, 150);
        assert_eq!(usage.output_tokens, 75);
        assert_eq!(usage.total_tokens, 225);
        assert!(usage.is_from_api());
    }

    #[test]
    fn test_token_counter_with_validation() {
        let mut counter = TokenCounter::default();

        let response_json = r#"{
            "usage": {
                "input_tokens": 100,
                "output_tokens": 100
            }
        }"#;

        let estimated = TokenUsage::from_estimation(95, 105, ConfidenceLevel::High);
        let usage = counter
            .count_from_response(response_json, Some(estimated), "claude-3-sonnet")
            .unwrap();

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 100);

        // Check validation was performed
        let stats = counter.get_validation_stats();
        assert_eq!(stats.total_validations, 1);
        assert_eq!(stats.failed_validations, 0);
    }

    #[test]
    fn test_confidence_levels_ordering() {
        assert!(ConfidenceLevel::Exact > ConfidenceLevel::High);
        assert!(ConfidenceLevel::High > ConfidenceLevel::Medium);
        assert!(ConfidenceLevel::Medium > ConfidenceLevel::Low);
    }
}
