use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::sync::Arc;

use crate::application::AuthService;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub username: String,
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let auth_service = req
        .app_data::<actix_web::web::Data<Arc<AuthService>>>()
        .expect("AuthService not found in app_data");

    match auth_service.verify_token(credentials.token()) {
        Ok((user_id, username)) => {
            req.extensions_mut()
                .insert(AuthenticatedUser { user_id, username });
            Ok(req)
        }
        Err(_) => Err((actix_web::error::ErrorUnauthorized("Invalid token"), req)),
    }
}
