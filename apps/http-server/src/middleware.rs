use crate::errors::AppError;
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

pub const JWT_SECRET: &[u8] = b"supersecretjwtkey";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // User ID
    pub exp: usize,
}

pub async fn auth_guard(mut req: Request, next: Next) -> Result<Response, AppError> {
    let mut token_opt = None;

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            token_opt = Some(auth[7..].to_string());
        }
    }

    if token_opt.is_none() {
        if let Some(query) = req.uri().query() {
            for pair in query.split('&') {
                let kv: Vec<&str> = pair.split('=').collect();
                if kv.len() == 2 && kv[0] == "token" {
                    token_opt = Some(kv[1].to_string());
                    break;
                }
            }
        }
    }

    if let Some(token) = token_opt {
        let validation = Validation::default();
        if let Ok(token_data) = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(JWT_SECRET),
            &validation,
        ) {
            req.extensions_mut().insert(token_data.claims);
            return Ok(next.run(req).await);
        }
    }
    Err(AppError::Unauthorized("Invalid or missing Authorization token".to_string()))
}
