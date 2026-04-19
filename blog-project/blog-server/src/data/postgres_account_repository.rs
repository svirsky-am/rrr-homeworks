use async_trait::async_trait;
use sqlx::{postgres::PgRow, PgPool, Row};
use tracing::{error, info};
use uuid::Uuid;

use crate::data::account_repository::AccountRepository;
use crate::domain::{Account, DomainError};

fn map_row(row: PgRow) -> Result<Account, DomainError> {
    let decode_err = |e: sqlx::Error| DomainError::Internal(format!("row decode error: {}", e));

    Ok(Account {
        id: row.try_get("id").map_err(decode_err)?,
        owner_id: row.try_get("owner_id").map_err(decode_err)?,
        balance: row.try_get("balance").map_err(decode_err)?,
        created_at: row.try_get("created_at").map_err(decode_err)?,
        updated_at: row.try_get("updated_at").map_err(decode_err)?,
    })
}

#[derive(Clone)]
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn create(&self, mut account: Account) -> Result<Account, DomainError> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (owner_id, balance)
            VALUES ($1, $2)
            RETURNING id, owner_id, balance, created_at, updated_at
            "#,
        )
        .bind(account.owner_id)
        .bind(account.balance)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to create account: {}", e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        account = map_row(row)?;
        info!(account_id = account.id, owner = %account.owner_id, "account created");
        Ok(account)
    }

    async fn get(&self, id: i32) -> Result<Option<Account>, DomainError> {
        let maybe_row = sqlx::query(
            r#"
            SELECT id, owner_id, balance, created_at, updated_at
            FROM accounts
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to fetch account {}: {}", id, e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        match maybe_row {
            Some(row) => Ok(Some(map_row(row)?)),
            None => Ok(None),
        }
    }

    async fn update(&self, account: Account) -> Result<Account, DomainError> {
        let row = sqlx::query(
            r#"
            UPDATE accounts
            SET balance = $1, updated_at = NOW()
            WHERE id = $2
            RETURNING id, owner_id, balance, created_at, updated_at
            "#,
        )
        .bind(account.balance)
        .bind(account.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to update account {}: {}", account.id, e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        match row {
            Some(row) => {
                let account = map_row(row)?;
                info!(account_id = account.id, balance = account.balance, "account updated");
                Ok(account)
            }
            None => Err(DomainError::AccountNotFound(account.id)),
        }
    }

    async fn list_for_user(&self, owner_id: Uuid) -> Result<Vec<Account>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, owner_id, balance, created_at, updated_at
            FROM accounts
            WHERE owner_id = $1
            ORDER BY id
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("failed to list accounts for user {}: {}", owner_id, e);
            DomainError::Internal(format!("database error: {}", e))
        })?;

        rows.into_iter()
            .map(map_row)
            .collect::<Result<Vec<_>, _>>()
    }
}

