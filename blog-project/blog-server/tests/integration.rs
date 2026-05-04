// blog-server/tests/integration.rs

use blog_server::{ServerConfig, init_services, run_server};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

// ==================== КОНФИГУРАЦИЯ ====================

/// Тестовая конфигурация — использует отдельную БД и фиксированные порты
fn test_config() -> ServerConfig {
    ServerConfig {
        // http_addr: "127.0.0.1:8081".into(),
        http_addr: "127.0.0.1:0".into(),
        grpc_addr: "127.0.0.1:50052".into(),
        database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
            "postgres://blog_admin:blog_pass@127.0.0.1:8432/r_blog_base_test".into()
        }),
        jwt_secret: "test-secret-min-32-chars-for-integration-tests!".into(),
        run_migrations: true,
    }
}
/// Очистка тестовой БД между тестами (быстрая и безопасная)
async fn cleanup_test_db(database_url: &str) {
    let pool = PgPool::connect(database_url)
        .await
        .expect("Failed to connect to test DB");

    // TRUNCATE очищает данные, но сохраняет таблицы и схему
    // RESTART IDENTITY сбрасывает автоинкрементные ID (BIGSERIAL)
    // CASCADE очищает связанные данные, если есть внешние ключи
    let result = sqlx::query("TRUNCATE posts, users RESTART IDENTITY CASCADE")
        .execute(&pool)
        .await;

    match result {
        Ok(_) => eprintln!("🧹 Test DB cleaned (TRUNCATE)"),
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("42P01") => {
            // Таблицы ещё не созданы (первый запуск) — это ок, миграции создадут их
            eprintln!("⚠️  Tables not found (first run) — skipping truncate");
        }
        Err(e) => {
            // Другие ошибки — логируем, но не паникуем, чтобы не ломать все тесты
            eprintln!("⚠️  Cleanup warning: {}", e);
        }
    }

    pool.close().await;
}
// ==================== ТЕСТОВЫЙ КЛИЕНТ ====================

/// Клиент для тестов с поддержкой авторизации
#[derive(Clone)]
struct AuthenticatedClient {
    client: Arc<Client>,
    base_url: String,
    token: Option<String>,
}

impl AuthenticatedClient {
    fn new(base_url: &str) -> Self {
        Self {
            client: Arc::new(
                Client::builder()
                    .timeout(Duration::from_secs(10))
                    .build()
                    .expect("Failed to build reqwest client"),
            ),
            base_url: base_url.to_string(),
            token: None,
        }
    }

    fn with_token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    async fn register(&self, username: &str, email: &str, password: &str) -> RegisterResponse {
        let resp = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .header("Content-Type", "application/json")
            .json(&json!({ "username": username, "email": email, "password": password }))
            .send()
            .await
            .expect("Register request failed");

        let status = resp.status(); // Сохраняем статус
        let body = resp.text().await.unwrap_or_else(|_| "<empty>".into()); //  Теперь resp можно потребить

        if !status.is_success() {
            panic!("Register failed with {}: {}", status, body);
        }

        serde_json::from_str(&body).expect("Failed to parse register response")
    }

    async fn login(&self, username: &str, password: &str) -> LoginResponse {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .header("Content-Type", "application/json")
            .json(&json!({ "username": username, "password": password }))
            .send()
            .await
            .expect("Login request failed");

        let status = resp.status(); // Сохраняем статус
        let body = resp.text().await.unwrap_or_else(|_| "<empty>".into());

        if !status.is_success() {
            panic!("Login failed with {}: {}", status, body);
        }

        serde_json::from_str(&body).expect("Failed to parse login response")
    }

    async fn create_post(&self, title: &str, content: &str) -> reqwest::Response {
        let mut req = self
            .client
            .post(format!("{}/api/posts", self.base_url))
            .header("Content-Type", "application/json")
            .json(&json!({ "title": title, "content": content }));

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req.send().await.expect("Create post request failed")
    }

    async fn get_posts(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/api/posts", self.base_url))
            .send()
            .await
            .expect("Get posts request failed")
    }

    async fn get_post(&self, id: i64) -> reqwest::Response {
        self.client
            .get(format!("{}/api/posts/{}", self.base_url, id))
            .send()
            .await
            .expect("Get post request failed")
    }

    async fn update_post(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> reqwest::Response {
        let mut req = self
            .client
            .put(format!("{}/api/posts/{}", self.base_url, id))
            .header("Content-Type", "application/json")
            .json(&json!({ "title": title, "content": content }));

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req.send().await.expect("Update post request failed")
    }

    async fn delete_post(&self, id: i64) -> reqwest::Response {
        let mut req = self
            .client
            .delete(format!("{}/api/posts/{}", self.base_url, id));

        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req.send().await.expect("Delete post request failed")
    }
}

// ==================== МОДЕЛИ ОТВЕТОВ ====================

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    user: User,
}

#[derive(Debug, Deserialize, Clone)]
struct User {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Post {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct ListPostsResponse {
    posts: Vec<Post>,
    total: i64,
    limit: i64,
    offset: i64,
}

// ==================== ТЕСТЫ ====================

/// Тест полного флоу с одним клиентом:
/// регистрация -> логин -> создание поста -> чтение -> обновление -> удаление
#[tokio::test]
async fn test_full_flow_single_client() {
    let config = test_config();

    // Очищаем БД перед тестом
    cleanup_test_db(&config.database_url).await;

    // Запускаем сервер
    let server = run_server(config.clone())
        .await
        .expect("Failed to start server");

    // Даём серверу время на старт
    tokio::time::sleep(Duration::from_secs(1)).await;

    let base_url = format!("http://{}", server.http_addr);
    let client = AuthenticatedClient::new(&base_url.clone());

    // === 1. Регистрация ===
    let username = format!("test_user_{}", Uuid::new_v4());
    let email = format!("{}@test.com", username);
    let password = "secure_password_123";

    let register_resp = client.register(&username, &email, password).await;
    let token = register_resp.token;
    let user_id = register_resp.user.id;

    let auth_client = client.with_token(token.clone());

    // === 2. Создание поста ===
    let create_resp = auth_client.create_post("Test Title", "Test Content").await;
    assert_eq!(create_resp.status(), 201, "Create post should return 201");
    let created_post: Post = create_resp.json().await.expect("Should parse created post");
    assert_eq!(created_post.title, "Test Title");
    assert_eq!(created_post.author_id, user_id);

    // === 3. Получение списка постов (публичный эндпоинт) ===
    let list_resp = auth_client.get_posts().await;
    assert_eq!(list_resp.status(), 200, "List posts should return 200");
    let list: ListPostsResponse = list_resp.json().await.expect("Should parse posts list");
    assert!(!list.posts.is_empty(), "Posts list should not be empty");
    assert!(
        list.posts.iter().any(|p| p.id == created_post.id),
        "Created post should be in list"
    );

    // === 4. Получение конкретного поста (публичный эндпоинт) ===
    let get_resp = auth_client.get_post(created_post.id).await;
    assert_eq!(get_resp.status(), 200, "Get post should return 200");
    let post: Post = get_resp.json().await.expect("Should parse post");
    assert_eq!(post.id, created_post.id);

    // === 5. Обновление поста (только автор) ===
    let update_resp = auth_client
        .update_post(created_post.id, Some("Updated Title"), None)
        .await;
    assert_eq!(update_resp.status(), 200, "Update post should return 200");
    let updated: Post = update_resp.json().await.expect("Should parse updated post");
    assert_eq!(updated.title, "Updated Title");
    assert_eq!(updated.id, created_post.id);

    // === 6. Удаление поста (только автор) ===
    let delete_resp = auth_client.delete_post(created_post.id).await;
    assert_eq!(delete_resp.status(), 204, "Delete post should return 204");

    // === 7. Проверка, что пост удалён ===
    let get_deleted = auth_client.get_post(created_post.id).await;
    assert_eq!(get_deleted.status(), 404, "Deleted post should return 404");

    // === 8. Проверка, что список пуст ===
    let list_after = auth_client.get_posts().await;
    let list_after_resp: ListPostsResponse =
        list_after.json().await.expect("Should parse empty list");
    assert_eq!(
        list_after_resp.total, 0,
        "Posts list should be empty after deletion"
    );

    // Останавливаем сервер
    server.handle.abort();

    //  Даём время на освобождение порта (TCP TIME_WAIT ~ 30-60 сек, но 200ms часто достаточно)
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// Тест с несколькими клиентами, работающими параллельно
#[tokio::test]
async fn test_concurrent_clients() {
    let config = test_config();

    // Очищаем БД перед тестом
    cleanup_test_db(&config.database_url).await;

    // Запускаем сервер
    let server = run_server(config.clone())
        .await
        .expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let base_url = format!("http://{}", server.http_addr);

    // Запускаем 3 клиента параллельно
    let tasks = (0..3).map(|i| {
        let url = base_url.clone();
        tokio::spawn(async move {
            let client = AuthenticatedClient::new(&url);

            // Уникальные данные для каждого клиента
            let username = format!("client_{}_{}", i, Uuid::new_v4());
            let email = format!("{}@test.com", username);
            let password = "password123";

            // === Регистрация (возвращает токен сразу) ===
            let register = client.register(&username, &email, password).await;
            let auth_client = client.with_token(register.token);

            // === Создание поста ===
            let create = auth_client
                .create_post(
                    &format!("Post from client {}", i),
                    &format!("Content from client {}", i),
                )
                .await;

            assert_eq!(
                create.status(),
                201,
                "Client {} create post should succeed",
                i
            );
            let post: Post = create.json().await.expect("Should parse created post");

            // === Чтение списка постов ===
            let list = auth_client.get_posts().await;
            assert_eq!(list.status(), 200, "Client {} list posts should succeed", i);

            (i, post.id)
        })
    });

    // Ждём завершения всех задач
    let results: Vec<_> = futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(|r| r.expect("Task should not panic"))
        .collect();

    // Проверяем, что все 3 клиента успешно создали посты
    assert_eq!(results.len(), 3, "All 3 clients should complete");

    // === Проверка: публичный список содержит все посты ===
    let public_client = AuthenticatedClient::new(&base_url);
    let list = public_client.get_posts().await;
    let list_resp: ListPostsResponse = list.json().await.expect("Should parse list");

    assert!(
        list_resp.total >= 3,
        "Should have at least 3 posts, got {}",
        list_resp.total
    );

    // Проверяем, что все созданные посты есть в списке
    for (_, post_id) in &results {
        assert!(
            list_resp.posts.iter().any(|p| p.id == *post_id),
            "Post {} should be in public list",
            post_id
        );
    }

    // Останавливаем сервер
    server.handle.abort();
    //  Даём время на освобождение порта (TCP TIME_WAIT ~ 30-60 сек, но 200ms часто достаточно)
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// Тест проверки авторизации: доступ к защищённым эндпоинтам без токена
#[tokio::test]
async fn test_unauthorized_access() {
    let config = test_config();

    // Очищаем БД перед тестом
    cleanup_test_db(&config.database_url).await;

    // Запускаем сервер
    let server = run_server(config.clone())
        .await
        .expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let base_url = format!("http://{}", server.http_addr);
    let client = AuthenticatedClient::new(&base_url);

    // === 1. Попытка создать пост без токена -> 401 ===
    let create = client.create_post("No Auth", "Should fail").await;
    assert_eq!(
        create.status(),
        401,
        "Create post without token should return 401"
    );

    // === 2. Регистрация двух пользователей ===
    let user1 = client.register("user1", "u1@test.com", "pass12321").await;
    let user2 = client.register("user2", "u2@test.com", "pass2sdfSDF").await;

    let client1 = client.clone().with_token(user1.token);
    let client2 = client.clone().with_token(user2.token);

    // === 3. User1 создаёт пост ===
    let post_resp = client1.create_post("User1 Post", "Content").await;
    assert_eq!(
        post_resp.status(),
        201,
        "User1 should be able to create post"
    );
    let post: Post = post_resp.json().await.expect("Should parse created post");

    // === 4. User2 пытается обновить пост User1 -> 403 Forbidden ===
    let update = client2.update_post(post.id, Some("Hacked"), None).await;
    assert_eq!(
        update.status(),
        403,
        "User2 should not be able to update User1's post"
    );

    // === 5. User2 пытается удалить пост User1 -> 403 Forbidden ===
    let delete = client2.delete_post(post.id).await;
    assert_eq!(
        delete.status(),
        403,
        "User2 should not be able to delete User1's post"
    );

    // === 6. User1 может обновить свой пост ===
    let update_own = client1
        .update_post(post.id, Some("Updated by Owner"), None)
        .await;
    assert_eq!(
        update_own.status(),
        200,
        "User1 should be able to update own post"
    );

    // === 7. User1 может удалить свой пост ===
    let delete_own = client1.delete_post(post.id).await;
    assert_eq!(
        delete_own.status(),
        204,
        "User1 should be able to delete own post"
    );

    // === 8. Проверка, что пост удалён ===
    let get_deleted = client1.get_post(post.id).await;
    assert_eq!(get_deleted.status(), 404, "Deleted post should return 404");

    // Останавливаем сервер
    server.handle.abort();
    //  Даём время на освобождение порта (TCP TIME_WAIT ~ 30-60 сек, но 200ms часто достаточно)
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// Тест пагинации списка постов
#[tokio::test]
async fn test_posts_pagination() {
    let config = test_config();

    cleanup_test_db(&config.database_url).await;

    let server = run_server(config.clone())
        .await
        .expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let base_url = format!("http://{}", server.http_addr);

    // Регистрируем пользователя и создаём 5 постов
    let client = AuthenticatedClient::new(&base_url.clone());
    let register = client.register("pager", "pager@test.com", "pass6ye").await;
    let auth_client = client.with_token(register.token);

    for i in 0..5 {
        let resp = auth_client
            .create_post(&format!("Post {}", i), &format!("Content {}", i))
            .await;
        assert_eq!(resp.status(), 201);
    }

    // === Тест пагинации: limit=2, offset=0 ===
    let list1 = auth_client
        .client
        .get(format!("{}/api/posts?limit=2&offset=0", base_url))
        .send()
        .await
        .expect("Request failed");

    let list1_resp: ListPostsResponse = list1.json().await.expect("Parse failed");
    assert_eq!(list1_resp.posts.len(), 2, "Should return 2 posts");
    assert_eq!(list1_resp.total, 5, "Total should be 5");
    assert_eq!(list1_resp.limit, 2);
    assert_eq!(list1_resp.offset, 0);

    // === Тест пагинации: limit=2, offset=2 ===
    let list2 = auth_client
        .client
        .get(format!("{}/api/posts?limit=2&offset=2", base_url))
        .send()
        .await
        .expect("Request failed");

    let list2_resp: ListPostsResponse = list2.json().await.expect("Parse failed");
    assert_eq!(list2_resp.posts.len(), 2, "Should return 2 posts");
    assert_eq!(list2_resp.offset, 2);

    // === Тест пагинации: limit=10, offset=0 (все посты) ===
    let list3 = auth_client
        .client
        .get(format!("{}/api/posts?limit=10&offset=0", base_url))
        .send()
        .await
        .expect("Request failed");

    let list3_resp: ListPostsResponse = list3.json().await.expect("Parse failed");
    assert_eq!(list3_resp.posts.len(), 5, "Should return all 5 posts");

    server.handle.abort();
    //  Даём время на освобождение порта (TCP TIME_WAIT ~ 30-60 сек, но 200ms часто достаточно)
    tokio::time::sleep(Duration::from_millis(200)).await;
}

/// Тест валидации входных данных
#[tokio::test]
async fn test_validation_errors() {
    let config = test_config();

    cleanup_test_db(&config.database_url).await;

    let server = run_server(config.clone())
        .await
        .expect("Failed to start server");
    tokio::time::sleep(Duration::from_secs(1)).await;

    let base_url = format!("http://{}", server.http_addr);
    let client = AuthenticatedClient::new(&base_url);

    // === Регистрация с пустым username -> 400 ===
    let resp = client
        .client
        .post(format!("{}/api/auth/register", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": "", "email": "test@test.com", "password": "pass" }))
        .send()
        .await
        .expect("Request failed");

    // Сервер может вернуть 400 или 422 в зависимости от валидации
    assert!(
        resp.status() == 400 || resp.status() == 422,
        "Empty username should return validation error, got {}",
        resp.status()
    );

    // === Регистрация с невалидным email -> 400 ===
    let resp = client
        .client
        .post(format!("{}/api/auth/register", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": "test", "email": "not-an-email", "password": "pass" }))
        .send()
        .await
        .expect("Request failed");

    assert!(
        resp.status() == 400 || resp.status() == 422,
        "Invalid email should return validation error, got {}",
        resp.status()
    );

    // === Регистрация с коротким паролем -> 400 ===
    let resp = client
        .client
        .post(format!("{}/api/auth/register", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": "test", "email": "test@test.com", "password": "123" }))
        .send()
        .await
        .expect("Request failed");

    assert!(
        resp.status() == 400 || resp.status() == 422,
        "Short password should return validation error, got {}",
        resp.status()
    );

    server.handle.abort();
    //  Даём время на освобождение порта (TCP TIME_WAIT ~ 30-60 сек, но 200ms часто достаточно)
    tokio::time::sleep(Duration::from_millis(200)).await;
}
