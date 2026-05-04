use crate::{application, blog, config::ServerConfig, data, infrastructure, presentation};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::transport::Server as TonicServer;

/// Инициализация зависимостей сервера (выносится для переиспользования в тестах)
pub async fn init_services(
    pool: PgPool,
    jwt_secret: &str,
) -> (
    web::Data<application::AuthService>,
    web::Data<application::BlogService>,
    web::Data<infrastructure::JwtService>,
    Arc<application::AuthService>, // Для gRPC
    Arc<application::BlogService>, // Для gRPC
) {
    let jwt_service = infrastructure::JwtService::new(jwt_secret);

    let user_repo = data::UserRepository::new(pool.clone());
    let post_repo = data::PostRepository::new(pool.clone());

    let auth_service = application::AuthService::new(user_repo, jwt_service.clone());
    let blog_service = application::BlogService::new(post_repo);

    // Для HTTP
    let auth_data = web::Data::new(auth_service.clone());
    let blog_data = web::Data::new(blog_service.clone());
    let jwt_data = web::Data::new(jwt_service.clone());

    // Для gRPC (с Arc для явного шаринга)
    let grpc_user_repo = Arc::new(data::UserRepository::new(pool.clone()));
    let grpc_post_repo = Arc::new(data::PostRepository::new(pool.clone()));

    let grpc_auth_service = Arc::new(application::AuthService::new(
        grpc_user_repo.as_ref().clone(),
        jwt_service.clone(),
    ));
    let grpc_blog_service = Arc::new(application::BlogService::new(
        grpc_post_repo.as_ref().clone(),
    ));

    (
        auth_data,
        blog_data,
        jwt_data,
        grpc_auth_service,
        grpc_blog_service,
    )
}
/// Структура с метаданными о старте приложения для тестов
pub struct StartedServer {
    pub http_addr: String, // "127.0.0.1:40365"
    pub grpc_addr: String,
    pub handle: tokio::task::JoinHandle<std::io::Result<()>>,
}

/// Запуск серверa (экспортируется для тестов)
/// StartedServer для управления жизненным циклом с метаданными о запуске
pub async fn run_server(config: ServerConfig) -> Result<StartedServer, std::io::Error> {
    if config.run_migrations {
        infrastructure::logging::init();
    }

    let pool = infrastructure::create_pool(&config.database_url)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::ConnectionRefused, e))?;

    if config.run_migrations {
        infrastructure::run_migrations(&pool)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }

    let (auth_data, blog_data, jwt_data, grpc_auth_service, grpc_blog_service) =
        init_services(pool, &config.jwt_secret).await;

    let auth_middleware = HttpAuthentication::bearer(presentation::jwt_validator);
    let jwt_service_for_grpc = infrastructure::JwtService::new(&config.jwt_secret);

    let grpc_addr = config.grpc_addr.clone();

    // Создаём TcpListener для получения реального порта вместо http.
    use std::net::TcpListener;
    let http_listener = TcpListener::bind(&config.http_addr)?;
    let actual_http_addr = http_listener.local_addr()?.to_string(); // "127.0.0.1:40365"
    let actual_http_addr_for_log = actual_http_addr.clone();

    // HTTP сервер
    let http_server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(auth_data.clone())
            .app_data(blog_data.clone())
            .app_data(jwt_data.clone())
            .service(
                presentation::public_scope()
                    .service(presentation::protected_scope().wrap(auth_middleware.clone())),
            )
    })
    // .bind(&http_addr)?
    .listen(http_listener)? // для мока в тестах
    .run();

    // gRPC сервер
    let grpc_service = presentation::BlogGrpcService::new(
        grpc_auth_service,
        grpc_blog_service,
        jwt_service_for_grpc,
    );

    let listener_grpc = TcpListener::bind(&config.grpc_addr)?;
    let actual_grpc_addr = listener_grpc.local_addr()?.to_string(); // "127.0.0.1:40365"
    let actual_grpc_addr_for_log = actual_grpc_addr.clone();

    let grpc_server = TonicServer::builder()
        .add_service(blog::blog_service_server::BlogServiceServer::new(
            grpc_service,
        ))
        .serve(listener_grpc.local_addr()?);

    // Запускаем в отдельной задаче
    let handle = tokio::spawn(async move {
        tracing::info!(
            "🌐 HTTP server listening on http://{}",
            actual_http_addr_for_log
        );
        tracing::info!(
            "🔗 gRPC server listening on http://{}",
            actual_grpc_addr_for_log
        );

        tokio::select! {
            result = http_server => {
                tracing::error!("HTTP server failed: {:?}", result);
                result
            }
            result = grpc_server => {
                tracing::error!("gRPC server failed: {:?}", result);
                result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            }
        }
    });

    Ok(StartedServer {
        http_addr: actual_http_addr.to_string(),
        grpc_addr: actual_grpc_addr.to_string(), // аналогично для gRPC
        handle,
    })
}
