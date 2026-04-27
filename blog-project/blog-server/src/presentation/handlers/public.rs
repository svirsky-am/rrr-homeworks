use actix_web::{post, web, HttpResponse, Responder, Scope};
use chrono::Utc;
use tracing::info;

use crate::application::auth_service::AuthService;
use crate::data::user_repository::{PostgresUserRepository, UserRepository};
use crate::domain::BlogError;
use crate::presentation::dto::{HealthResponse, LoginRequest, RegisterRequest, TokenResponse, UserResponse};

pub fn scope() -> Scope {
    web::scope("")
        .route("/health", web::get().to(health))
        .service(register)
        .service(login)
        .service(token)
}

async fn health() -> impl Responder {
    HttpResponse::Ok().json(HealthResponse {
        status: "ok",
        timestamp: Utc::now(),
    })
}

#[post("/auth/register")]
async fn register(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<RegisterRequest>,
) -> Result<impl Responder, BlogError> {
    let user = service
        .register(payload.username.clone(), payload.email.clone(), payload.password.clone())
        .await?;

    info!(user_id = %user.id, email = %user.email, "user registered");

    let jwt_token = service.keys().generate_token(user.id)
        .map_err(|err| BlogError::Internal(err.to_string()))?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "token": jwt_token,
        "user": UserResponse::from(user)
    })))
}

#[post("/auth/login")]
async fn login(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, BlogError> {
    let jwt = service.login(&payload.username, &payload.password).await?;
    let user = service.repo.find_by_username(&payload.username.to_lowercase())
        .await
        .map_err(BlogError::from)?
        .ok_or_else(|| BlogError::InvalidCredentials)?;
    
    info!(username = %payload.username, "user logged in");
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": jwt,
        "user": UserResponse::from(user)
    })))
}

#[post("/auth/token")]
async fn token(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, BlogError> {
    let jwt = service.login(&payload.username, &payload.password).await?;
    let user = service.repo.find_by_username(&payload.username.to_lowercase())
        .await
        .map_err(BlogError::from)?
        .ok_or_else(|| BlogError::InvalidCredentials)?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": jwt,
        "user": UserResponse::from(user)
    })))
}

