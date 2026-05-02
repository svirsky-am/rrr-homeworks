pub mod blog {
    tonic::include_proto!("blog");
}

mod domain;
mod application;
mod data;
mod infrastructure;
mod presentation;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_cors::Cors;
use tonic::transport::Server;
use std::sync::Arc;
use std::env;



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Инициализация
    infrastructure::logging::init();
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL required");
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-min-32-chars-change-in-prod".into());

    // 2. Подключение к БД и миграции
    let pool = infrastructure::create_pool(&database_url)
        .await.expect("Failed to create DB pool");
    infrastructure::run_migrations(&pool)
        .await.expect("Failed to run migrations");

    // 3. Инициализация сервисов
    let jwt_service = infrastructure::JwtService::new(&jwt_secret);
    
    // ✅ СТАЛО (правильно — без лишнего Arc):
    let user_repo = data::UserRepository::new(pool.clone());  // ← Без Arc
    let post_repo = data::PostRepository::new(pool.clone());   // ← Без Arc
    
    // Сервисы создаются без Arc — web::Data обернёт их сам
    let auth_service = application::AuthService::new(
        user_repo,           // ← Передаём по значению (Repo внутри уже Clone)
        jwt_service.clone()
    );
    let blog_service = application::BlogService::new(post_repo);

    // 4. Shared data для Actix
    // web::Data<T> уже использует Arc внутри — тип будет правильным ✅
    let auth_data = web::Data::new(auth_service);  // web::Data<AuthService>
    let blog_data = web::Data::new(blog_service);   // web::Data<BlogService>
    let jwt_data = web::Data::new(jwt_service.clone());



    // 6. JWT middleware для защищённых маршрутов
    let auth_middleware = HttpAuthentication::bearer(presentation::jwt_validator);

    // 7. HTTP сервер
    let http_addr = "0.0.0.0:3000";
    let http_server = HttpServer::new(
        move || {
        // 5. CORS для WASM-фронтенда
        let cors = Cors::default()
            .allow_any_origin() // В продакшене: .allowed_origin("https://your-frontend.com")
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
            // Public routes
            .service(presentation::register)
            .service(presentation::login)
            .service(presentation::get_post)
            .service(presentation::list_posts)
            // Protected routes with JWT middleware
            .service(
                web::scope("/api/posts")
                    .wrap(auth_middleware.clone())
                    .service(presentation::create_post)
                    .service(presentation::update_post)
                    .service(presentation::delete_post)
            )
    })
    .bind(http_addr)?
    .run();

    // 8. gRPC сервер
    

    let grpc_user_repo = Arc::new(data::UserRepository::new(pool.clone()));
    let grpc_post_repo = Arc::new(data::PostRepository::new(pool.clone()));
    // gRPC нуждается в явном Arc для шаринга между потоками
    let grpc_auth_service = Arc::new(application::AuthService::new(
        grpc_user_repo.as_ref().clone(),  // ← Arc<UserRepository> → UserRepository
        jwt_service.clone()
    ));

    let grpc_blog_service = Arc::new(application::BlogService::new(
        grpc_post_repo.as_ref().clone()  // ← Arc<PostRepository> → PostRepository
    ));

    let grpc_service = presentation::BlogGrpcService::new(
        grpc_auth_service,
        grpc_blog_service,
        jwt_service.clone(),
    );

    let grpc_addr = "0.0.0.0:50051".parse().unwrap();
    let grpc_server = Server::builder()
        .add_service(blog::blog_service_server::BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    // 9. Запуск обоих серверов параллельно
    tracing::info!("🌐 HTTP server listening on http://{}", http_addr);
    tracing::info!("🔗 gRPC server listening on http://{}", grpc_addr);

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
}