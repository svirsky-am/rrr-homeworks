// use std::sync::Arc;
use crate::domain::{RegisterRequest, LoginRequest, UserPublic, DomainError, AppResult};
use crate::data::UserRepository;
use crate::infrastructure::JwtService;

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_service: JwtService) -> Self {
        Self { user_repo, jwt_service }
    }
    
    fn validate_registration(req: &RegisterRequest) -> Result<(), DomainError> {
        if req.username.trim().is_empty() {
            return Err(DomainError::Validation("Username cannot be empty".into()));
        }
        if req.username.len() < 3 || req.username.len() > 50 {
            return Err(DomainError::Validation("Username must be 3-50 characters".into()));
        }
        if !req.email.contains('@') || !req.email.contains('.') {
            return Err(DomainError::Validation("Invalid email format".into()));
        }
        if req.password.len() < 6 {
            return Err(DomainError::Validation("Password must be at least 6 characters".into()));
        }
        Ok(())
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<(String, UserPublic)> {
        // Валидация в сервисе — работает для HTTP и gRPC!
        Self::validate_registration(&req)?;
        
        let user = self.user_repo.create(req).await?;
        let token = self.jwt_service.generate_token(user.id, user.username.clone())?;
        Ok((token, UserPublic::from(user)))
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<(String, UserPublic)> {
        // Опционально: валидация логина тоже
        if req.username.trim().is_empty() || req.password.is_empty() {
            return Err(DomainError::InvalidCredentials);
        }
        
        let user = self.user_repo.find_by_username(&req.username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        if !self.user_repo.verify_password(&req.password, &user.password_hash)? {
            return Err(DomainError::InvalidCredentials);
        }

        let token = self.jwt_service.generate_token(user.id, user.username.clone())?;
        Ok((token, UserPublic::from(user)))
    }
}