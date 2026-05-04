
use blog_server::{run_server, ServerConfig, blog::blog_service_client::BlogServiceClient};
use blog_server::blog::{
    RegisterRequest, LoginRequest, CreatePostRequest, UpdatePostRequest, 
    PostId, ListPostsRequest,
};
use tonic::{transport::Channel, Request, metadata::MetadataValue};
use sqlx::PgPool;
use std::{time::Duration, str::FromStr};
use uuid::Uuid;

// ==================== КОНФИГУРАЦИЯ ====================

fn test_config() -> ServerConfig {
    ServerConfig {
        http_addr: "127.0.0.1:0".into(),
        grpc_addr: "127.0.0.1:0".into(),
        database_url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base_test".into()),
        jwt_secret: "test-secret-min-32-chars-for-grpc-integration!".into(),
        run_migrations: true,
    }
}

async fn cleanup_test_db(database_url: &str) {
    let pool = PgPool::connect(database_url).await.expect("Failed to connect to test DB");
    
    let result = sqlx::query("TRUNCATE posts, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await;
    
    match result {
        Ok(_) => eprintln!("🧹 gRPC test DB cleaned (TRUNCATE)"),
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("42P01") => {
            eprintln!("⚠️  Tables not found (first run) — skipping truncate");
        }
        Err(e) => eprintln!("⚠️  Cleanup warning: {}", e),
    }
    
    pool.close().await;
}

// ==================== ВСПОМОГАТЕЛЬНЫЕ ФУНКЦИИ ====================

/// Создаёт Request с JWT-токеном в метаданных
fn auth_request<T>(token: &str, message: T) -> Request<T> {
    let mut request = Request::new(message);
    let auth_value = MetadataValue::from_str(&format!("Bearer {}", token))
        .expect("Invalid token format");
    request.metadata_mut().insert("authorization", auth_value);
    request
}

/// Ждём доступности gRPC сервера
async fn wait_for_grpc_server(grpc_addr: &str, max_attempts: usize) -> Result<(), String> {
    for attempt in 1..=max_attempts {
        match Channel::from_shared(format!("http://{}", grpc_addr))
            .map_err(|e| e.to_string())?
            .connect()
            .await
        {
            Ok(_) => return Ok(()),
            Err(_) if attempt < max_attempts => {
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => return Err(format!("Failed to connect to gRPC server: {}", e)),
        }
    }
    Err("Server did not become ready".into())
}

// ==================== ТЕСТЫ ====================

#[tokio::test]
async fn test_grpc_login() {
    let config = test_config();
    cleanup_test_db(&config.database_url).await;
    
    let server = run_server(config.clone()).await.expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;
    wait_for_grpc_server(&server.grpc_addr, 5).await.expect("gRPC server not ready");
    
    let channel = Channel::from_shared(format!("http://{}", server.grpc_addr))
        .expect("Invalid grpc_addr")
        .connect()
        .await
        .expect("Failed to connect to gRPC");
    
    let mut client = BlogServiceClient::new(channel);
    
    // === Регистрация пользователя ===
    let username = format!("grpc_user_{}", Uuid::new_v4());
    let _register = client
        .register(Request::new(RegisterRequest {
            username: username.clone(),
            email: format!("{}@test.com", username),
            password: "pass123".into(),
        }))
        .await
        .expect("Register should succeed");
    
    // === Логин ===
    let login_resp = client
        .login(Request::new(LoginRequest {
            username: username.clone(),
            password: "pass123".into(),
        }))
        .await
        .expect("Login should succeed")
        .into_inner();
    
    assert!(!login_resp.token.is_empty(), "Token should not be empty");
    assert_eq!(login_resp.user.as_ref().unwrap().username, username);
    
    server.handle.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_grpc_full_flow() {
    let config = test_config();
    cleanup_test_db(&config.database_url).await;
    
    let server = run_server(config.clone()).await.expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;
    wait_for_grpc_server(&server.grpc_addr, 5).await.expect("gRPC server not ready");
    
    let channel = Channel::from_shared(format!("http://{}", server.grpc_addr))
        .expect("Invalid grpc_addr")
        .connect()
        .await
        .expect("Failed to connect");
    
    let mut client = BlogServiceClient::new(channel);
    
    // === 1. Регистрация ===
    let username = format!("flow_user_{}", Uuid::new_v4());
    let register = client
        .register(Request::new(RegisterRequest {
            username: username.clone(),
            email: format!("{}@test.com", username),
            password: "pass123".into(),
        }))
        .await
        .expect("Register should succeed")
        .into_inner();
    
    let token = register.token;
    let user_id = register.user.as_ref().unwrap().id;
    
    // === 2. Создание поста (с авторизацией) ===
    let create_request = auth_request(&token, CreatePostRequest {
        title: "gRPC Post".into(),
        content: "Created via gRPC".into(),
    });
    
    let create_resp = client
        .create_post(create_request)
        .await
        .expect("Create post should succeed")
        .into_inner();
    
    let post = create_resp.post.expect("Post should be returned");
    assert_eq!(post.title, "gRPC Post");
    assert_eq!(post.author_id, user_id);
    
    // === 3. Получение поста (публичный метод) ===
    let get_resp = client
        .get_post(Request::new(PostId { id: post.id }))
        .await
        .expect("Get post should succeed")
        .into_inner();
    
    assert_eq!(get_resp.post.as_ref().unwrap().id, post.id);
    
    // === 4. Список постов (публичный) ===
    let list_resp = client
        .list_posts(Request::new(ListPostsRequest {
            limit: Some(10),
            offset: Some(0),
        }))
        .await
        .expect("List posts should succeed")
        .into_inner();
    
    assert!(!list_resp.posts.is_empty());
    assert!(list_resp.total >= 1);
    
    // === 5. Обновление поста (только автор) ===
    let update_request = auth_request(&token, UpdatePostRequest {
        id: post.id,
        title: Some("Updated via gRPC".into()),
        content: None,
    });
    
    let update_resp = client
        .update_post(update_request)
        .await
        .expect("Update post should succeed")
        .into_inner();
    
    assert_eq!(update_resp.post.as_ref().unwrap().title, "Updated via gRPC");
    
    // === 6. Удаление поста (только автор) ===
    let delete_request = auth_request(&token, PostId { id: post.id });
    
    let delete_resp = client
        .delete_post(delete_request)
        .await
        .expect("Delete post should succeed")
        .into_inner();
    
    assert!(delete_resp.success);
    
    // === 7. Проверка, что пост удалён ===
    let get_deleted = client
        .get_post(Request::new(PostId { id: post.id }))
        .await;
    
    assert_eq!(
        get_deleted.unwrap_err().code(),
        tonic::Code::NotFound,
        "Deleted post should return NotFound"
    );
    
    server.handle.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_grpc_unauthorized_access() {
    let config = test_config();
    cleanup_test_db(&config.database_url).await;
    
    let server = run_server(config.clone()).await.expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;
    wait_for_grpc_server(&server.grpc_addr, 5).await.expect("gRPC server not ready");
    
    let channel = Channel::from_shared(format!("http://{}", server.grpc_addr))
        .expect("Invalid grpc_addr")
        .connect()
        .await
        .expect("Failed to connect");
    
    let mut client = BlogServiceClient::new(channel);
    
    // === Попытка создать пост без токена -> Unauthenticated ===
    let create_err = client
        .create_post(Request::new(CreatePostRequest {
            title: "No Auth".into(),
            content: "Should fail".into(),
        }))
        .await
        .unwrap_err();
    
    assert_eq!(
        create_err.code(),
        tonic::Code::Unauthenticated,
        "Create post without token should return Unauthenticated"
    );
    
    // === Регистрация двух пользователей ===
    let user1 = client
        .register(Request::new(RegisterRequest {
            username: "user1_grpc".into(),
            email: "u1@test.com".into(),
            password: "pass123".into(),
        }))
        .await
        .expect("Register user1")
        .into_inner();
    
    let user2 = client
        .register(Request::new(RegisterRequest {
            username: "user2_grpc".into(),
            email: "u2@test.com".into(),
            password: "pass123".into(),
        }))
        .await
        .expect("Register user2")
        .into_inner();
    
    // === User1 создаёт пост ===
    let post = client
        .create_post(auth_request(&user1.token, CreatePostRequest {
            title: "User1 Post".into(),
            content: "Content".into(),
        }))
        .await
        .expect("User1 create post")
        .into_inner()
        .post
        .expect("Post should be returned");
    
    // === User2 пытается обновить пост User1 -> PermissionDenied ===
    let update_err = client
        .update_post(auth_request(&user2.token, UpdatePostRequest {
            id: post.id,
            title: Some("Hacked".into()),
            content: None,
        }))
        .await
        .unwrap_err();
    
    assert_eq!(
        update_err.code(),
        tonic::Code::PermissionDenied,
        "User2 should not be able to update User1's post"
    );
    
    // === User2 пытается удалить пост User1 -> PermissionDenied ===
    let delete_err = client
        .delete_post(auth_request(&user2.token, PostId { id: post.id }))
        .await
        .unwrap_err();
    
    assert_eq!(
        delete_err.code(),
        tonic::Code::PermissionDenied,
        "User2 should not be able to delete User1's post"
    );
    
    // === User1 может удалить свой пост ===
    let delete = client
        .delete_post(auth_request(&user1.token, PostId { id: post.id }))
        .await
        .expect("User1 should delete own post")
        .into_inner();
    
    assert!(delete.success);
    
    server.handle.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_grpc_pagination() {
    let config = test_config();
    cleanup_test_db(&config.database_url).await;
    
    let server = run_server(config.clone()).await.expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;
    wait_for_grpc_server(&server.grpc_addr, 5).await.expect("gRPC server not ready");
    
    let channel = Channel::from_shared(format!("http://{}", server.grpc_addr))
        .expect("Invalid grpc_addr")
        .connect()
        .await
        .expect("Failed to connect");
    
    let mut client = BlogServiceClient::new(channel);
    
    // Регистрируемся и создаём 5 постов
    let username = format!("pager_{}", Uuid::new_v4());
    let register = client
        .register(Request::new(RegisterRequest {
            username: username.clone(),
            email: format!("{}@test.com", username),
            password: "pass123".into(),
        }))
        .await
        .expect("Register")
        .into_inner();
    
    for i in 0..5 {
        client
            .create_post(auth_request(&register.token, CreatePostRequest {
                title: format!("Post {}", i),
                content: format!("Content {}", i),
            }))
            .await
            .expect("Create post");
    }
    
    // === Пагинация: limit=2, offset=0 ===
    let page1 = client
        .list_posts(Request::new(ListPostsRequest {
            limit: Some(2),
            offset: Some(0),
        }))
        .await
        .expect("List posts page 1")
        .into_inner();
    
    assert_eq!(page1.posts.len(), 2);
    assert_eq!(page1.total, 5);
    assert_eq!(page1.limit, 2);
    assert_eq!(page1.offset, 0);
    
    // === Пагинация: limit=2, offset=2 ===
    let page2 = client
        .list_posts(Request::new(ListPostsRequest {
            limit: Some(2),
            offset: Some(2),
        }))
        .await
        .expect("List posts page 2")
        .into_inner();
    
    assert_eq!(page2.posts.len(), 2);
    assert_eq!(page2.offset, 2);
    
    // === Получить все посты ===
    let all = client
        .list_posts(Request::new(ListPostsRequest {
            limit: Some(100),
            offset: Some(0),
        }))
        .await
        .expect("List all posts")
        .into_inner();
    
    assert_eq!(all.posts.len(), 5);
    
    server.handle.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_grpc_validation_errors() {
    let config = test_config();
    cleanup_test_db(&config.database_url).await;
    
    let server = run_server(config.clone()).await.expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;
    wait_for_grpc_server(&server.grpc_addr, 5).await.expect("gRPC server not ready");
    
    let channel = Channel::from_shared(format!("http://{}", server.grpc_addr))
        .expect("Invalid grpc_addr")
        .connect()
        .await
        .expect("Failed to connect");
    
    let mut client = BlogServiceClient::new(channel);
    
    // === Пустой username -> InvalidArgument ===
    let err = client
        .register(Request::new(RegisterRequest {
            username: "".into(),
            email: "test@test.com".into(),
            password: "pass123".into(),
        }))
        .await
        .unwrap_err();
    
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    
    // === Невалидный email -> InvalidArgument ===
    let err = client
        .register(Request::new(RegisterRequest {
            username: "testuser".into(),
            email: "not-an-email".into(),
            password: "pass123".into(),
        }))
        .await
        .unwrap_err();
    
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    
    // === Короткий пароль -> InvalidArgument ===
    let err = client
        .register(Request::new(RegisterRequest {
            username: "testuser".into(),
            email: "test@test.com".into(),
            password: "123".into(),
        }))
        .await
        .unwrap_err();
    
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
    
    server.handle.abort();
    tokio::time::sleep(Duration::from_millis(100)).await;
}