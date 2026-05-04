// src/presentation/grpc_service.rs
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::blog::blog_service_server::BlogService as GrpcBlogService;
use crate::blog::{
    AuthResponse, CreatePostRequest, DeleteResponse, ListPostsRequest, ListPostsResponse,
    LoginRequest, Post as GrpcPost, PostId, PostResponse, RegisterRequest, UpdatePostRequest,
    User as GrpcUser,
};

use crate::application::{AuthService, BlogService};
use crate::domain::{
    CreatePost as DomainCreatePost, LoginRequest as DomainLogin, RegisterRequest as DomainRegister,
    UpdatePost as DomainUpdatePost, UserPublic,
};
use crate::infrastructure::JwtService;

#[derive(Clone)]
pub struct BlogGrpcService {
    auth_service: Arc<AuthService>,
    blog_service: Arc<BlogService>,
    jwt_service: JwtService,
}

impl BlogGrpcService {
    pub fn new(
        auth_service: Arc<AuthService>,
        blog_service: Arc<BlogService>,
        jwt_service: JwtService,
    ) -> Self {
        Self {
            auth_service,
            blog_service,
            jwt_service,
        }
    }

    fn extract_token(&self, metadata: &tonic::metadata::MetadataMap) -> Result<String, Status> {
        metadata
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(String::from)
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization header"))
    }

    fn verify_grpc_token(&self, token: &str) -> Result<(i64, String), Status> {
        let claims = self
            .jwt_service
            .verify_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;
        Ok((claims.user_id, claims.username))
    }

    fn domain_to_grpc_user(&self, user: UserPublic) -> GrpcUser {
        GrpcUser {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.to_rfc3339(),
        }
    }

    fn domain_to_grpc_post(&self, post: crate::domain::Post) -> GrpcPost {
        GrpcPost {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

#[tonic::async_trait]
impl GrpcBlogService for BlogGrpcService {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let domain_req = DomainRegister {
            username: req.username,
            email: req.email,
            password: req.password,
        };

        match self.auth_service.register(domain_req).await {
            Ok((token, user)) => {
                let grpc_user = self.domain_to_grpc_user(user);
                Ok(Response::new(AuthResponse {
                    token,
                    user: Some(grpc_user),
                }))
            }
            Err(crate::domain::DomainError::Validation(msg)) => {
                Err(Status::invalid_argument(msg)) // 400 InvalidArgument
            }
            Err(crate::domain::DomainError::UserAlreadyExists) => {
                Err(Status::already_exists("User already exists"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let req = request.into_inner();
        let domain_req = DomainLogin {
            username: req.username,
            password: req.password,
        };

        match self.auth_service.login(domain_req).await {
            Ok((token, user)) => {
                let grpc_user = self.domain_to_grpc_user(user);
                Ok(Response::new(AuthResponse {
                    token,
                    user: Some(grpc_user),
                }))
            }
            // Валидация -> InvalidArgument
            Err(crate::domain::DomainError::Validation(msg)) => Err(Status::invalid_argument(msg)),
            // Неверные креды -> Unauthenticated
            Err(crate::domain::DomainError::InvalidCredentials) => {
                Err(Status::unauthenticated("Invalid credentials"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn get_post(&self, request: Request<PostId>) -> Result<Response<PostResponse>, Status> {
        let id = request.into_inner().id;

        match self.blog_service.get_post(id).await {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(self.domain_to_grpc_post(post)),
            })),
            Err(crate::domain::DomainError::PostNotFound) => {
                Err(Status::not_found("Post not found"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let req = request.into_inner();
        let limit = req.limit.unwrap_or(10).min(100);
        let offset = req.offset.unwrap_or(0);

        match self.blog_service.list_posts(limit, offset).await {
            Ok((posts, total)) => {
                let grpc_posts = posts
                    .into_iter()
                    .map(|p| self.domain_to_grpc_post(p))
                    .collect();
                Ok(Response::new(ListPostsResponse {
                    posts: grpc_posts,
                    total,
                    limit,
                    offset,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let metadata = request.metadata();
        let token = self.extract_token(metadata)?;
        let (user_id, _) = self.verify_grpc_token(&token)?;

        let req = request.into_inner();
        let domain_req = DomainCreatePost {
            title: req.title,
            content: req.content,
        };

        match self.blog_service.create_post(user_id, domain_req).await {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(self.domain_to_grpc_post(post)),
            })),
            Err(crate::domain::DomainError::Validation(msg)) => Err(Status::invalid_argument(msg)),
            Err(e) => Err(Status::invalid_argument(e.to_string())),
        }
    }

    async fn update_post(
        &self,
        request: Request<UpdatePostRequest>,
    ) -> Result<Response<PostResponse>, Status> {
        let metadata = request.metadata();
        let token = self.extract_token(metadata)?;
        let (user_id, _) = self.verify_grpc_token(&token)?;

        let req = request.into_inner();
        let domain_req = DomainUpdatePost {
            title: req.title.filter(|s| !s.is_empty()),
            content: req.content.filter(|s| !s.is_empty()),
        };

        match self
            .blog_service
            .update_post(req.id, user_id, domain_req)
            .await
        {
            Ok(post) => Ok(Response::new(PostResponse {
                post: Some(self.domain_to_grpc_post(post)),
            })),
            Err(crate::domain::DomainError::PostNotFound) => {
                Err(Status::not_found("Post not found"))
            }
            Err(crate::domain::DomainError::Forbidden) => {
                Err(Status::permission_denied("Not the author"))
            }
            Err(e) => Err(Status::invalid_argument(e.to_string())),
        }
    }

    async fn delete_post(
        &self,
        request: Request<PostId>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let metadata = request.metadata();
        let token = self.extract_token(metadata)?;
        let (user_id, _) = self.verify_grpc_token(&token)?;

        let id = request.into_inner().id;

        match self.blog_service.delete_post(id, user_id).await {
            Ok(_) => Ok(Response::new(DeleteResponse { success: true })),
            Err(crate::domain::DomainError::PostNotFound) => {
                Err(Status::not_found("Post not found"))
            }
            Err(crate::domain::DomainError::Forbidden) => {
                Err(Status::permission_denied("Not the author"))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
