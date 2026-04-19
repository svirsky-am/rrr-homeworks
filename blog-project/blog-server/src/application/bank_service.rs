use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::data::account_repository::AccountRepository;
use crate::domain::{Account, Amount, BankError, DomainError, Transfer};

#[derive(Clone)]
pub struct BankService<R: AccountRepository + 'static> {
    repo: Arc<R>,
}

impl<R> BankService<R>
where
    R: AccountRepository + 'static,
{
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn create_account(
        &self,
        owner_id: Uuid,
        initial_balance: i64,
    ) -> Result<Account, BankError> {
        let account = Account::new(0, owner_id, initial_balance).map_err(BankError::from)?;
        self.repo.create(account).await.map_err(BankError::from)
    }

    #[instrument(skip(self))]
    pub async fn get_account(&self, id: i32) -> Result<Account, BankError> {
        match self.repo.get(id).await.map_err(BankError::from)? {
            Some(account) => Ok(account),
            None => Err(BankError::NotFound(format!("account {}", id))),
        }
    }

    #[instrument(skip(self))]
    pub async fn list_accounts(&self, owner_id: Uuid) -> Result<Vec<Account>, BankError> {
        self.repo.list_for_user(owner_id).await.map_err(BankError::from)
    }

    #[instrument(skip(self))]
    pub async fn deposit(&self, id: i32, amount: i64) -> Result<Account, BankError> {
        let mut account = self.get_account(id).await?;
        let amount = Amount::new(amount).map_err(BankError::from)?;
        account.deposit(amount);
        self.repo.update(account).await.map_err(BankError::from)
    }

    #[instrument(skip(self))]
    pub async fn withdraw(&self, id: i32, amount: i64) -> Result<Account, BankError> {
        let mut account = self.get_account(id).await?;
        let amount = Amount::new(amount).map_err(BankError::from)?;
        account
            .withdraw(amount)
            .map_err(|err| match err {
                DomainError::InsufficientFunds(acc) => BankError::InsufficientFunds(acc),
                other => BankError::from(other),
            })?;
        self.repo.update(account).await.map_err(BankError::from)
    }

    #[instrument(skip(self))]
    pub async fn transfer(&self, from: i32, to: i32, amount: i64) -> Result<(), BankError> {
        let transfer = Transfer::new(from, to, amount).map_err(BankError::from)?;
        let mut from_account = self.get_account(transfer.from).await?;
        let mut to_account = self.get_account(transfer.to).await?;

        from_account
            .withdraw(transfer.amount)
            .map_err(|err| match err {
                DomainError::InsufficientFunds(acc) => BankError::InsufficientFunds(acc),
                other => BankError::from(other),
            })?;
        to_account.deposit(transfer.amount);

        self.repo.update(from_account).await.map_err(BankError::from)?;
        self.repo.update(to_account).await.map_err(BankError::from)?;
        Ok(())
    }
}

