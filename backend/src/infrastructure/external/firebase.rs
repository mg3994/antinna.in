use firebase_admin_sdk::{FirebaseApp, yup_oauth2};
use crate::infrastructure::config::FirebaseAdminConfig;
use tracing::{error, info, warn};

pub async fn init(config: &FirebaseAdminConfig) -> Option<FirebaseApp> {
    // 1. Resolve Key Source
    let service_account_key = if let Some(path) = &config.service_account_path {
        info!("Firebase: Loading from file path: {}", path);
        Some(yup_oauth2::read_service_account_key(path)
            .await
            .expect("Failed to read Firebase service account file"))

    } else if let Some(json_str) = &config.service_account_json {
        info!("Firebase: Loading from inline JSON string");
        let key: yup_oauth2::ServiceAccountKey = serde_json::from_str(json_str)
            .expect("Failed to parse inline Firebase service account JSON");
        Some(key)

    } else {
        // Source C: Manual Fields - Strict Check
        // We check every field you listed as required
        match (
            &config.r#type,
            &config.project_id,
            &config.private_key_id,
            &config.private_key,
            &config.client_email,
            &config.client_id,
            &config.auth_uri,
            &config.token_uri,
            &config.auth_provider_x509_cert_url,
            &config.client_x509_cert_url,
        ) {
            (
                Some(key_type),
                Some(proj_id),
                Some(pk_id),
                Some(pk),
                Some(email),
                Some(c_id),
                Some(a_uri),
                Some(t_uri),
                Some(ap_url),
                Some(cx_url),
            ) => {
                info!("Firebase: Manual config detected for project_id: {}", proj_id);
                Some(yup_oauth2::ServiceAccountKey {
                    key_type: Some(key_type.clone()),
                    project_id: Some(proj_id.clone()),
                    private_key_id: Some(pk_id.clone()),
                    private_key: pk.clone(),
                    client_email: email.clone(),
                    client_id: Some(c_id.clone()),
                    auth_uri: Some(a_uri.clone()),
                    token_uri: t_uri.clone(),
                    auth_provider_x509_cert_url: Some(ap_url.clone()),
                    client_x509_cert_url: Some(cx_url.clone()),
                })
            },
            _ => {
                // This captures if ANY of the fields above were None
                error!("Firebase Init Failed: One or more manual config fields are missing!");
                None
            }
        }
    };

    // 2. Final Construction
    match service_account_key {
        Some(key) => {
            info!("FirebaseApp successfully initialized.");
            Some(FirebaseApp::new(key))
        },
        None => {
            warn!("FirebaseApp initialization skipped: No valid configuration source (path, json, or manual fields) was fully provided.");
            None
        }
    }
}