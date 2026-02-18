use std::sync::Arc;

use chrono::DateTime;

use firebase_admin_sdk::FirebaseApp;
use salvo::async_trait;
use crate::interface::http::AppResult;
use crate::core::services::auth_service::{AuthService, ExternalAuthUser, ExternalIdentity};
use crate::core::errors::{AppError};


pub struct FirebaseAdapter {
    pub app: Arc<FirebaseApp>,
}

#[async_trait]
impl AuthService for FirebaseAdapter {
    async fn verify_token(&self, id_token: &str) -> AppResult<ExternalAuthUser> {
        let auth = self.app.auth();
        let token_claims = auth
            .verify_id_token(id_token)
            .await
            .map_err(|e| AppError::unauthorized(format!("Firebase Token verification failed: {}", e)))?;

        let provider_id = match token_claims.claims.get("firebase") {
            Some(serde_json::Value::Object(f)) => match f.get("sign_in_provider") {
                Some(serde_json::Value::String(p)) => p.clone(),
                _ => "unknown".to_string(),
            },
            _ => "unknown".to_string(),
        };

        let mut identities = Vec::new();
        let email = token_claims.email.clone();
        let phone_number = match token_claims.claims.get("phone_number") {
            Some(serde_json::Value::String(s)) => Some(s.to_string()),
            _ => None,
        };

        if let Some(serde_json::Value::Object(firebase)) = token_claims.claims.get("firebase") {
            if let Some(serde_json::Value::Object(ids_map)) = firebase.get("identities") {
                for (slug, ids_val) in ids_map {
                    let provider_slug = slug.as_str();
                    let provider_category = match provider_slug {
                        "email" => "password",
                        "phone" => "phone",
                        other => other,
                    };

                    if let serde_json::Value::Array(arr) = ids_val {
                        if let Some(serde_json::Value::String(p_uid)) = arr.get(0) {
                            let identifier = match provider_category {
                                "password" => email.clone(),
                                "phone" => phone_number.clone(),
                                _ => None,
                            };

                            identities.push(ExternalIdentity {
                                provider_slug: provider_category.to_string(),
                                provider_uid: p_uid.clone(),
                                identifier,
                            });
                        }
                    }
                }
            }
        }

        let expiration = DateTime::from_timestamp(token_claims.exp as i64, 0)
            .ok_or_else(|| AppError::internal("Invalid expiration timestamp from Firebase"))?;

        Ok(ExternalAuthUser {
            uid: token_claims.sub,
            email,
            email_verified: token_claims.email_verified.unwrap_or(false),
            display_name: token_claims.name,
            photo_url: token_claims.picture,
            phone_number: phone_number.clone(),
            expiration,
            identities,
            provider_id,
        })
    }
}
