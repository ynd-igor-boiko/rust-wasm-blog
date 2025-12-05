pub mod error;
pub mod post;
pub mod user;

pub use error::DomainError;
pub use post::{CreatePostRequest, Post, UpdatePostRequest};
pub use user::{LoginRequest, RegisterRequest, User};
