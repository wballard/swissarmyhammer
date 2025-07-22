//! API interception testing suite
//!
//! Comprehensive end-to-end tests for the complete API interception pipeline,
//! validating integration between MCP handlers, cost tracking, token counting,
//! and cost calculation systems. Tests ensure the complete system works reliably
//! under various conditions including concurrent operations, error scenarios,
//! and performance requirements.
//!
//! This module is organized into specialized sub-modules:
//! - `api_interception_helpers`: Common test utilities and helper functions
//! - `api_interception_functional`: Core functionality and integration tests
//! - `api_interception_performance`: Performance benchmarks and overhead tests
//! - `api_interception_reliability`: Error handling and edge case tests
//!
//! The modular structure improves maintainability and allows focused testing
//! of different aspects of the API interception system.

// Re-export the test modules for integration testing
pub use super::api_interception_helpers::*;
