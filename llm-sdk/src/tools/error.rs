//! Tool-related error types

use thiserror::Error;

/// Errors that can occur during tool execution
#[derive(Error, Debug)]
pub enum ToolExecutionError {
    /// Failed to parse tool arguments
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),

    /// Tool handler not found
    #[error("tool not found: {0}")]
    ToolNotFound(String),

    /// Payload type mismatch
    #[error("payload type mismatch: expected {expected}, got {actual}")]
    PayloadMismatch {
        expected: String,
        actual: String,
    },

    /// File operation error
    #[error("file error: {0}")]
    FileError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Approval denied
    #[error("approval denied: {0}")]
    ApprovalDenied(String),

    /// Execution failed
    #[error("execution failed: {0}")]
    ExecutionFailed(String),

    /// Internal error (for errors that should be reported to the model)
    #[error("{0}")]
    RespondToModel(String),

    /// Fatal error (should stop execution)
    #[error("fatal error: {0}")]
    Fatal(String),
}

/// Legacy error type alias for backwards compatibility
pub type ToolError = ToolExecutionError;

impl ToolExecutionError {
    /// Create an error that should be reported back to the model
    pub fn respond_to_model<S: Into<String>>(message: S) -> Self {
        ToolExecutionError::RespondToModel(message.into())
    }

    /// Create a fatal error
    pub fn fatal<S: Into<String>>(message: S) -> Self {
        ToolExecutionError::Fatal(message.into())
    }
}
