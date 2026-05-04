use crate::{AuthResponse, BlogClientError, ListPostsResponse, Post};
use reqwest::Client;
use serde_json::json;

pub async fn register(
    c: &Client,
    url: &str,
    u: &str,
    e: &str,
    p: &str,
) -> Result<AuthResponse, BlogClientError> {
    let r = c
        .post(format!("{}/api/auth/register", url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": u, "email": e, "password": p }))
        .send()
        .await?;
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest(format!(
            "Register failed: {}",
            r.text().await.unwrap_or_default()
        )));
    }
    Ok(r.json().await?)
}

pub async fn login(
    c: &Client,
    url: &str,
    u: &str,
    p: &str,
) -> Result<AuthResponse, BlogClientError> {
    let r = c
        .post(format!("{}/api/auth/login", url))
        .header("Content-Type", "application/json")
        .json(&json!({ "username": u, "password": p }))
        .send()
        .await?;
    if !r.status().is_success() {
        return Err(BlogClientError::Unauthorized("Invalid credentials".into()));
    }
    Ok(r.json().await?)
}

pub async fn create_post(
    c: &Client,
    url: &str,
    t: &str,
    title: &str,
    content: &str,
) -> Result<Post, BlogClientError> {
    let r = c
        .post(format!("{}/api/posts", url))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", t))
        .json(&json!({ "title": title, "content": content }))
        .send()
        .await?;
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Create post failed".into()));
    }
    Ok(r.json().await?)
}

pub async fn get_post(c: &Client, url: &str, id: i64) -> Result<Post, BlogClientError> {
    let r = c.get(format!("{}/api/posts/{}", url, id)).send().await?;
    if r.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Get post failed".into()));
    }
    Ok(r.json().await?)
}

pub async fn update_post(
    c: &Client,
    url: &str,
    t: &str,
    id: i64,
    title: Option<&str>,
    content: Option<&str>,
) -> Result<Post, BlogClientError> {
    let r = c
        .put(format!("{}/api/posts/{}", url, id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", t))
        .json(&json!({ "title": title, "content": content }))
        .send()
        .await?;
    if r.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    if r.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(BlogClientError::PermissionDenied("Not the author".into()));
    }
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Update post failed".into()));
    }
    Ok(r.json().await?)
}

pub async fn delete_post(c: &Client, url: &str, t: &str, id: i64) -> Result<(), BlogClientError> {
    let r = c
        .delete(format!("{}/api/posts/{}", url, id))
        .header("Authorization", format!("Bearer {}", t))
        .send()
        .await?;
    if r.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(BlogClientError::NotFound(format!("Post {} not found", id)));
    }
    if r.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(BlogClientError::PermissionDenied("Not the author".into()));
    }
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest("Delete post failed".into()));
    }
    Ok(())
}

pub async fn list_posts(
    c: &Client,
    url: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<ListPostsResponse, BlogClientError> {
    let mut u = format!("{}/api/posts", url);
    let mut p = Vec::new();
    if let Some(l) = limit {
        p.push(format!("limit={}", l));
    }
    if let Some(o) = offset {
        p.push(format!("offset={}", o));
    }
    if !p.is_empty() {
        u.push_str(&format!("?{}", p.join("&")));
    }
    let r = c.get(&u).send().await?;
    if !r.status().is_success() {
        return Err(BlogClientError::InvalidRequest("List posts failed".into()));
    }
    Ok(r.json().await?)
}
