// use std::sync::Arc;
use crate::data::PostRepository;
use crate::domain::{AppResult, CreatePost, DomainError, Post, UpdatePost};

#[derive(Clone)]
pub struct BlogService {
    post_repo: PostRepository,
}

impl BlogService {
    pub fn new(post_repo: PostRepository) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(&self, author_id: i64, input: CreatePost) -> AppResult<Post> {
        if input.title.trim().is_empty() {
            return Err(DomainError::Validation("Title cannot be empty".into()));
        }
        self.post_repo.create(author_id, input).await
    }

    pub async fn get_post(&self, id: i64) -> AppResult<Post> {
        self.post_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn list_posts(&self, limit: i64, offset: i64) -> AppResult<(Vec<Post>, i64)> {
        self.post_repo.list(limit, offset).await
    }

    pub async fn update_post(&self, id: i64, author_id: i64, input: UpdatePost) -> AppResult<Post> {
        self.post_repo
            .update(id, author_id, input)
            .await?
            .ok_or(DomainError::Forbidden)
    }

    pub async fn delete_post(&self, id: i64, author_id: i64) -> AppResult<()> {
        let deleted = self.post_repo.delete(id, author_id).await?;
        if !deleted {
            return Err(DomainError::Forbidden);
        }
        Ok(())
    }
}
