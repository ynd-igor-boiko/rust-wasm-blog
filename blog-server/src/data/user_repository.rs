use sqlx::PgPool;

use crate::domain::{DomainError, User};

#[derive(Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: &str, email: &str, password_hash: &str) -> Result<User, DomainError> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, created_at
            "#,
            username,
            email,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_username_key")
                    || db_err.constraint() == Some("users_email_key")
                {
                    return DomainError::UserAlreadyExists;
                }
            }
            DomainError::DatabaseError(e.to_string())
        })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1"#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>, DomainError> {
        sqlx::query_as!(
            User,
            r#"SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }
}
