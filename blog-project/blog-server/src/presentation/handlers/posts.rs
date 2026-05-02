use actix_web::{delete, get, post, put, web, HttpResponse, Scope};
// use actix_web::web::Deserialize;
// use actix_web::error::JsonPayloadError::Deserialize;
use tracing::info;

use crate::application::blog_service::BlogService;
use crate::data::post_repository::PostgresPostRepository;
use crate::domain::BlogError;
use crate::presentation::auth::AuthenticatedUser;
use crate::presentation::dto::{PostResponse, PostsListResponse};
use crate::data::post::{CreatePostRequest, Post, UpdatePostRequest};

pub fn scope() -> Scope {
    web::scope("/posts")
        .service(create_post)
        .service(list_posts)
        .service(get_post)
        .service(update_post)
        .service(delete_post)
}

#[post("")]
async fn create_post(
    user: AuthenticatedUser,
    blog: web::Data<BlogService<PostgresPostRepository>>,
    payload: web::Json<CreatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let post = blog
        .create_post(payload.title.clone(), payload.content.clone(), user.id)
        .await?;

    info!(
        post_id = post.id,
        author_id = %user.id,
        "post created"
    );

    Ok(HttpResponse::Created().json(PostResponse::from(post)))
}

#[get("/{id}")]
async fn get_post(
    path: web::Path<i64>,
    blog: web::Data<BlogService<PostgresPostRepository>>,
) -> Result<HttpResponse, BlogError> {
    let id = path.into_inner();
    let post = blog.get_post(id).await?;

    info!(post_id = id, "post fetched");

    Ok(HttpResponse::Ok().json(PostResponse::from(post)))
}

#[put("/{id}")]
async fn update_post(
    user: AuthenticatedUser,
    path: web::Path<i64>,
    blog: web::Data<BlogService<PostgresPostRepository>>,
    payload: web::Json<UpdatePostRequest>,
) -> Result<HttpResponse, BlogError> {
    let id = path.into_inner();
    let post = blog
        .update_post(id, payload.title.clone(), payload.content.clone(), user.id)
        .await?;

    info!(
        post_id = id,
        author_id = %user.id,
        "post updated"
    );

    Ok(HttpResponse::Ok().json(PostResponse::from(post)))
}

#[delete("/{id}")]
async fn delete_post(
    user: AuthenticatedUser,
    path: web::Path<i64>,
    blog: web::Data<BlogService<PostgresPostRepository>>,
) -> Result<HttpResponse, BlogError> {
    let id = path.into_inner();
    blog.delete_post(id, user.id).await?;

    info!(
        post_id = id,
        author_id = %user.id,
        "post deleted"
    );

    Ok(HttpResponse::NoContent().finish())
}

#[get("/list")]
async fn list_posts(
    query: web::Query<ListPostsQuery>,
    blog: web::Data<BlogService<PostgresPostRepository>>,
) -> Result<HttpResponse, BlogError> {
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    let posts = blog.list_posts(limit, offset).await?;
    let total = blog.count_posts().await?;

    let response = PostsListResponse {
        posts: posts.into_iter().map(PostResponse::from).collect(),
        total,
        limit,
        offset,
    };

    info!(
        limit,
        offset,
        total,
        returned_count = response.posts.len(),
        "posts listed"
    );

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, serde::Deserialize)]
struct ListPostsQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}
