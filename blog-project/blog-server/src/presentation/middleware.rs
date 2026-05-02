// src/presentation/middleware.rs
use actix_web::{
    dev::ServiceRequest,
    error::ErrorUnauthorized,
    Error, HttpMessage, web,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::infrastructure::JwtService;
use crate::domain::DomainError;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

// ✅ Исправленная функция: используем match для избежания borrow/move конфликта
pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    
    // ✅ 1. Извлекаем jwt_service через match (без closure, который захватывает req)
    let jwt_service = match req.app_data::<web::Data<JwtService>>() {
        Some(service) => service,
        None => {
            let err = ErrorUnauthorized("JWT service not configured");
            return Err((err, req));  // ✅ Возвращаем сразу — нет конфликта заимствований
        }
    };

    let token = credentials.token();

    // ✅ 2. Проверяем токен
    let claims = match jwt_service.verify_token(token) {
        Ok(c) => c,
        Err(_) => {
            let err = ErrorUnauthorized("Invalid token");
            return Err((err, req));  // ✅ Возвращаем сразу
        }
    };

    let auth_user = AuthenticatedUser {
        user_id: claims.user_id,
        username: claims.username,
    };

    // ✅ 3. req больше не заимствован — можно безопасно модифицировать
    req.extensions_mut().insert(auth_user);
    Ok(req)
}

/// Helper для извлечения пользователя из запроса
pub fn get_authenticated_user(req: &actix_web::HttpRequest) -> Result<AuthenticatedUser, DomainError> {
    req.extensions()
        .get::<AuthenticatedUser>()
        .cloned()
        .ok_or(DomainError::Forbidden)
}

