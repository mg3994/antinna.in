use salvo::jwt_auth::{ConstDecoder, JwtAuth};
use crate::config::JwtConfig;
use crate::hoops::jwt::JwtClaims;
use crate::utils;

//
// Extraction: It uses your finders to automatically look into the Header, Query, and Cookies.
//
// Validation: It performs the exact same check as decode_token (Signature + Expiration).
//
// Depot Storage: If the token is valid, it extracts the data and puts it into the Salvo Depot. This means your later code doesn't just know the token is valid; it has immediate access to the uid inside it.
//
// Flow Control: Because force_passed(false) is set, it won't block the request if the token is invalid; it just won't put anything in the Depot.
// For your auth_guard (Middlewares): Use the data produced by auth_hoop. If depot.get::<JwtClaims>() is None, the token was either missing, fake, or expired.
pub fn auth_hoop(config: &JwtConfig) -> JwtAuth<JwtClaims, ConstDecoder> {
    JwtAuth::new(ConstDecoder::from_secret(
        config.secret.to_owned().as_bytes(),
    ))
        .finders(utils::get_token_finders())
        .force_passed(false)
}