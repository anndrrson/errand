use axum::{
    extract::{FromRequestParts, State},
    http::{header, request::Parts, StatusCode},
    Json,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db, error::AppError, AppState};
use errand_types::{AuthTokenResponse, LoginRequest, SignupRequest};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,     // user_id as string
    email: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Auth("Missing authorization header".into()))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Auth("Invalid authorization format".into()))?;

        let app = state.as_ref();
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(app.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Auth(format!("Invalid token: {e}")))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid user_id in token".into()))?;

        Ok(AuthUser {
            user_id,
            email: token_data.claims.email,
        })
    }
}

/// POST /api/auth/signup -- create account with email + password
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthTokenResponse>, AppError> {
    // Validate email (basic)
    if !req.email.contains('@') || req.email.len() < 5 {
        return Err(AppError::BadRequest("Invalid email".into()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }

    // Check if user already exists
    if db::get_user_by_email(&state.pool, &req.email).await?.is_some() {
        return Err(AppError::BadRequest("Email already registered".into()));
    }

    // Hash password with argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {e}")))?
        .to_string();

    // Create user
    let user_id = db::create_user(&state.pool, &req.email, &password_hash).await?;

    // Grant 10 free signup credits
    db::init_credit_balance(&state.pool, user_id).await?;
    db::add_credit_transaction(&state.pool, user_id, 10, "signup_bonus").await?;

    // Issue JWT
    let token = issue_jwt(&state.config.jwt_secret, user_id, &req.email)?;

    Ok(Json(AuthTokenResponse { token, user_id }))
}

/// POST /api/auth/login -- authenticate with email + password
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthTokenResponse>, AppError> {
    let (user_id, password_hash) = db::get_user_by_email(&state.pool, &req.email)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".into()))?;

    // Verify password
    let parsed_hash = PasswordHash::new(&password_hash)
        .map_err(|_| AppError::Internal("Stored hash is invalid".into()))?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Auth("Invalid email or password".into()))?;

    let token = issue_jwt(&state.config.jwt_secret, user_id, &req.email)?;

    Ok(Json(AuthTokenResponse { token, user_id }))
}

fn issue_jwt(secret: &str, user_id: Uuid, email: &str) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + 86400, // 24 hours
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Token creation failed: {e}")))
}
