//! Constants for MCP server configuration

/// Constants for issue branch management
pub const ISSUE_BRANCH_PREFIX: &str = "issue/";
/// Width for zero-padded issue numbers (e.g., 000001)
pub const ISSUE_NUMBER_WIDTH: usize = 6;

/// Minimum valid issue number
pub const MIN_ISSUE_NUMBER: u32 = 1;
/// Maximum valid issue number
pub const MAX_ISSUE_NUMBER: u32 = 999999;