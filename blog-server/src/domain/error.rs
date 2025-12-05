use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("user not found")]
    UserNotFound,

    #[error("username or email already taken")]
    UserAlreadyExists,

    #[error("wrong username or password")]
    InvalidCredentials,

    #[error("post not found")]
    PostNotFound,

    #[error("access denied")]
    Forbidden,

    #[error("{0}")]
    InvalidInput(String),

    #[error("db error: {0}")]
    DatabaseError(String),

    #[error("{0}")]
    InternalError(String),
}
