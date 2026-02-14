use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FirebaseAdminConfig {
    pub service_account_path: Option<String>, // prefer this
    pub service_account_json: Option<String>,
}