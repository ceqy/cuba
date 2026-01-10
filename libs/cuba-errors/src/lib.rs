#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Internal error")]
    Internal,
}
