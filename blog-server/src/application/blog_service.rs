use crate::data::PostgresPostRepository;
use crate::domain::{DomainError, Post};

#[derive(Clone)]
pub struct BlogService {
    post_repo: PostgresPostRepository,
}

impl BlogService {
    pub fn new(post_repo: PostgresPostRepository) -> Self {
        Self { post_repo }
    }

    pub async fn create_post(
        &self,
        title: &str,
        content: &str,
        author_id: i64,
    ) -> Result<Post, DomainError> {
        self.post_repo.create(title, content, author_id).await
    }

    pub async fn get_post(&self, id: i64) -> Result<Post, DomainError> {
        self.post_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)
    }

    pub async fn update_post(
        &self,
        id: i64,
        title: &str,
        content: &str,
        user_id: i64,
    ) -> Result<Post, DomainError> {
        let post = self
            .post_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repo.update(id, title, content).await
    }

    pub async fn delete_post(&self, id: i64, user_id: i64) -> Result<(), DomainError> {
        let post = self
            .post_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::PostNotFound)?;

        if post.author_id != user_id {
            return Err(DomainError::Forbidden);
        }

        self.post_repo.delete(id).await?;
        Ok(())
    }

    pub async fn list_posts(&self, limit: i32, offset: i32) -> Result<(Vec<Post>, i64), DomainError> {
        self.post_repo.list(limit, offset).await
    }
}
