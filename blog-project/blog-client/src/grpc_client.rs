use tonic::Request;
use crate::blog::blog_service_client::BlogServiceClient;
use crate::blog::*;
use crate::{AuthResponse, Post, ListPostsResponse, BlogClientError};
use tonic::metadata::MetadataValue;

fn auth_request<T>(token: &str, message: T) -> Request<T> {
    let mut req = Request::new(message);
    req.metadata_mut().insert(
        "authorization",
        MetadataValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    req
}

pub async fn register(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let resp = client
        .register(Request::new(RegisterRequest {
            username: username.into(),
            email: email.into(),
            password: password.into(),
        }))
        .await?
        .into_inner();
    
    Ok(AuthResponse {
        token: resp.token,
        user: resp.user.map(|u| crate::User {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }).ok_or(BlogClientError::Internal("No user in response".into()))?,
    })
}

pub async fn login(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    username: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let resp = client
        .login(Request::new(LoginRequest {
            username: username.into(),
            password: password.into(),
        }))
        .await?
        .into_inner();
    
    Ok(AuthResponse {
        token: resp.token,
        user: resp.user.map(|u| crate::User {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at,
        }).ok_or(BlogClientError::Internal("No user in response".into()))?,
    })
}

pub async fn create_post(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    token: &str,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let resp = client
        .create_post(auth_request(token, CreatePostRequest {
            title: title.into(),
            content: content.into(),
        }))
        .await?
        .into_inner();
    
    resp.post.map(|p| Post {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }).ok_or(BlogClientError::Internal("No post in response".into()))
}

pub async fn get_post(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    id: i64,
) -> Result<Post, BlogClientError> {
    let resp = client
        .get_post(Request::new(PostId { id }))
        .await?
        .into_inner();
    
    resp.post.map(|p| Post {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }).ok_or(BlogClientError::NotFound(format!("Post {} not found", id)))
}

pub async fn update_post(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    token: &str,
    id: i64,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<Post, BlogClientError> {
    let resp = client
        .update_post(auth_request(token, UpdatePostRequest {
            id,
            title: title.map(Into::into),
            content: content.map(Into::into),
        }))
        .await?
        .into_inner();
    
    resp.post.map(|p| Post {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }).ok_or(BlogClientError::Internal("No post in response".into()))
}

pub async fn delete_post(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    token: &str,
    id: i64,
) -> Result<(), BlogClientError> {
    let resp = client
        .delete_post(auth_request(token, PostId { id }))
        .await?
        .into_inner();
    
    if resp.success {
        Ok(())
    } else {
        Err(BlogClientError::Internal("Delete failed".into()))
    }
}

pub async fn list_posts(
    client: &mut BlogServiceClient<tonic::transport::Channel>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<ListPostsResponse, BlogClientError> {
    let resp = client
        .list_posts(Request::new(ListPostsRequest { limit, offset }))
        .await?
        .into_inner();
    
    Ok(ListPostsResponse {
        posts: resp.posts.into_iter().map(|p| Post {
            id: p.id,
            title: p.title,
            content: p.content,
            author_id: p.author_id,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }).collect(),
        total: resp.total,
        limit: resp.limit,
        offset: resp.offset,
    })
}