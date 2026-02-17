use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, ModelError>;

// Additional error types for other traits
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session error: {0}")]
    Session(String),
    #[error("Max turns exceeded")]
    MaxTurnsExceeded,
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
