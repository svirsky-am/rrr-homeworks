pub mod blog {
    tonic::include_proto!("blog");
}

pub mod application;
pub mod config;
pub mod data;
pub mod domain;
pub mod infrastructure;
pub mod presentation;

pub use application::{AuthService, BlogService};
pub use config::ServerConfig;
pub use infrastructure::{JwtService, create_pool, run_migrations};
pub use presentation::{AuthenticatedUser, jwt_validator, protected_scope, public_scope};

// Функции для тестов
pub use server::init_services;
pub use server::run_server;

// Внутренний модуль с логикой запуска (не публичный для внешних пользователей)
mod server;
