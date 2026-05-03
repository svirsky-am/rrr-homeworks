pub mod blog {
    tonic::include_proto!("blog");
}

pub mod domain;
pub mod application;
pub mod data;
pub mod infrastructure;
pub mod presentation;
pub mod config;

pub use application::{AuthService, BlogService};
pub use infrastructure::{JwtService, create_pool, run_migrations};
pub use presentation::{public_scope, protected_scope, jwt_validator, AuthenticatedUser};
pub use config::ServerConfig;

// Функции для тестов
pub use server::init_services;
pub use server::run_server;

// Внутренний модуль с логикой запуска (не публичный для внешних пользователей)
mod server;