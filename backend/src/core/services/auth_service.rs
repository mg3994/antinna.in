

use chrono::{DateTime, Utc};
use salvo::async_trait;
use crate::interface::http::AppResult;

#[derive(Debug, Clone)]
pub struct ExternalAuthUser {
    pub uid: String,
    pub email: Option<String>,
    pub email_verified: bool,
    pub display_name: Option<String>,
    pub photo_url: Option<String>,
    pub phone_number: Option<String>,
    pub expiration: DateTime<Utc>,
    pub identities: Vec<ExternalIdentity>,
    pub provider_id: String, // e.g., "google.com", "password"
}

#[derive(Debug, Clone)]
pub struct ExternalIdentity {
    pub provider_slug: String,
    pub provider_uid: String,
    pub identifier: Option<String>,
}

/// Port trait for authentication services (Domain Port)
#[async_trait]
pub trait AuthService: Send + Sync {
    /// Verify an external token and return user information
    async fn verify_token(&self, id_token: &str) -> AppResult<ExternalAuthUser>;
}
