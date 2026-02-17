use anyhow::Result;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Validation};

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use crate::config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims {
    pub uid: String, // users(id) stored as String in JWT standard usually, but Uuid works if serializer handles it
    pub sid: Uuid,   // Session ID (The new addition)
    pub exp: i64,    // Expiration timestamp
    pub iat: i64,    // Issued at
}



pub fn generate_jwt_token(uid: impl Into<Uuid>, sid: impl Into<Uuid>) -> Result<(String, i64)> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::seconds(config::get().jwt.expiry);
    let claim = JwtClaims {
        uid: uid.into().to_string(),
        sid: sid.into(), // Convert the session ID here
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
    };
    let token: String = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &EncodingKey::from_secret(config::get().jwt.secret.as_bytes()),
    )?;
    Ok((token, exp.unix_timestamp()))
}

/// Helper: Decodes and validates signature. Returns claims if valid.
pub fn get_token_claims(token: &str) -> Option<JwtClaims> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(config::get().jwt.secret.as_bytes()),
        &validation,
    )
        .ok()
        .map(|data| data.claims)
}
// #[allow(dead_code)]
/// Use this for fast UI checks
pub fn is_jwt_token_signature_valid(token: &str) -> bool {
    get_token_claims(token).is_some()
}

/// Use this for secure API checks
pub async fn is_jwt_session_active(token: &str, pool: &sqlx::PgPool) -> bool {
    // 1. Reuse the signature check logic
    let claims = match get_token_claims(token) {
        Some(c) => c,
        None => return false,
    };

    // 2. Check Database State
    let result = sqlx::query!(
        r#"
        SELECT user_id
        FROM users_sessions
        WHERE id = $1
          AND revoked_at IS NULL
          AND auth_exp > to_timestamp($2)
        "#,
        claims.sid,
        claims.exp as f64 // FIX: Cast i64 to f64 for Postgres compatibility
    )
        .fetch_optional(pool)
        .await;

    match result {
        Ok(Some(row)) => row.user_id.to_string() == claims.uid,
        _ => false,
    }
}