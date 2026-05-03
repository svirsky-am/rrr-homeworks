use thiserror::Error;



//  это можно было бы  взять из сервера 
#[derive(Debug, Error)]
pub enum BlogClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
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
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, BlogClientError>;