// Условная компиляция для разных платформ
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod wasm_client;
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub use wasm_client::*;

#[cfg(not(target_arch = "wasm32"))]
mod http_client;
#[cfg(not(target_arch = "wasm32"))]
mod grpc_client;

#[cfg(not(target_arch = "wasm32"))]
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub use error::BlogClientError;

// Общие типы (доступны везде)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

// Транспорт для не-WASM платформ
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[cfg(not(target_arch = "wasm32"))]
pub struct BlogClient {
    #[cfg(feature = "http")]
    http_client: Option<reqwest::Client>,
    #[cfg(feature = "grpc")]
    grpc_client: Option<blog::blog_service_client::BlogServiceClient<tonic::transport::Channel>>,
    transport: Transport,
    token: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl BlogClient {
    /// Создание клиента с выбранным транспортом
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        Ok(Self {
            #[cfg(feature = "http")]
            http_client: if matches!(transport, Transport::Http(_)) {
                Some(reqwest::Client::new())
            } else { None },
            #[cfg(feature = "grpc")]
            grpc_client: if let Transport::Grpc(addr) = &transport {
                let channel = tonic::transport::Channel::from_shared(addr.clone())?
                    .connect()
                    .await?;
                Some(blog::blog_service_client::BlogServiceClient::new(channel))
            } else { None },
            transport,
            token: None,
        })
    }

    /// Установить JWT-токен
    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    /// Получить текущий токен
    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    /// Регистрация пользователя
    pub async fn register(&mut self, username: &str, email: &str, password: &str) 
        -> Result<AuthResponse, BlogClientError> 
    {
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::register(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    username, email, password
                ).await.map(|r| {
                    self.token = Some(r.token.clone());
                    r
                })
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::register(
                    self.grpc_client.as_ref().unwrap(),
                    username, email, password
                ).await.map(|r| {
                    self.token = Some(r.token.clone());
                    r
                })
            }
        }
    }

    /// Вход в систему
    pub async fn login(&mut self, username: &str, password: &str) 
        -> Result<AuthResponse, BlogClientError> 
    {
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::login(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    username, password
                ).await.map(|r| {
                    self.token = Some(r.token.clone());
                    r
                })
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::login(
                    self.grpc_client.as_ref().unwrap(),
                    username, password
                ).await.map(|r| {
                    self.token = Some(r.token.clone());
                    r
                })
            }
        }
    }

    /// Создание поста (требует токен)
    pub async fn create_post(&self, title: &str, content: &str) 
        -> Result<Post, BlogClientError> 
    {
        let token = self.token.clone()
            .ok_or(BlogClientError::Unauthorized("Not authenticated".into()))?;
        
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::create_post(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    &token,
                    title, content
                ).await
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::create_post(
                    self.grpc_client.as_ref().unwrap(),
                    &token,
                    title, content
                ).await
            }
        }
    }

    /// Получение поста по ID (публичный)
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::get_post(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    id
                ).await
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::get_post(
                    self.grpc_client.as_ref().unwrap(),
                    id
                ).await
            }
        }
    }

    /// Обновление поста (требует токен, только автор)
    pub async fn update_post(&self, id: i64, title: Option<&str>, content: Option<&str>) 
        -> Result<Post, BlogClientError> 
    {
        let token = self.token.clone()
            .ok_or(BlogClientError::Unauthorized("Not authenticated".into()))?;
        
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::update_post(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    &token,
                    id, title, content
                ).await
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::update_post(
                    self.grpc_client.as_ref().unwrap(),
                    &token,
                    id, title, content
                ).await
            }
        }
    }

    /// Удаление поста (требует токен, только автор)
    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let token = self.token.clone()
            .ok_or(BlogClientError::Unauthorized("Not authenticated".into()))?;
        
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::delete_post(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    &token,
                    id
                ).await
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::delete_post(
                    self.grpc_client.as_ref().unwrap(),
                    &token,
                    id
                ).await
            }
        }
    }

    /// Список постов с пагинацией (публичный)
    pub async fn list_posts(&self, limit: Option<i64>, offset: Option<i64>) 
        -> Result<ListPostsResponse, BlogClientError> 
    {
        match &self.transport {
            #[cfg(feature = "http")]
            Transport::Http(base_url) => {
                http_client::list_posts(
                    self.http_client.as_ref().unwrap(),
                    base_url,
                    limit, offset
                ).await
            }
            #[cfg(feature = "grpc")]
            Transport::Grpc(_) => {
                grpc_client::list_posts(
                    self.grpc_client.as_ref().unwrap(),
                    limit, offset
                ).await
            }
        }
    }
}