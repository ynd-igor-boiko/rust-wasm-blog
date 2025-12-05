use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::application::{AuthService, BlogService};
use crate::domain::{DomainError, Post as DomainPost, User as DomainUser};

pub mod blog_proto {
    tonic::include_proto!("blog");
}

use blog_proto::blog_service_server::BlogService as GrpcBlogService;
use blog_proto::{
    AuthResponse, CreatePostRequest, DeletePostRequest, DeletePostResponse, GetPostRequest,
    ListPostsRequest, ListPostsResponse, LoginRequest, Post, PostResponse, RegisterRequest, User,
};

fn to_proto_user(u: DomainUser) -> User {
    User {
        id: u.id,
        username: u.username,
        email: u.email,
        created_at: u.created_at.to_rfc3339(),
    }
}

fn to_proto_post(p: DomainPost) -> Post {
    Post {
        id: p.id,
        title: p.title,
        content: p.content,
        author_id: p.author_id,
        author_username: p.author_username,
        created_at: p.created_at.to_rfc3339(),
        updated_at: p.updated_at.to_rfc3339(),
    }
}

fn map_err(e: DomainError) -> Status {
    match e {
        DomainError::UserNotFound | DomainError::PostNotFound => Status::not_found(e.to_string()),
        DomainError::UserAlreadyExists => Status::already_exists(e.to_string()),
        DomainError::InvalidCredentials => Status::unauthenticated(e.to_string()),
        DomainError::Forbidden => Status::permission_denied(e.to_string()),
        DomainError::InvalidInput(msg) => Status::invalid_argument(msg),
        _ => Status::internal("internal error"),
    }
}

pub struct BlogGrpcService {
    auth: Arc<AuthService>,
    posts: Arc<BlogService>,
}

impl BlogGrpcService {
    pub fn new(auth: Arc<AuthService>, posts: Arc<BlogService>) -> Self {
        Self { auth, posts }
    }

    fn get_bearer_token<T>(&self, req: &Request<T>) -> Result<(i64, String), Status> {
        let hdr = req.metadata()
            .get("authorization")
            .ok_or_else(|| Status::unauthenticated("missing auth header"))?
            .to_str()
            .map_err(|_| Status::unauthenticated("bad auth header"))?;

        let token = hdr.strip_prefix("Bearer ")
            .ok_or_else(|| Status::unauthenticated("expected Bearer token"))?;

        self.auth.verify_token(token)
            .map_err(|_| Status::unauthenticated("invalid token"))
    }
}

#[tonic::async_trait]
impl GrpcBlogService for BlogGrpcService {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<AuthResponse>, Status> {
        let r = request.into_inner();
        let (token, user) = self.auth
            .register(&r.username, &r.email, &r.password)
            .await
            .map_err(map_err)?;

        Ok(Response::new(AuthResponse {
            token,
            user: Some(to_proto_user(user)),
        }))
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<AuthResponse>, Status> {
        let r = request.into_inner();
        let (token, user) = self.auth.login(&r.username, &r.password).await.map_err(map_err)?;

        Ok(Response::new(AuthResponse { token, user: Some(to_proto_user(user)) }))
    }

    async fn create_post(&self, request: Request<CreatePostRequest>) -> Result<Response<PostResponse>, Status> {
        let (uid, _) = self.get_bearer_token(&request)?;
        let r = request.into_inner();

        let post = self.posts.create_post(&r.title, &r.content, uid).await.map_err(map_err)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn get_post(&self, request: Request<GetPostRequest>) -> Result<Response<PostResponse>, Status> {
        let post = self.posts.get_post(request.into_inner().id).await.map_err(map_err)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn update_post(&self, request: Request<blog_proto::UpdatePostRequest>) -> Result<Response<PostResponse>, Status> {
        let (uid, _) = self.get_bearer_token(&request)?;
        let r = request.into_inner();

        let post = self.posts.update_post(r.id, &r.title, &r.content, uid).await.map_err(map_err)?;
        Ok(Response::new(PostResponse { post: Some(to_proto_post(post)) }))
    }

    async fn delete_post(&self, request: Request<DeletePostRequest>) -> Result<Response<DeletePostResponse>, Status> {
        let (uid, _) = self.get_bearer_token(&request)?;
        self.posts.delete_post(request.into_inner().id, uid).await.map_err(map_err)?;
        Ok(Response::new(DeletePostResponse { success: true }))
    }

    async fn list_posts(&self, request: Request<ListPostsRequest>) -> Result<Response<ListPostsResponse>, Status> {
        let r = request.into_inner();
        let lim = if r.limit > 0 { r.limit } else { 10 };
        let off = r.offset.max(0);

        let (items, total) = self.posts.list_posts(lim, off).await.map_err(map_err)?;

        Ok(Response::new(ListPostsResponse {
            posts: items.into_iter().map(to_proto_post).collect(),
            total,
            limit: lim,
            offset: off,
        }))
    }
}
