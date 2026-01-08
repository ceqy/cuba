use cuba_errors::ServiceError;
use tonic::Status;

pub fn map_service_error(err: ServiceError) -> Status {
    match err {
        ServiceError::NotFound => Status::not_found("Resource not found"),
        ServiceError::InvalidInput(msg) => Status::invalid_argument(msg),
        ServiceError::Unauthorized(msg) => Status::unauthenticated(msg),
        ServiceError::InternalServerError(e) => Status::internal(e.to_string()),
        ServiceError::DatabaseError(e) => Status::internal(format!("Database error: {}", e)),
        _ => Status::internal("An internal error occurred"),
    }
}
