pub mod error;
pub mod post;
pub mod user;

pub use error::{AppResult, DomainError};
pub use post::{CreatePost, Post, UpdatePost};
pub use user::{LoginRequest, RegisterRequest, User, UserPublic};
