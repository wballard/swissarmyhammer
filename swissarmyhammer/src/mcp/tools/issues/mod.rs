//! Issue management tools for MCP operations
//!
//! This module provides all issue-related tools using the tool registry pattern.
//! Each tool is in its own submodule with dedicated implementation and description.

pub mod create;
pub mod mark_complete;
pub mod all_complete;
pub mod update;
pub mod current;
pub mod work;
pub mod merge;
pub mod next;

use crate::mcp::tool_registry::ToolRegistry;

/// Register all issue-related tools with the registry
pub fn register_issue_tools(registry: &mut ToolRegistry) {
    registry.register(create::CreateIssueTool::new());
    registry.register(mark_complete::MarkCompleteIssueTool::new());
    registry.register(all_complete::AllCompleteIssueTool::new());
    registry.register(update::UpdateIssueTool::new());
    registry.register(current::CurrentIssueTool::new());
    registry.register(work::WorkIssueTool::new());
    registry.register(merge::MergeIssueTool::new());
    registry.register(next::NextIssueTool::new());
}