pub trait I18nService: Send + Sync {
    /// Get a translated message by key and language
    fn translate(&self, key: &str, lang: &str) -> String;

    /// Get list of supported languages
    fn supported_languages(&self) -> Vec<String>;
}