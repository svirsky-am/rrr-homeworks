use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};

// === Модели (дублируем из blog-client для независимости) ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
}

// === Основной WASM-экспорт ===

#[wasm_bindgen]
pub struct BlogApp {
    server_url: String,
}

#[wasm_bindgen]
impl BlogApp {
    /// Конструктор
    #[wasm_bindgen(constructor)]
    pub fn new(server_url: String) -> Self {
        Self { server_url }
    }

    /// Проверка аутентификации
    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.get_token().is_some()
    }

    /// Регистрация
    #[wasm_bindgen]
    pub async fn register(
        &self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        let body_str = serde_json::to_string(&serde_json::json!({
            "username": username,
            "email": email,
            "password": password
        }))
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        let resp = Request::post(&format!("{}/api/auth/register", self.server_url))
            .header("Content-Type", "application/json")
            .body(body_str) // Теперь передаём JsValue
            .map_err(|e| JsValue::from_str(&format!("Request error: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if !resp.ok() {
            let text = resp.text().await.unwrap_or_else(|_| "Unknown error".into());
            return Err(JsValue::from_str(&format!("Register failed: {}", text)));
        }

        let auth: AuthResponse = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        // Сохраняем токен
        self.save_token(&auth.token);

        serde_wasm_bindgen::to_value(&auth).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Логин
    #[wasm_bindgen]
    pub async fn login(&self, username: String, password: String) -> Result<JsValue, JsValue> {
        let body_str = serde_json::to_string(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        let resp = Request::post(&format!("{}/api/auth/login", self.server_url))
            .header("Content-Type", "application/json")
            .body(body_str)
            .map_err(|e| JsValue::from_str(&format!("Request error: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if !resp.ok() {
            return Err(JsValue::from_str("Invalid credentials"));
        }

        let auth: AuthResponse = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        self.save_token(&auth.token);

        serde_wasm_bindgen::to_value(&auth).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Выход (удаляет токен)
    #[wasm_bindgen]
    pub fn logout(&self) {
        self.clear_token();
    }

    /// Загрузка списка постов
    #[wasm_bindgen]
    pub async fn load_posts(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<JsValue, JsValue> {
        let mut url = format!("{}/api/posts", self.server_url);
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }
        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        let resp = Request::get(&url)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if !resp.ok() {
            return Err(JsValue::from_str("Failed to load posts"));
        }

        let list: ListPostsResponse = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_wasm_bindgen::to_value(&list).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Получение поста по ID
    #[wasm_bindgen]
    pub async fn get_post(&self, id: i32) -> Result<JsValue, JsValue> {
        let resp = Request::get(&format!("{}/api/posts/{}", self.server_url, id))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if resp.status() == 404 {
            return Err(JsValue::from_str("Post not found"));
        }
        if !resp.ok() {
            return Err(JsValue::from_str("Failed to get post"));
        }

        let post: Post = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Создание поста (требует токен)
    #[wasm_bindgen]
    pub async fn create_post(&self, title: String, content: String) -> Result<JsValue, JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let body_str = serde_json::to_string(&serde_json::json!({
            "title": title,
            "content": content
        }))
        .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        let resp = Request::post(&format!("{}/api/posts", self.server_url))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body_str)
            .map_err(|e| JsValue::from_str(&format!("Request error: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if !resp.ok() {
            return Err(JsValue::from_str("Failed to create post"));
        }

        let post: Post = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Обновление поста (требует токен, только автор)
    #[wasm_bindgen]
    pub async fn update_post(
        &self,
        id: i32,
        title: Option<String>,
        content: Option<String>,
    ) -> Result<JsValue, JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let mut data = serde_json::Map::new();
        if let Some(t) = title {
            data.insert("title".into(), serde_json::Value::String(t));
        }
        if let Some(c) = content {
            data.insert("content".into(), serde_json::Value::String(c));
        }

        let body_str = serde_json::to_string(&data)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))?;

        let resp = Request::put(&format!("{}/api/posts/{}", self.server_url, id))
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", token))
            .body(body_str)
            .map_err(|e| JsValue::from_str(&format!("Request error: {}", e)))?
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if resp.status() == 404 {
            return Err(JsValue::from_str("Post not found"));
        }
        if resp.status() == 403 {
            return Err(JsValue::from_str("Not the author"));
        }
        if !resp.ok() {
            return Err(JsValue::from_str("Failed to update post"));
        }

        let post: Post = resp
            .json()
            .await
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_wasm_bindgen::to_value(&post).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Удаление поста (требует токен, только автор)
    #[wasm_bindgen]
    pub async fn delete_post(&self, id: i32) -> Result<(), JsValue> {
        let token = self
            .get_token()
            .ok_or_else(|| JsValue::from_str("Not authenticated"))?;

        let resp = Request::delete(&format!("{}/api/posts/{}", self.server_url, id))
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Send error: {}", e)))?;

        if resp.status() == 404 {
            return Err(JsValue::from_str("Post not found"));
        }
        if resp.status() == 403 {
            return Err(JsValue::from_str("Not the author"));
        }
        if !resp.ok() {
            return Err(JsValue::from_str("Failed to delete post"));
        }

        Ok(())
    }

    // === Работа с localStorage ===

    fn storage(&self) -> Option<Storage> {
        window()?.local_storage().ok()?
    }

    fn save_token(&self, token: &str) {
        if let Some(storage) = self.storage() {
            let _ = storage.set_item("blog_token", token);
        }
    }

    fn get_token(&self) -> Option<String> {
        self.storage()?.get_item("blog_token").ok()?
    }

    fn clear_token(&self) {
        if let Some(storage) = self.storage() {
            let _ = storage.remove_item("blog_token");
        }
    }
}

// === Глобальная инициализация для удобства ===

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
