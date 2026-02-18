use std::sync::OnceLock;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::Deserialize;

mod log_config;
pub use log_config::LogConfig;
mod db_config;
mod firebase_admin_config;
mod firebase_web_config;

pub use db_config::DbConfig;
pub use crate::infrastructure::config::firebase_admin_config::FirebaseAdminConfig;
pub use crate::infrastructure::config::firebase_web_config::FirebaseWebConfig;


pub static CONFIG: OnceLock<crate::infrastructure::config::ServerConfig> = OnceLock::new();
pub fn init()  {
    let raw_config = Figment::new()
        .merge(Toml::file(
            std::env::var("APP_CONFIG").as_deref().unwrap_or("config.toml"),
        ))
        .merge(Env::prefixed("APP_").global());

    let mut config = match raw_config.extract::<ServerConfig>() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("It looks like your config is invalid. The following error occurred: {e}");
            std::process::exit(1);
        }
    };
    if config.db.url.is_empty() {
        config.db.url = std::env::var("DATABASE_URL").unwrap_or_default();
    }
    if config.db.url.is_empty() {
        eprintln!("DATABASE_URL is not set");
        std::process::exit(1);
    }
    crate::infrastructure::config::CONFIG
        .set(config)
        .expect("config should be set");

}

pub fn get() -> &'static crate::infrastructure::config::ServerConfig {
    crate::infrastructure::config::CONFIG.get().expect("config should be set")
}



#[derive(Deserialize, Clone, Debug)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,

    #[serde(default = "default_locale")]
    pub default_locale: String,

    pub db: DbConfig,
    pub log: LogConfig,
    pub jwt: JwtConfig,
    pub tls: Option<TlsConfig>,

    pub firebase: FirebaseConfig,

}

#[derive(Deserialize, Clone, Debug)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry: i64,
}
#[derive(Deserialize, Clone, Debug)]
pub struct TlsConfig {
    pub cert: String,
    pub key: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FirebaseConfig {
    pub admin: Option<FirebaseAdminConfig>,
    pub web: Option<FirebaseWebConfig>,
}

#[allow(dead_code)]
pub fn default_false() -> bool {
    false
}
#[allow(dead_code)]
pub fn default_true() -> bool {
    true
}

fn default_listen_addr() -> String {
    "127.0.0.1:8008".into()
}


fn default_locale() -> String {
    "en".into()
}

//
fn default_helper_threads() -> usize {
    10
}
fn default_db_pool_size() -> u32 {
    10
}
fn default_tcp_timeout() -> u64 {
    10000
}
fn default_connection_timeout() -> u64 {
    30000
}
fn default_statement_timeout() -> u64 {
    30000
}

//
fn default_filter_level() -> String {
    "info".into()
}
fn default_directory() -> String {
    "./logs".into()
}
fn default_file_name() -> String {
    "app.log".into()
}
fn default_rolling() -> String {
    "daily".into()
}
fn default_format() -> String {
    FORMAT_FULL.into()
}
//


pub const FORMAT_PRETTY: &str = "pretty";
pub const FORMAT_COMPACT: &str = "compact";
pub const FORMAT_JSON: &str = "json";
pub const FORMAT_FULL: &str = "full";