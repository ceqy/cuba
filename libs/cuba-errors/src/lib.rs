use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Messaging error: {0}")]
    MessagingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(#[from] config::ConfigError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Item not found")]
    NotFound,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
}
