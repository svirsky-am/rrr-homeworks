pub mod middleware;
pub mod http_handlers;
pub mod grpc_service;

pub use middleware::{AuthenticatedUser, jwt_validator};
pub use http_handlers::{register, login, get_post, list_posts, create_post, update_post, delete_post, debug_routes,  health_check};
pub use grpc_service::BlogGrpcService;

/// Публичные маршруты (без авторизации)
pub fn public_scope() -> actix_web::Scope {
    actix_web::web::scope("/api")
        .service(register)      // POST /api/auth/register
        .service(login)         // POST /api/auth/login
        .service(get_post)      // GET  /api/posts/{id}
        .service(list_posts) // GET  /api/posts
        .service(health_check)
}

/// Защищённые маршруты (требуют JWT)
pub fn protected_scope() -> actix_web::Scope {
    actix_web::web::scope("/posts")
        .service(create_post)   // POST /api/posts
        .service(update_post)   // PUT  /api/posts/{id}
        .service(delete_post)   // DELETE /api/posts/{id}
        .service(debug_routes)   // GET  /api/posts/debug (для отладки без постов)
}