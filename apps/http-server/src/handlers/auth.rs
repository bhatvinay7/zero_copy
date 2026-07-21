use crate::errors::AppError;
use crate::middleware::{Claims, JWT_SECRET};
use crate::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use db::models::NewUser;
use db::{create_user, get_user_by_email};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<UserDto>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct UserDto {
    pub id: i32,
    pub email: String,
}

pub async fn signup_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;

    if get_user_by_email(&mut conn, &payload.email)?.is_some() {
        return Err(AppError::BadRequest("User already exists".to_string()));
    }

    let hashed_password = hash(&payload.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.into()))?;

    let new_user = NewUser {
        email: payload.email,
        password_hash: hashed_password,
    };

    let user = create_user(&mut conn, &new_user)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            success: true,
            token: None,
            user: Some(UserDto {
                id: user.id,
                email: user.email,
            }),
            error: None,
        }),
    ))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = state.db_pool.get().map_err(|e| AppError::Internal(e.into()))?;

    let user = get_user_by_email(&mut conn, &payload.email)?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    // Generate JWT
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    ).map_err(|e| AppError::Internal(e.into()))?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            token: Some(token),
            user: Some(UserDto {
                id: user.id,
                email: user.email,
            }),
            error: None,
        }),
    ))
}
