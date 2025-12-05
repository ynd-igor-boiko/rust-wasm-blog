use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

use crate::error::BlogClientError;
use crate::{AuthResponse, ListPostsResponse, Post, User};

pub mod blog_proto {
    tonic::include_proto!("blog");
}

use blog_proto::blog_service_client::BlogServiceClient;

pub struct GrpcBlogClient {
    client: BlogServiceClient<Channel>,
}

impl GrpcBlogClient {
    pub async fn new(url: String) -> Result<Self, BlogClientError> {
        let channel = Channel::from_shared(url)?.connect().await?;
        Ok(Self {
            client: BlogServiceClient::new(channel),
        })
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = Request::new(blog_proto::RegisterRequest {
            username: username.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        });

        let response = self.client.register(request).await?.into_inner();
        let user = response.user.ok_or(BlogClientError::ServerError(
            "Missing user in response".to_string(),
        ))?;

        Ok(AuthResponse {
            token: response.token,
            user: User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at,
            },
        })
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let request = Request::new(blog_proto::LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        });

        let response = self.client.login(request).await?.into_inner();
        let user = response.user.ok_or(BlogClientError::ServerError(
            "Missing user in response".to_string(),
        ))?;

        Ok(AuthResponse {
            token: response.token,
            user: User {
                id: user.id,
                username: user.username,
                email: user.email,
                created_at: user.created_at,
            },
        })
    }

    pub async fn create_post(
        &mut self,
        token: &str,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(blog_proto::CreatePostRequest {
            title: title.to_string(),
            content: content.to_string(),
        });

        let auth_value: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
        request.metadata_mut().insert("authorization", auth_value);

        let response = self.client.create_post(request).await?.into_inner();
        let post = response.post.ok_or(BlogClientError::ServerError(
            "Missing post in response".to_string(),
        ))?;

        Ok(self.proto_to_post(post))
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        let request = Request::new(blog_proto::GetPostRequest { id });
        let response = self.client.get_post(request).await?.into_inner();
        let post = response.post.ok_or(BlogClientError::ServerError(
            "Missing post in response".to_string(),
        ))?;

        Ok(self.proto_to_post(post))
    }

    pub async fn update_post(
        &mut self,
        token: &str,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let mut request = Request::new(blog_proto::UpdatePostRequest {
            id,
            title: title.to_string(),
            content: content.to_string(),
        });

        let auth_value: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
        request.metadata_mut().insert("authorization", auth_value);

        let response = self.client.update_post(request).await?.into_inner();
        let post = response.post.ok_or(BlogClientError::ServerError(
            "Missing post in response".to_string(),
        ))?;

        Ok(self.proto_to_post(post))
    }

    pub async fn delete_post(&mut self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let mut request = Request::new(blog_proto::DeletePostRequest { id });

        let auth_value: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();
        request.metadata_mut().insert("authorization", auth_value);

        self.client.delete_post(request).await?;
        Ok(())
    }

    pub async fn list_posts(
        &mut self,
        limit: i32,
        offset: i32,
    ) -> Result<ListPostsResponse, BlogClientError> {
        let request = Request::new(blog_proto::ListPostsRequest { limit, offset });
        let response = self.client.list_posts(request).await?.into_inner();

        Ok(ListPostsResponse {
            posts: response.posts.into_iter().map(|p| self.proto_to_post(p)).collect(),
            total: response.total,
            limit: response.limit,
            offset: response.offset,
        })
    }

    fn proto_to_post(&self, post: blog_proto::Post) -> Post {
        Post {
            id: post.id,
            title: post.title,
            content: post.content,
            author_id: post.author_id,
            author_username: post.author_username,
            created_at: post.created_at,
            updated_at: post.updated_at,
        }
    }
}
