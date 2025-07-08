//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod action_parser;
mod actions;
mod definition;
mod error_utils;
mod executor;
mod parser;
mod run;
mod state;
mod storage;
#[cfg(test)]
mod test_helpers;
mod transition;

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
pub use parser::{MermaidParser, ParseError, ParseResult};
pub use run::{WorkflowRun, WorkflowRunId, WorkflowRunStatus};
pub use state::{State, StateError, StateId, StateResult, StateType};
pub use storage::{
    FileSystemWorkflowRunStorage, FileSystemWorkflowStorage, MemoryWorkflowRunStorage,
    MemoryWorkflowStorage, WorkflowResolver, WorkflowRunStorageBackend, WorkflowSource,
    WorkflowStorage, WorkflowStorageBackend,
};
pub use transition::{ConditionType, Transition, TransitionCondition};
