use thiserror::Error;
use tonic::codegen::http::uri::InvalidUri;

#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("gRPC status: {0}")]
    Grpc(#[from] tonic::Status),
    #[error("gRPC transport: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] InvalidUri),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, BlogClientError>;
