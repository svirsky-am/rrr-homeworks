// src/presentation/middleware.rs
use crate::domain::DomainError;
use crate::infrastructure::JwtService;
use actix_web::{Error, HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<JwtService>>() {
        Some(service) => service,
        None => {
            let err = ErrorUnauthorized("JWT service not configured");
            return Err((err, req));
        }
    };

    let token = credentials.token();

    let claims = match jwt_service.verify_token(token) {
        Ok(c) => c,
        Err(_) => {
            let err = ErrorUnauthorized("Invalid token");
            return Err((err, req));
        }
    };

    let auth_user = AuthenticatedUser {
        user_id: claims.user_id,
        username: claims.username,
    };

    req.extensions_mut().insert(auth_user);
    Ok(req)
}

/// Helper для извлечения пользователя из запроса
pub fn get_authenticated_user(
    req: &actix_web::HttpRequest,
) -> Result<AuthenticatedUser, DomainError> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or(DomainError::Forbidden)
}
