use salvo::jwt_auth::{ConstDecoder, JwtAuth};
use crate::config::JwtConfig;
use crate::hoops::jwt::JwtClaims;
use crate::{db, utils};

use salvo::prelude::*;

use sqlx::{Postgres, Transaction};


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


// #[handler]
// pub async fn my_handler(depot: &mut Depot, res: &mut Response) {
//     // Remove the transaction from depot
//     let mut tx = depot.remove::<Transaction<'static, Postgres>>("tx").unwrap();
//
//     // ... run queries ...
//
//     // Commit is vital
//     tx.commit().await.ok();
// }

pub struct DbRlsMiddleware {
    pub auth: JwtAuth<JwtClaims, ConstDecoder>,
}

#[handler]
impl DbRlsMiddleware {
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 1. Run the JWT Auth extraction
        self.auth.handle(req, depot, res, ctrl).await;

        // 1.1 Check if JwtAuth failed.
        // If JwtAuth was unsuccessful, it sets the state to Unauthorized or Forbidden.
        if depot.jwt_auth_state() != JwtAuthState::Authorized {
            // JwtAuth already called ctrl.skip_rest() and set status_code,
            // so we just return here to stop our RLS logic.
            return;
        }

        // 2. Use the official Extension Trait to get the claims
        // .jwt_auth_data::<C>() is provided by the salvo::jwt_auth::JwtAuthDepotExt trait
        if let Some(token_data) = depot.jwt_auth_data::<JwtClaims>() {
            let pool = db::pool();
            match pool.begin().await {
                Ok(mut tx) => {
                    // 3. Set the RLS variable using the uid from your claims
                    // Note: current_setting in Postgres always returns text,
                    // so we pass the string and cast in SQL.
                    let setup = sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
                        .bind(&token_data.claims.uid)
                        .execute(&mut *tx)
                        .await;
                    if setup.is_ok() {
                        // 4. Inject transaction into depot for the handler
                        depot.insert("tx", tx);

                        // FIX: call_next needs req, depot, res passed into it
                        ctrl.call_next(req, depot, res).await;
                    } else {
                        res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                        ctrl.skip_rest();
                    }
                }
                Err(_) => {
                    res.status_code(StatusCode::SERVICE_UNAVAILABLE);
                    ctrl.skip_rest();
                }
            }
        }
        else {
            // FIX: Use .status_code() with no args to GET (returns Option)
            // OR simply set it directly if we know we are in a failure state.
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();

        } //<- complete it


    }
}
pub fn auth_db_rls_hoop(config: &JwtConfig) -> DbRlsMiddleware { //<- modify this
    let auth = JwtAuth::new(ConstDecoder::from_secret(
        config.secret.to_owned().as_bytes(),
    ))
        .finders(utils::get_token_finders())

        .force_passed(false);

    DbRlsMiddleware { auth}
}