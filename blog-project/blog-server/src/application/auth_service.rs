use std::sync::Arc;

use tracing::instrument;

use crate::data::user_repository::UserRepository;
use crate::domain::{user::User, BankError};
use crate::infrastructure::security::{hash_password, verify_password, JwtKeys};

#[derive(Clone)]
pub struct AuthService<R: UserRepository + 'static> {
    repo: Arc<R>,
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
    
    pub async fn get_user(&self, id: uuid::Uuid) -> Result<User, BankError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(BankError::from)?
            .ok_or_else(|| BankError::NotFound(format!("user {}", id)))
    }

    #[instrument(skip(self))]
    pub async fn register(&self, email: String, password: String) -> Result<User, BankError> {
        let hash = hash_password(&password).map_err(|err| BankError::Internal(err.to_string()))?;
        let user = User::new(email.to_lowercase(), hash);
        self.repo.create(user).await.map_err(BankError::from)
    }

    #[instrument(skip(self))]
    pub async fn login(&self, email: &str, password: &str) -> Result<String, BankError> {
        let user = self
            .repo
            .find_by_email(&email.to_lowercase())
            .await
            .map_err(BankError::from)?
            .ok_or_else(|| BankError::Unauthorized)?;

        let valid = verify_password(password, &user.password_hash)
            .map_err(|_| BankError::Unauthorized)?;
        if !valid {
            return Err(BankError::Unauthorized);
        }

        self.keys
            .generate_token(user.id)
            .map_err(|err| BankError::Internal(err.to_string()))
    }
}


