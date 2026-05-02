use sqlx::PgPool;
use crate::domain::{User, RegisterRequest, DomainError, AppResult};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

#[derive(Clone)]
pub struct UserRepository { pool: PgPool }

impl UserRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }

    pub async fn create(&self, req: RegisterRequest) -> AppResult<User> {
        // Хешируем пароль через Argon2
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(req.password.as_bytes(), &salt)
            .map_err(|e| DomainError::Hash(e.to_string()))?
            .to_string();

        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            "#,
            req.username,
            req.email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => 
                DomainError::UserAlreadyExists,
            _ => DomainError::Database(e),
        })?;

        Ok(user)
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1"#,
            username
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = argon2::PasswordHash::new(hash)
            .map_err(|e| DomainError::Hash(e.to_string()))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}