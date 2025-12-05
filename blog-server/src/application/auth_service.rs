use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use tracing::{info, warn};

use crate::data::PostgresUserRepository;
use crate::domain::{DomainError, User};
use crate::infrastructure::JwtService;

const MIN_USERNAME_LEN: usize = 3;
const MAX_USERNAME_LEN: usize = 32;
const MIN_PASSWORD_LEN: usize = 8;

#[derive(Clone)]
pub struct AuthService {
    user_repo: PostgresUserRepository,
    jwt: JwtService,
}

impl AuthService {
    pub fn new(user_repo: PostgresUserRepository, jwt: JwtService) -> Self {
        Self { user_repo, jwt }
    }

    pub async fn register(
        &self,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<(String, User), DomainError> {
        self.validate_registration(username, email, password)?;

        let hash = self.hash_password(password)?;
        let user = self.user_repo.create(username, email, &hash).await?;
        let token = self.jwt.generate_token(user.id, &user.username)?;

        info!(user_id = user.id, "new user registered");
        Ok((token, user))
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(String, User), DomainError> {
        let user = self.user_repo
            .find_by_username(username)
            .await?
            .ok_or(DomainError::InvalidCredentials)?;

        self.verify_password(password, &user.password_hash)?;

        let token = self.jwt.generate_token(user.id, &user.username)?;
        info!(user_id = user.id, "user logged in");
        Ok((token, user))
    }

    pub fn verify_token(&self, token: &str) -> Result<(i64, String), DomainError> {
        let claims = self.jwt.verify_token(token)?;
        Ok((claims.user_id, claims.username))
    }

    fn validate_registration(&self, username: &str, email: &str, password: &str) -> Result<(), DomainError> {
        let uname = username.trim();
        if uname.len() < MIN_USERNAME_LEN || uname.len() > MAX_USERNAME_LEN {
            return Err(DomainError::InvalidInput(
                format!("username must be {}-{} characters", MIN_USERNAME_LEN, MAX_USERNAME_LEN)
            ));
        }
        if !uname.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DomainError::InvalidInput("username can only contain letters, numbers and underscores".into()));
        }

        if !email.contains('@') || email.len() > 255 {
            return Err(DomainError::InvalidInput("invalid email".into()));
        }

        if password.len() < MIN_PASSWORD_LEN {
            return Err(DomainError::InvalidInput(
                format!("password must be at least {} characters", MIN_PASSWORD_LEN)
            ));
        }
        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| {
                warn!("password hashing failed: {}", e);
                DomainError::InternalError("failed to process password".into())
            })
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<(), DomainError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|_| DomainError::InternalError("corrupted password hash".into()))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|_| DomainError::InvalidCredentials)
    }
}
