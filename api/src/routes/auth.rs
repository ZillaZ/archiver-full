use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub api_key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub exp: usize,
}

impl Claims {
    pub fn client_id(&self) -> Result<Uuid, StatusCode> {
        Uuid::parse_str(&self.sub).map_err(|_| StatusCode::UNAUTHORIZED)
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let api_key = Uuid::new_v4().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(api_key.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    sqlx::query("INSERT INTO api_keys (key_hash, name) VALUES ($1, $2)")
        .bind(&hash)
        .bind(&req.name)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;

    Ok(Json(RegisterResponse {
        api_key,
        name: req.name,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let rows = sqlx::query_as::<_, (String, String, String)>(
        "SELECT key_hash, name, id::text FROM api_keys",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for (stored_hash, name, id) in &rows {
        let parsed_hash = PasswordHash::new(stored_hash).map_err(|_| StatusCode::UNAUTHORIZED)?;
        if Argon2::default()
            .verify_password(req.api_key.as_bytes(), &parsed_hash)
            .is_ok()
        {
            let exp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + 86400 * 7;

            let claims = Claims {
                sub: id.clone(),
                name: name.clone(),
                exp,
            };

            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            )
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Json(LoginResponse { token }));
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    if path.starts_with("/v1/auth") {
        return Ok(next.run(req).await);
    }

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/register", axum::routing::post(register))
        .route("/login", axum::routing::post(login))
}