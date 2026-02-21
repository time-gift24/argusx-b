use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message", rename_all = "camelCase")]
pub enum ModelError {
    InvalidRequest(String),
    AuthenticationError(String),
    RateLimitError(String),
    ServerError(String),
    NetworkError(String),
    ParseError(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            ModelError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            ModelError::RateLimitError(msg) => write!(f, "Rate limit exceeded: {}", msg),
            ModelError::ServerError(msg) => write!(f, "Server error: {}", msg),
            ModelError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ModelError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

pub type Result<T> = std::result::Result<T, ModelError>;

// Additional error types for other traits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message", rename_all = "camelCase")]
pub enum SessionError {
    Session(Option<String>),
    MaxTurnsExceeded,
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::Session(Some(msg)) => write!(f, "Session error: {}", msg),
            SessionError::Session(None) => write!(f, "Session error"),
            SessionError::MaxTurnsExceeded => write!(f, "Max turns exceeded"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<ModelError> for SessionError {
    fn from(err: ModelError) -> Self {
        SessionError::Session(Some(err.to_string()))
    }
}

impl From<SessionError> for ModelError {
    fn from(err: SessionError) -> Self {
        ModelError::ParseError(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum ToolExecutionError {
    #[error("Tool execution error: {0}")]
    Execution(String),
    #[error("Tool not found: {0}")]
    NotFound(String),
}

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory error: {0}")]
    Memory(String),
}

#[derive(Error, Debug)]
pub enum PlanError {
    #[error("Plan error: {0}")]
    Plan(String),
}

#[derive(Error, Debug)]
pub enum PlanValidationError {
    #[error("Plan validation error: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
pub enum ApprovalError {
    #[error("Approval error: {0}")]
    Approval(String),
}

#[derive(Error, Debug)]
pub enum ResumeError {
    #[error("Resume error: {0}")]
    Resume(String),
}
