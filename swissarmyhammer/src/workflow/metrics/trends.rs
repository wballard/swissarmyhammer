//! Resource trend tracking for workflow execution

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Maximum number of data points to keep in resource trends
pub const MAX_TREND_DATA_POINTS: usize = 100;

/// Resource trend tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrends {
    /// Memory usage trend (bytes over time)
    pub memory_trend: Vec<(DateTime<Utc>, u64)>,
    /// CPU usage trend (percentage over time)
    pub cpu_trend: Vec<(DateTime<Utc>, f64)>,
    /// Throughput trend (runs per hour)
    pub throughput_trend: Vec<(DateTime<Utc>, f64)>,
    /// Cost trend (total cost over time)
    pub cost_trend: Vec<(DateTime<Utc>, Decimal)>,
    /// Token efficiency trend (output/input ratio over time)
    pub token_efficiency_trend: Vec<(DateTime<Utc>, f64)>,
    /// Average cost per API call trend
    pub avg_cost_per_call_trend: Vec<(DateTime<Utc>, Decimal)>,
}

impl ResourceTrends {
    /// Create new resource trends tracker
    pub fn new() -> Self {
        Self {
            memory_trend: Vec::new(),
            cpu_trend: Vec::new(),
            throughput_trend: Vec::new(),
            cost_trend: Vec::new(),
            token_efficiency_trend: Vec::new(),
            avg_cost_per_call_trend: Vec::new(),
        }
    }

    /// Generic method to add data point to trend
    fn add_trend_point<T>(trend: &mut Vec<(DateTime<Utc>, T)>, value: T) {
        trend.push((Utc::now(), value));
        if trend.len() > MAX_TREND_DATA_POINTS {
            trend.remove(0);
        }
    }

    /// Add memory usage data point
    pub fn add_memory_point(&mut self, memory_bytes: u64) {
        Self::add_trend_point(&mut self.memory_trend, memory_bytes);
    }

    /// Add CPU usage data point
    pub fn add_cpu_point(&mut self, cpu_percentage: f64) {
        Self::add_trend_point(&mut self.cpu_trend, cpu_percentage);
    }

    /// Add throughput data point
    pub fn add_throughput_point(&mut self, runs_per_hour: f64) {
        Self::add_trend_point(&mut self.throughput_trend, runs_per_hour);
    }

    /// Add cost data point
    pub fn add_cost_point(&mut self, total_cost: Decimal) {
        Self::add_trend_point(&mut self.cost_trend, total_cost);
    }

    /// Add token efficiency data point
    pub fn add_token_efficiency_point(&mut self, efficiency_ratio: f64) {
        Self::add_trend_point(&mut self.token_efficiency_trend, efficiency_ratio);
    }

    /// Add average cost per call data point
    pub fn add_avg_cost_per_call_point(&mut self, avg_cost: Decimal) {
        Self::add_trend_point(&mut self.avg_cost_per_call_trend, avg_cost);
    }
}

impl Default for ResourceTrends {
    fn default() -> Self {
        Self::new()
    }
}
