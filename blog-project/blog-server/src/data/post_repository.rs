use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::data::post::{CreatePostRequest, Post, UpdatePostRequest};
use crate::domain::BlogError;

#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn create(&self, post: Post) -> Result<Post, BlogError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, BlogError>;
    async fn update(&self, post: Post) -> Result<Post, BlogError>;
    async fn delete(&self, id: i64) -> Result<(), BlogError>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, BlogError>;
    async fn count(&self) -> Result<i64, BlogError>;
}

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(Debug, FromRow)]
struct PostRow {
    id: i64,
    title: String,
    content: String,
    author_id: Uuid,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        Post {
            id: row.id,
            title: row.title,
            content: row.content,
            author_id: row.author_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn create(&self, post: Post) -> Result<Post, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to create post: {}", e);
            BlogError::Internal(format!("database error: {}", e))
        })?;

        Ok(row.into())
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Post>, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to find post by id {}: {}", id, e);
            BlogError::Internal(format!("database error: {}", e))
        })?;

        Ok(row.map(Into::into))
    }

    async fn update(&self, post: Post) -> Result<Post, BlogError> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            UPDATE posts
            SET title = $1, content = $2, updated_at = NOW()
            WHERE id = $3
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
        )
        .bind(&post.title)
        .bind(&post.content)
        .bind(post.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to update post {}: {}", post.id, e);
            if e.as_database_error()
                .and_then(|db| db.constraint())
                .map(|c| c.contains("posts_pkey"))
                == Some(true)
            {
                BlogError::NotFound(format!("post {}", post.id))
            } else {
                BlogError::Internal(format!("database error: {}", e))
            }
        })?;

        Ok(row.into())
    }

    async fn delete(&self, id: i64) -> Result<(), BlogError> {
        let result = sqlx::query(
            r#"
            DELETE FROM posts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to delete post {}: {}", id, e);
            BlogError::Internal(format!("database error: {}", e))
        })?;

        if result.rows_affected() == 0 {
            return Err(BlogError::NotFound(format!("post {}", id)));
        }

        Ok(())
    }

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Post>, BlogError> {
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to list posts: {}", e);
            BlogError::Internal(format!("database error: {}", e))
        })?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn count(&self) -> Result<i64, BlogError> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM posts
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("failed to count posts: {}", e);
            BlogError::Internal(format!("database error: {}", e))
        })?;

        Ok(row.0)
    }
}
