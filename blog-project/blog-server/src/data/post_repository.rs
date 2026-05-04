use crate::domain::{AppResult, CreatePost, Post, UpdatePost};
use sqlx::PgPool;

#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, author_id: i64, input: CreatePost) -> AppResult<Post> {
        Ok(sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
            input.title,
            input.content,
            author_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<Post>> {
        Ok(sqlx::query_as!(
            Post,
            r#"SELECT id, title, content, author_id, created_at, updated_at FROM posts WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> AppResult<(Vec<Post>, i64)> {
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT id, title, content, author_id, created_at, updated_at
            FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let total = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM posts"#)
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);

        Ok((posts, total))
    }

    pub async fn update(
        &self,
        id: i64,
        author_id: i64,
        input: UpdatePost,
    ) -> AppResult<Option<Post>> {
        // Проверяем, что пользователь — автор
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1 AND author_id = $2)"#,
            id,
            author_id
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(false);

        if !exists {
            return Ok(None);
        }

        Ok(sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET 
                title = COALESCE($3, title),
                content = COALESCE($4, content),
                updated_at = NOW()
            WHERE id = $1 AND author_id = $2
            RETURNING id, title, content, author_id, created_at, updated_at
            "#,
            id,
            author_id,
            input.title,
            input.content
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn delete(&self, id: i64, author_id: i64) -> AppResult<bool> {
        let result = sqlx::query!(
            r#"DELETE FROM posts WHERE id = $1 AND author_id = $2"#,
            id,
            author_id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
