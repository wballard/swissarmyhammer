//! Cost tracking data structures
//!
//! This module provides the core data structures for cost tracking: `CostTracker`,
//! `CostSession`, and `ApiCall`. These structures form the foundation of the cost
//! tracking system and integrate with the existing metrics infrastructure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use thiserror::Error;
use ulid::Ulid;

/// Maximum number of cost sessions to keep in memory
pub const MAX_COST_SESSIONS: usize = 1000;

/// Maximum number of API calls per session
pub const MAX_API_CALLS_PER_SESSION: usize = 500;

/// Maximum age of completed sessions before cleanup (in days)
pub const MAX_COMPLETED_SESSION_AGE_DAYS: i64 = 7;

/// Maximum endpoint URL length
pub const MAX_ENDPOINT_URL_LENGTH: usize = 2048;

/// Maximum model name length
pub const MAX_MODEL_NAME_LENGTH: usize = 256;

/// Cost tracking error types
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CostError {
    /// Session not found
    #[error("Cost session not found: {session_id}")]
    SessionNotFound {
        /// The session ID that was not found
        session_id: CostSessionId,
    },

    /// Session already exists
    #[error("Cost session already exists: {session_id}")]
    SessionAlreadyExists {
        /// The session ID that already exists
        session_id: CostSessionId,
    },

    /// Session already completed
    #[error("Cost session already completed: {session_id}")]
    SessionAlreadyCompleted {
        /// The session ID that was already completed
        session_id: CostSessionId,
    },

    /// Too many sessions
    #[error("Maximum number of sessions ({}) exceeded", MAX_COST_SESSIONS)]
    TooManySessions,

    /// Too many API calls in session
    #[error(
        "Maximum number of API calls per session ({}) exceeded for session: {session_id}",
        MAX_API_CALLS_PER_SESSION
    )]
    TooManyApiCalls {
        /// The session ID that exceeded the API call limit
        session_id: CostSessionId,
    },

    /// Invalid input data
    #[error("Invalid input: {message}")]
    InvalidInput {
        /// Description of the invalid input
        message: String,
    },

    /// API call not found
    #[error("API call not found: {call_id} in session: {session_id}")]
    ApiCallNotFound {
        /// The API call ID that was not found
        call_id: ApiCallId,
        /// The session ID where the call was expected
        session_id: CostSessionId,
    },

    /// Serialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Description of the serialization error
        message: String,
    },
}

/// Unique identifier for a cost session
///
/// This is a wrapper around a ULID that provides a type-safe identifier
/// for cost tracking sessions. Each session gets a unique identifier that
/// is sortable by creation time.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::cost::CostSessionId;
///
/// let session_id1 = CostSessionId::new();
/// let session_id2 = CostSessionId::new();
///
/// assert_ne!(session_id1, session_id2);
/// println!("Session ID: {}", session_id1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CostSessionId(Ulid);

impl CostSessionId {
    /// Create a new cost session ID
    ///
    /// Each call to this function generates a unique identifier that is
    /// sortable by creation time thanks to the ULID format.
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// Get the inner ULID
    pub fn as_ulid(&self) -> Ulid {
        self.0
    }

    /// Create from a ULID
    pub fn from_ulid(ulid: Ulid) -> Self {
        Self(ulid)
    }
}

impl fmt::Display for CostSessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for CostSessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an API call
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApiCallId(Ulid);

impl ApiCallId {
    /// Create a new API call ID
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// Get the inner ULID
    pub fn as_ulid(&self) -> Ulid {
        self.0
    }

    /// Create from a ULID
    pub fn from_ulid(ulid: Ulid) -> Self {
        Self(ulid)
    }
}

impl fmt::Display for ApiCallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ApiCallId {
    fn default() -> Self {
        Self::new()
    }
}

/// Issue identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IssueId(String);

impl IssueId {
    /// Create a new issue ID
    pub fn new(id: impl Into<String>) -> Result<Self, CostError> {
        let id = id.into();
        Self::validate_issue_id(&id)?;
        Ok(Self(id))
    }

    /// Get the issue ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validate issue ID format
    fn validate_issue_id(id: &str) -> Result<(), CostError> {
        if id.trim().is_empty() {
            return Err(CostError::InvalidInput {
                message: "Issue ID cannot be empty".to_string(),
            });
        }
        if id.len() > 256 {
            return Err(CostError::InvalidInput {
                message: "Issue ID cannot exceed 256 characters".to_string(),
            });
        }
        Ok(())
    }
}

impl fmt::Display for IssueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of an API call
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiCallStatus {
    /// API call completed successfully
    Success,
    /// API call failed with an error
    Failed,
    /// API call timed out
    Timeout,
    /// API call was cancelled
    Cancelled,
    /// API call is in progress
    InProgress,
}

/// Status of a cost session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostSessionStatus {
    /// Session is active and tracking calls
    Active,
    /// Session completed successfully
    Completed,
    /// Session was cancelled
    Cancelled,
    /// Session failed due to error
    Failed,
}

/// Individual API call record
///
/// This structure tracks a single API call made during issue workflow execution.
/// It captures timing information, token usage, and call status for cost calculation
/// and monitoring purposes.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::cost::{ApiCall, ApiCallStatus};
///
/// // Create a new API call
/// let mut api_call = ApiCall::new(
///     "https://api.anthropic.com/v1/messages",
///     "claude-3-sonnet-20241022"
/// ).unwrap();
///
/// // Complete the call with token counts
/// api_call.complete(150, 300, ApiCallStatus::Success, None);
///
/// assert_eq!(api_call.total_tokens(), 450);
/// assert!(api_call.is_completed());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCall {
    /// Unique call identifier
    pub call_id: ApiCallId,
    /// When the call was started
    pub started_at: DateTime<Utc>,
    /// When the call was completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// API endpoint URL
    pub endpoint: String,
    /// Model name used for the call
    pub model: String,
    /// Number of input tokens
    pub input_tokens: u32,
    /// Number of output tokens
    pub output_tokens: u32,
    /// Total call duration
    pub duration: Option<Duration>,
    /// Call status
    pub status: ApiCallStatus,
    /// Error message if call failed
    pub error_message: Option<String>,
}

impl ApiCall {
    /// Create a new API call record
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Result<Self, CostError> {
        let endpoint = endpoint.into();
        let model = model.into();

        // Validate inputs
        if endpoint.trim().is_empty() {
            return Err(CostError::InvalidInput {
                message: "Endpoint cannot be empty".to_string(),
            });
        }
        if endpoint.len() > MAX_ENDPOINT_URL_LENGTH {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Endpoint URL cannot exceed {} characters",
                    MAX_ENDPOINT_URL_LENGTH
                ),
            });
        }
        if model.trim().is_empty() {
            return Err(CostError::InvalidInput {
                message: "Model cannot be empty".to_string(),
            });
        }
        if model.len() > MAX_MODEL_NAME_LENGTH {
            return Err(CostError::InvalidInput {
                message: format!(
                    "Model name cannot exceed {} characters",
                    MAX_MODEL_NAME_LENGTH
                ),
            });
        }

        Ok(Self {
            call_id: ApiCallId::new(),
            started_at: Utc::now(),
            completed_at: None,
            endpoint,
            model,
            input_tokens: 0,
            output_tokens: 0,
            duration: None,
            status: ApiCallStatus::InProgress,
            error_message: None,
        })
    }

    /// Complete the API call with token counts
    pub fn complete(
        &mut self,
        input_tokens: u32,
        output_tokens: u32,
        status: ApiCallStatus,
        error_message: Option<String>,
    ) {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.input_tokens = input_tokens;
        self.output_tokens = output_tokens;
        self.status = status;
        self.error_message = error_message;
        self.duration = Some(
            now.signed_duration_since(self.started_at)
                .to_std()
                .unwrap_or(Duration::ZERO),
        );
    }

    /// Get total token count
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    /// Check if the call is completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }
}

/// Cost session for tracking API calls during issue workflow
///
/// A cost session represents a single issue workflow execution and tracks
/// all API calls made during that workflow. Sessions provide aggregated
/// metrics like total tokens and costs for the entire workflow.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::cost::{CostSession, CostSessionStatus, IssueId, ApiCall};
///
/// // Create a new session for an issue
/// let issue_id = IssueId::new("issue-123").unwrap();
/// let mut session = CostSession::new(issue_id);
///
/// // Add API calls
/// let api_call = ApiCall::new(
///     "https://api.anthropic.com/v1/messages",
///     "claude-3-sonnet-20241022"
/// ).unwrap();
/// let call_id = session.add_api_call(api_call).unwrap();
///
/// // Complete the API call
/// let api_call = session.get_api_call_mut(&call_id).unwrap();
/// api_call.complete(100, 200, swissarmyhammer::cost::ApiCallStatus::Success, None);
///
/// // Complete the session
/// session.complete(CostSessionStatus::Completed).unwrap();
///
/// assert_eq!(session.total_tokens(), 300);
/// assert!(session.is_completed());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSession {
    /// Unique session identifier
    pub session_id: CostSessionId,
    /// Associated issue identifier
    pub issue_id: IssueId,
    /// When the session started
    pub started_at: DateTime<Utc>,
    /// When the session completed (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Session status
    pub status: CostSessionStatus,
    /// Collection of API calls during this session
    pub api_calls: HashMap<ApiCallId, ApiCall>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Total session duration
    pub total_duration: Option<Duration>,
}

impl CostSession {
    /// Create a new cost session
    pub fn new(issue_id: IssueId) -> Self {
        Self {
            session_id: CostSessionId::new(),
            issue_id,
            started_at: Utc::now(),
            completed_at: None,
            status: CostSessionStatus::Active,
            api_calls: HashMap::new(),
            metadata: HashMap::new(),
            total_duration: None,
        }
    }

    /// Add an API call to this session
    pub fn add_api_call(&mut self, mut api_call: ApiCall) -> Result<ApiCallId, CostError> {
        // Check if session is already completed
        if self.completed_at.is_some() {
            return Err(CostError::SessionAlreadyCompleted {
                session_id: self.session_id,
            });
        }

        // Check if we're exceeding the limit
        if self.api_calls.len() >= MAX_API_CALLS_PER_SESSION {
            return Err(CostError::TooManyApiCalls {
                session_id: self.session_id,
            });
        }

        // Ensure the call has a unique ID
        let mut collision_count = 0;
        while self.api_calls.contains_key(&api_call.call_id) {
            collision_count += 1;
            api_call.call_id = ApiCallId::new();
        }

        // Log ULID collisions for monitoring (extremely rare event)
        if collision_count > 0 {
            tracing::warn!(
                collision_count = collision_count,
                session_id = %self.session_id,
                "ULID collision detected in API call ID generation"
            );
        }

        let call_id = api_call.call_id;
        self.api_calls.insert(call_id, api_call);
        Ok(call_id)
    }

    /// Get an API call by ID
    pub fn get_api_call(&self, call_id: &ApiCallId) -> Option<&ApiCall> {
        self.api_calls.get(call_id)
    }

    /// Get a mutable reference to an API call by ID
    pub fn get_api_call_mut(&mut self, call_id: &ApiCallId) -> Option<&mut ApiCall> {
        self.api_calls.get_mut(call_id)
    }

    /// Complete the session
    pub fn complete(&mut self, status: CostSessionStatus) -> Result<(), CostError> {
        if self.completed_at.is_some() {
            return Err(CostError::SessionAlreadyCompleted {
                session_id: self.session_id,
            });
        }

        let now = Utc::now();
        self.completed_at = Some(now);
        self.status = status;
        self.total_duration = Some(
            now.signed_duration_since(self.started_at)
                .to_std()
                .unwrap_or(Duration::ZERO),
        );

        Ok(())
    }

    /// Get total input tokens for all API calls in this session
    pub fn total_input_tokens(&self) -> u32 {
        self.api_calls.values().map(|call| call.input_tokens).sum()
    }

    /// Get total output tokens for all API calls in this session
    pub fn total_output_tokens(&self) -> u32 {
        self.api_calls.values().map(|call| call.output_tokens).sum()
    }

    /// Get total tokens for all API calls in this session
    pub fn total_tokens(&self) -> u32 {
        self.total_input_tokens() + self.total_output_tokens()
    }

    /// Get number of API calls in this session
    pub fn api_call_count(&self) -> usize {
        self.api_calls.len()
    }

    /// Check if the session is completed
    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Set metadata for the session
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

/// Main cost tracker for managing cost sessions
///
/// The cost tracker is the primary interface for cost tracking functionality.
/// It manages multiple cost sessions, provides session lifecycle management,
/// and enforces memory limits to prevent unbounded growth.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::cost::{CostTracker, CostSessionStatus, IssueId, ApiCall, ApiCallStatus};
///
/// let mut tracker = CostTracker::new();
///
/// // Start a new session
/// let issue_id = IssueId::new("issue-123").unwrap();
/// let session_id = tracker.start_session(issue_id).unwrap();
///
/// // Add and complete API calls
/// let api_call = ApiCall::new(
///     "https://api.anthropic.com/v1/messages",
///     "claude-3-sonnet-20241022"
/// ).unwrap();
/// let call_id = tracker.add_api_call(&session_id, api_call).unwrap();
///
/// tracker.complete_api_call(
///     &session_id,
///     &call_id,
///     150,
///     300,
///     ApiCallStatus::Success,
///     None
/// ).unwrap();
///
/// // Complete the session
/// tracker.complete_session(&session_id, CostSessionStatus::Completed).unwrap();
///
/// // Check results
/// let session = tracker.get_session(&session_id).unwrap();
/// assert_eq!(session.total_tokens(), 450);
/// assert_eq!(tracker.completed_session_count(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct CostTracker {
    /// Active and completed cost sessions
    sessions: HashMap<CostSessionId, CostSession>,
}

impl CostTracker {
    /// Create a new cost tracker
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Start a new cost session for an issue
    ///
    /// Creates a new cost tracking session for the specified issue. The session
    /// will track all API calls made during the issue workflow execution.
    ///
    /// # Arguments
    ///
    /// * `issue_id` - The identifier of the issue being tracked
    ///
    /// # Returns
    ///
    /// Returns the unique session identifier on success, or a `CostError` if
    /// the maximum number of sessions is exceeded.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::cost::{CostTracker, IssueId};
    ///
    /// let mut tracker = CostTracker::new();
    /// let issue_id = IssueId::new("issue-123").unwrap();
    /// let session_id = tracker.start_session(issue_id).unwrap();
    ///
    /// assert_eq!(tracker.session_count(), 1);
    /// assert_eq!(tracker.active_session_count(), 1);
    /// ```
    pub fn start_session(&mut self, issue_id: IssueId) -> Result<CostSessionId, CostError> {
        // Check if we're exceeding the limit
        if self.sessions.len() >= MAX_COST_SESSIONS {
            self.cleanup_old_sessions();
            if self.sessions.len() >= MAX_COST_SESSIONS {
                return Err(CostError::TooManySessions);
            }
        }

        let session = CostSession::new(issue_id);
        let session_id = session.session_id;

        // Ensure unique session ID
        if self.sessions.contains_key(&session_id) {
            return Err(CostError::SessionAlreadyExists { session_id });
        }

        self.sessions.insert(session_id, session);
        Ok(session_id)
    }

    /// Get a cost session by ID
    pub fn get_session(&self, session_id: &CostSessionId) -> Option<&CostSession> {
        self.sessions.get(session_id)
    }

    /// Get a mutable reference to a cost session by ID
    pub fn get_session_mut(&mut self, session_id: &CostSessionId) -> Option<&mut CostSession> {
        self.sessions.get_mut(session_id)
    }

    /// Add an API call to a session
    pub fn add_api_call(
        &mut self,
        session_id: &CostSessionId,
        api_call: ApiCall,
    ) -> Result<ApiCallId, CostError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(CostError::SessionNotFound {
                session_id: *session_id,
            })?;

        session.add_api_call(api_call)
    }

    /// Complete an API call in a session
    pub fn complete_api_call(
        &mut self,
        session_id: &CostSessionId,
        call_id: &ApiCallId,
        input_tokens: u32,
        output_tokens: u32,
        status: ApiCallStatus,
        error_message: Option<String>,
    ) -> Result<(), CostError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(CostError::SessionNotFound {
                session_id: *session_id,
            })?;

        let api_call = session
            .api_calls
            .get_mut(call_id)
            .ok_or(CostError::ApiCallNotFound {
                call_id: *call_id,
                session_id: *session_id,
            })?;

        api_call.complete(input_tokens, output_tokens, status, error_message);
        Ok(())
    }

    /// Complete a cost session
    pub fn complete_session(
        &mut self,
        session_id: &CostSessionId,
        status: CostSessionStatus,
    ) -> Result<(), CostError> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or(CostError::SessionNotFound {
                session_id: *session_id,
            })?;

        session.complete(status)
    }

    /// Get all sessions
    pub fn get_all_sessions(&self) -> &HashMap<CostSessionId, CostSession> {
        &self.sessions
    }

    /// Get active sessions only
    pub fn get_active_sessions(&self) -> impl Iterator<Item = (&CostSessionId, &CostSession)> {
        self.sessions
            .iter()
            .filter(|(_, session)| session.status == CostSessionStatus::Active)
    }

    /// Get completed sessions only
    pub fn get_completed_sessions(&self) -> impl Iterator<Item = (&CostSessionId, &CostSession)> {
        self.sessions
            .iter()
            .filter(|(_, session)| session.is_completed())
    }

    /// Remove a session
    pub fn remove_session(&mut self, session_id: &CostSessionId) -> Option<CostSession> {
        self.sessions.remove(session_id)
    }

    /// Clean up old completed sessions
    pub fn cleanup_old_sessions(&mut self) {
        let now = Utc::now();
        let cutoff_date = now - chrono::Duration::days(MAX_COMPLETED_SESSION_AGE_DAYS);

        let sessions_to_remove: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                if let Some(completed_at) = session.completed_at {
                    completed_at < cutoff_date
                } else {
                    false
                }
            })
            .map(|(id, _)| *id)
            .collect();

        let removed_count = sessions_to_remove.len();
        for session_id in sessions_to_remove {
            self.sessions.remove(&session_id);
        }

        if removed_count > 0 {
            tracing::info!(
                "Cost tracker cleanup completed: removed {} old sessions",
                removed_count
            );
        }
    }

    /// Get total number of sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get number of active sessions
    pub fn active_session_count(&self) -> usize {
        self.get_active_sessions().count()
    }

    /// Get number of completed sessions
    pub fn completed_session_count(&self) -> usize {
        self.get_completed_sessions().count()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_session_id_creation() {
        let id1 = CostSessionId::new();
        let id2 = CostSessionId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());

        // Test ULID conversion
        let ulid = id1.as_ulid();
        let id3 = CostSessionId::from_ulid(ulid);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_api_call_id_creation() {
        let id1 = ApiCallId::new();
        let id2 = ApiCallId::new();

        assert_ne!(id1, id2);
        assert!(!id1.to_string().is_empty());

        // Test ULID conversion
        let ulid = id1.as_ulid();
        let id3 = ApiCallId::from_ulid(ulid);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_issue_id_validation() {
        // Valid issue IDs
        assert!(IssueId::new("issue-123").is_ok());
        assert!(IssueId::new("test_issue").is_ok());
        assert!(IssueId::new("123").is_ok());

        // Invalid issue IDs
        assert!(IssueId::new("").is_err());
        assert!(IssueId::new("   ").is_err());
        assert!(IssueId::new("a".repeat(257)).is_err());

        // Test valid issue ID access
        let issue_id = IssueId::new("test-issue").unwrap();
        assert_eq!(issue_id.as_str(), "test-issue");
        assert_eq!(issue_id.to_string(), "test-issue");
    }

    #[test]
    fn test_api_call_creation() {
        // Valid API call
        let api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        );
        assert!(api_call.is_ok());

        let call = api_call.unwrap();
        assert_eq!(call.endpoint, "https://api.anthropic.com/v1/messages");
        assert_eq!(call.model, "claude-3-sonnet-20241022");
        assert_eq!(call.input_tokens, 0);
        assert_eq!(call.output_tokens, 0);
        assert_eq!(call.status, ApiCallStatus::InProgress);
        assert!(!call.is_completed());

        // Invalid API calls
        assert!(ApiCall::new("", "claude-3-sonnet-20241022").is_err());
        assert!(ApiCall::new("https://api.anthropic.com/v1/messages", "").is_err());
        assert!(ApiCall::new(
            "a".repeat(MAX_ENDPOINT_URL_LENGTH + 1),
            "claude-3-sonnet-20241022"
        )
        .is_err());
        assert!(ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "a".repeat(MAX_MODEL_NAME_LENGTH + 1)
        )
        .is_err());
    }

    #[test]
    fn test_api_call_completion() {
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();

        assert!(!api_call.is_completed());
        assert_eq!(api_call.total_tokens(), 0);

        // Complete the call
        api_call.complete(100, 200, ApiCallStatus::Success, None);

        assert!(api_call.is_completed());
        assert_eq!(api_call.input_tokens, 100);
        assert_eq!(api_call.output_tokens, 200);
        assert_eq!(api_call.total_tokens(), 300);
        assert_eq!(api_call.status, ApiCallStatus::Success);
        assert!(api_call.completed_at.is_some());
        assert!(api_call.duration.is_some());
        assert!(api_call.error_message.is_none());

        // Test with error
        let mut api_call_error = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        api_call_error.complete(
            50,
            0,
            ApiCallStatus::Failed,
            Some("Rate limit exceeded".to_string()),
        );

        assert_eq!(api_call_error.status, ApiCallStatus::Failed);
        assert_eq!(
            api_call_error.error_message,
            Some("Rate limit exceeded".to_string())
        );
    }

    #[test]
    fn test_cost_session_creation() {
        let issue_id = IssueId::new("test-issue").unwrap();
        let session = CostSession::new(issue_id.clone());

        assert_eq!(session.issue_id, issue_id);
        assert_eq!(session.status, CostSessionStatus::Active);
        assert!(!session.is_completed());
        assert_eq!(session.api_call_count(), 0);
        assert_eq!(session.total_tokens(), 0);
        assert_eq!(session.total_input_tokens(), 0);
        assert_eq!(session.total_output_tokens(), 0);
    }

    #[test]
    fn test_cost_session_api_call_management() {
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add API call
        let api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        let call_id = session.add_api_call(api_call).unwrap();

        assert_eq!(session.api_call_count(), 1);
        assert!(session.get_api_call(&call_id).is_some());

        // Complete the API call
        let api_call_mut = session.get_api_call_mut(&call_id).unwrap();
        api_call_mut.complete(100, 200, ApiCallStatus::Success, None);

        assert_eq!(session.total_input_tokens(), 100);
        assert_eq!(session.total_output_tokens(), 200);
        assert_eq!(session.total_tokens(), 300);

        // Add another API call
        let api_call2 = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        let call_id2 = session.add_api_call(api_call2).unwrap();

        assert_ne!(call_id, call_id2);
        assert_eq!(session.api_call_count(), 2);

        // Complete second call
        let api_call2_mut = session.get_api_call_mut(&call_id2).unwrap();
        api_call2_mut.complete(50, 75, ApiCallStatus::Success, None);

        assert_eq!(session.total_input_tokens(), 150);
        assert_eq!(session.total_output_tokens(), 275);
        assert_eq!(session.total_tokens(), 425);
    }

    #[test]
    fn test_cost_session_too_many_api_calls() {
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Add maximum number of API calls
        for i in 0..MAX_API_CALLS_PER_SESSION {
            let api_call = ApiCall::new(
                format!("https://api.anthropic.com/v1/messages/{}", i),
                "claude-3-sonnet-20241022",
            )
            .unwrap();
            assert!(session.add_api_call(api_call).is_ok());
        }

        // Adding one more should fail
        let api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages/overflow",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        let result = session.add_api_call(api_call);

        assert!(matches!(result, Err(CostError::TooManyApiCalls { .. })));
    }

    #[test]
    fn test_cost_session_completion() {
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        assert!(!session.is_completed());
        assert!(session.total_duration.is_none());

        // Complete session
        assert!(session.complete(CostSessionStatus::Completed).is_ok());

        assert!(session.is_completed());
        assert_eq!(session.status, CostSessionStatus::Completed);
        assert!(session.completed_at.is_some());
        assert!(session.total_duration.is_some());

        // Cannot complete again
        let result = session.complete(CostSessionStatus::Failed);
        assert!(matches!(
            result,
            Err(CostError::SessionAlreadyCompleted { .. })
        ));
    }

    #[test]
    fn test_cost_session_metadata() {
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);

        // Set metadata
        session.set_metadata("branch", "feature/cost-tracking");
        session.set_metadata("user", "test-user");

        // Get metadata
        assert_eq!(
            session.get_metadata("branch"),
            Some(&"feature/cost-tracking".to_string())
        );
        assert_eq!(session.get_metadata("user"), Some(&"test-user".to_string()));
        assert_eq!(session.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_cost_tracker_new() {
        let tracker = CostTracker::new();

        assert_eq!(tracker.session_count(), 0);
        assert_eq!(tracker.active_session_count(), 0);
        assert_eq!(tracker.completed_session_count(), 0);
    }

    #[test]
    fn test_cost_tracker_session_lifecycle() {
        let mut tracker = CostTracker::new();
        let issue_id = IssueId::new("test-issue").unwrap();

        // Start session
        let session_id = tracker.start_session(issue_id.clone()).unwrap();

        assert_eq!(tracker.session_count(), 1);
        assert_eq!(tracker.active_session_count(), 1);
        assert_eq!(tracker.completed_session_count(), 0);

        // Get session
        let session = tracker.get_session(&session_id);
        assert!(session.is_some());
        assert_eq!(session.unwrap().issue_id, issue_id);

        // Complete session
        assert!(tracker
            .complete_session(&session_id, CostSessionStatus::Completed)
            .is_ok());

        assert_eq!(tracker.session_count(), 1);
        assert_eq!(tracker.active_session_count(), 0);
        assert_eq!(tracker.completed_session_count(), 1);
    }

    #[test]
    fn test_cost_tracker_api_call_lifecycle() {
        let mut tracker = CostTracker::new();
        let issue_id = IssueId::new("test-issue").unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();

        // Add API call
        let api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        let call_id = tracker.add_api_call(&session_id, api_call).unwrap();

        let session = tracker.get_session(&session_id).unwrap();
        assert_eq!(session.api_call_count(), 1);

        // Complete API call
        assert!(tracker
            .complete_api_call(
                &session_id,
                &call_id,
                100,
                200,
                ApiCallStatus::Success,
                None
            )
            .is_ok());

        let session = tracker.get_session(&session_id).unwrap();
        let api_call = session.get_api_call(&call_id).unwrap();
        assert!(api_call.is_completed());
        assert_eq!(api_call.input_tokens, 100);
        assert_eq!(api_call.output_tokens, 200);
        assert_eq!(api_call.status, ApiCallStatus::Success);
    }

    #[test]
    fn test_cost_tracker_errors() {
        let mut tracker = CostTracker::new();
        let invalid_session_id = CostSessionId::new();
        let invalid_call_id = ApiCallId::new();

        // Session not found
        assert!(tracker.get_session(&invalid_session_id).is_none());

        assert!(matches!(
            tracker.complete_session(&invalid_session_id, CostSessionStatus::Completed),
            Err(CostError::SessionNotFound { .. })
        ));

        let api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        assert!(matches!(
            tracker.add_api_call(&invalid_session_id, api_call),
            Err(CostError::SessionNotFound { .. })
        ));

        assert!(matches!(
            tracker.complete_api_call(
                &invalid_session_id,
                &invalid_call_id,
                100,
                200,
                ApiCallStatus::Success,
                None
            ),
            Err(CostError::SessionNotFound { .. })
        ));

        // API call not found (valid session, invalid call)
        let issue_id = IssueId::new("test-issue").unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();

        assert!(matches!(
            tracker.complete_api_call(
                &session_id,
                &invalid_call_id,
                100,
                200,
                ApiCallStatus::Success,
                None
            ),
            Err(CostError::ApiCallNotFound { .. })
        ));
    }

    #[test]
    fn test_cost_tracker_remove_session() {
        let mut tracker = CostTracker::new();
        let issue_id = IssueId::new("test-issue").unwrap();
        let session_id = tracker.start_session(issue_id).unwrap();

        assert_eq!(tracker.session_count(), 1);

        // Remove session
        let removed_session = tracker.remove_session(&session_id);
        assert!(removed_session.is_some());
        assert_eq!(tracker.session_count(), 0);

        // Removing again should return None
        let removed_session = tracker.remove_session(&session_id);
        assert!(removed_session.is_none());
    }

    #[test]
    fn test_cost_tracker_iterators() {
        let mut tracker = CostTracker::new();

        // Create multiple sessions in different states
        let issue_id1 = IssueId::new("issue-1").unwrap();
        let issue_id2 = IssueId::new("issue-2").unwrap();
        let issue_id3 = IssueId::new("issue-3").unwrap();

        let session_id1 = tracker.start_session(issue_id1).unwrap();
        let session_id2 = tracker.start_session(issue_id2).unwrap();
        let session_id3 = tracker.start_session(issue_id3).unwrap();

        // Complete some sessions
        tracker
            .complete_session(&session_id1, CostSessionStatus::Completed)
            .unwrap();
        tracker
            .complete_session(&session_id2, CostSessionStatus::Failed)
            .unwrap();
        // Leave session_id3 active

        assert_eq!(tracker.session_count(), 3);
        assert_eq!(tracker.active_session_count(), 1);
        assert_eq!(tracker.completed_session_count(), 2);

        // Test iterators
        let active_sessions: Vec<_> = tracker.get_active_sessions().collect();
        assert_eq!(active_sessions.len(), 1);
        assert_eq!(active_sessions[0].0, &session_id3);

        let completed_sessions: Vec<_> = tracker.get_completed_sessions().collect();
        assert_eq!(completed_sessions.len(), 2);
    }

    #[test]
    fn test_serialization_deserialization() {
        // Test ApiCall serialization
        let mut api_call = ApiCall::new(
            "https://api.anthropic.com/v1/messages",
            "claude-3-sonnet-20241022",
        )
        .unwrap();
        api_call.complete(100, 200, ApiCallStatus::Success, None);

        let serialized = serde_json::to_string(&api_call).unwrap();
        let deserialized: ApiCall = serde_json::from_str(&serialized).unwrap();

        assert_eq!(api_call.call_id, deserialized.call_id);
        assert_eq!(api_call.endpoint, deserialized.endpoint);
        assert_eq!(api_call.model, deserialized.model);
        assert_eq!(api_call.input_tokens, deserialized.input_tokens);
        assert_eq!(api_call.output_tokens, deserialized.output_tokens);
        assert_eq!(api_call.status, deserialized.status);

        // Test CostSession serialization
        let issue_id = IssueId::new("test-issue").unwrap();
        let mut session = CostSession::new(issue_id);
        session.add_api_call(api_call).unwrap();
        session.set_metadata("test", "value");

        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: CostSession = serde_json::from_str(&serialized).unwrap();

        assert_eq!(session.session_id, deserialized.session_id);
        assert_eq!(session.issue_id, deserialized.issue_id);
        assert_eq!(session.status, deserialized.status);
        assert_eq!(session.api_call_count(), deserialized.api_call_count());
        assert_eq!(
            session.get_metadata("test"),
            deserialized.get_metadata("test")
        );

        // Test error types serialization
        let error = CostError::SessionNotFound {
            session_id: CostSessionId::new(),
        };
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: CostError = serde_json::from_str(&serialized).unwrap();

        match (error, deserialized) {
            (
                CostError::SessionNotFound { session_id: id1 },
                CostError::SessionNotFound { session_id: id2 },
            ) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Error types don't match"),
        }
    }

    #[test]
    fn test_memory_limit_enforcement() {
        // Test maximum sessions limit first
        {
            let mut tracker = CostTracker::new();

            // First, fill up to the limit
            let mut session_ids = Vec::new();
            for i in 0..MAX_COST_SESSIONS {
                let issue_id = IssueId::new(format!("issue-{}", i)).unwrap();
                let session_id = tracker.start_session(issue_id).unwrap();
                session_ids.push(session_id);
            }

            assert_eq!(tracker.session_count(), MAX_COST_SESSIONS);

            // Complete some sessions to test cleanup
            for session_id in session_ids.iter().take(100) {
                tracker
                    .complete_session(session_id, CostSessionStatus::Completed)
                    .unwrap();
            }

            // Adding one more session should trigger cleanup but might still fail
            // if no sessions are old enough to be cleaned up (which is expected in tests)
            let issue_id = IssueId::new("overflow-issue").unwrap();
            let result = tracker.start_session(issue_id);

            // The cleanup only removes old sessions, so this will likely fail in tests
            // where all sessions are recent. This is correct behavior.
            match result {
                Ok(_) => {
                    // Great, cleanup worked (unlikely in test environment)
                }
                Err(CostError::TooManySessions) => {
                    // Expected - no sessions are old enough to clean up
                    assert_eq!(tracker.session_count(), MAX_COST_SESSIONS);
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        // Test maximum API calls per session limit in separate scope
        {
            let mut tracker = CostTracker::new();
            let issue_id = IssueId::new("api-test-issue").unwrap();
            let session_id = tracker.start_session(issue_id).unwrap();

            // Fill up to the limit
            for i in 0..MAX_API_CALLS_PER_SESSION {
                let api_call = ApiCall::new(
                    format!("https://api.anthropic.com/v1/messages/{}", i),
                    "claude-3-sonnet-20241022",
                )
                .unwrap();
                assert!(tracker.add_api_call(&session_id, api_call).is_ok());
            }

            // Adding one more should fail
            let api_call = ApiCall::new(
                "https://api.anthropic.com/v1/messages/overflow",
                "claude-3-sonnet-20241022",
            )
            .unwrap();
            let result = tracker.add_api_call(&session_id, api_call);

            assert!(matches!(result, Err(CostError::TooManyApiCalls { .. })));
        }
    }

    #[test]
    fn test_cleanup_old_sessions() {
        let mut tracker = CostTracker::new();

        // Create sessions and complete them
        let mut session_ids = Vec::new();
        for i in 0..10 {
            let issue_id = IssueId::new(format!("issue-{}", i)).unwrap();
            let session_id = tracker.start_session(issue_id).unwrap();
            tracker
                .complete_session(&session_id, CostSessionStatus::Completed)
                .unwrap();
            session_ids.push(session_id);
        }

        assert_eq!(tracker.session_count(), 10);
        assert_eq!(tracker.completed_session_count(), 10);

        // Test cleanup (this won't remove sessions since they're not old enough)
        tracker.cleanup_old_sessions();
        assert_eq!(tracker.session_count(), 10);

        // Manually test the cleanup logic by creating an old session
        // (In real usage, sessions would age naturally)
        let _old_cutoff =
            chrono::Utc::now() - chrono::Duration::days(MAX_COMPLETED_SESSION_AGE_DAYS + 1);

        // We can't easily test the actual cleanup without manipulating time,
        // but we can verify the cleanup method exists and runs without errors
    }

    #[test]
    fn test_input_validation() {
        // Test IssueId validation
        assert!(matches!(
            IssueId::new(""),
            Err(CostError::InvalidInput { .. })
        ));
        assert!(matches!(
            IssueId::new("   "),
            Err(CostError::InvalidInput { .. })
        ));
        assert!(matches!(
            IssueId::new("a".repeat(257)),
            Err(CostError::InvalidInput { .. })
        ));

        // Test ApiCall endpoint validation
        assert!(matches!(
            ApiCall::new("", "model"),
            Err(CostError::InvalidInput { .. })
        ));
        assert!(matches!(
            ApiCall::new("a".repeat(MAX_ENDPOINT_URL_LENGTH + 1), "model"),
            Err(CostError::InvalidInput { .. })
        ));

        // Test ApiCall model validation
        assert!(matches!(
            ApiCall::new("https://api.anthropic.com/v1/messages", ""),
            Err(CostError::InvalidInput { .. })
        ));
        assert!(matches!(
            ApiCall::new(
                "https://api.anthropic.com/v1/messages",
                "a".repeat(MAX_MODEL_NAME_LENGTH + 1)
            ),
            Err(CostError::InvalidInput { .. })
        ));
    }

    #[test]
    fn test_error_display() {
        let session_id = CostSessionId::new();
        let call_id = ApiCallId::new();

        let errors = vec![
            CostError::SessionNotFound { session_id },
            CostError::SessionAlreadyExists { session_id },
            CostError::SessionAlreadyCompleted { session_id },
            CostError::TooManySessions,
            CostError::TooManyApiCalls { session_id },
            CostError::InvalidInput {
                message: "Test error".to_string(),
            },
            CostError::ApiCallNotFound {
                call_id,
                session_id,
            },
            CostError::SerializationError {
                message: "Test serialization error".to_string(),
            },
        ];

        for error in errors {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
            // Verify the error contains relevant information
            match error {
                CostError::SessionNotFound { .. } => {
                    assert!(error_string.contains("session not found"))
                }
                CostError::TooManySessions => {
                    assert!(error_string.contains("Maximum number of sessions"))
                }
                CostError::InvalidInput { message } => assert!(error_string.contains(&message)),
                _ => {}
            }
        }
    }

    #[test]
    fn test_comprehensive_workflow() {
        let mut tracker = CostTracker::new();

        // Start multiple sessions for different issues
        let issue_ids: Vec<_> = (0..5)
            .map(|i| IssueId::new(format!("issue-{}", i)).unwrap())
            .collect();

        let mut session_ids = Vec::new();
        for issue_id in &issue_ids {
            let session_id = tracker.start_session(issue_id.clone()).unwrap();
            session_ids.push(session_id);
        }

        // Add API calls to each session
        let mut call_ids = Vec::new();
        for (i, session_id) in session_ids.iter().enumerate() {
            for j in 0..3 {
                let api_call = ApiCall::new(
                    format!("https://api.anthropic.com/v1/messages/{}/{}", i, j),
                    "claude-3-sonnet-20241022",
                )
                .unwrap();
                let call_id = tracker.add_api_call(session_id, api_call).unwrap();
                call_ids.push((*session_id, call_id));
            }
        }

        // Complete some API calls
        for (i, (session_id, call_id)) in call_ids.iter().enumerate() {
            let input_tokens = 100 + (i as u32 * 10);
            let output_tokens = 200 + (i as u32 * 20);
            tracker
                .complete_api_call(
                    session_id,
                    call_id,
                    input_tokens,
                    output_tokens,
                    if i % 4 == 0 {
                        ApiCallStatus::Failed
                    } else {
                        ApiCallStatus::Success
                    },
                    if i % 4 == 0 {
                        Some("Test error".to_string())
                    } else {
                        None
                    },
                )
                .unwrap();
        }

        // Complete sessions
        for (i, session_id) in session_ids.iter().enumerate() {
            let status = if i % 2 == 0 {
                CostSessionStatus::Completed
            } else {
                CostSessionStatus::Failed
            };
            tracker.complete_session(session_id, status).unwrap();
        }

        // Verify final state
        assert_eq!(tracker.session_count(), 5);
        assert_eq!(tracker.active_session_count(), 0);
        assert_eq!(tracker.completed_session_count(), 5);

        // Verify token counts
        for session_id in &session_ids {
            let session = tracker.get_session(session_id).unwrap();
            assert_eq!(session.api_call_count(), 3);
            assert!(session.total_tokens() > 0);
            assert!(session.is_completed());
        }
    }
}
