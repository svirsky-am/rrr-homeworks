use tonic::{Request, metadata::MetadataValue};
use std::convert::TryFrom;
use crate::blog::blog_service_client::BlogServiceClient;
use crate::blog::*;
use crate::{AuthResponse, Post, ListPostsResponse, BlogClientError};

fn auth_req<T>(token: &str, msg: T) -> Request<T> {
    let mut r = Request::new(msg);
    r.metadata_mut().insert("authorization", MetadataValue::try_from(&format!("Bearer {}", token)).expect("Invalid token"));
    r
}

pub async fn register(c: &mut BlogServiceClient<tonic::transport::Channel>, u: &str, e: &str, p: &str) -> Result<AuthResponse, BlogClientError> {
    let r = c.register(Request::new(RegisterRequest { username: u.into(), email: e.into(), password: p.into() })).await?.into_inner();
    Ok(AuthResponse { token: r.token, user: r.user.map(|x| crate::User { id: x.id, username: x.username, email: x.email, created_at: x.created_at }).ok_or(BlogClientError::Internal("No user".to_string()))? })
}

pub async fn login(c: &mut BlogServiceClient<tonic::transport::Channel>, u: &str, p: &str) -> Result<AuthResponse, BlogClientError> {
    let r = c.login(Request::new(LoginRequest { username: u.into(), password: p.into() })).await?.into_inner();
    Ok(AuthResponse { token: r.token, user: r.user.map(|x| crate::User { id: x.id, username: x.username, email: x.email, created_at: x.created_at }).ok_or(BlogClientError::Internal("No user".to_string()))? })
}

pub async fn create_post(c: &mut BlogServiceClient<tonic::transport::Channel>, t: &str, title: &str, content: &str) -> Result<Post, BlogClientError> {
    let r = c.create_post(auth_req(t, CreatePostRequest { title: title.into(), content: content.into() })).await?.into_inner();
    r.post.map(|x| Post { id: x.id, title: x.title, content: x.content, author_id: x.author_id, created_at: x.created_at, updated_at: x.updated_at }).ok_or(BlogClientError::Internal("No post".to_string()))
}

pub async fn get_post(c: &mut BlogServiceClient<tonic::transport::Channel>, id: i64) -> Result<Post, BlogClientError> {
    let r = c.get_post(Request::new(PostId { id })).await?.into_inner();
    r.post.map(|x| Post { id: x.id, title: x.title, content: x.content, author_id: x.author_id, created_at: x.created_at, updated_at: x.updated_at }).ok_or(BlogClientError::Internal("No post".to_string()))
}

pub async fn update_post(c: &mut BlogServiceClient<tonic::transport::Channel>, t: &str, id: i64, title: Option<&str>, content: Option<&str>) -> Result<Post, BlogClientError> {
    let r = c.update_post(auth_req(t, UpdatePostRequest { id, title: title.map(Into::into), content: content.map(Into::into) })).await?.into_inner();
    r.post.map(|x| Post { id: x.id, title: x.title, content: x.content, author_id: x.author_id, created_at: x.created_at, updated_at: x.updated_at }).ok_or(BlogClientError::Internal("No post".to_string()))
}

pub async fn delete_post(c: &mut BlogServiceClient<tonic::transport::Channel>, t: &str, id: i64) -> Result<(), BlogClientError> {
    let r = c.delete_post(auth_req(t, PostId { id })).await?.into_inner();
    if r.success { Ok(()) } else { Err(BlogClientError::Internal("Delete failed".to_string())) }
}

pub async fn list_posts(c: &mut BlogServiceClient<tonic::transport::Channel>, limit: Option<i64>, offset: Option<i64>) -> Result<ListPostsResponse, BlogClientError> {
    let r = c.list_posts(Request::new(ListPostsRequest { limit, offset })).await?.into_inner();
    Ok(ListPostsResponse { posts: r.posts.into_iter().map(|x| Post { id: x.id, title: x.title, content: x.content, author_id: x.author_id, created_at: x.created_at, updated_at: x.updated_at }).collect(), total: r.total, limit: r.limit, offset: r.offset })
}