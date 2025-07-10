//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod action_parser;
mod actions;
#[cfg(test)]
mod actions_tests;
mod cache;
mod definition;
mod error_utils;
mod executor;
mod graph;
#[cfg(test)]
mod graph_tests;
mod metrics;
mod parser;
mod run;
mod state;
mod storage;
#[cfg(test)]
mod test_helpers;
mod transition;
mod transition_key;
mod visualization;
#[cfg(test)]
mod visualization_tests;

pub use actions::{
    parse_action_from_description, Action, ActionError, ActionResult, LogAction, LogLevel,
    PromptAction, SetVariableAction, SubWorkflowAction, WaitAction,
};
pub use cache::{
    CacheStats, CelProgramCache, TransitionCache, TransitionPath, WorkflowCache,
    WorkflowCacheManager,
};
pub use definition::{Workflow, WorkflowError, WorkflowName, WorkflowResult};
pub use error_utils::{
    command_succeeded, extract_stderr, extract_stdout, handle_claude_command_error,
    handle_command_error, handle_command_error_with_mapper,
};
pub use executor::{
    ExecutionEvent, ExecutionEventType, ExecutorError, ExecutorResult, WorkflowExecutor,
};
pub use graph::{GraphError, GraphResult, WorkflowGraphAnalyzer};
pub use metrics::{
    GlobalMetrics, MemoryMetrics, ResourceTrends, RunMetrics, StateExecutionCount, WorkflowMetrics,
    WorkflowSummaryMetrics,
};
pub use parser::{MermaidParser, ParseError, ParseResult};
pub use run::{WorkflowRun, WorkflowRunId, WorkflowRunStatus};
pub use state::{
    CompensationKey, ErrorContext, State, StateError, StateId, StateResult, StateType,
};
pub use storage::{
    CompressedWorkflowStorage, FileSystemWorkflowRunStorage, FileSystemWorkflowStorage,
    MemoryWorkflowRunStorage, MemoryWorkflowStorage, WorkflowResolver, WorkflowRunStorageBackend,
    WorkflowStorage, WorkflowStorageBackend,
};
pub use transition::{ConditionType, Transition, TransitionCondition};
pub use transition_key::TransitionKey;
pub use visualization::{
    ColorScheme, ExecutionStep, ExecutionTrace, ExecutionVisualizer, VisualizationFormat,
    VisualizationOptions,
};
