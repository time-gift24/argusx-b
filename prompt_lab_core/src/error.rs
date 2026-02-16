use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptLabError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid value for {field}: {value}")]
    InvalidEnum { field: &'static str, value: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("{entity} with id={id} was not found")]
    NotFound { entity: &'static str, id: i64 },
}

pub type Result<T> = std::result::Result<T, PromptLabError>;
