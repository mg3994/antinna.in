use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FirebaseAdminConfig {
    // Preferred: path to service account JSON file
    pub service_account_path: Option<String>,
    // Alternative: inline service account JSON string
    pub service_account_json: Option<String>,

    // Optional: inline service account fields (if not using path or JSON string)
    #[serde(rename = "type", default = "default_service_account_type")]
    pub r#type: Option<String>,
    pub project_id: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: Option<String>,
    pub client_email: Option<String>,
    pub client_id: Option<String>,
    #[serde(default = "default_auth_uri")]
    pub auth_uri: Option<String>,
    #[serde(default = "default_token_uri")]
    pub token_uri: Option<String>,
    #[serde(default = "default_auth_provider_x509_cert_url")]
    pub auth_provider_x509_cert_url: Option<String>,
    pub client_x509_cert_url: Option<String>,
    #[serde(default = "default_universe_domain")]
    pub universe_domain: Option<String>
}


fn default_service_account_type() -> Option<String> {
    Some("service_account".to_string())
}

fn default_auth_uri() -> Option<String> {
    Some("https://accounts.google.com/o/oauth2/auth".to_string())
}

fn default_token_uri() -> Option<String> {

    Some("https://oauth2.googleapis.com/token".to_string())
}


fn default_auth_provider_x509_cert_url() -> Option<String> {
    Some("https://www.googleapis.com/oauth2/v1/certs".to_string())
}

fn default_universe_domain() -> Option<String> {
    Some("googleapis.com".to_string())
}

