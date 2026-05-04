use crate::application::{AuthService, BlogService};
use crate::domain::{CreatePost, LoginRequest, RegisterRequest, UpdatePost};
use crate::presentation::middleware::get_authenticated_user;
use actix_web::{HttpResponse, delete, get, post, put, web};

fn validate_registration(req: &RegisterRequest) -> Result<(), actix_web::Error> {
    if req.username.trim().is_empty() {
        return Err(actix_web::error::ErrorBadRequest(
            "Username cannot be empty",
        ));
    }
    if req.username.len() < 3 || req.username.len() > 50 {
        return Err(actix_web::error::ErrorBadRequest(
            "Username must be 3-50 characters",
        ));
    }
    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(actix_web::error::ErrorBadRequest("Invalid email format"));
    }
    if req.password.len() < 6 {
        return Err(actix_web::error::ErrorBadRequest(
            "Password must be at least 6 characters",
        ));
    }
    Ok(())
}

#[post("/auth/register")]
pub async fn register(
    service: web::Data<AuthService>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let req = body.into_inner();
    validate_registration(&req)?;
    let (token, user) = service
        .register(req)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // req.validate()
    //     .map_err(|e| {
    //         let msg = e.errors().values()
    //             .next()
    //             .and_then(|v| v.first())
    //             .map(|err| err.to_string())
    //             .unwrap_or_else(|| "Validation failed".into());
    //         actix_web::error::ErrorBadRequest(msg)
    //     })?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "token": token,
        "user": user
    })))
}

#[post("/auth/login")]
pub async fn login(
    service: web::Data<AuthService>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let req = body.into_inner();
    // validate_registration(&req)?;
    let (token, user) = service.login(req).await.map_err(|e| match e {
        crate::domain::DomainError::InvalidCredentials => {
            actix_web::error::ErrorUnauthorized("Invalid credentials")
        }
        _ => actix_web::error::ErrorInternalServerError(e),
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": user
    })))
}

#[get("/posts/{id}")]
pub async fn get_post(
    service: web::Data<BlogService>,
    path: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    let post = service
        .get_post(path.into_inner())
        .await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => {
                actix_web::error::ErrorNotFound("Post not found")
            }
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;

    Ok(HttpResponse::Ok().json(post))
}

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

#[get("/posts")]
pub async fn list_posts(
    service: web::Data<BlogService>,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, actix_web::Error> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);

    let (posts, total) = service
        .list_posts(limit, offset)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "posts": posts,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

#[post("")]
pub async fn create_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    body: web::Json<CreatePost>,
) -> Result<HttpResponse, actix_web::Error> {
    let body_req = body.into_inner();
    // validate_registration(&body_req)?;
    let auth_user = get_authenticated_user(&req).map_err(actix_web::error::ErrorUnauthorized)?;

    let post = service
        .create_post(auth_user.user_id, body_req)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(post))
}

#[put("/{id}")]
pub async fn update_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
    body: web::Json<UpdatePost>,
) -> Result<HttpResponse, actix_web::Error> {
    let body_req = body.into_inner();
    // validate_registration(&body_req)?;
    let auth_user = get_authenticated_user(&req).map_err(actix_web::error::ErrorUnauthorized)?;

    let post = service
        .update_post(path.into_inner(), auth_user.user_id, body_req)
        .await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => {
                actix_web::error::ErrorNotFound("Post not found")
            }
            crate::domain::DomainError::Forbidden => {
                actix_web::error::ErrorForbidden("Not the author")
            }
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;

    Ok(HttpResponse::Ok().json(post))
}

#[delete("/{id}")]
pub async fn delete_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_user = get_authenticated_user(&req).map_err(actix_web::error::ErrorUnauthorized)?;

    service
        .delete_post(path.into_inner(), auth_user.user_id)
        .await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => {
                actix_web::error::ErrorNotFound("Post not found")
            }
            crate::domain::DomainError::Forbidden => {
                actix_web::error::ErrorForbidden("Not the author")
            }
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[get("/debug/routes")]
pub async fn debug_routes() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "public": [
            "POST /api/auth/register",
            "POST /api/auth/login", 
            "GET /api/posts/{id}",
            "GET /api/posts"
        ],
        "protected": [
            "POST /api/posts",
            "PUT /api/posts/{id}",
            "DELETE /api/posts/{id}"
        ],
        "hint": "If POST /api/posts returns 404, check that handler uses #[post(\"\")] not #[post(\"/api/posts\")]"
    }))
}
