use std::sync::OnceLock;
use crate::config;
use crate::config::FirebaseAdminConfig;
use firebase_admin_sdk::{FirebaseApp, yup_oauth2};



// admin
pub static FIREBASE_ADMIN: OnceLock<FirebaseApp> = OnceLock::new();

/// Initialize Firebase Admin SDK with service account JSON
pub async  fn init(config: &FirebaseAdminConfig) {
    // Load the service account key (e.g., from a file)
    if let Some(firebase_service_account_path) = &config.service_account_path {
        // Load the service account key (JSON)
        let service_account_key =
            yup_oauth2::read_service_account_key(firebase_service_account_path)
                .await
                .expect("Failed to read Firebase service account file");

        // Initialize FirebaseApp
        let app = FirebaseApp::new(service_account_key);
        // Option 1: safer manual check
        if crate::firebase::FIREBASE_ADMIN.set(app).is_err() {
            println!("Firebase Admin already initialized, skipping");
        }
    }

}

/// Get global Firebase Admin instance
pub fn firebase_admin() -> &'static FirebaseApp {
    FIREBASE_ADMIN.get().expect("Firebase Admin is not initialized")
}



//

// WEB

//
// Provides SSR injection of Firebase Web config and auto-initialization script.
///! Generate Firebase Web config script for SSR injection, including automatic `initializeApp()`.
pub fn ssr_script() -> String {
    let web = config::get()
        .firebase
        .web
        .as_ref()
        .expect("Firebase Web config not set");

    format!(
        r#"<script src="https://www.gstatic.com/firebasejs/9.22.1/firebase-app.js"></script>
<script src="https://www.gstatic.com/firebasejs/9.22.1/firebase-messaging.js"></script>
<script>
  // Firebase Web config
  window.FIREBASE_CONFIG = {{
    apiKey: "{}",
    authDomain: "{}",
    projectId: "{}",
    storageBucket: "{}",
    messagingSenderId: "{}",
    appId: "{}"
  }};

  window.VAPID_KEY = "{}";

  // Auto-initialize Firebase in browser
  const app = firebase.initializeApp(window.FIREBASE_CONFIG);
  const messaging = firebase.messaging();
</script>"#,
        web.api_key,
        web.auth_domain,
        web.project_id,
        web.storage_bucket.as_deref().unwrap_or(""),
        web.messaging_sender_id,
        web.app_id,
        web.vapid_public_key
    )
}
