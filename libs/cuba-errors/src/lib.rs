use thiserror::Error;
use tonic::{Code, Status};

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

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Item already exists: {0}")]
    AlreadyExists(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
}

impl From<ServiceError> for Status {
    fn from(error: ServiceError) -> Self {
        match error {
            ServiceError::DatabaseError(e) => {
                // Check if it's a unique constraint violation
                if let Some(db_err) = e.as_database_error() {
                     if db_err.code().as_deref() == Some("23505") { // Unique violation
                         return Status::already_exists(e.to_string());
                     }
                }
                Status::internal(format!("Database error: {}", e))
            },
            ServiceError::MessagingError(e) => Status::internal(format!("Messaging error: {}", e)),
            ServiceError::ConfigurationError(e) => Status::internal(format!("Config error: {}", e)),
            ServiceError::InvalidInput(e) => Status::invalid_argument(e),
            ServiceError::ValidationError(e) => Status::invalid_argument(e),
            ServiceError::NotFound(e) => Status::not_found(e),
            ServiceError::AlreadyExists(e) => Status::already_exists(e),
            ServiceError::Unauthorized(e) => Status::unauthenticated(e),
            ServiceError::Forbidden(e) => Status::permission_denied(e),
            ServiceError::Conflict(e) => Status::failed_precondition(e),
            ServiceError::InternalServerError(e) => Status::internal(format!("Internal error: {}", e)),
        }
    }
}
