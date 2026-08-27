//! Error types for the entity crate.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for entity operations.
pub type Result<T> = std::result::Result<T, EntityError>;

/// Errors that can occur in entity operations.
#[derive(Debug, Error)]
pub enum EntityError {
    /// Entity file not found.
    #[error("entity not found: {entity_type}/{id}")]
    NotFound {
        /// The entity type that was read.
        entity_type: String,
        /// The id of the entity that does not exist.
        id: String,
    },

    /// Missing frontmatter delimiters in a body-field entity.
    #[error("invalid frontmatter in {path}: expected --- delimiters")]
    InvalidFrontmatter {
        /// The file that holds no frontmatter delimiters.
        path: PathBuf,
    },

    /// A field value emitted a bare `---` line into the frontmatter, so the
    /// entity could not be written without destroying it on the next read.
    #[error("cannot write {path}: {source}")]
    FrontmatterDelimiter {
        /// The file the write was refused for.
        path: PathBuf,
        /// The delimiter line that made the write unsafe.
        source: swissarmyhammer_common::frontmatter::DelimiterInFrontmatter,
    },

    /// YAML parse error.
    #[error("YAML error in {path}: {source}")]
    Yaml {
        /// The file whose YAML could not be read.
        path: PathBuf,
        /// The YAML error from the parser.
        source: serde_yaml_ng::Error,
    },

    /// Unknown entity type (not defined in FieldsContext).
    #[error("unknown entity type: {entity_type}")]
    UnknownEntityType {
        /// The entity type name that no `FieldsContext` defines.
        entity_type: String,
    },

    /// Field validation failed.
    #[error("validation failed for field '{field}': {message}")]
    ValidationFailed {
        /// The name of the field that failed validation.
        field: String,
        /// The reason the value is not valid.
        message: String,
    },

    /// Computed field derivation failed.
    #[error("compute error for field '{field}': {message}")]
    ComputeError {
        /// The name of the computed field.
        field: String,
        /// The reason the value could not be computed.
        message: String,
    },

    /// A text diff patch could not be parsed or applied.
    #[error("patch apply error: {0}")]
    PatchApply(String),

    /// A non-string field change is stale: the entity's current value does not
    /// match the expected value from the changelog entry.
    #[error("stale change on field '{field}': expected {expected}, found {actual}")]
    StaleChange {
        /// The name of the field the change applies to.
        field: String,
        /// The value the changelog entry expected to find.
        expected: serde_json::Value,
        /// The value the entity holds now.
        actual: serde_json::Value,
    },

    /// An undo or redo was attempted on an unsupported operation type
    /// (e.g. trying to undo an "undo" or "redo" entry directly).
    #[error("unsupported undo/redo operation type: '{op}'")]
    UnsupportedUndoOp {
        /// The name of the operation that undo and redo do not support.
        op: String,
    },

    /// A changelog ULID was not found in the index.
    #[error("changelog entry not found: {ulid}")]
    ChangelogEntryNotFound {
        /// The ULID of the changelog entry the index does not hold.
        ulid: String,
    },

    /// Transaction undo/redo failed partway through. Rollback was attempted.
    ///
    /// When a multi-entry transaction undo or redo fails on one entry after
    /// some entries have already been reversed, the system attempts to roll
    /// back the completed entries to restore consistency. This error reports
    /// both the original failure and whether rollback succeeded.
    #[error(
        "transaction partial failure on entry {failed_entry}: {original_error} \
         (completed {completed_count} entries, rollback {rollback_status})",
        completed_count = completed.len(),
        rollback_status = if *rollback_succeeded { "succeeded" } else { "failed" }
    )]
    TransactionPartialFailure {
        /// The original error that caused the failure.
        original_error: String,
        /// Entry ULIDs that were successfully reversed before the failure.
        completed: Vec<String>,
        /// The entry ULID that failed.
        failed_entry: String,
        /// Whether rollback of completed entries succeeded.
        rollback_succeeded: bool,
    },

    /// Cannot restore from trash because the data file is missing.
    #[error("cannot restore from trash: data file not found at {path}")]
    RestoreFromTrashFailed {
        /// The data file in the trash that does not exist.
        path: PathBuf,
    },

    /// A path required to have a parent directory or a filename component,
    /// but did not (e.g. the filesystem root, or an empty path).
    #[error("invalid path {path}: {reason}")]
    InvalidPath {
        /// The path that is not valid.
        path: PathBuf,
        /// The component the path does not have.
        reason: String,
    },

    /// Attachment source file not found.
    #[error("attachment source file not found: {path}")]
    AttachmentSourceNotFound {
        /// The source file the attachment was to be copied from.
        path: PathBuf,
    },

    /// Enriched attachment object references a file that no longer exists.
    #[error("attachment file not found for field '{field}': {filename}")]
    AttachmentNotFound {
        /// The name of the attachment field.
        field: String,
        /// The name of the attachment file that does not exist.
        filename: String,
    },

    /// Attachment file exceeds max size.
    #[error(
        "attachment file too large for field '{field}': {size} bytes exceeds max {max_bytes} bytes"
    )]
    AttachmentTooLarge {
        /// The name of the attachment field.
        field: String,
        /// The size of the attachment file, in bytes.
        size: u64,
        /// The largest size the field accepts, in bytes.
        max_bytes: u64,
    },

    /// YAML serialization/deserialization error (without file path context).
    #[error("YAML error: {0}")]
    YamlSerde(#[from] serde_yaml_ng::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// An error from the underlying `StoreHandle`.
    #[error("store error: {0}")]
    Store(#[from] swissarmyhammer_store::StoreError),
}
