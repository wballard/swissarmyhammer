//! Cost aggregation reporting and export capabilities
//!
//! This module provides comprehensive reporting capabilities for cost aggregation
//! data, including multiple export formats and customizable report generation.

use super::ProjectCostSummary;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Report generation errors
#[derive(Error, Debug)]
pub enum ReportError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// I/O error during export
    #[error("Export I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// Invalid report configuration
    #[error("Invalid configuration: {0}")]
    Configuration(String),
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format for programmatic access
    Json,
    /// CSV format for spreadsheet analysis
    Csv,
    /// Markdown format for human-readable reports
    Markdown,
    /// HTML format for web viewing
    Html,
    /// Plain text format
    Text,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Csv => write!(f, "csv"),
            ExportFormat::Markdown => write!(f, "markdown"),
            ExportFormat::Html => write!(f, "html"),
            ExportFormat::Text => write!(f, "text"),
        }
    }
}

/// Report configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Include detailed cost breakdown
    pub include_breakdown: bool,
    /// Include trend analysis
    pub include_trends: bool,
    /// Include efficiency metrics
    pub include_efficiency: bool,
    /// Include outlier analysis
    pub include_outliers: bool,
    /// Include predictions
    pub include_predictions: bool,
    /// Number of decimal places for cost values
    pub cost_precision: u8,
    /// Currency symbol to use
    pub currency_symbol: String,
    /// Date format for timestamps
    pub date_format: String,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            include_breakdown: true,
            include_trends: true,
            include_efficiency: true,
            include_outliers: true,
            include_predictions: false,
            cost_precision: 4,
            currency_symbol: "$".to_string(),
            date_format: "%Y-%m-%d %H:%M:%S UTC".to_string(),
        }
    }
}

/// Aggregated report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Project cost summary
    pub summary: ProjectCostSummary,
    /// Additional report sections
    pub sections: HashMap<String, ReportSection>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report title
    pub title: String,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report configuration used
    pub config: ReportConfig,
    /// Report format
    pub format: ExportFormat,
    /// Report version
    pub version: String,
}

/// Individual report section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: ReportContent,
    /// Section order for display
    pub order: u32,
}

/// Report content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportContent {
    /// Plain text content
    Text(String),
    /// Table data
    Table(TableData),
    /// Chart/visualization data
    Chart(ChartData),
    /// Key-value pairs
    KeyValue(HashMap<String, String>),
}

/// Table data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    /// Column headers
    pub headers: Vec<String>,
    /// Table rows
    pub rows: Vec<Vec<String>>,
}

/// Chart data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// Chart type (line, bar, pie, etc.)
    pub chart_type: String,
    /// Chart data points
    pub data_points: Vec<ChartPoint>,
    /// Chart labels
    pub labels: Vec<String>,
}

/// Individual chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    /// X-axis value (typically time)
    pub x: String,
    /// Y-axis value (typically cost)
    pub y: f64,
    /// Optional series name for multi-series charts
    pub series: Option<String>,
}

/// Report generator with multiple format support
#[derive(Default)]
pub struct ReportGenerator {
    /// Default report configuration
    config: ReportConfig,
}


impl ReportGenerator {
    /// Create a new report generator with custom configuration
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate a comprehensive aggregated report
    pub fn generate_report(
        &self,
        summary: ProjectCostSummary,
        format: ExportFormat,
    ) -> Result<AggregatedReport, ReportError> {
        let metadata = ReportMetadata {
            title: "Project Cost Analysis Report".to_string(),
            generated_at: Utc::now(),
            config: self.config.clone(),
            format,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let mut sections = HashMap::new();

        // Executive summary section
        if let Some(exec_section) = self.create_executive_summary(&summary)? {
            sections.insert("executive_summary".to_string(), exec_section);
        }

        // Cost breakdown section
        if self.config.include_breakdown {
            if let Some(breakdown_section) = self.create_cost_breakdown(&summary)? {
                sections.insert("cost_breakdown".to_string(), breakdown_section);
            }
        }

        // Trend analysis section
        if self.config.include_trends {
            if let Some(trends_section) = self.create_trends_section(&summary)? {
                sections.insert("trends".to_string(), trends_section);
            }
        }

        // Efficiency metrics section
        if self.config.include_efficiency {
            if let Some(efficiency_section) = self.create_efficiency_section(&summary)? {
                sections.insert("efficiency".to_string(), efficiency_section);
            }
        }

        // Outliers section
        if self.config.include_outliers && !summary.outliers.is_empty() {
            if let Some(outliers_section) = self.create_outliers_section(&summary)? {
                sections.insert("outliers".to_string(), outliers_section);
            }
        }

        Ok(AggregatedReport {
            metadata,
            summary,
            sections,
        })
    }

    /// Export report to specified format
    pub fn export_report(
        &self,
        report: &AggregatedReport,
        format: ExportFormat,
    ) -> Result<String, ReportError> {
        match format {
            ExportFormat::Json => self.export_json(report),
            ExportFormat::Csv => self.export_csv(report),
            ExportFormat::Markdown => self.export_markdown(report),
            ExportFormat::Html => self.export_html(report),
            ExportFormat::Text => self.export_text(report),
        }
    }

    /// Create executive summary section
    fn create_executive_summary(
        &self,
        summary: &ProjectCostSummary,
    ) -> Result<Option<ReportSection>, ReportError> {
        let total_cost = self.format_currency(summary.total_cost);
        let avg_cost = self.format_currency(summary.average_cost_per_issue);
        let median_cost = self.format_currency(summary.median_cost_per_issue);

        let content = format!(
            "Total project cost: {}\n\
             Issues analyzed: {}\n\
             Average cost per issue: {}\n\
             Median cost per issue: {}\n\
             Trend direction: {:?}\n\
             Analysis period: {} to {}",
            total_cost,
            summary.total_issues,
            avg_cost,
            median_cost,
            summary.cost_trend.trend_direction,
            self.format_date(summary.period.start),
            self.format_date(summary.period.end)
        );

        Ok(Some(ReportSection {
            title: "Executive Summary".to_string(),
            content: ReportContent::Text(content),
            order: 1,
        }))
    }

    /// Create cost breakdown section
    fn create_cost_breakdown(
        &self,
        summary: &ProjectCostSummary,
    ) -> Result<Option<ReportSection>, ReportError> {
        let mut rows = Vec::new();

        for (category, cost) in &summary.cost_breakdown {
            let percentage = if summary.total_cost > Decimal::ZERO {
                (*cost / summary.total_cost * Decimal::new(100, 0))
                    .round_dp(2)
                    .to_string()
            } else {
                "0.00".to_string()
            };

            rows.push(vec![
                category.clone(),
                self.format_currency(*cost),
                format!("{}%", percentage),
            ]);
        }

        let table = TableData {
            headers: vec![
                "Category".to_string(),
                "Cost".to_string(),
                "Percentage".to_string(),
            ],
            rows,
        };

        Ok(Some(ReportSection {
            title: "Cost Breakdown".to_string(),
            content: ReportContent::Table(table),
            order: 2,
        }))
    }

    /// Create trends section
    fn create_trends_section(
        &self,
        summary: &ProjectCostSummary,
    ) -> Result<Option<ReportSection>, ReportError> {
        let trend = &summary.cost_trend;

        let mut key_values = HashMap::new();
        key_values.insert("Direction".to_string(), format!("{:?}", trend.trend_direction));
        key_values.insert("Growth Rate".to_string(), format!("{:.2}%", trend.growth_rate * 100.0));
        key_values.insert("Confidence".to_string(), format!("{:.1}%", trend.confidence * 100.0));
        key_values.insert("Daily Data Points".to_string(), trend.daily_costs.len().to_string());
        key_values.insert("Weekly Data Points".to_string(), trend.weekly_costs.len().to_string());
        key_values.insert("Monthly Data Points".to_string(), trend.monthly_costs.len().to_string());

        Ok(Some(ReportSection {
            title: "Trend Analysis".to_string(),
            content: ReportContent::KeyValue(key_values),
            order: 3,
        }))
    }

    /// Create efficiency section
    fn create_efficiency_section(
        &self,
        summary: &ProjectCostSummary,
    ) -> Result<Option<ReportSection>, ReportError> {
        let efficiency = &summary.efficiency_metrics;

        let mut key_values = HashMap::new();
        key_values.insert(
            "Cost per API Call".to_string(),
            self.format_currency(efficiency.cost_per_api_call),
        );
        key_values.insert(
            "Cost per Token".to_string(),
            self.format_currency(efficiency.cost_per_token),
        );
        key_values.insert(
            "Cost per Session".to_string(),
            self.format_currency(efficiency.cost_per_session),
        );
        key_values.insert(
            "Average Session Duration".to_string(),
            format!("{:.1} minutes", efficiency.avg_session_duration_minutes),
        );
        key_values.insert(
            "Token Efficiency".to_string(),
            format!("{:.2}", efficiency.token_efficiency),
        );
        key_values.insert(
            "Efficiency Score".to_string(),
            format!("{:.1}%", efficiency.efficiency_score * 100.0),
        );

        Ok(Some(ReportSection {
            title: "Efficiency Metrics".to_string(),
            content: ReportContent::KeyValue(key_values),
            order: 4,
        }))
    }

    /// Create outliers section
    fn create_outliers_section(
        &self,
        summary: &ProjectCostSummary,
    ) -> Result<Option<ReportSection>, ReportError> {
        let mut rows = Vec::new();

        for outlier in &summary.outliers {
            rows.push(vec![
                outlier.issue_id.clone(),
                self.format_currency(outlier.cost),
                format!("{:?}", outlier.outlier_type),
                format!("{:.2}", outlier.standard_deviations),
                outlier.reason.clone(),
            ]);
        }

        let table = TableData {
            headers: vec![
                "Issue ID".to_string(),
                "Cost".to_string(),
                "Type".to_string(),
                "Std Deviations".to_string(),
                "Reason".to_string(),
            ],
            rows,
        };

        Ok(Some(ReportSection {
            title: "Cost Outliers".to_string(),
            content: ReportContent::Table(table),
            order: 5,
        }))
    }

    /// Export report as JSON
    fn export_json(&self, report: &AggregatedReport) -> Result<String, ReportError> {
        serde_json::to_string_pretty(report)
            .map_err(|e| ReportError::Serialization(e.to_string()))
    }

    /// Export report as CSV
    fn export_csv(&self, report: &AggregatedReport) -> Result<String, ReportError> {
        let mut output = String::new();

        // Add summary information as CSV header
        output.push_str("# Project Cost Analysis Report\n");
        output.push_str(&format!("# Generated: {}\n", self.format_date(report.metadata.generated_at)));
        output.push_str(&format!("# Total Cost: {}\n", self.format_currency(report.summary.total_cost)));
        output.push_str(&format!("# Total Issues: {}\n", report.summary.total_issues));
        output.push('\n');

        // Export each table section
        for section in report.sections.values() {
            if let ReportContent::Table(table_data) = &section.content {
                output.push_str(&format!("# {}\n", section.title));
                output.push_str(&table_data.headers.join(","));
                output.push('\n');

                for row in &table_data.rows {
                    output.push_str(&row.join(","));
                    output.push('\n');
                }
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Export report as Markdown
    fn export_markdown(&self, report: &AggregatedReport) -> Result<String, ReportError> {
        let mut output = String::new();

        // Report header
        output.push_str(&format!("# {}\n\n", report.metadata.title));
        output.push_str(&format!("Generated: {}\n\n", self.format_date(report.metadata.generated_at)));

        // Export sections in order
        let mut sorted_sections: Vec<_> = report.sections.iter().collect();
        sorted_sections.sort_by_key(|(_, section)| section.order);

        for (_, section) in sorted_sections {
            output.push_str(&format!("## {}\n\n", section.title));

            match &section.content {
                ReportContent::Text(text) => {
                    output.push_str(text);
                    output.push_str("\n\n");
                }
                ReportContent::Table(table) => {
                    // Markdown table format
                    output.push_str(&format!("| {} |\n", table.headers.join(" | ")));
                    output.push_str(&format!("|{}|\n", table.headers.iter().map(|_| "---").collect::<Vec<_>>().join("|")));

                    for row in &table.rows {
                        output.push_str(&format!("| {} |\n", row.join(" | ")));
                    }
                    output.push('\n');
                }
                ReportContent::KeyValue(kv) => {
                    for (key, value) in kv {
                        output.push_str(&format!("- **{}**: {}\n", key, value));
                    }
                    output.push('\n');
                }
                ReportContent::Chart(_) => {
                    output.push_str("*Chart data available in JSON format*\n\n");
                }
            }
        }

        Ok(output)
    }

    /// Export report as HTML
    fn export_html(&self, report: &AggregatedReport) -> Result<String, ReportError> {
        let mut output = String::new();

        output.push_str(&format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .summary {{ background-color: #f9f9f9; padding: 20px; margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <p>Generated: {}</p>
"#,
            report.metadata.title,
            report.metadata.title,
            self.format_date(report.metadata.generated_at)
        ));

        // Export sections
        let mut sorted_sections: Vec<_> = report.sections.iter().collect();
        sorted_sections.sort_by_key(|(_, section)| section.order);

        for (_, section) in sorted_sections {
            output.push_str(&format!("    <h2>{}</h2>\n", section.title));

            match &section.content {
                ReportContent::Text(text) => {
                    output.push_str(&format!("    <div class=\"summary\"><pre>{}</pre></div>\n", text));
                }
                ReportContent::Table(table) => {
                    output.push_str("    <table>\n");
                    output.push_str("        <thead><tr>");
                    for header in &table.headers {
                        output.push_str(&format!("<th>{}</th>", header));
                    }
                    output.push_str("</tr></thead>\n");

                    output.push_str("        <tbody>\n");
                    for row in &table.rows {
                        output.push_str("            <tr>");
                        for cell in row {
                            output.push_str(&format!("<td>{}</td>", cell));
                        }
                        output.push_str("</tr>\n");
                    }
                    output.push_str("        </tbody>\n    </table>\n");
                }
                ReportContent::KeyValue(kv) => {
                    output.push_str("    <ul>\n");
                    for (key, value) in kv {
                        output.push_str(&format!("        <li><strong>{}:</strong> {}</li>\n", key, value));
                    }
                    output.push_str("    </ul>\n");
                }
                ReportContent::Chart(_) => {
                    output.push_str("    <p><em>Chart data available in JSON format</em></p>\n");
                }
            }
        }

        output.push_str("</body>\n</html>\n");
        Ok(output)
    }

    /// Export report as plain text
    fn export_text(&self, report: &AggregatedReport) -> Result<String, ReportError> {
        let mut output = String::new();

        output.push_str(&format!("{}\n", report.metadata.title));
        output.push_str(&format!("{}\n", "=".repeat(report.metadata.title.len())));
        output.push_str(&format!("Generated: {}\n\n", self.format_date(report.metadata.generated_at)));

        let mut sorted_sections: Vec<_> = report.sections.iter().collect();
        sorted_sections.sort_by_key(|(_, section)| section.order);

        for (_, section) in sorted_sections {
            output.push_str(&format!("{}\n", section.title));
            output.push_str(&format!("{}\n", "-".repeat(section.title.len())));

            match &section.content {
                ReportContent::Text(text) => {
                    output.push_str(text);
                    output.push_str("\n\n");
                }
                ReportContent::Table(table) => {
                    // Simple text table format
                    let col_widths: Vec<usize> = table.headers
                        .iter()
                        .enumerate()
                        .map(|(i, header)| {
                            let max_row_width = table.rows
                                .iter()
                                .map(|row| row.get(i).map(|s| s.len()).unwrap_or(0))
                                .max()
                                .unwrap_or(0);
                            header.len().max(max_row_width)
                        })
                        .collect();

                    // Header row
                    for (i, header) in table.headers.iter().enumerate() {
                        output.push_str(&format!("{:<width$}", header, width = col_widths[i] + 2));
                    }
                    output.push('\n');

                    // Separator
                    for &width in &col_widths {
                        output.push_str(&format!("{:<width$}", "-".repeat(width), width = width + 2));
                    }
                    output.push('\n');

                    // Data rows
                    for row in &table.rows {
                        for (i, cell) in row.iter().enumerate() {
                            output.push_str(&format!("{:<width$}", cell, width = col_widths[i] + 2));
                        }
                        output.push('\n');
                    }
                    output.push('\n');
                }
                ReportContent::KeyValue(kv) => {
                    for (key, value) in kv {
                        output.push_str(&format!("{}: {}\n", key, value));
                    }
                    output.push('\n');
                }
                ReportContent::Chart(_) => {
                    output.push_str("(Chart data available in JSON format)\n\n");
                }
            }
        }

        Ok(output)
    }

    /// Format a currency value
    fn format_currency(&self, value: Decimal) -> String {
        format!(
            "{}{}",
            self.config.currency_symbol,
            value.round_dp(self.config.cost_precision as u32)
        )
    }

    /// Format a date according to configuration
    fn format_date(&self, date: DateTime<Utc>) -> String {
        date.format(&self.config.date_format).to_string()
    }
}