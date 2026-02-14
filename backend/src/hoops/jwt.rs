use anyhow::Result;
use jsonwebtoken::{decode, Algorithm, DecodingKey, EncodingKey, Validation};

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use crate::config;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims {
    uid: String,
    exp: i64,
}



pub fn generate_jwt_token(uid: impl Into<Uuid>) -> Result<(String, i64)> {
    let exp = OffsetDateTime::now_utc() + Duration::seconds(config::get().jwt.expiry);
    let claim = JwtClaims {
        uid: uid.into().to_string(),
        exp: exp.unix_timestamp(),
    };
    let token: String = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claim,
        &EncodingKey::from_secret(config::get().jwt.secret.as_bytes()),
    )?;
    Ok((token, exp.unix_timestamp()))
}

// #[allow(dead_code)]
pub fn is_jwt_token_valid(token: &str) -> bool {
    let validation = Validation::new(Algorithm::HS256);
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(config::get().jwt.secret.as_bytes()),
        &validation,
    )
    .is_ok()
}
