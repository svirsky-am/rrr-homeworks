use actix_web::{post, web, HttpResponse, Responder, Scope};
use chrono::Utc;
use tracing::info;

use crate::application::auth_service::AuthService;
use crate::data::user_repository::PostgresUserRepository;
use crate::domain::BankError;
use crate::presentation::dto::{HealthResponse, LoginRequest, RegisterRequest, TokenResponse};

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
) -> Result<impl Responder, BankError> {
    let user = service
        .register(payload.email.clone(), payload.password.clone())
        .await?;

    info!(user_id = %user.id, email = %user.email, "user registered");

    Ok(HttpResponse::Created().json(serde_json::json!({
        "user_id": user.id,
        "email": user.email
    })))
}

#[post("/auth/login")]
async fn login(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, BankError> {
    let jwt = service.login(&payload.email, &payload.password).await?;
    info!(email = %payload.email, "user logged in");
    Ok(HttpResponse::Ok().json(TokenResponse { access_token: jwt }))
}

#[post("/auth/token")]
async fn token(
    service: web::Data<AuthService<PostgresUserRepository>>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, BankError> {
    let jwt = service.login(&payload.email, &payload.password).await?;
    Ok(HttpResponse::Ok().json(TokenResponse { access_token: jwt }))
}

