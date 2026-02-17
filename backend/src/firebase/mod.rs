use std::sync::OnceLock;
use crate::config;
use crate::config::FirebaseAdminConfig;
use firebase_admin_sdk::{FirebaseApp, yup_oauth2};

use salvo::prelude::*;
use crate::models::FirebaseFeatures;


//
// window.addEventListener("fb:token", async (e) => {
// const token = e.detail.idToken;
//
// await fetch("/api/auth/refresh_session", {
// method: "POST",
// headers: { "Content-Type": "application/json" },
// body: JSON.stringify({ id_token: token }),
// });
// });
//
// window.addEventListener("fb:logout", () => {
// console.log("User logged out");
// });




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
/// Helper to get the web config safely
fn get_web_config() -> &'static crate::config::FirebaseWebConfig {
    config::get()
        .firebase
        .web
        .as_ref()
        .expect("Firebase Web config not set")
}

//

/// The raw JS object for Firebase Config
fn get_js_config() -> String {
    let web = get_web_config();
    format!(
        r#"{{
    apiKey: "{}",
    authDomain: "{}",
    projectId: "{}",
    storageBucket: "{}",
    messagingSenderId: "{}",
    appId: "{}"
  }}"#,
        web.api_key, web.auth_domain, web.project_id,
        web.storage_bucket.as_deref().unwrap_or(""),
        web.messaging_sender_id, web.app_id
    )
}
// Provides SSR injection of Firebase Web config and auto-initialization script.
///! Generate Firebase Web config script for CSR injection, including automatic `initializeApp()`.
/// For auth.html SSR injection
pub fn csr_script(features: FirebaseFeatures) -> String {
    let config_js = get_js_config();
    let vapid_key = &get_web_config().vapid_public_key;

    let mut scripts = String::from(r#"<script src="https://www.gstatic.com/firebasejs/9.22.1/firebase-app-compat.js"></script>"#);
    let mut init_logic = String::new();

    if features.auth {
        scripts.push_str(r#"<script src="https://www.gstatic.com/firebasejs/9.22.1/firebase-auth-compat.js"></script>"#);
        init_logic.push_str(
            r#"
window.fbAuth = firebase.auth(app);

// ---- Firebase auth lifecycle bridge ----
let __lastFbToken = null;

firebase.auth().onIdTokenChanged(async (user) => {
    if (!user) {
        window.dispatchEvent(new CustomEvent("fb:logout"));
        return;
    }

    try {
        const idToken = await user.getIdToken();

        // prevent duplicate refresh calls
        if (__lastFbToken === idToken) return;
        __lastFbToken = idToken;

        window.dispatchEvent(
            new CustomEvent("fb:token", {
                detail: { idToken }
            })
        );
    } catch (err) {
        console.error("Token refresh error:", err);
    }
});
"#
        );
    }

    if features.messaging {
        scripts.push_str(r#"<script src="https://www.gstatic.com/firebasejs/9.22.1/firebase-messaging-compat.js"></script>"#);
        init_logic.push_str("window.fbMessaging = firebase.messaging(app);\n");
    }

    format!(
        r#"
        {}
        <script>
          window.FIREBASE_CONFIG = {};
          window.VAPID_KEY = "{}";
          const app = firebase.initializeApp(window.FIREBASE_CONFIG);
          {}
        </script>
        "#,
        scripts, config_js, vapid_key, init_logic
    )
}



/// The dynamic Service Worker logic (firebase-messaging-sw.js)
pub fn sw_script() -> String {
    let config_js = get_js_config();

    format!(
        r#"
importScripts('https://www.gstatic.com/firebasejs/9.22.1/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/9.22.1/firebase-messaging-compat.js');

// Initialize the Firebase app in the service worker
const app = firebase.initializeApp({});

// Retrieve an instance of Firebase Messaging and pass the app explicitly
const messaging = firebase.messaging(app);

// This listener handles the background notification when the app is not in focus
messaging.onBackgroundMessage((payload) => {{
    console.log('[firebase-messaging-sw.js] Received background message: ', payload);

    const notificationTitle = payload.notification.title || 'New Message';
    const notificationOptions = {{
        body: payload.notification.body || 'You have a new update.',
        icon: payload.notification.image || '/favicon.ico',
        data: payload.data, // Custom data from server
    }};

    self.registration.showNotification(notificationTitle, notificationOptions);
}});

// Optional: Handle notification click to open the app
self.addEventListener('notificationclick', (event) => {{
    event.notification.close();
    event.waitUntil(
        clients.matchAll({{ type: 'window', includeUncontrolled: true }}).then((clientList) => {{
            if (clientList.length > 0) {{
                return clientList[0].focus();
            }}
            return clients.openWindow('/');
        }})
    );
}});
"#,
        config_js
    )
}


#[handler]
pub async fn firebase_sw_handler(res: &mut Response) {
    // Render as JavaScript content type
    res.render(Text::Js(crate::firebase::sw_script()));
}