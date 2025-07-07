//! Workflow system data structures and types
//!
//! This module provides the core types for representing and executing workflows
//! based on Mermaid state diagrams.

mod state;
mod transition;
mod workflow;
mod run;

pub use state::{State, StateId};
pub use transition::{Transition, TransitionCondition};
pub use workflow::{Workflow, WorkflowName};
pub use run::{WorkflowRun, WorkflowRunId, WorkflowRunStatus};