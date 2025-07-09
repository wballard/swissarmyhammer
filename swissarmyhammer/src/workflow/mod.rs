//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod action_parser;
mod actions;
mod definition;
mod error_utils;
mod executor;
mod graph;
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

pub use actions::{
    parse_action_from_description, Action, ActionError, ActionResult, LogAction, LogLevel,
    PromptAction, SetVariableAction, SubWorkflowAction, WaitAction,
};
pub use definition::{Workflow, WorkflowError, WorkflowName, WorkflowResult};
pub use error_utils::{
    handle_command_error, handle_command_error_with_mapper, handle_claude_command_error,
    command_succeeded, extract_stderr, extract_stdout,
};
pub use executor::{
    ExecutionEvent, ExecutionEventType, ExecutorError, ExecutorResult, WorkflowExecutor,
};
pub use graph::{GraphError, GraphResult, WorkflowGraphAnalyzer};
pub use metrics::{
    WorkflowMetrics, RunMetrics, WorkflowSummaryMetrics, GlobalMetrics, MemoryMetrics,
    StateExecutionCount, ResourceTrends,
};
pub use parser::{MermaidParser, ParseError, ParseResult};
pub use run::{WorkflowRun, WorkflowRunId, WorkflowRunStatus};
pub use state::{State, StateError, StateId, StateResult, StateType, CompensationKey, ErrorContext};
pub use storage::{
    FileSystemWorkflowRunStorage, FileSystemWorkflowStorage, MemoryWorkflowRunStorage,
    MemoryWorkflowStorage, WorkflowResolver, WorkflowRunStorageBackend, WorkflowSource,
    WorkflowStorage, WorkflowStorageBackend,
};
pub use transition::{ConditionType, Transition, TransitionCondition};
pub use transition_key::TransitionKey;
pub use visualization::{
    ExecutionVisualizer, ExecutionTrace, ExecutionStep, VisualizationFormat,
    VisualizationOptions, ColorScheme,
};
