//! Advanced cost analytics queries

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use sqlx::{FromRow, Row};
use std::collections::HashMap;
use thiserror::Error;

/// Query errors specific to cost analytics
#[derive(Error, Debug)]
pub enum QueryError {
    /// Database query error
    #[error("Query error: {0}")]
    Database(#[from] sqlx::Error),

    /// Invalid query parameters
    #[error("Invalid query parameters: {message}")]
    InvalidParameters {
        /// Detailed error message describing invalid parameters
        message: String,
    },

    /// No data found for query
    #[error("No data found for the specified query parameters")]
    NoDataFound,
}

/// Time period for cost analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePeriod {
    /// Last 24 hours
    Day,
    /// Last 7 days
    Week,
    /// Last 30 days
    Month,
    /// Last 90 days
    Quarter,
    /// Last 365 days
    Year,
    /// Custom date range
    Custom,
}

impl TimePeriod {
    /// Get the duration in days for this time period
    pub fn duration_days(&self) -> Option<i64> {
        match self {
            TimePeriod::Day => Some(1),
            TimePeriod::Week => Some(7),
            TimePeriod::Month => Some(30),
            TimePeriod::Quarter => Some(90),
            TimePeriod::Year => Some(365),
            TimePeriod::Custom => None,
        }
    }

    /// Get the start date for this time period from now
    pub fn start_date(&self) -> Option<DateTime<Utc>> {
        self.duration_days()
            .map(|days| Utc::now() - chrono::Duration::days(days))
    }
}

/// Parameters for cost trend queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendQuery {
    /// Time period for analysis
    pub period: TimePeriod,
    /// Custom start date (required if period is Custom)
    pub start_date: Option<DateTime<Utc>>,
    /// Custom end date (optional, defaults to now if period is Custom)
    pub end_date: Option<DateTime<Utc>>,
    /// Group results by this interval (in hours)
    pub group_by_hours: Option<u32>,
    /// Filter by specific issue IDs
    pub issue_ids: Option<Vec<String>>,
    /// Filter by pricing model
    pub pricing_model: Option<String>,
}

impl TrendQuery {
    /// Create a new trend query for a specific time period
    pub fn new(period: TimePeriod) -> Self {
        Self {
            period,
            start_date: None,
            end_date: None,
            group_by_hours: None,
            issue_ids: None,
            pricing_model: None,
        }
    }

    /// Create a custom trend query with date range
    pub fn custom(start_date: DateTime<Utc>, end_date: Option<DateTime<Utc>>) -> Self {
        Self {
            period: TimePeriod::Custom,
            start_date: Some(start_date),
            end_date,
            group_by_hours: None,
            issue_ids: None,
            pricing_model: None,
        }
    }

    /// Add issue ID filter
    pub fn with_issue_ids(mut self, issue_ids: Vec<String>) -> Self {
        self.issue_ids = Some(issue_ids);
        self
    }

    /// Add pricing model filter
    pub fn with_pricing_model(mut self, pricing_model: String) -> Self {
        self.pricing_model = Some(pricing_model);
        self
    }

    /// Group results by hour intervals
    pub fn group_by_hours(mut self, hours: u32) -> Self {
        self.group_by_hours = Some(hours);
        self
    }

    /// Validate the query parameters
    pub fn validate(&self) -> Result<(), QueryError> {
        match self.period {
            TimePeriod::Custom => {
                if self.start_date.is_none() {
                    return Err(QueryError::InvalidParameters {
                        message: "start_date is required for custom time period".to_string(),
                    });
                }

                if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
                    if start >= end {
                        return Err(QueryError::InvalidParameters {
                            message: "start_date must be before end_date".to_string(),
                        });
                    }
                }
            }
            _ => {
                if self.start_date.is_some() || self.end_date.is_some() {
                    return Err(QueryError::InvalidParameters {
                        message: "start_date and end_date should only be set for custom period"
                            .to_string(),
                    });
                }
            }
        }

        if let Some(hours) = self.group_by_hours {
            if hours == 0 || hours > 8760 {
                // Max 1 year in hours
                return Err(QueryError::InvalidParameters {
                    message: "group_by_hours must be between 1 and 8760".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get the effective start and end dates for this query
    pub fn get_date_range(&self) -> Result<(DateTime<Utc>, DateTime<Utc>), QueryError> {
        match self.period {
            TimePeriod::Custom => {
                let start = self
                    .start_date
                    .ok_or_else(|| QueryError::InvalidParameters {
                        message: "start_date is required for custom period".to_string(),
                    })?;
                let end = self.end_date.unwrap_or_else(Utc::now);
                Ok((start, end))
            }
            _ => {
                let start =
                    self.period
                        .start_date()
                        .ok_or_else(|| QueryError::InvalidParameters {
                            message: "Invalid time period".to_string(),
                        })?;
                let end = Utc::now();
                Ok((start, end))
            }
        }
    }
}

/// Cost trend data point
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CostTrend {
    /// Time bucket for this data point
    pub time_bucket: DateTime<Utc>,
    /// Total cost for this time period (stored as f64, converted from Decimal)
    pub total_cost: f64,
    /// Total number of API calls
    pub total_calls: i64,
    /// Total input tokens
    pub total_input_tokens: i64,
    /// Total output tokens
    pub total_output_tokens: i64,
    /// Number of sessions
    pub session_count: i64,
    /// Average cost per call (stored as f64, converted from Decimal)
    pub avg_cost_per_call: f64,
    /// Average tokens per call
    pub avg_tokens_per_call: i64,
}

impl CostTrend {
    /// Get total cost as Decimal
    pub fn total_cost_decimal(&self) -> Decimal {
        Decimal::try_from(self.total_cost).unwrap_or(Decimal::ZERO)
    }

    /// Get average cost per call as Decimal
    pub fn avg_cost_per_call_decimal(&self) -> Decimal {
        Decimal::try_from(self.avg_cost_per_call).unwrap_or(Decimal::ZERO)
    }
}

/// Issue cost summary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IssueCostSummary {
    /// Issue ID
    pub issue_id: String,
    /// Total cost for this issue (stored as f64)
    pub total_cost: f64,
    /// Total number of sessions
    pub session_count: i64,
    /// Total number of API calls
    pub total_calls: i64,
    /// Total input tokens
    pub total_input_tokens: i64,
    /// Total output tokens
    pub total_output_tokens: i64,
    /// Average cost per session (stored as f64)
    pub avg_cost_per_session: f64,
    /// First session date
    pub first_session: DateTime<Utc>,
    /// Last session date
    pub last_session: DateTime<Utc>,
}

impl IssueCostSummary {
    /// Get total cost as Decimal
    pub fn total_cost_decimal(&self) -> Decimal {
        Decimal::try_from(self.total_cost).unwrap_or(Decimal::ZERO)
    }

    /// Get average cost per session as Decimal
    pub fn avg_cost_per_session_decimal(&self) -> Decimal {
        Decimal::try_from(self.avg_cost_per_session).unwrap_or(Decimal::ZERO)
    }
}

/// Model usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelUsage {
    /// Model name
    pub model: String,
    /// Total cost for this model (stored as f64)
    pub total_cost: f64,
    /// Total number of calls
    pub call_count: i64,
    /// Total input tokens
    pub total_input_tokens: i64,
    /// Total output tokens
    pub total_output_tokens: i64,
    /// Average cost per call (stored as f64)
    pub avg_cost_per_call: f64,
    /// Usage percentage
    pub usage_percentage: f64,
}

impl ModelUsage {
    /// Get total cost as Decimal
    pub fn total_cost_decimal(&self) -> Decimal {
        Decimal::try_from(self.total_cost).unwrap_or(Decimal::ZERO)
    }

    /// Get average cost per call as Decimal
    pub fn avg_cost_per_call_decimal(&self) -> Decimal {
        Decimal::try_from(self.avg_cost_per_call).unwrap_or(Decimal::ZERO)
    }
}

/// Cost analytics interface
pub struct CostAnalytics<'a> {
    pool: &'a SqlitePool,
}

impl<'a> CostAnalytics<'a> {
    /// Create a new cost analytics instance
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    /// Get cost trends over time
    ///
    /// Returns cost data aggregated by time periods according to the query parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Trend query parameters
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swissarmyhammer::cost::database::{CostAnalytics, TrendQuery, TimePeriod};
    ///
    /// # tokio_test::block_on(async {
    /// # let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    /// let analytics = CostAnalytics::new(&pool);
    /// let query = TrendQuery::new(TimePeriod::Week).group_by_hours(24);
    ///
    /// let trends = analytics.get_cost_trends(&query).await?;
    /// for trend in trends {
    ///     println!("Date: {}, Cost: ${}", trend.time_bucket, trend.total_cost);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn get_cost_trends(&self, query: &TrendQuery) -> Result<Vec<CostTrend>, QueryError> {
        query.validate()?;
        let (start_date, end_date) = query.get_date_range()?;

        // Determine grouping interval
        let group_hours = query.group_by_hours.unwrap_or(24);
        let interval_minutes = group_hours * 60;

        let mut sql = format!(
            r#"
            SELECT 
                datetime(
                    (strftime('%s', started_at) / ({} * 60)) * ({} * 60), 
                    'unixepoch'
                ) as time_bucket,
                COALESCE(SUM(total_cost), 0) as total_cost,
                COALESCE(SUM(total_calls), 0) as total_calls,
                COALESCE(SUM(total_input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(total_output_tokens), 0) as total_output_tokens,
                COUNT(*) as session_count,
                CASE 
                    WHEN SUM(total_calls) > 0 THEN COALESCE(SUM(total_cost), 0) / SUM(total_calls)
                    ELSE 0
                END as avg_cost_per_call,
                CASE 
                    WHEN SUM(total_calls) > 0 THEN (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0)) / SUM(total_calls)
                    ELSE 0
                END as avg_tokens_per_call
            FROM cost_sessions 
            WHERE started_at >= ? AND started_at <= ?
            "#,
            interval_minutes, interval_minutes
        );

        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send>> =
            vec![Box::new(start_date), Box::new(end_date)];

        // Add issue ID filter
        if let Some(ref issue_ids) = query.issue_ids {
            if !issue_ids.is_empty() {
                let placeholders = issue_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                sql.push_str(&format!(" AND issue_id IN ({})", placeholders));
                for issue_id in issue_ids {
                    params.push(Box::new(issue_id.clone()));
                }
            }
        }

        // Add pricing model filter
        if let Some(ref pricing_model) = query.pricing_model {
            sql.push_str(" AND pricing_model = ?");
            params.push(Box::new(pricing_model.clone()));
        }

        sql.push_str(" GROUP BY time_bucket ORDER BY time_bucket");

        // Execute query using raw SQL since we have dynamic parameters
        let mut query_builder = sqlx::query_as::<_, CostTrend>(&sql);

        // Bind parameters
        query_builder = query_builder.bind(start_date).bind(end_date);

        if let Some(ref issue_ids) = query.issue_ids {
            for issue_id in issue_ids {
                query_builder = query_builder.bind(issue_id);
            }
        }

        if let Some(ref pricing_model) = query.pricing_model {
            query_builder = query_builder.bind(pricing_model);
        }

        let trends = query_builder.fetch_all(self.pool).await?;

        if trends.is_empty() {
            return Err(QueryError::NoDataFound);
        }

        Ok(trends)
    }

    /// Get cost summary by issue
    ///
    /// Returns aggregated cost data for each issue in the specified time period.
    pub async fn get_issue_cost_summary(
        &self,
        period: TimePeriod,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<IssueCostSummary>, QueryError> {
        let query = if period == TimePeriod::Custom {
            TrendQuery::custom(
                start_date.ok_or_else(|| QueryError::InvalidParameters {
                    message: "start_date required for custom period".to_string(),
                })?,
                end_date,
            )
        } else {
            TrendQuery::new(period)
        };

        let (start_date, end_date) = query.get_date_range()?;

        let summaries = sqlx::query_as::<_, IssueCostSummary>(
            r#"
            SELECT 
                issue_id,
                COALESCE(SUM(total_cost), 0) as total_cost,
                COUNT(*) as session_count,
                COALESCE(SUM(total_calls), 0) as total_calls,
                COALESCE(SUM(total_input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(total_output_tokens), 0) as total_output_tokens,
                CASE 
                    WHEN COUNT(*) > 0 THEN COALESCE(SUM(total_cost), 0) / COUNT(*)
                    ELSE 0
                END as avg_cost_per_session,
                MIN(started_at) as first_session,
                MAX(started_at) as last_session
            FROM cost_sessions 
            WHERE started_at >= ? AND started_at <= ?
            GROUP BY issue_id
            ORDER BY total_cost DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool)
        .await?;

        if summaries.is_empty() {
            return Err(QueryError::NoDataFound);
        }

        Ok(summaries)
    }

    /// Get model usage statistics
    ///
    /// Returns usage statistics for each model in the specified time period.
    pub async fn get_model_usage(
        &self,
        period: TimePeriod,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<ModelUsage>, QueryError> {
        let query = if period == TimePeriod::Custom {
            TrendQuery::custom(
                start_date.ok_or_else(|| QueryError::InvalidParameters {
                    message: "start_date required for custom period".to_string(),
                })?,
                end_date,
            )
        } else {
            TrendQuery::new(period)
        };

        let (start_date, end_date) = query.get_date_range()?;

        // First get total call count for percentage calculation
        let total_calls: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(ac.input_tokens + ac.output_tokens), 0)
            FROM api_calls ac
            JOIN cost_sessions cs ON ac.session_id = cs.id
            WHERE cs.started_at >= ? AND cs.started_at <= ?
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(self.pool)
        .await?;

        let usage_stats = sqlx::query(
            r#"
            SELECT 
                ac.model,
                COALESCE(SUM(ac.cost), 0) as total_cost,
                COUNT(*) as call_count,
                COALESCE(SUM(ac.input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(ac.output_tokens), 0) as total_output_tokens,
                CASE 
                    WHEN COUNT(*) > 0 THEN COALESCE(SUM(ac.cost), 0) / COUNT(*)
                    ELSE 0
                END as avg_cost_per_call
            FROM api_calls ac
            JOIN cost_sessions cs ON ac.session_id = cs.id
            WHERE cs.started_at >= ? AND cs.started_at <= ?
            GROUP BY ac.model
            ORDER BY total_cost DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool)
        .await?;

        let mut model_usage = Vec::new();
        for row in usage_stats {
            let call_count: i64 = row.get("call_count");
            let usage_percentage = if total_calls > 0 {
                (call_count as f64 / total_calls as f64) * 100.0
            } else {
                0.0
            };

            model_usage.push(ModelUsage {
                model: row.get("model"),
                total_cost: row.get("total_cost"),
                call_count,
                total_input_tokens: row.get("total_input_tokens"),
                total_output_tokens: row.get("total_output_tokens"),
                avg_cost_per_call: row.get("avg_cost_per_call"),
                usage_percentage,
            });
        }

        if model_usage.is_empty() {
            return Err(QueryError::NoDataFound);
        }

        Ok(model_usage)
    }

    /// Get daily cost summary for a specific date range
    pub async fn get_daily_costs(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<HashMap<String, f64>, QueryError> {
        if start_date >= end_date {
            return Err(QueryError::InvalidParameters {
                message: "start_date must be before end_date".to_string(),
            });
        }

        let results = sqlx::query(
            r#"
            SELECT 
                DATE(started_at) as date,
                COALESCE(SUM(total_cost), 0) as total_cost
            FROM cost_sessions
            WHERE DATE(started_at) >= DATE(?) AND DATE(started_at) <= DATE(?)
            GROUP BY DATE(started_at)
            ORDER BY DATE(started_at)
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool)
        .await?;

        let mut daily_costs = HashMap::new();
        for row in results {
            let date: String = row.get("date");
            let cost: f64 = row.get("total_cost");
            daily_costs.insert(date, cost);
        }

        Ok(daily_costs)
    }

    /// Export cost data as CSV
    pub async fn export_csv(
        &self,
        period: TimePeriod,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, QueryError> {
        let summaries = self
            .get_issue_cost_summary(period, start_date, end_date)
            .await?;

        let mut csv = String::new();
        csv.push_str("issue_id,total_cost,session_count,total_calls,total_input_tokens,total_output_tokens,avg_cost_per_session,first_session,last_session\n");

        for summary in summaries {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                summary.issue_id,
                summary.total_cost,
                summary.session_count,
                summary.total_calls,
                summary.total_input_tokens,
                summary.total_output_tokens,
                summary.avg_cost_per_session,
                summary.first_session.to_rfc3339(),
                summary.last_session.to_rfc3339()
            ));
        }

        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_period_duration() {
        assert_eq!(TimePeriod::Day.duration_days(), Some(1));
        assert_eq!(TimePeriod::Week.duration_days(), Some(7));
        assert_eq!(TimePeriod::Month.duration_days(), Some(30));
        assert_eq!(TimePeriod::Quarter.duration_days(), Some(90));
        assert_eq!(TimePeriod::Year.duration_days(), Some(365));
        assert_eq!(TimePeriod::Custom.duration_days(), None);
    }

    #[test]
    fn test_time_period_start_date() {
        let now = Utc::now();
        let day_start = TimePeriod::Day.start_date().unwrap();
        let week_start = TimePeriod::Week.start_date().unwrap();

        assert!(day_start > week_start);
        assert!(day_start <= now);
        assert!(week_start <= now);
        assert!(TimePeriod::Custom.start_date().is_none());
    }

    #[test]
    fn test_trend_query_creation() {
        let query = TrendQuery::new(TimePeriod::Week);
        assert_eq!(query.period, TimePeriod::Week);
        assert!(query.start_date.is_none());
        assert!(query.end_date.is_none());
        assert!(query.group_by_hours.is_none());
        assert!(query.issue_ids.is_none());
        assert!(query.pricing_model.is_none());
    }

    #[test]
    fn test_trend_query_custom() {
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();
        let query = TrendQuery::custom(start, Some(end));

        assert_eq!(query.period, TimePeriod::Custom);
        assert_eq!(query.start_date, Some(start));
        assert_eq!(query.end_date, Some(end));
    }

    #[test]
    fn test_trend_query_validation() {
        // Valid standard period query
        let query = TrendQuery::new(TimePeriod::Week);
        assert!(query.validate().is_ok());

        // Valid custom query with dates
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();
        let query = TrendQuery::custom(start, Some(end));
        assert!(query.validate().is_ok());

        // Invalid custom query without start date
        let mut query = TrendQuery::new(TimePeriod::Custom);
        query.start_date = None;
        assert!(query.validate().is_err());

        // Invalid custom query with start >= end
        let start = Utc::now();
        let end = start - chrono::Duration::hours(1);
        let query = TrendQuery::custom(start, Some(end));
        assert!(query.validate().is_err());

        // Invalid group_by_hours
        let query = TrendQuery::new(TimePeriod::Week).group_by_hours(0);
        assert!(query.validate().is_err());

        let query = TrendQuery::new(TimePeriod::Week).group_by_hours(10000);
        assert!(query.validate().is_err());
    }

    #[test]
    fn test_trend_query_date_range() {
        // Standard period
        let query = TrendQuery::new(TimePeriod::Week);
        let (start, end) = query.get_date_range().unwrap();
        assert!(start < end);
        assert!(end <= Utc::now());

        // Custom period
        let custom_start = Utc::now() - chrono::Duration::days(5);
        let custom_end = Utc::now() - chrono::Duration::days(1);
        let query = TrendQuery::custom(custom_start, Some(custom_end));
        let (start, end) = query.get_date_range().unwrap();
        assert_eq!(start, custom_start);
        assert_eq!(end, custom_end);

        // Custom period with default end
        let query = TrendQuery::custom(custom_start, None);
        let (start, end) = query.get_date_range().unwrap();
        assert_eq!(start, custom_start);
        assert!(end <= Utc::now());
    }

    #[test]
    fn test_trend_query_builder_pattern() {
        let query = TrendQuery::new(TimePeriod::Month)
            .group_by_hours(12)
            .with_issue_ids(vec!["issue1".to_string(), "issue2".to_string()])
            .with_pricing_model("paid".to_string());

        assert_eq!(query.period, TimePeriod::Month);
        assert_eq!(query.group_by_hours, Some(12));
        assert_eq!(
            query.issue_ids,
            Some(vec!["issue1".to_string(), "issue2".to_string()])
        );
        assert_eq!(query.pricing_model, Some("paid".to_string()));
        assert!(query.validate().is_ok());
    }
}
