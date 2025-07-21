//! Performance monitoring and metrics collection
//!
//! This module provides real-time performance monitoring to ensure cost tracking
//! meets performance targets and detect regressions.

use crate::cost::performance::PerformanceError;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Performance monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Maximum number of measurements to keep
    pub max_measurements: usize,
    /// Measurement window duration
    pub measurement_window_secs: u64,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Enable percentile calculations
    pub enable_percentiles: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            max_measurements: 10000,
            measurement_window_secs: 3600, // 1 hour
            alert_thresholds: AlertThresholds::default(),
            enable_percentiles: true,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// API call overhead threshold in milliseconds
    pub api_overhead_ms: u64,
    /// Memory usage threshold as percentage of total memory
    pub memory_usage_pct: f32,
    /// Cache hit rate threshold percentage
    pub cache_hit_rate_pct: f32,
    /// Error rate threshold percentage
    pub error_rate_pct: f32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            api_overhead_ms: 50,
            memory_usage_pct: 5.0,
            cache_hit_rate_pct: 80.0,
            error_rate_pct: 1.0,
        }
    }
}

/// Individual performance measurement
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Timestamp of measurement
    pub timestamp: Instant,
    /// Duration of operation in microseconds
    pub duration_micros: u64,
    /// Operation type
    pub operation_type: OperationType,
    /// Success/failure status
    pub success: bool,
    /// Memory usage at time of measurement (if available)
    pub memory_usage_bytes: Option<usize>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Type of operation being measured
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// API call interception and tracking
    ApiCallTracking,
    /// Token counting operation
    TokenCounting,
    /// Storage operation
    StorageOperation,
    /// Aggregation calculation
    AggregationCalculation,
    /// Cache operation
    CacheOperation,
    /// Memory allocation/deallocation
    MemoryOperation,
}

/// Performance statistics for a specific operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    /// Number of measurements
    pub count: usize,
    /// Total duration in microseconds
    pub total_duration_micros: u64,
    /// Average duration in microseconds
    pub avg_duration_micros: f64,
    /// Minimum duration in microseconds
    pub min_duration_micros: u64,
    /// Maximum duration in microseconds
    pub max_duration_micros: u64,
    /// Success rate percentage
    pub success_rate_pct: f64,
    /// Percentiles (if enabled)
    pub percentiles: Option<Percentiles>,
}

/// Percentile measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

impl OperationStats {
    /// Check if operation meets performance target
    pub fn meets_target(&self, target_ms: u64) -> bool {
        self.avg_duration_micros <= (target_ms as f64 * 1000.0)
    }

    /// Check if percentiles meet target (if available)
    pub fn percentiles_meet_target(&self, target_ms: u64) -> bool {
        if let Some(ref p) = self.percentiles {
            let target_micros = target_ms as f64 * 1000.0;
            p.p95 <= target_micros && p.p99 <= (target_micros * 1.5) // Allow 50% variance for p99
        } else {
            true
        }
    }
}

/// Performance alert
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    /// Alert type
    pub alert_type: AlertType,
    /// Current value that triggered the alert
    pub current_value: f64,
    /// Threshold that was exceeded
    pub threshold: f64,
    /// When the alert was generated
    pub timestamp: Instant,
    /// Additional context
    pub context: String,
}

/// Type of performance alert
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertType {
    /// API overhead exceeded target
    ApiOverheadExceeded,
    /// Memory usage exceeded limit
    MemoryUsageExceeded,
    /// Cache hit rate below threshold
    CacheHitRateLow,
    /// Error rate above threshold
    ErrorRateHigh,
    /// Performance regression detected
    PerformanceRegression,
}

/// Real-time performance monitor
pub struct PerformanceMonitor {
    /// Configuration
    config: MonitoringConfig,
    /// Raw measurements
    measurements: Arc<RwLock<VecDeque<PerformanceMeasurement>>>,
    /// Computed statistics by operation type
    stats: Arc<RwLock<std::collections::HashMap<OperationType, OperationStats>>>,
    /// Recent alerts
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    /// Global performance metrics
    global_metrics: Arc<RwLock<GlobalPerformanceMetrics>>,
}

impl std::fmt::Debug for PerformanceMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceMonitor")
            .field("config", &self.config)
            .field("measurements_count", &self.measurements.read().unwrap().len())
            .field("stats_count", &self.stats.read().unwrap().len())
            .finish()
    }
}

/// Global performance metrics across all operations
#[derive(Debug, Clone, Default)]
pub struct GlobalPerformanceMetrics {
    /// Total operations monitored
    pub total_operations: usize,
    /// Total duration across all operations
    pub total_duration_micros: u64,
    /// Overall average duration
    pub overall_avg_duration_micros: f64,
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    /// Alert summary
    pub alert_summary: AlertSummary,
    /// Performance trend
    pub performance_trend: PerformanceTrend,
}

/// Memory usage statistics
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// Current memory usage in bytes
    pub current_usage_bytes: usize,
    /// Peak memory usage in bytes
    pub peak_usage_bytes: usize,
    /// Average memory usage in bytes
    pub avg_usage_bytes: f64,
    /// Memory usage as percentage of system memory
    pub usage_percentage: f32,
}

/// Alert summary statistics
#[derive(Debug, Clone, Default)]
pub struct AlertSummary {
    /// Total alerts generated
    pub total_alerts: usize,
    /// Alerts by type
    pub alerts_by_type: std::collections::HashMap<AlertType, usize>,
    /// Recent alert count (last hour)
    pub recent_alert_count: usize,
}

/// Performance trend analysis
#[derive(Debug, Clone, Default)]
pub struct PerformanceTrend {
    /// Is performance improving
    pub improving: bool,
    /// Trend direction (positive = getting slower, negative = getting faster)
    pub trend_direction: f64,
    /// Confidence in trend analysis (0-100%)
    pub confidence_pct: f64,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            measurements: Arc::new(RwLock::new(VecDeque::new())),
            stats: Arc::new(RwLock::new(std::collections::HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            global_metrics: Arc::new(RwLock::new(GlobalPerformanceMetrics::default())),
        }
    }

    /// Record a performance measurement
    pub fn record_measurement(&self, measurement: PerformanceMeasurement) {
        let mut measurements = self.measurements.write().unwrap();
        
        // Add new measurement
        measurements.push_back(measurement.clone());
        
        // Maintain window size
        if measurements.len() > self.config.max_measurements {
            measurements.pop_front();
        }
        
        // Clean up old measurements outside time window
        let cutoff_time = Instant::now() - Duration::from_secs(self.config.measurement_window_secs);
        while let Some(oldest) = measurements.front() {
            if oldest.timestamp < cutoff_time {
                measurements.pop_front();
            } else {
                break;
            }
        }
        
        drop(measurements);
        
        // Update statistics
        self.update_statistics();
        
        // Check for alerts
        self.check_alerts(&measurement);
    }

    /// Record API call overhead
    pub fn record_api_call_overhead(&self, duration: Duration, success: bool) {
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            duration_micros: duration.as_micros() as u64,
            operation_type: OperationType::ApiCallTracking,
            success,
            memory_usage_bytes: None,
            metadata: std::collections::HashMap::new(),
        };
        
        self.record_measurement(measurement);
    }

    /// Record token counting performance
    pub fn record_token_counting(&self, duration: Duration, success: bool, token_count: u32) {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("token_count".to_string(), token_count.to_string());
        
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            duration_micros: duration.as_micros() as u64,
            operation_type: OperationType::TokenCounting,
            success,
            memory_usage_bytes: None,
            metadata,
        };
        
        self.record_measurement(measurement);
    }

    /// Update statistics from current measurements
    fn update_statistics(&self) {
        let measurements = self.measurements.read().unwrap();
        let mut stats = self.stats.write().unwrap();
        let mut global = self.global_metrics.write().unwrap();
        
        // Clear existing stats
        stats.clear();
        
        // Group measurements by operation type
        let mut by_type: std::collections::HashMap<OperationType, Vec<&PerformanceMeasurement>> = 
            std::collections::HashMap::new();
        
        for measurement in measurements.iter() {
            by_type.entry(measurement.operation_type)
                .or_default()
                .push(measurement);
        }
        
        // Calculate stats for each operation type
        for (op_type, type_measurements) in by_type {
            let op_stats = self.calculate_operation_stats(&type_measurements);
            stats.insert(op_type, op_stats);
        }
        
        // Update global metrics
        global.total_operations = measurements.len();
        global.total_duration_micros = measurements.iter()
            .map(|m| m.duration_micros)
            .sum();
        global.overall_avg_duration_micros = if global.total_operations > 0 {
            global.total_duration_micros as f64 / global.total_operations as f64
        } else {
            0.0
        };
        
        // Update performance trend
        global.performance_trend = self.calculate_performance_trend(&measurements);
    }

    /// Calculate statistics for a specific operation type
    fn calculate_operation_stats(&self, measurements: &[&PerformanceMeasurement]) -> OperationStats {
        if measurements.is_empty() {
            return OperationStats {
                count: 0,
                total_duration_micros: 0,
                avg_duration_micros: 0.0,
                min_duration_micros: 0,
                max_duration_micros: 0,
                success_rate_pct: 0.0,
                percentiles: None,
            };
        }
        
        let count = measurements.len();
        let total_duration: u64 = measurements.iter().map(|m| m.duration_micros).sum();
        let avg_duration = total_duration as f64 / count as f64;
        
        let min_duration = measurements.iter()
            .map(|m| m.duration_micros)
            .min()
            .unwrap_or(0);
            
        let max_duration = measurements.iter()
            .map(|m| m.duration_micros)
            .max()
            .unwrap_or(0);
        
        let successful_count = measurements.iter()
            .filter(|m| m.success)
            .count();
        let success_rate = (successful_count as f64 / count as f64) * 100.0;
        
        let percentiles = if self.config.enable_percentiles {
            Some(self.calculate_percentiles(measurements))
        } else {
            None
        };
        
        OperationStats {
            count,
            total_duration_micros: total_duration,
            avg_duration_micros: avg_duration,
            min_duration_micros: min_duration,
            max_duration_micros: max_duration,
            success_rate_pct: success_rate,
            percentiles,
        }
    }

    /// Calculate percentiles for measurements
    fn calculate_percentiles(&self, measurements: &[&PerformanceMeasurement]) -> Percentiles {
        let mut durations: Vec<u64> = measurements.iter()
            .map(|m| m.duration_micros)
            .collect();
        durations.sort();
        
        let len = durations.len();
        let p50_idx = len / 2;
        let p90_idx = (len * 90) / 100;
        let p95_idx = (len * 95) / 100;
        let p99_idx = (len * 99) / 100;
        
        Percentiles {
            p50: durations.get(p50_idx).copied().unwrap_or(0) as f64,
            p90: durations.get(p90_idx).copied().unwrap_or(0) as f64,
            p95: durations.get(p95_idx).copied().unwrap_or(0) as f64,
            p99: durations.get(p99_idx).copied().unwrap_or(0) as f64,
        }
    }

    /// Calculate performance trend from measurements
    fn calculate_performance_trend(&self, measurements: &VecDeque<PerformanceMeasurement>) -> PerformanceTrend {
        if measurements.len() < 10 {
            return PerformanceTrend::default();
        }
        
        // Simple linear regression to detect trend
        // Take the most recent measurements, but keep them in chronological order for trend analysis
        let mut recent: Vec<f64> = measurements.iter()
            .rev()
            .take(100) // Last 100 measurements
            .map(|m| m.duration_micros as f64)
            .collect();
        
        // Reverse back to chronological order (oldest first) for proper trend calculation
        recent.reverse();
        
        let n = recent.len() as f64;
        let x_mean = (n - 1.0) / 2.0; // 0, 1, 2, ... n-1
        let y_mean = recent.iter().sum::<f64>() / n;
        
        let mut numerator = 0.0;
        let mut denominator = 0.0;
        
        for (i, &y) in recent.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }
        
        let slope = if denominator != 0.0 {
            numerator / denominator
        } else {
            0.0
        };
        
        let improving = slope < 0.0; // Negative slope means decreasing duration (improving)
        let confidence = (slope.abs() * 10.0).min(100.0); // Simple confidence measure
        
        PerformanceTrend {
            improving,
            trend_direction: slope,
            confidence_pct: confidence,
        }
    }

    /// Check for performance alerts
    fn check_alerts(&self, measurement: &PerformanceMeasurement) {
        let mut alerts = self.alerts.write().unwrap();
        
        // Check API overhead
        if measurement.operation_type == OperationType::ApiCallTracking {
            let duration_ms = measurement.duration_micros as f64 / 1000.0;
            if duration_ms > self.config.alert_thresholds.api_overhead_ms as f64 {
                let alert = PerformanceAlert {
                    alert_type: AlertType::ApiOverheadExceeded,
                    current_value: duration_ms,
                    threshold: self.config.alert_thresholds.api_overhead_ms as f64,
                    timestamp: Instant::now(),
                    context: "API call overhead exceeded threshold".to_string(),
                };
                alerts.push(alert);
            }
        }
        
        // Limit alert history
        if alerts.len() > 1000 {
            alerts.drain(0..500); // Remove oldest half
        }
    }

    /// Get statistics for a specific operation type
    pub fn get_operation_stats(&self, op_type: OperationType) -> Option<OperationStats> {
        self.stats.read().unwrap().get(&op_type).cloned()
    }

    /// Get all operation statistics
    pub fn get_all_stats(&self) -> std::collections::HashMap<OperationType, OperationStats> {
        self.stats.read().unwrap().clone()
    }

    /// Get global performance metrics
    pub fn get_global_metrics(&self) -> GlobalPerformanceMetrics {
        self.global_metrics.read().unwrap().clone()
    }

    /// Get recent alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().unwrap();
        alerts.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Check if performance targets are met
    pub fn check_performance_targets(&self) -> Result<(), PerformanceError> {
        let stats = self.stats.read().unwrap();
        
        // Check API call overhead
        if let Some(api_stats) = stats.get(&OperationType::ApiCallTracking) {
            let target_ms = self.config.alert_thresholds.api_overhead_ms;
            if !api_stats.meets_target(target_ms) {
                return Err(PerformanceError::TargetExceeded {
                    metric: "API call overhead".to_string(),
                    actual: (api_stats.avg_duration_micros / 1000.0) as u64,
                    target: target_ms,
                });
            }
        }
        
        Ok(())
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let stats = self.stats.read().unwrap();
        let global = self.global_metrics.read().unwrap();
        let alerts = self.alerts.read().unwrap();
        
        let mut report = String::new();
        report.push_str("Performance Monitor Report\n");
        report.push_str("========================\n\n");
        
        // Global metrics
        report.push_str(&format!("Total Operations: {}\n", global.total_operations));
        report.push_str(&format!("Overall Avg Duration: {:.2}μs\n", global.overall_avg_duration_micros));
        report.push_str(&format!("Performance Trend: {} (confidence: {:.1}%)\n", 
                                if global.performance_trend.improving { "Improving" } else { "Degrading" },
                                global.performance_trend.confidence_pct));
        report.push_str(&format!("Recent Alerts: {}\n\n", alerts.len()));
        
        // Operation-specific stats
        for (op_type, op_stats) in stats.iter() {
            report.push_str(&format!("Operation: {:?}\n", op_type));
            report.push_str(&format!("  Count: {}\n", op_stats.count));
            report.push_str(&format!("  Avg Duration: {:.2}μs\n", op_stats.avg_duration_micros));
            report.push_str(&format!("  Min/Max: {}/{}μs\n", op_stats.min_duration_micros, op_stats.max_duration_micros));
            report.push_str(&format!("  Success Rate: {:.1}%\n", op_stats.success_rate_pct));
            
            if let Some(ref p) = op_stats.percentiles {
                report.push_str(&format!("  P50/P90/P95/P99: {:.1}/{:.1}/{:.1}/{:.1}μs\n", 
                                       p.p50, p.p90, p.p95, p.p99));
            }
            
            report.push('\n');
        }
        
        report
    }
}

/// Performance metrics aggregator for collecting metrics from multiple sources
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Monitor instance
    monitor: PerformanceMonitor,
}

impl PerformanceMetrics {
    /// Create new performance metrics aggregator
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            monitor: PerformanceMonitor::new(config),
        }
    }
    
    /// Record API call performance
    pub fn record_api_call(&self, duration: Duration, success: bool) {
        self.monitor.record_api_call_overhead(duration, success);
    }
    
    /// Record token counting performance
    pub fn record_token_counting(&self, duration: Duration, success: bool, token_count: u32) {
        self.monitor.record_token_counting(duration, success, token_count);
    }
    
    /// Get performance report
    pub fn get_report(&self) -> String {
        self.monitor.generate_report()
    }
    
    /// Check if targets are met
    pub fn check_targets(&self) -> Result<(), PerformanceError> {
        self.monitor.check_performance_targets()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        
        assert_eq!(config.max_measurements, 10000);
        assert_eq!(config.measurement_window_secs, 3600);
        assert_eq!(config.alert_thresholds.api_overhead_ms, 50);
        assert!(config.enable_percentiles);
    }

    #[test]
    fn test_performance_measurement() {
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            duration_micros: 25000, // 25ms
            operation_type: OperationType::ApiCallTracking,
            success: true,
            memory_usage_bytes: Some(1024),
            metadata: std::collections::HashMap::new(),
        };
        
        assert_eq!(measurement.operation_type, OperationType::ApiCallTracking);
        assert!(measurement.success);
        assert_eq!(measurement.memory_usage_bytes, Some(1024));
    }

    #[test]
    fn test_operation_stats_targets() {
        let stats = OperationStats {
            count: 100,
            total_duration_micros: 2_500_000, // 2.5 seconds total
            avg_duration_micros: 25_000.0, // 25ms average
            min_duration_micros: 10_000,
            max_duration_micros: 50_000,
            success_rate_pct: 99.0,
            percentiles: None,
        };
        
        assert!(stats.meets_target(30)); // 25ms < 30ms target
        assert!(!stats.meets_target(20)); // 25ms > 20ms target
    }

    #[test]
    fn test_performance_monitor() {
        let config = MonitoringConfig {
            max_measurements: 100,
            ..Default::default()
        };
        
        let monitor = PerformanceMonitor::new(config);
        
        // Record some measurements
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            duration_micros: 30_000, // 30ms
            operation_type: OperationType::ApiCallTracking,
            success: true,
            memory_usage_bytes: None,
            metadata: std::collections::HashMap::new(),
        };
        
        monitor.record_measurement(measurement);
        
        // Check stats
        let stats = monitor.get_operation_stats(OperationType::ApiCallTracking);
        assert!(stats.is_some());
        
        let api_stats = stats.unwrap();
        assert_eq!(api_stats.count, 1);
        assert_eq!(api_stats.avg_duration_micros, 30_000.0);
        assert_eq!(api_stats.success_rate_pct, 100.0);
    }

    #[test]
    fn test_performance_monitor_alerts() {
        let config = MonitoringConfig {
            alert_thresholds: AlertThresholds {
                api_overhead_ms: 25, // Low threshold for testing
                ..Default::default()
            },
            ..Default::default()
        };
        
        let monitor = PerformanceMonitor::new(config);
        
        // Record measurement that exceeds threshold
        let measurement = PerformanceMeasurement {
            timestamp: Instant::now(),
            duration_micros: 30_000, // 30ms > 25ms threshold
            operation_type: OperationType::ApiCallTracking,
            success: true,
            memory_usage_bytes: None,
            metadata: std::collections::HashMap::new(),
        };
        
        monitor.record_measurement(measurement);
        
        // Should have generated alert
        let alerts = monitor.get_recent_alerts(10);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].alert_type, AlertType::ApiOverheadExceeded);
    }

    #[test]
    fn test_percentiles_calculation() {
        let config = MonitoringConfig {
            enable_percentiles: true,
            ..Default::default()
        };
        
        let monitor = PerformanceMonitor::new(config);
        
        // Record multiple measurements with different durations
        let durations = [10_000, 20_000, 30_000, 40_000, 50_000]; // 10ms to 50ms
        
        for duration in durations.iter() {
            let measurement = PerformanceMeasurement {
                timestamp: Instant::now(),
                duration_micros: *duration,
                operation_type: OperationType::TokenCounting,
                success: true,
                memory_usage_bytes: None,
                metadata: std::collections::HashMap::new(),
            };
            
            monitor.record_measurement(measurement);
            std::thread::sleep(Duration::from_millis(1)); // Ensure different timestamps
        }
        
        let stats = monitor.get_operation_stats(OperationType::TokenCounting).unwrap();
        assert!(stats.percentiles.is_some());
        
        let percentiles = stats.percentiles.unwrap();
        assert_eq!(percentiles.p50, 30_000.0); // Median should be middle value
    }

    #[test]
    fn test_performance_metrics_aggregator() {
        let config = MonitoringConfig::default();
        let metrics = PerformanceMetrics::new(config);
        
        // Record some measurements
        metrics.record_api_call(Duration::from_millis(20), true);
        metrics.record_token_counting(Duration::from_millis(5), true, 150);
        
        // Generate report
        let report = metrics.get_report();
        assert!(report.contains("Performance Monitor Report"));
        assert!(report.contains("ApiCallTracking"));
        assert!(report.contains("TokenCounting"));
    }

    #[test]
    fn test_performance_trend_calculation() {
        let config = MonitoringConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Record measurements with improving trend (decreasing duration)
        for i in 0..20 {
            let duration = 50_000 - (i * 1000); // 50ms down to 31ms
            let measurement = PerformanceMeasurement {
                timestamp: Instant::now(),
                duration_micros: duration as u64,
                operation_type: OperationType::ApiCallTracking,
                success: true,
                memory_usage_bytes: None,
                metadata: std::collections::HashMap::new(),
            };
            
            monitor.record_measurement(measurement);
            std::thread::sleep(Duration::from_millis(1));
        }
        
        let global_metrics = monitor.get_global_metrics();
        assert!(global_metrics.performance_trend.improving);
        assert!(global_metrics.performance_trend.trend_direction < 0.0);
    }
}