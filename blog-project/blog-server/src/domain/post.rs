use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
}

impl Post {
    pub fn new(author_id: i64, title: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by DB
            title,
            content,
            author_id,
            created_at: now,
            updated_at: now,
        }
    }
}