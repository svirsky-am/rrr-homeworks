pub mod post;
pub mod user;
pub mod error;

pub use post::{Post, CreatePost, UpdatePost};
pub use user::{User, RegisterRequest, LoginRequest, UserPublic};
pub use error::{DomainError, AppResult};