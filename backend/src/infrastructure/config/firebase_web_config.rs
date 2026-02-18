use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FirebaseWebConfig {
    pub api_key: String,
    pub auth_domain: String,
    pub project_id: String,
    pub storage_bucket: Option<String>,
    pub messaging_sender_id: String,
    pub app_id: String,
    pub measurement_id: Option<String>,
    pub vapid_public_key: String,
}
