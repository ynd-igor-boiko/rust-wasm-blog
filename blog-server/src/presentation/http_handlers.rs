use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::application::{AuthService, BlogService};
use crate::domain::{CreatePostRequest, DomainError, LoginRequest, Post, RegisterRequest, UpdatePostRequest, User};
use crate::presentation::middleware::{jwt_validator, AuthenticatedUser};

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    user: UserDto,
}

#[derive(Serialize)]
struct UserDto {
    id: i64,
    username: String,
    email: String,
    created_at: String,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            created_at: u.created_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct PostDto {
    id: i64,
    title: String,
    content: String,
    author_id: i64,
    author_username: String,
    created_at: String,
    updated_at: String,
}

impl From<Post> for PostDto {
    fn from(p: Post) -> Self {
        Self {
            id: p.id,
            title: p.title,
            content: p.content,
            author_id: p.author_id,
            author_username: p.author_username,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Serialize)]
struct PostListResponse {
    posts: Vec<PostDto>,
    total: i64,
    limit: i32,
    offset: i32,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    limit: Option<i32>,
    offset: Option<i32>,
}

fn err_response(e: DomainError) -> HttpResponse {
    let body = serde_json::json!({"error": e.to_string()});
    match e {
        DomainError::UserNotFound | DomainError::PostNotFound => HttpResponse::NotFound().json(body),
        DomainError::UserAlreadyExists => HttpResponse::Conflict().json(body),
        DomainError::InvalidCredentials => HttpResponse::Unauthorized().json(body),
        DomainError::Forbidden => HttpResponse::Forbidden().json(body),
        DomainError::InvalidInput(_) => HttpResponse::BadRequest().json(body),
        _ => HttpResponse::InternalServerError().json(serde_json::json!({"error": "internal error"})),
    }
}

pub async fn register(
    auth: web::Data<Arc<AuthService>>,
    payload: web::Json<RegisterRequest>,
) -> HttpResponse {
    match auth.register(&payload.username, &payload.email, &payload.password).await {
        Ok((token, user)) => {
            HttpResponse::Created().json(AuthResponse { token, user: user.into() })
        }
        Err(e) => err_response(e),
    }
}

pub async fn login(
    auth: web::Data<Arc<AuthService>>,
    payload: web::Json<LoginRequest>,
) -> HttpResponse {
    match auth.login(&payload.username, &payload.password).await {
        Ok((token, user)) => HttpResponse::Ok().json(AuthResponse { token, user: user.into() }),
        Err(e) => err_response(e),
    }
}

pub async fn create_post(
    req: HttpRequest,
    svc: web::Data<Arc<BlogService>>,
    payload: web::Json<CreatePostRequest>,
) -> HttpResponse {
    let caller = req.extensions().get::<AuthenticatedUser>().cloned().unwrap();

    match svc.create_post(&payload.title, &payload.content, caller.user_id).await {
        Ok(post) => HttpResponse::Created().json(PostDto::from(post)),
        Err(e) => err_response(e),
    }
}

pub async fn get_post(
    svc: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> HttpResponse {
    match svc.get_post(path.into_inner()).await {
        Ok(p) => HttpResponse::Ok().json(PostDto::from(p)),
        Err(e) => err_response(e),
    }
}

pub async fn update_post(
    req: HttpRequest,
    svc: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
    payload: web::Json<UpdatePostRequest>,
) -> HttpResponse {
    let caller = req.extensions().get::<AuthenticatedUser>().cloned().unwrap();
    let post_id = path.into_inner();

    match svc.update_post(post_id, &payload.title, &payload.content, caller.user_id).await {
        Ok(p) => HttpResponse::Ok().json(PostDto::from(p)),
        Err(e) => err_response(e),
    }
}

pub async fn delete_post(
    req: HttpRequest,
    svc: web::Data<Arc<BlogService>>,
    path: web::Path<i64>,
) -> HttpResponse {
    let caller = req.extensions().get::<AuthenticatedUser>().cloned().unwrap();

    match svc.delete_post(path.into_inner(), caller.user_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => err_response(e),
    }
}

pub async fn list_posts(
    svc: web::Data<Arc<BlogService>>,
    q: web::Query<PaginationQuery>,
) -> HttpResponse {
    let limit = q.limit.unwrap_or(10);
    let offset = q.offset.unwrap_or(0);

    match svc.list_posts(limit, offset).await {
        Ok((posts, total)) => {
            let items: Vec<PostDto> = posts.into_iter().map(Into::into).collect();
            HttpResponse::Ok().json(PostListResponse {
                posts: items,
                total,
                limit,
                offset,
            })
        }
        Err(e) => err_response(e),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let auth_guard = HttpAuthentication::bearer(jwt_validator);

    cfg.service(
        web::scope("/api")
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login)),
            )
            .service(
                web::scope("/posts")
                    .route("", web::get().to(list_posts))
                    .route("/{id}", web::get().to(get_post))
                    .service(
                        web::scope("")
                            .wrap(auth_guard)
                            .route("", web::post().to(create_post))
                            .route("/{id}", web::put().to(update_post))
                            .route("/{id}", web::delete().to(delete_post)),
                    ),
            ),
    );
}
