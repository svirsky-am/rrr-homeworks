pub mod error;
pub mod user;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use error::{BankError, DomainError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub owner_id: Uuid,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(id: i32, owner_id: Uuid, initial_balance: i64) -> Result<Self, DomainError> {
        if initial_balance < 0 {
            return Err(DomainError::Validation(
                "initial balance must be non-negative".into(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id,
            owner_id,
            balance: initial_balance,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn deposit(&mut self, amount: Amount) {
        self.balance += amount.0;
        self.updated_at = Utc::now();
    }

    pub fn withdraw(&mut self, amount: Amount) -> Result<(), DomainError> {
        if self.balance < amount.0 {
            return Err(DomainError::InsufficientFunds(self.id));
        }
        self.balance -= amount.0;
        self.updated_at = Utc::now();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Amount(pub i64);

impl Amount {
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value <= 0 {
            return Err(DomainError::Validation("amount must be positive".into()));
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub from: i32,
    pub to: i32,
    pub amount: Amount,
}

impl Transfer {
    pub fn new(from: i32, to: i32, amount: i64) -> Result<Self, DomainError> {
        if from == to {
            return Err(DomainError::Validation(
                "source and destination accounts must differ".into(),
            ));
        }
        Ok(Self {
            from,
            to,
            amount: Amount::new(amount)?,
        })
    }
}

