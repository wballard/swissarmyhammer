//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod state;
mod transition;
mod definition;
mod run;
mod parser;
mod storage;
mod executor;

pub use state::{State, StateId};
pub use transition::{Transition, TransitionCondition};
pub use definition::{Workflow, WorkflowName};
pub use run::{WorkflowRun, WorkflowRunId, WorkflowRunStatus};
pub use parser::{MermaidParser, ParseError, ParseResult};
pub use storage::{
    WorkflowStorage, WorkflowStorageBackend, WorkflowRunStorageBackend,
    MemoryWorkflowStorage, MemoryWorkflowRunStorage,
    FileSystemWorkflowStorage, FileSystemWorkflowRunStorage,
    WorkflowResolver, WorkflowSource,
};
pub use executor::{
    WorkflowExecutor, ExecutorError, ExecutorResult, 
    ExecutionEvent, ExecutionEventType,
};