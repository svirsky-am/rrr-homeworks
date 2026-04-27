use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Account, DomainError};

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn create(&self, account: Account) -> Result<Account, DomainError>;
    async fn get(&self, id: i32) -> Result<Option<Account>, DomainError>;
    async fn update(&self, account: Account) -> Result<Account, DomainError>;
    async fn list_for_user(&self, owner_id: Uuid) -> Result<Vec<Account>, DomainError>;
}

