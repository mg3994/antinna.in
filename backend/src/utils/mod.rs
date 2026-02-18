use argon2::{
    Argon2, PasswordHash,
    password_hash::{SaltString, rand_core::OsRng},
};
use rand::{ RngExt};
use std::iter;
use salvo::jwt_auth::{CookieFinder, HeaderFinder, JwtTokenFinder, QueryFinder};
// added by Manish
use salvo::prelude::*;

#[allow(dead_code)]
#[inline]
pub fn random_string(limit: usize) -> String {
    iter::repeat(())
        .map(|_| rand::rng().sample(rand::distr::Alphanumeric))
        .map(char::from)
        .take(limit)
        .collect()
}

// pub fn verify_password(password: &str, password_hash: &str) -> anyhow::Result<()> {
//     let hash = PasswordHash::new(&password_hash)
//         .map_err(|e| anyhow::anyhow!("invalid password hash: {}", e))?;
//     let result = hash.verify_password(&[&Argon2::default()], password);
//     match result {
//         Ok(_) => Ok(()),
//         Err(_) => Err(anyhow::anyhow!("invalid password")),
//     }
// }

// pub fn hash_password(password: &str) -> anyhow::Result<String> {
//     let salt = SaltString::generate(&mut OsRng);
//     Ok(PasswordHash::generate(Argon2::default(), password, &salt)
//         .map_err(|e| anyhow::anyhow!("failed to generate password hash: {}", e))?
//         .to_string())
// }
//  Added by Manish

// This function defines the "Source of Truth" for where tokens live
pub fn get_token_finders() -> Vec<Box<dyn JwtTokenFinder>> {
    vec![
        Box::new(HeaderFinder::new()),
        Box::new(QueryFinder::new("token")),
        Box::new(CookieFinder::new("jwt_token")),
        Box::new(CookieFinder::new("token")),
    ]
}
pub async fn extract_jwt_token_manually(req: &mut Request) -> Option<String> {
    for finder in get_token_finders() {
        // find_token returns a Future, so we must .await it
        if let Some(token) = finder.find_token(req).await {
            return Some(token);
        }
    }
    None
}
/// Returns true if the server is configured to use TLS (HTTPS/Quinn)
pub fn is_secure_context() -> bool {
    crate::infrastructure::config::get().tls.is_some()
}

