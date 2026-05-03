use std::env;

/// Конфигурация сервера (для гибкости в интеграционных тестах)
#[derive(Clone)]
pub struct ServerConfig {
    pub http_addr: String,
    pub grpc_addr: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub run_migrations: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_addr: "0.0.0.0:3000".into(),
            grpc_addr: "0.0.0.0:50051".into(),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL required"),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-min-32-chars-change-in-prod".into()),
            run_migrations: true,
        }
    }
}