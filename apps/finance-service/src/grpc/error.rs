use tonic::Status;
use cuba_errors::ServiceError;

pub fn map_service_error(err: ServiceError) -> Status {
    match err {
        ServiceError::NotFound => Status::not_found("not found"),
        ServiceError::Unauthorized(msg) => Status::unauthenticated(msg),
        ServiceError::InvalidInput(msg) => Status::invalid_argument(msg),
        ServiceError::DatabaseError(e) => {
            tracing::error!("Database error: {:?}", e);
            Status::internal("internal database error")
        }
        ServiceError::InternalServerError(e) => {
            tracing::error!("Internal error: {:?}", e);
            Status::internal("internal server error")
        }
        _ => Status::internal("unknown internal error"),
    }
}
