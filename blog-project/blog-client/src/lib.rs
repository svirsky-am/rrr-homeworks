// 1. Сгенерированный gRPC-код
pub mod blog {
    tonic::include_proto!("blog");
}

// 2. Модули
pub mod error;
mod grpc_client;
mod http_client;
pub use error::BlogClientError;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// 3. Общие типы
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

// 4. Транспорт (выбирается ОДИН раз при создании)
pub enum TransportClient {
    Http(reqwest::Client, String), // client + base_url
    Grpc(Arc<Mutex<blog::blog_service_client::BlogServiceClient<tonic::transport::Channel>>>),
}

// 5. Клиент
pub struct BlogClient {
    transport: TransportClient,
    token: Option<String>,
}

impl BlogClient {
    /// Создать HTTP-клиент
    pub async fn new_http(base_url: String) -> Result<Self, BlogClientError> {
        Ok(Self {
            transport: TransportClient::Http(reqwest::Client::new(), base_url),
            token: None,
        })
    }

    /// Создать gRPC-клиент
    pub async fn new_grpc(grpc_url: String) -> Result<Self, BlogClientError> {
        let channel = tonic::transport::Channel::from_shared(grpc_url)?
            .connect()
            .await?;
        let client = blog::blog_service_client::BlogServiceClient::new(channel);
        Ok(Self {
            transport: TransportClient::Grpc(Arc::new(Mutex::new(client))),
            token: None,
        })
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }
    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    // === REGISTER ===
    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let resp = match &mut self.transport {
            TransportClient::Http(c, url) => {
                http_client::register(c, url, username, email, password).await?
            }
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::register(&mut *guard, username, email, password).await?
            }
        };
        self.token = Some(resp.token.clone());
        Ok(resp)
    }

    // === LOGIN ===
    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let resp = match &mut self.transport {
            TransportClient::Http(c, url) => http_client::login(c, url, username, password).await?,
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::login(&mut *guard, username, password).await?
            }
        };
        self.token = Some(resp.token.clone());
        Ok(resp)
    }

    // === CREATE POST ===
    pub async fn create_post(&self, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| BlogClientError::Unauthorized("Not authenticated".into()))?;
        match &self.transport {
            TransportClient::Http(c, url) => {
                http_client::create_post(c, url, token, title, content).await
            }
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::create_post(&mut *guard, token, title, content).await
            }
        }
    }

    // === GET POST ===
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        match &self.transport {
            TransportClient::Http(c, url) => http_client::get_post(c, url, id).await,
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::get_post(&mut *guard, id).await
            }
        }
    }

    // === UPDATE POST ===
    pub async fn update_post(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<Post, BlogClientError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| BlogClientError::Unauthorized("Not authenticated".into()))?;
        match &self.transport {
            TransportClient::Http(c, url) => {
                http_client::update_post(c, url, token, id, title, content).await
            }
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::update_post(&mut *guard, token, id, title, content).await
            }
        }
    }

    // === DELETE POST ===
    pub async fn delete_post(&self, id: i64) -> Result<(), BlogClientError> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| BlogClientError::Unauthorized("Not authenticated".into()))?;
        match &self.transport {
            TransportClient::Http(c, url) => http_client::delete_post(c, url, token, id).await,
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::delete_post(&mut *guard, token, id).await
            }
        }
    }

    // === LIST POSTS ===
    pub async fn list_posts(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<ListPostsResponse, BlogClientError> {
        match &self.transport {
            TransportClient::Http(c, url) => http_client::list_posts(c, url, limit, offset).await,
            TransportClient::Grpc(arc) => {
                let mut guard = arc.lock().await;
                grpc_client::list_posts(&mut *guard, limit, offset).await
            }
        }
    }
}
