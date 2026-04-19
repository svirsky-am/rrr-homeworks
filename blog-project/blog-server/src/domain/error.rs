use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("insufficient funds on account {0}")]
    InsufficientFunds(i32),
    #[error("account not found: {0}")]
    AccountNotFound(i32),
    #[error("user not found: {0}")]
    UserNotFound(Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum BankError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("insufficient funds on account {0}")]
    InsufficientFunds(i32),
    #[error("internal server error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl ResponseError for BankError {
    fn status_code(&self) -> StatusCode {
        match self {
            BankError::Validation(_) => StatusCode::BAD_REQUEST,
            BankError::NotFound(_) => StatusCode::NOT_FOUND,
            BankError::Unauthorized => StatusCode::UNAUTHORIZED,
            BankError::InsufficientFunds(_) => StatusCode::BAD_REQUEST,
            BankError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = self.to_string();
        let details = match self {
            BankError::Validation(msg) => Some(json!({ "message": msg })),
            BankError::NotFound(resource) => Some(json!({ "resource": resource })),
            BankError::Unauthorized => None,
            BankError::InsufficientFunds(account) => {
                Some(json!({ "account_id": account, "reason": "insufficient_funds" }))
            }
            BankError::Internal(_) => None,
        };
        let body = ErrorBody {
            error: &message,
            details,
        };
        HttpResponse::build(self.status_code()).json(body)
    }
}

impl From<DomainError> for BankError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::Validation(msg) => BankError::Validation(msg),
            DomainError::InsufficientFunds(acc) => BankError::InsufficientFunds(acc),
            DomainError::AccountNotFound(acc) => BankError::NotFound(format!("account {}", acc)),
            DomainError::UserNotFound(id) => BankError::NotFound(format!("user {}", id)),
            DomainError::Internal(msg) => BankError::Internal(msg),
        }
    }
}

