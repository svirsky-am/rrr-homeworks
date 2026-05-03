// blog-client/src/http_client.rs
use reqwest::Client;
use serde_json::json;
use crate::{AuthResponse, Post, ListPostsResponse, BlogClientError};

pub async fn register(
    client: &Client,
    base_url: &str,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let resp = client
        .post(format!("{}/api/auth/register", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": username, "email": email, "password": password }))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_else(|_| "<empty>".into());
        return Err(BlogClientError::InvalidRequest(format!("Register failed: {}", body)));
    }
    
    Ok(resp.json::<AuthResponse>().await?)
}

pub async fn login(
    client: &Client,
    base_url: &str,
    username: &str,
    password: &str,
) -> Result<AuthResponse, BlogClientError> {
    let resp = client
        .post(format!("{}/api/auth/login", base_url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(BlogClientError::Unauthorized("Invalid credentials".into()));
    }
    
    Ok(resp.json::<AuthResponse>().await?)
}

pub async fn create_post(
    client: &Client,
    base_url: &str,
    token: &str,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let resp = client
        .post(format!("{}/api/posts", base_url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "title": title, "content": content }))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Create post failed".into()));
    }
    
    Ok(resp.json::<Post>().await?)
}

pub async fn get_post(
    client: &Client,
    base_url: &str,
    id: i64,
) -> Result<Post, BlogClientError> {
    let resp = client
        .get(format!("{}/api/posts/{}", base_url, id))
        .send()
        .await?;
    
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    
    if !resp.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Get post failed".into()));
    }
    
    Ok(resp.json::<Post>().await?)
}

pub async fn update_post(
    client: &Client,
    base_url: &str,
    token: &str,
    id: i64,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<Post, BlogClientError> {
    let resp = client
        .put(format!("{}/api/posts/{}", base_url, id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "title": title, "content": content }))
        .send()
        .await?;
    
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(BlogClientError::PermissionDenied("Not the author".into()));
    }
    if !resp.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Update post failed".into()));
    }
    
    Ok(resp.json::<Post>().await?)
}

pub async fn delete_post(
    client: &Client,
    base_url: &str,
    token: &str,
    id: i64,
) -> Result<(), BlogClientError> {
    let resp = client
        .delete(format!("{}/api/posts/{}", base_url, id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(BlogClientError::PermissionDenied("Not the author".into()));
    }
    if !resp.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Delete post failed".into()));
    }
    
    Ok(())
}

pub async fn list_posts(
    client: &Client,
    base_url: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<ListPostsResponse, BlogClientError> {
    let mut url = format!("{}/api/posts", base_url);
    let mut params = Vec::new();
    if let Some(l) = limit { params.push(format!("limit={}", l)); }
    if let Some(o) = offset { params.push(format!("offset={}", o)); }
    if !params.is_empty() {
        url.push_str(&format!("?{}", params.join("&")));
    }
    
    let resp = client.get(&url).send().await?;
    
    if !resp.status().is_success() {
        return Err(BlogClientError::InvalidRequest("List posts failed".into()));
    }
    
    Ok(resp.json::<ListPostsResponse>().await?)
}