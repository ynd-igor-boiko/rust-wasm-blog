use sqlx::PgPool;
use tracing::{debug, error};

use crate::domain::{DomainError, Post};

const MAX_LIMIT: i32 = 100;

#[derive(Clone)]
pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, title: &str, content: &str, author_id: i64) -> Result<Post, DomainError> {
        debug!(author_id, "creating post");

        sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, content, author_id)
            VALUES ($1, $2, $3)
            RETURNING
                posts.id,
                posts.title,
                posts.content,
                posts.author_id,
                (SELECT username FROM users WHERE id = posts.author_id) as "author_username!",
                posts.created_at,
                posts.updated_at
            "#,
            title, content, author_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("insert post failed: {}", e);
            DomainError::DatabaseError(e.to_string())
        })
    }

    pub async fn find_by_id(&self, post_id: i64) -> Result<Option<Post>, DomainError> {
        sqlx::query_as!(
            Post,
            r#"
            SELECT p.id, p.title, p.content, p.author_id,
                   u.username as author_username,
                   p.created_at, p.updated_at
            FROM posts p
            JOIN users u ON p.author_id = u.id
            WHERE p.id = $1
            "#,
            post_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn update(&self, post_id: i64, title: &str, content: &str) -> Result<Post, DomainError> {
        debug!(post_id, "updating post");

        sqlx::query_as!(
            Post,
            r#"
            UPDATE posts SET title = $2, content = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING
                posts.id, posts.title, posts.content, posts.author_id,
                (SELECT username FROM users WHERE id = posts.author_id) as "author_username!",
                posts.created_at, posts.updated_at
            "#,
            post_id, title, content
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!(post_id, "update failed: {}", e);
            DomainError::DatabaseError(e.to_string())
        })
    }

    pub async fn delete(&self, post_id: i64) -> Result<bool, DomainError> {
        let res = sqlx::query!("DELETE FROM posts WHERE id = $1", post_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        Ok(res.rows_affected() > 0)
    }

    pub async fn list(&self, limit: i32, offset: i32) -> Result<(Vec<Post>, i64), DomainError> {
        let limit = limit.min(MAX_LIMIT).max(1);
        let offset = offset.max(0);

        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT p.id, p.title, p.content, p.author_id,
                   u.username as author_username,
                   p.created_at, p.updated_at
            FROM posts p
            JOIN users u ON p.author_id = u.id
            ORDER BY p.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64, offset as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM posts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?
            .unwrap_or(0);

        Ok((posts, total))
    }
}
