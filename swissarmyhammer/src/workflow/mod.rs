//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod actions;
mod definition;
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
    PromptAction, SetVariableAction, WaitAction,
};
pub use definition::{Workflow, WorkflowError, WorkflowName, WorkflowResult};
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
