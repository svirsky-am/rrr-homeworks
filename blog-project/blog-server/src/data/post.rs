use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    pub fn new(id: i64, title: String, content: String, author_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            content,
            author_id,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}
