use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String, DomainError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .ok_or_else(|| DomainError::InternalError("Failed to calculate expiration".into()))?
            .timestamp();

        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp: expiration,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| DomainError::InternalError(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, DomainError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| DomainError::InvalidCredentials)
    }
}
