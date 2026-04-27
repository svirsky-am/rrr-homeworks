use std::sync::Arc;

use tracing::instrument;

use crate::data::user_repository::UserRepository;
use crate::domain::{user::User, BlogError};
use crate::infrastructure::security::{hash_password, verify_password, JwtKeys};

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
    pub repo: Arc<R>,
    keys: JwtKeys,
}

impl<R> AuthService<R>
where
    R: UserRepository + 'static,
{
    pub fn new(repo: Arc<R>, keys: JwtKeys) -> Self {
        Self { repo, keys }
    }

    pub fn keys(&self) -> &JwtKeys {
        &self.keys
    }
    
    pub async fn get_user(&self, id: uuid::Uuid) -> Result<User, BlogError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(BlogError::from)?
            .ok_or_else(|| BlogError::UserNotFound(id))
    }

    #[instrument(skip(self))]
    pub async fn register(&self, username: String, email: String, password: String) -> Result<User, BlogError> {
        let hash = hash_password(&password).map_err(|err| BlogError::Internal(err.to_string()))?;
        let user = User::new(username, email.to_lowercase(), hash);
        self.repo.create(user).await.map_err(|e| match e {
            crate::domain::DomainError::Validation(msg) => BlogError::UserAlreadyExists,
            other => BlogError::from(other),
        })
    }

    #[instrument(skip(self))]
    pub async fn login(&self, username: &str, password: &str) -> Result<String, BlogError> {
        let user = self
            .repo
            .find_by_username(&username.to_lowercase())
            .await
            .map_err(BlogError::from)?
            .ok_or_else(|| BlogError::InvalidCredentials)?;

        let valid = verify_password(password, &user.password_hash)
            .map_err(|_| BlogError::InvalidCredentials)?;
        if !valid {
            return Err(BlogError::InvalidCredentials);
        }

        self.keys
            .generate_token(user.id)
            .map_err(|err| BlogError::Internal(err.to_string()))
    }
}


