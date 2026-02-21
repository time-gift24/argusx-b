use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransientError {
    #[error("network error: {message}")]
    Network {
        message: String,
        retry_after_ms: Option<u64>,
    },
    #[error("rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        retry_after_ms: Option<u64>,
    },
    #[error("service unavailable: {message}")]
    ServiceUnavailable {
        message: String,
        retry_after_ms: Option<u64>,
    },
}

impl TransientError {
    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            TransientError::Network { retry_after_ms, .. }
            | TransientError::RateLimit { retry_after_ms, .. }
            | TransientError::ServiceUnavailable { retry_after_ms, .. } => *retry_after_ms,
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuntimeError {
    #[error("turn already exists: {turn_id}")]
    TurnAlreadyExists { turn_id: String },
    #[error("turn not found: {turn_id}")]
    TurnNotFound { turn_id: String },
    #[error("invalid event: {message}")]
    InvalidEvent { message: String },
    #[error("protocol violation: {message}")]
    ProtocolViolation { message: String },
    #[error("cancelled: {message}")]
    Cancelled { message: String },
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error(transparent)]
    Transient(#[from] TransientError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error("tool error: {message}")]
    Tool { message: String },
    #[error("model error: {message}")]
    Model { message: String },
    #[error("checkpoint error: {message}")]
    Checkpoint { message: String },
    #[error("internal error: {message}")]
    Internal { message: String },
}

impl AgentError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, AgentError::Transient(_))
    }
}
