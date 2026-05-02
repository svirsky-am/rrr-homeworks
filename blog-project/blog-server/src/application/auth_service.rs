// use std::sync::Arc;
use crate::data::UserRepository;
use crate::infrastructure::JwtService;
use crate::domain::{RegisterRequest, LoginRequest, UserPublic, DomainError, AppResult};

#[derive(Clone)]
pub struct AuthService {
    user_repo: UserRepository,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(user_repo: UserRepository, jwt_service: JwtService) -> Self {
        Self { user_repo, jwt_service }
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<(String, UserPublic)> {
        let user = self.user_repo.create(req).await?;
        let token = self.jwt_service.generate_token(user.id, user.username.clone())?;
        Ok((token, UserPublic::from(user)))
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<(String, UserPublic)> {
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