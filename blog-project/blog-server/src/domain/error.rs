use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Post not found")]
    PostNotFound,
    #[error("Forbidden: not the author")]
    Forbidden,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Hash error: {0}")]
    Hash(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Internal: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ResponseError for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            DomainError::UserNotFound | DomainError::PostNotFound => StatusCode::NOT_FOUND,
            DomainError::UserAlreadyExists => StatusCode::CONFLICT,
            DomainError::InvalidCredentials | DomainError::Forbidden => StatusCode::UNAUTHORIZED,
            DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            DomainError::Database(_) | DomainError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: match self {
                DomainError::UserNotFound => "user_not_found",
                DomainError::UserAlreadyExists => "user_already_exists",
                DomainError::InvalidCredentials => "invalid_credentials",
                DomainError::PostNotFound => "post_not_found",
                DomainError::Forbidden => "forbidden",
                DomainError::Validation(_) => "validation_error",
                DomainError::Database(_) => "database_error",
                DomainError::Jwt(_) => "jwt_error",
                DomainError::Hash(_) => "hash_error",
                DomainError::Internal(_) => "internal_error",
            }
            .to_string(),
            message: self.to_string(),
        })
    }
}

pub type AppResult<T> = Result<T, DomainError>;
