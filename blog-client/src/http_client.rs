use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::BlogClientError;
use crate::{AuthResponse, ListPostsResponse, Post};

#[derive(Serialize)]
struct RegisterReq {
    username: String,
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginReq {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct PostReq {
    title: String,
    content: String,
}

#[derive(Deserialize)]
struct ErrBody {
    error: String,
}

pub struct HttpBlogClient {
    client: Client,
    base: String,
}

impl HttpBlogClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create http client");

        Self { client, base: base_url }
    }

    pub async fn register(&self, username: &str, email: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        let resp = self.client
            .post(format!("{}/api/auth/register", self.base))
            .json(&RegisterReq {
                username: username.into(),
                email: email.into(),
                password: password.into(),
            })
            .send()
            .await?;

        self.parse_json_or_err(resp).await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, BlogClientError> {
        let resp = self.client
            .post(format!("{}/api/auth/login", self.base))
            .json(&LoginReq { username: username.into(), password: password.into() })
            .send()
            .await?;

        self.parse_json_or_err(resp).await
    }

    pub async fn create_post(&self, token: &str, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let resp = self.client
            .post(format!("{}/api/posts", self.base))
            .bearer_auth(token)
            .json(&PostReq { title: title.into(), content: content.into() })
            .send()
            .await?;

        self.parse_json_or_err(resp).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, BlogClientError> {
        let resp = self.client.get(format!("{}/api/posts/{}", self.base, id)).send().await?;
        self.parse_json_or_err(resp).await
    }

    pub async fn update_post(&self, token: &str, id: i64, title: &str, content: &str) -> Result<Post, BlogClientError> {
        let resp = self.client
            .put(format!("{}/api/posts/{}", self.base, id))
            .bearer_auth(token)
            .json(&PostReq { title: title.into(), content: content.into() })
            .send()
            .await?;

        self.parse_json_or_err(resp).await
    }

    pub async fn delete_post(&self, token: &str, id: i64) -> Result<(), BlogClientError> {
        let resp = self.client
            .delete(format!("{}/api/posts/{}", self.base, id))
            .bearer_auth(token)
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(self.map_status_err(resp).await)
        }
    }

    pub async fn list_posts(&self, limit: i32, offset: i32) -> Result<ListPostsResponse, BlogClientError> {
        let url = format!("{}/api/posts?limit={}&offset={}", self.base, limit, offset);
        let resp = self.client.get(&url).send().await?;
        self.parse_json_or_err(resp).await
    }

    async fn parse_json_or_err<T: serde::de::DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, BlogClientError> {
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            Err(self.map_status_err(resp).await)
        }
    }

    async fn map_status_err(&self, resp: reqwest::Response) -> BlogClientError {
        let status = resp.status().as_u16();
        let msg = resp.json::<ErrBody>().await.map(|e| e.error).unwrap_or_default();

        match status {
            401 => BlogClientError::Unauthorized,
            403 => BlogClientError::Forbidden,
            404 => BlogClientError::NotFound,
            409 => BlogClientError::Conflict(msg),
            400 => BlogClientError::InvalidRequest(msg),
            _ => BlogClientError::ServerError(msg),
        }
    }
}
