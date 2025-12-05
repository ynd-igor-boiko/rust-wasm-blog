pub mod error;
pub mod grpc_client;
pub mod http_client;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use error::BlogClientError;
pub use grpc_client::GrpcBlogClient;
pub use http_client::HttpBlogClient;

#[derive(Debug, Clone)]
pub enum Transport {
    Http(String),
    Grpc(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub author_id: i64,
    pub author_username: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsResponse {
    pub posts: Vec<Post>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

pub struct BlogClient {
    transport: Transport,
    http_client: Option<HttpBlogClient>,
    grpc_client: Option<GrpcBlogClient>,
    token: Option<String>,
}

impl BlogClient {
    pub async fn new(transport: Transport) -> Result<Self, BlogClientError> {
        match &transport {
            Transport::Http(url) => Ok(Self {
                transport,
                http_client: Some(HttpBlogClient::new(url.clone())),
                grpc_client: None,
                token: None,
            }),
            Transport::Grpc(url) => Ok(Self {
                transport,
                http_client: None,
                grpc_client: Some(GrpcBlogClient::new(url.clone()).await?),
                token: None,
            }),
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn get_token(&self) -> Option<&String> {
        self.token.as_ref()
    }

    pub async fn register(
        &mut self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.register(username, email, password).await?,
            (_, Some(client)) => client.register(username, email, password).await?,
            _ => return Err(BlogClientError::NotConnected),
        };

        self.token = Some(response.token.clone());
        Ok(response)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<AuthResponse, BlogClientError> {
        let response = match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.login(username, password).await?,
            (_, Some(client)) => client.login(username, password).await?,
            _ => return Err(BlogClientError::NotConnected),
        };

        self.token = Some(response.token.clone());
        Ok(response)
    }

    pub async fn create_post(
        &mut self,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;

        match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.create_post(token, title, content).await,
            (_, Some(client)) => client.create_post(token, title, content).await,
            _ => Err(BlogClientError::NotConnected),
        }
    }

    pub async fn get_post(&mut self, id: i64) -> Result<Post, BlogClientError> {
        match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.get_post(id).await,
            (_, Some(client)) => client.get_post(id).await,
            _ => Err(BlogClientError::NotConnected),
        }
    }

    pub async fn update_post(
        &mut self,
        id: i64,
        title: &str,
        content: &str,
    ) -> Result<Post, BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;

        match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.update_post(token, id, title, content).await,
            (_, Some(client)) => client.update_post(token, id, title, content).await,
            _ => Err(BlogClientError::NotConnected),
        }
    }

    pub async fn delete_post(&mut self, id: i64) -> Result<(), BlogClientError> {
        let token = self.token.as_ref().ok_or(BlogClientError::Unauthorized)?;

        match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.delete_post(token, id).await,
            (_, Some(client)) => client.delete_post(token, id).await,
            _ => Err(BlogClientError::NotConnected),
        }
    }

    pub async fn list_posts(
        &mut self,
        limit: i32,
        offset: i32,
    ) -> Result<ListPostsResponse, BlogClientError> {
        match (&mut self.http_client, &mut self.grpc_client) {
            (Some(client), _) => client.list_posts(limit, offset).await,
            (_, Some(client)) => client.list_posts(limit, offset).await,
            _ => Err(BlogClientError::NotConnected),
        }
    }
}
