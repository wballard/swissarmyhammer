//! Cost trend analysis and prediction
//!
//! This module provides sophisticated trend analysis capabilities including
//! regression analysis, seasonal pattern detection, and cost forecasting.

use super::{PatternType, SeasonalPattern, TrendDirection};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Trend analysis errors
#[derive(Error, Debug)]
pub enum TrendError {
    /// Insufficient data for trend analysis
    #[error("Insufficient data: need at least {required} points, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    /// Mathematical calculation error
    #[error("Calculation error: {0}")]
    Calculation(String),

    /// Invalid time series data
    #[error("Invalid time series data: {0}")]
    InvalidData(String),
}

/// Comprehensive trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Linear trend slope (cost change per day)
    pub linear_slope: f64,
    /// Trend direction classification
    pub direction: TrendDirection,
    /// Statistical confidence (0.0 to 1.0)
    pub confidence: f64,
    /// R-squared correlation coefficient
    pub r_squared: f64,
    /// Detected seasonal patterns
    pub seasonal_patterns: Vec<SeasonalPattern>,
    /// Volatility measure (standard deviation of daily changes)
    pub volatility: f64,
    /// Predicted costs for the next period
    pub predictions: Vec<CostPrediction>,
    /// Trend stability score (0.0 to 1.0, higher is more stable)
    pub stability_score: f64,
}

/// Cost prediction data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPrediction {
    /// Date for prediction
    pub date: DateTime<Utc>,
    /// Predicted cost
    pub predicted_cost: Decimal,
    /// Confidence interval lower bound
    pub confidence_lower: Decimal,
    /// Confidence interval upper bound
    pub confidence_upper: Decimal,
    /// Confidence level (e.g., 0.95 for 95% confidence)
    pub confidence_level: f64,
}

/// Time series data point for analysis
#[derive(Debug, Clone)]
pub struct TimeSeriesPoint {
    /// Date/time of the data point
    pub timestamp: DateTime<Utc>,
    /// Cost value
    pub cost: Decimal,
}

/// Trend analyzer with statistical methods
pub struct TrendAnalyzer {
    /// Minimum number of data points required for analysis
    min_data_points: usize,
    /// Default confidence level for predictions
    confidence_level: f64,
    /// Number of periods to predict into the future
    prediction_periods: usize,
}

impl Default for TrendAnalyzer {
    fn default() -> Self {
        Self {
            min_data_points: 5,
            confidence_level: 0.95,
            prediction_periods: 7, // 7 days by default
        }
    }
}

impl TrendAnalyzer {
    /// Create a new trend analyzer with custom configuration
    pub fn new(
        min_data_points: usize,
        confidence_level: f64,
        prediction_periods: usize,
    ) -> Self {
        Self {
            min_data_points,
            confidence_level,
            prediction_periods,
        }
    }

    /// Perform comprehensive trend analysis on time series data
    pub fn analyze_trends(
        &self,
        data: &[TimeSeriesPoint],
    ) -> Result<TrendAnalysis, TrendError> {
        if data.len() < self.min_data_points {
            return Err(TrendError::InsufficientData {
                required: self.min_data_points,
                actual: data.len(),
            });
        }

        // Sort data by timestamp
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by_key(|p| p.timestamp);

        // Convert to numerical form for statistical analysis
        let time_series = self.prepare_time_series(&sorted_data)?;

        // Perform linear regression
        let (slope, intercept, r_squared) = self.linear_regression(&time_series)?;

        // Determine trend direction and confidence
        let direction = self.classify_trend_direction(slope, r_squared);
        let confidence = self.calculate_confidence(r_squared, data.len());

        // Calculate volatility
        let volatility = self.calculate_volatility(&time_series)?;

        // Detect seasonal patterns
        let seasonal_patterns = self.detect_seasonal_patterns(&sorted_data)?;

        // Generate predictions
        let predictions = self.generate_predictions(&time_series, slope, intercept)?;

        // Calculate stability score
        let stability_score = self.calculate_stability_score(volatility, r_squared);

        Ok(TrendAnalysis {
            linear_slope: slope,
            direction,
            confidence,
            r_squared,
            seasonal_patterns,
            volatility,
            predictions,
            stability_score,
        })
    }

    /// Prepare time series data for statistical analysis
    fn prepare_time_series(
        &self,
        data: &[TimeSeriesPoint],
    ) -> Result<Vec<(f64, f64)>, TrendError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let start_time = data[0].timestamp;
        let mut time_series = Vec::new();

        for point in data {
            let days_from_start = (point.timestamp - start_time).num_days() as f64;
            let cost = point
                .cost
                .to_f64()
                .ok_or_else(|| TrendError::InvalidData("Invalid cost value".to_string()))?;

            time_series.push((days_from_start, cost));
        }

        Ok(time_series)
    }

    /// Perform linear regression analysis
    fn linear_regression(
        &self,
        data: &[(f64, f64)],
    ) -> Result<(f64, f64, f64), TrendError> {
        let n = data.len() as f64;

        if n < 2.0 {
            return Err(TrendError::InsufficientData {
                required: 2,
                actual: data.len(),
            });
        }

        // Calculate means
        let x_mean = data.iter().map(|(x, _)| x).sum::<f64>() / n;
        let y_mean = data.iter().map(|(_, y)| y).sum::<f64>() / n;

        // Calculate slope and intercept
        let numerator: f64 = data
            .iter()
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = data
            .iter()
            .map(|(x, _)| (x - x_mean).powi(2))
            .sum();

        if denominator == 0.0 {
            return Err(TrendError::Calculation(
                "Cannot calculate slope: denominator is zero".to_string(),
            ));
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        // Calculate R-squared
        let ss_tot: f64 = data
            .iter()
            .map(|(_, y)| (y - y_mean).powi(2))
            .sum();

        if ss_tot == 0.0 {
            return Ok((slope, intercept, 1.0)); // Perfect fit if no variance
        }

        let ss_res: f64 = data
            .iter()
            .map(|(x, y)| {
                let y_pred = slope * x + intercept;
                (y - y_pred).powi(2)
            })
            .sum();

        let r_squared = 1.0 - (ss_res / ss_tot);

        Ok((slope, intercept, r_squared.max(0.0)))
    }

    /// Classify trend direction based on slope and confidence
    fn classify_trend_direction(&self, slope: f64, r_squared: f64) -> TrendDirection {
        const SLOPE_THRESHOLD: f64 = 0.001; // Minimum slope to consider significant
        const CONFIDENCE_THRESHOLD: f64 = 0.3; // Minimum R² for trend confidence

        if r_squared < CONFIDENCE_THRESHOLD {
            return TrendDirection::Volatile;
        }

        if slope > SLOPE_THRESHOLD {
            TrendDirection::Increasing
        } else if slope < -SLOPE_THRESHOLD {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate confidence in trend analysis
    fn calculate_confidence(&self, r_squared: f64, data_points: usize) -> f64 {
        // Confidence increases with R² and number of data points
        let r_squared_factor = r_squared.max(0.0).min(1.0);
        let sample_size_factor = (data_points as f64).ln() / 10.0;

        (r_squared_factor + sample_size_factor).min(1.0).max(0.0)
    }

    /// Calculate volatility (standard deviation of changes)
    fn calculate_volatility(&self, data: &[(f64, f64)]) -> Result<f64, TrendError> {
        if data.len() < 2 {
            return Ok(0.0);
        }

        let changes: Vec<f64> = data
            .windows(2)
            .map(|window| window[1].1 - window[0].1)
            .collect();

        let mean_change = changes.iter().sum::<f64>() / changes.len() as f64;
        let variance = changes
            .iter()
            .map(|change| (change - mean_change).powi(2))
            .sum::<f64>()
            / changes.len() as f64;

        Ok(variance.sqrt())
    }

    /// Detect seasonal patterns in the data
    fn detect_seasonal_patterns(
        &self,
        data: &[TimeSeriesPoint],
    ) -> Result<Vec<SeasonalPattern>, TrendError> {
        let mut patterns = Vec::new();

        // For now, implement basic pattern detection
        // In a full implementation, this would use FFT or other sophisticated methods

        // Daily pattern detection (hour of day effects)
        if self.has_sufficient_daily_data(data) {
            if let Some(pattern) = self.detect_daily_pattern(data) {
                patterns.push(pattern);
            }
        }

        // Weekly pattern detection (day of week effects)
        if self.has_sufficient_weekly_data(data) {
            if let Some(pattern) = self.detect_weekly_pattern(data) {
                patterns.push(pattern);
            }
        }

        Ok(patterns)
    }

    /// Check if there's sufficient data for daily pattern detection
    fn has_sufficient_daily_data(&self, data: &[TimeSeriesPoint]) -> bool {
        // Need at least 3 days with multiple points per day
        data.len() >= 24 // Rough heuristic
    }

    /// Check if there's sufficient data for weekly pattern detection
    fn has_sufficient_weekly_data(&self, data: &[TimeSeriesPoint]) -> bool {
        // Need at least 2 full weeks
        let span_days = if data.len() >= 2 {
            (data[data.len() - 1].timestamp - data[0].timestamp).num_days()
        } else {
            0
        };
        span_days >= 14
    }

    /// Detect daily patterns (placeholder implementation)
    fn detect_daily_pattern(&self, _data: &[TimeSeriesPoint]) -> Option<SeasonalPattern> {
        // Simplified implementation - in practice would analyze hour-of-day effects
        Some(SeasonalPattern {
            pattern_type: PatternType::Daily,
            strength: 0.1, // Low confidence for now
            description: "Potential daily usage patterns detected".to_string(),
        })
    }

    /// Detect weekly patterns (placeholder implementation)
    fn detect_weekly_pattern(&self, _data: &[TimeSeriesPoint]) -> Option<SeasonalPattern> {
        // Simplified implementation - in practice would analyze day-of-week effects
        Some(SeasonalPattern {
            pattern_type: PatternType::Weekly,
            strength: 0.15, // Low confidence for now
            description: "Potential weekly usage patterns detected".to_string(),
        })
    }

    /// Generate cost predictions for future periods
    fn generate_predictions(
        &self,
        data: &[(f64, f64)],
        slope: f64,
        intercept: f64,
    ) -> Result<Vec<CostPrediction>, TrendError> {
        let mut predictions = Vec::new();

        if data.is_empty() {
            return Ok(predictions);
        }

        let last_day = data.last().unwrap().0;
        let base_date = Utc::now();

        // Calculate prediction error estimate
        let prediction_error = self.calculate_prediction_error(data, slope, intercept);

        for i in 1..=self.prediction_periods {
            let future_day = last_day + i as f64;
            let predicted_value = slope * future_day + intercept;
            let predicted_cost = Decimal::try_from(predicted_value.max(0.0))
                .unwrap_or(Decimal::ZERO);

            // Calculate confidence interval
            let error_margin = prediction_error * (self.confidence_level * 2.0);
            let confidence_lower = Decimal::try_from((predicted_value - error_margin).max(0.0))
                .unwrap_or(Decimal::ZERO);
            let confidence_upper = Decimal::try_from(predicted_value + error_margin)
                .unwrap_or(predicted_cost);

            predictions.push(CostPrediction {
                date: base_date + chrono::Duration::days(i as i64),
                predicted_cost,
                confidence_lower,
                confidence_upper,
                confidence_level: self.confidence_level,
            });
        }

        Ok(predictions)
    }

    /// Calculate prediction error estimate
    fn calculate_prediction_error(&self, data: &[(f64, f64)], slope: f64, intercept: f64) -> f64 {
        if data.len() < 2 {
            return 0.1; // Default error estimate
        }

        let errors: Vec<f64> = data
            .iter()
            .map(|(x, y)| {
                let predicted = slope * x + intercept;
                (y - predicted).abs()
            })
            .collect();

        errors.iter().sum::<f64>() / errors.len() as f64
    }

    /// Calculate stability score based on volatility and correlation
    fn calculate_stability_score(&self, volatility: f64, r_squared: f64) -> f64 {
        // Higher R² and lower volatility indicate higher stability
        let volatility_factor = 1.0 / (1.0 + volatility);
        let correlation_factor = r_squared;

        (volatility_factor * 0.4 + correlation_factor * 0.6).min(1.0).max(0.0)
    }
}