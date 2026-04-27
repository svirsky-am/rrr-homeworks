use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

use crate::domain::user::User;
impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub initial: i64,
}

#[derive(Debug, Deserialize)]
pub struct AmountRequest {
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from: i32,
    pub to: i32,
    pub amount: i64,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: i32,
    pub owner_id: Uuid,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::domain::Account> for AccountResponse {
    fn from(account: crate::domain::Account) -> Self {
        Self {
            id: account.id,
            owner_id: account.owner_id,
            balance: account.balance,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ExchangeResponse {
    pub from: String,
    pub to: String,
    pub rate: f64,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<crate::data::post::Post> for PostResponse {
    fn from(post: crate::data::post::Post) -> Self {
        Self {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PostsListResponse {
    pub posts: Vec<PostResponse>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

