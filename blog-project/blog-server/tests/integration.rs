
use std::sync::Arc;
use axum::{Router, routing::get, Json};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_multiple_requests_with_arc_client() {
    // 1. Поднимаем минимальный тестовый сервер
    let app = Router::new()
        .route("/api/ping", get(|| async { Json(json!({"status": "ok"})) }));

    // Биндим на порт 0 → ОС сама выберет свободный порт
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Запускаем сервер в фоновой задаче
    // let server_handle = tokio::spawn(axum::serve(listener, app));
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("Server failed");
    });

    // 2. Создаём один экземпляр клиента и оборачиваем в Arc
    // В реальных тестах рекомендуется задавать таймаут, чтобы тест не висел бесконечно
    let client = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap()
    );

    // 3. Запускаем несколько параллельных запросов через один Arc-клиент
    let mut handles = Vec::new();
    for i in 0..3 {
        let client = client.clone(); // Клонирование Arc (операция счётчика ссылок, ~10ns)
        let url = format!("http://127.0.0.1:{}/api/ping", addr.port());

        handles.push(tokio::spawn(async move {
            let resp = client
                .get(&url)
                .send()
                .await
                .expect("Failed to send request");
            
            assert_eq!(resp.status(), 200);
            
            let body: serde_json::Value = resp
                .json()
                .await
                .expect("Failed to parse JSON");
            
            assert_eq!(body["status"], "ok");
            i // возвращаем индекс для проверки порядка завершения
        }));
    }

    // 4. Дожидаемся всех задач и проверяем результаты
    for (idx, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task panicked");
        assert_eq!(result, idx);
    }

    // 5. Останавливаем тестовый сервер
    server_handle.abort();
}