use salvo::jwt_auth::{ConstDecoder, JwtAuth, JwtAuthState, JwtAuthDepotExt}; // Added Ext traits
use salvo::prelude::*;
// use sqlx::{Postgres, Transaction};

use crate::infrastructure::config::JwtConfig;
use crate::hoops::jwt::JwtClaims;
use crate::{db, utils};

pub fn auth_hoop(config: &JwtConfig) -> JwtAuth<JwtClaims, ConstDecoder> {
    JwtAuth::new(ConstDecoder::from_secret(
        config.secret.as_bytes(),
    ))
        .finders(utils::get_token_finders())
        .force_passed(false)
}

pub struct DbRlsMiddleware {
    pub auth: JwtAuth<JwtClaims, ConstDecoder>,
}

#[handler]
impl DbRlsMiddleware {
    // Note: Removed 'req' and 'res' from the auth.handle call if you want
    // to manually control the flow, but usually, we pass them through.
    async fn handle(&self, req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
        // 1. Run the JWT Auth extraction manually
        self.auth.handle(req, depot, res, ctrl).await;

        // 2. Check if JwtAuth succeeded
        // If force_passed(false) was used in auth_hoop, it might have already stopped the flow.
        if depot.jwt_auth_state() != JwtAuthState::Authorized {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render("Unauthorized"); // <- yet , TODO ; this
            ctrl.skip_rest();
            return;
        }

        // 3. Get the claims
        let uid = if let Some(data) = depot.jwt_auth_data::<JwtClaims>() {
            data.claims.uid.clone()
        } else {
            res.status_code(StatusCode::UNAUTHORIZED);
            ctrl.skip_rest();
            return;
        };

        // 4. Start Transaction
        let pool = db::pool();
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(_) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                ctrl.skip_rest();
                return;
            }
        };

        // 5. RLS setup: Set the session variable in Postgres
        // Use 'true' for the third parameter so it only lasts for the current transaction
        let setup = sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(&uid)
            .execute(&mut *tx)
            .await;

        if setup.is_err() {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            ctrl.skip_rest();
            return;
        }

        // 6. Store transaction in Depot
        // IMPORTANT: The transaction must be committed/rolled back in the final handler
        depot.insert("tx", tx);

        ctrl.call_next(req, depot, res).await;
    }
}

pub fn auth_db_rls_hoop(config: &JwtConfig) -> DbRlsMiddleware {
    DbRlsMiddleware {
        auth: auth_hoop(config)
    }
}