//! Optional database storage for cost analytics
//!
//! This module provides an optional SQLite database backend for enhanced cost
//! analytics and reporting. The database storage is completely optional and
//! configurable - the primary storage remains markdown-based for simplicity.
//!
//! # Features
//!
//! - **Optional Storage**: Database storage is disabled by default
//! - **Analytics**: Advanced cost queries and aggregations  
//! - **Historical Analysis**: Long-term cost trend tracking
//! - **Cross-issue Reporting**: Compare costs across different issues
//! - **Export Capabilities**: Export data for external analytics tools
//!
//! # Configuration
//!
//! Database storage can be enabled via configuration:
//!
//! ```yaml
//! cost_tracking:
//!   database:
//!     enabled: true
//!     file_path: "./costs.db"
//!     connection_timeout_seconds: 30
//!     max_connections: 10
//!     retention_days: 365
//! ```

#[cfg(feature = "database")]
pub mod config;
#[cfg(feature = "database")]
pub mod migrations;
#[cfg(feature = "database")]
pub mod queries;
#[cfg(feature = "database")]
pub mod schema;

#[cfg(feature = "database")]
pub use config::{DatabaseConfig, DatabaseConfigError};
#[cfg(feature = "database")]
pub use migrations::{Migration, MigrationError, MigrationRunner};
#[cfg(feature = "database")]
pub use queries::{CostAnalytics, CostTrend, TrendQuery};
#[cfg(feature = "database")]
pub use schema::{CostDatabase, DatabaseError};

// Re-export main types when database feature is enabled
#[cfg(feature = "database")]
pub use schema::CostDatabase as Database;

// When database feature is disabled, provide stub types
#[cfg(not(feature = "database"))]
pub struct Database;

#[cfg(not(feature = "database"))]
impl Database {
    pub async fn new(_config: ()) -> Result<Self, &'static str> {
        Err("Database feature not enabled")
    }
}
