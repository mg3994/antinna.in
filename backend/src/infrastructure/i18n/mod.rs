use rust_embed::Embed;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Embed)]
#[folder = "locales/"]
struct Asset;

/// Global singleton (no third-party lazy crate)
static I18N: OnceLock<TranslationService> = OnceLock::new();

/// Initialize once at app startup
pub fn init_i18n(default_locale: String) {
    I18N.set(TranslationService::new(default_locale)) // what to pass here
        .expect("I18N already initialized");
}


/// Access anywhere safely
pub fn i18n() -> &'static TranslationService {
    I18N.get().expect("I18N not initialized")
}

#[derive(Debug)]
pub struct TranslationService {
    // lang_code -> (key -> message)
    data: HashMap<String, HashMap<String, String>>,
    pub(crate) default_locale: String,
}

use crate::core::services::I18nService;

impl I18nService for TranslationService {
    fn translate(&self, key: &str, lang: &str) -> String {
        self.translate(key, lang)
    }
    fn supported_languages(&self) -> Vec<String> {
        self.supported_languages()
    }
}

impl TranslationService {
    pub fn default_locale(&self) -> &str {
        &self.default_locale
    }
    pub fn new(default_locale: String) -> Self {
        let mut data = HashMap::new();

        // Load all locale files
        let locales = vec!["en", "hi"];// , "es", "fr"];

        for locale in locales {
            let filename = format!("{}.json", locale);
            if let Some(file) = Asset::get(&filename) {
                if let Ok(content) = std::str::from_utf8(file.data.as_ref()) {
                    if let Ok(json) = serde_json::from_str::<Value>(content) {
                        let mut flat_map = HashMap::new();
                        flatten_json("", &json, &mut flat_map);
                        data.insert(locale.to_string(), flat_map);
                    }
                }
            }
        }


        Self { data, default_locale }
    }

    /// Get a translated message by key and language
    /// Keys can be nested like "errors.not_found" or "auth.login_success"
    pub fn translate(&self, key: &str, lang: &str) -> String {
        // Try requested language first
        if let Some(messages) = self.data.get(lang) {
            if let Some(msg) = messages.get(key) {
                return msg.clone();
            }
        }
        // Fallback to English
        if let Some(messages) = self.data.get(&self.default_locale) {
            if let Some(msg) = messages.get(key) {
                return msg.clone();
            }
        }
        // Fallback to key itself if not found
        key.to_string()
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

/// Flatten nested JSON into dot-notation keys
/// Example: {"errors": {"not_found": "..."}} becomes {"errors.not_found": "..."}
fn flatten_json(prefix: &str, value: &Value, output: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                flatten_json(&new_prefix, val, output);
            }
        }
        Value::String(s) => {
            output.insert(prefix.to_string(), s.clone());
        }
        _ => {}
    }
}
