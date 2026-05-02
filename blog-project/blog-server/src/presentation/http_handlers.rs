use actix_web::{web, HttpResponse, get, post, put, delete};
use crate::application::{AuthService, BlogService};
use crate::domain::{RegisterRequest, LoginRequest, CreatePost, UpdatePost};
use crate::presentation::middleware::{get_authenticated_user, AuthenticatedUser};

#[post("/api/auth/register")]
pub async fn register(
    service: web::Data<AuthService>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let (token, user) = service.register(body.into_inner()).await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Created().json(serde_json::json!({
        "token": token,
        "user": user
    })))
}

#[post("/api/auth/login")]
pub async fn login(
    service: web::Data<AuthService>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let (token, user) = service.login(body.into_inner()).await
        .map_err(|e| match e {
            crate::domain::DomainError::InvalidCredentials => 
                actix_web::error::ErrorUnauthorized("Invalid credentials"),
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": token,
        "user": user
    })))
}

#[get("/api/posts/{id}")]
pub async fn get_post(
    service: web::Data<BlogService>,
    path: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    let post = service.get_post(path.into_inner()).await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => 
                actix_web::error::ErrorNotFound("Post not found"),
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;
    
    Ok(HttpResponse::Ok().json(post))
}

#[get("/api/posts")]
pub async fn list_posts(
    service: web::Data<BlogService>,
    query: web::Query<Pagination>,
) -> Result<HttpResponse, actix_web::Error> {
    let limit = query.limit.unwrap_or(10).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let (posts, total) = service.list_posts(limit, offset).await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "posts": posts,
        "total": total,
        "limit": limit,
        "offset": offset
    })))
}

#[post("/api/posts")]
pub async fn create_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    body: web::Json<CreatePost>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_user = get_authenticated_user(&req)
        .map_err(actix_web::error::ErrorUnauthorized)?;
    
    let post = service.create_post(auth_user.user_id, body.into_inner()).await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Created().json(post))
}

#[put("/api/posts/{id}")]
pub async fn update_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
    body: web::Json<UpdatePost>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_user = get_authenticated_user(&req)
        .map_err(actix_web::error::ErrorUnauthorized)?;
    
    let post = service.update_post(path.into_inner(), auth_user.user_id, body.into_inner()).await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => 
                actix_web::error::ErrorNotFound("Post not found"),
            crate::domain::DomainError::Forbidden => 
                actix_web::error::ErrorForbidden("Not the author"),
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;
    
    Ok(HttpResponse::Ok().json(post))
}

#[delete("/api/posts/{id}")]
pub async fn delete_post(
    service: web::Data<BlogService>,
    req: actix_web::HttpRequest,
    path: web::Path<i64>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_user = get_authenticated_user(&req)
        .map_err(actix_web::error::ErrorUnauthorized)?;
    
    service.delete_post(path.into_inner(), auth_user.user_id).await
        .map_err(|e| match e {
            crate::domain::DomainError::PostNotFound => 
                actix_web::error::ErrorNotFound("Post not found"),
            crate::domain::DomainError::Forbidden => 
                actix_web::error::ErrorForbidden("Not the author"),
            _ => actix_web::error::ErrorInternalServerError(e),
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}