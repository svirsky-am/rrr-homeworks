use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::data::post::{CreatePostRequest, Post, UpdatePostRequest};
use crate::data::post_repository::PostRepository;
use crate::domain::BlogError;

#[derive(Clone)]
pub struct BlogService<R: PostRepository + 'static> {
    repo: Arc<R>,
}

impl<R> BlogService<R>
where
    R: PostRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_post(
        &self,
        title: String,
        content: String,
        author_id: Uuid,
    ) -> Result<Post, BlogError> {
        let post = Post::new(0, title, content, author_id);
        self.repo.create(post).await
    }

    #[instrument(skip(self))]
    pub async fn get_post(&self, id: i64) -> Result<Post, BlogError> {
        match self.repo.find_by_id(id).await? {
            Some(post) => Ok(post),
            None => Err(BlogError::PostNotFound(id)),
        }
    }

    #[instrument(skip(self))]
    pub async fn update_post(
        &self,
        id: i64,
        title: String,
        content: String,
        author_id: Uuid,
    ) -> Result<Post, BlogError> {
        let mut post = self.get_post(id).await?;

        // Проверка, что пользователь является автором поста
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }

        post.title = title;
        post.content = content;

        self.repo.update(post).await
    }

    #[instrument(skip(self))]
    pub async fn delete_post(&self, id: i64, author_id: Uuid) -> Result<(), BlogError> {
        let post = self.get_post(id).await?;

        // Проверка, что пользователь является автором поста
        if post.author_id != author_id {
            return Err(BlogError::Forbidden);
        }

        self.repo.delete(id).await
    }

    #[instrument(skip(self))]
    pub async fn list_posts(&self, limit: i64, offset: i64) -> Result<Vec<Post>, BlogError> {
        self.repo.list(limit, offset).await
    }

    #[instrument(skip(self))]
    pub async fn count_posts(&self) -> Result<i64, BlogError> {
        self.repo.count().await
    }
}
