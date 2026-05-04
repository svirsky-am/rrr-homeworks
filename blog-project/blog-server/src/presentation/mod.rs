pub mod grpc_service;
pub mod http_handlers;
pub mod middleware;

pub use grpc_service::BlogGrpcService;
pub use http_handlers::{
    create_post, debug_routes, delete_post, get_post, health_check, list_posts, login, register,
    update_post,
};
pub use middleware::{AuthenticatedUser, jwt_validator};

/// Публичные маршруты (без авторизации)
pub fn public_scope() -> actix_web::Scope {
    actix_web::web::scope("/api")
        .service(register) // POST /api/auth/register
        .service(login) // POST /api/auth/login
        .service(get_post) // GET  /api/posts/{id}
        .service(list_posts) // GET  /api/posts
        .service(health_check)
}

/// Защищённые маршруты (требуют JWT)
pub fn protected_scope() -> actix_web::Scope {
    actix_web::web::scope("/posts")
        .service(create_post) // POST /api/posts
        .service(update_post) // PUT  /api/posts/{id}
        .service(delete_post) // DELETE /api/posts/{id}
        .service(debug_routes) // GET  /api/posts/debug (для отладки без постов)
}
