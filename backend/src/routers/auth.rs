use cookie::Cookie;
use askama::Template;
use firebase_admin_sdk::auth::verifier::FirebaseTokenClaims;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::hoops::jwt;
use crate::models::{FirebaseFeatures, FullUserRecord, Gender, ProviderType, User};
use crate::{db, firebase, json_ok, utils, AppError, AppResult, JsonResult};
use crate::firebase::firebase_admin;

#[handler]
pub async fn verify_link_page(req: &mut Request,res: &mut Response) -> AppResult<()> {
    #[derive(Template)]
    #[template(path = "verify_link.html")]
    struct VerifyTemplate {
        firebase_web_script: Option<String>,
    }

    // 1. Extract token using your universal finders (Header, Query, Cookies)
    if let Some(jwt_token) = utils::extract_jwt_token_manually(req).await { // check even from headers adn query
        // 2. Validate the token (checks signature + expiration)
        if jwt::is_jwt_token_signature_valid(&jwt_token) //TODO: in future is_jwt_session_active
        {
                res.render(Redirect::other("/users")); //
                return Ok(());
            }
        }

        let script = Some(crate::firebase::csr_script(FirebaseFeatures { auth: true, messaging: true }));
        let tmpl = VerifyTemplate { firebase_web_script: script };

        res.render(Text::Html(tmpl.render().unwrap()));
        Ok(())
    }

    #[handler]
    pub async fn auth_page(req: &mut Request, res: &mut Response) -> AppResult<()> {
        #[derive(Template)]
        #[template(path = "auth.html")]
        struct AuthTemplate {
            firebase_web_script: Option<String>, // Add this field
            providers: Vec<ProviderType>,
        }
        // 1. Extract token using your universal finders (Header, Query, Cookies)
        if let Some(jwt_token) = utils::extract_jwt_token_manually(req).await { // check even from headers adn query
            // 2. Validate the token (checks signature + expiration)
            if jwt::is_jwt_token_signature_valid(&jwt_token) { //TODO: in future is_jwt_session_active
                res.render(Redirect::other("/users")); //
                return Ok(());
            }
        }
        let conn = db::pool();
        let providers: Vec<ProviderType> = sqlx::query_as!(
        ProviderType,
      r#"
            SELECT slug, name
            FROM provider_types
            WHERE is_active = true
            ORDER BY
                CASE
                    WHEN slug = 'password' THEN 1
                    ELSE 2
                END,
                name ASC
            "#
              )
            .fetch_all(conn)
            .await?;
        // 2. Inject Firebase Script
        // Assuming csr_script() is accessible here
        let script = Some(firebase::csr_script(FirebaseFeatures { auth: true, messaging: true }));
        let hello_tmpl = AuthTemplate {
            firebase_web_script: script,
            providers,
        };
        res.render(Text::Html(hello_tmpl.render().unwrap()));
        Ok(())
    }

    #[derive(Deserialize, ToSchema, Default, Debug)]
    pub struct FirebaseLoginInData {
        pub id_token: String,
        pub fcm_token: Option<String>,
        pub device_id: String,
        pub user_agent: Option<String>,
    }
    // #[derive(Serialize, ToSchema, Default, Debug)]
    #[derive(Serialize, ToSchema, Debug)]

    pub struct FirebaseLoginOutData {
        pub id: Uuid,
        pub firebase_uid: String,
        pub session_id: Uuid,
        pub username: Option<String>, // New users might not have one yet
        pub display_name: Option<String>,
        pub bio: Option<String>,
        pub avatar_url: Option<String>,
        pub gender: Option<Gender>, // Use your Gender enum
        pub dob: Option<chrono::NaiveDate>,
        //
        pub current_auth_provider_slug: String,      // e.g., "google.com"
        pub current_provider_internal_uid:Option<String>,
        pub is_verified: bool,        // e.g., is_email_verified

        pub token: String,
        pub exp: i64,
    }
    #[endpoint(tags("auth"))]
    pub async fn post_authenticate(
        idata: JsonBody<FirebaseLoginInData>,
        req: &mut Request,
        res: &mut Response,
    ) -> JsonResult<FirebaseLoginOutData> {
        let idata = idata.into_inner();
        // the flow is no login not signup, just auth it will seprate out if a user is alredy there or we have to create a new one
        //  use firebase_admin() to verify the incomming token and check if that if firebase_uid is already there if not create a new user then create a jwt token and do rls related feeding for auth_identities based on response

        // 1. Verify Firebase ID token
        // 1. Verify and get claims
        let token_claims: FirebaseTokenClaims = firebase_admin().auth().verify_id_token(&idata.id_token).await
            .map_err(|e| StatusError::unauthorized().brief("Invalid Token"))?;
        //  inow check is there any valid firebase uid in users table exists if exists create token and go up next if not then we have to create a new user record
        // --- SEPARATING FIELDS ---

        println!("{:?}", &token_claims); // TODO: Uncomment in production

        // // The Unique Firebase ID (Local UID)
        let firebase_uid = &token_claims.sub;
        //
        // // Extract Provider Info from the 'firebase' claim map
        let firebase_internal = &token_claims.claims.get("firebase")
            .and_then(|v| v.as_object());
        //
        // // e.g., 'google.com', 'apple.com', or 'password' 'phone'
        let current_provider_name = firebase_internal
            .and_then(|f| f.get("sign_in_provider"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        //
        // // The Provider-specific UID (e.g., the actual Google ID)
        // // Firebase stores this in identities -> {provider_name} -> [0]
        let current_provider_internal_uid = firebase_internal
            .and_then(|f| f.get("identities"))
            .and_then(|i| i.get(current_provider_name))
            .and_then(|arr| arr.get(0))
            .and_then(|v| v.as_str());
        // //

        let is_email_verified = token_claims.email_verified.unwrap_or(false);
        // This serves as the "signal" for the DB trigger to use CURRENT_TIMESTAMP
        let signal_time = Some(chrono::Utc::now());

        // 2. Extract and Normalize Identities
        let mut normalized_accounts = Vec::new();
        if let Some(identities) = firebase_internal.and_then(|f| f.get("identities")).and_then(|v| v.as_object()) {
            for (slug, ids) in identities {
                if let Some(p_uid) = ids.as_array().and_then(|arr| arr.get(0)).and_then(|v| v.as_str()) {

                    // --- FIX: Normalize Firebase 'email' to your DB 'password' ---
                    let provider_slug = if slug == "email" { "password" } else { slug.as_str() };

                    // Logic to decide if we should tell the DB to verify this identity
                    let v_at = match provider_slug {
                        "google.com" | "apple.com" | "phone" => signal_time,
                        "password" => if is_email_verified { signal_time } else { None },
                        _ => None,
                    };

                    normalized_accounts.push((provider_slug.to_string(), p_uid.to_string(), v_at));
                }
            }
        }
        let display_name = &token_claims.name;
        let avatar_url = &token_claims.picture;


        let pool = db::pool();
        // we wiill try to feed user in users table it may be possible that that firebase_uid already exists (As our query is whee firebase_uid is...) on conflict we will ty to update those other stuffs ,first fetch users as user_record
        let mut tx = pool.begin().await.map_err(|e| AppError::Internal(e.to_string()))?;

        // 4. Minimal Upsert: Fetches the whole record even if it already existed.
        // We use the "firebase_uid = EXCLUDED.firebase_uid" trick to force a row return on conflict.
        // 4. Minimal Upsert using query_as!
        // 4. Combined Upsert & Join using query_as!
        let user_record = sqlx::query_as!(
    FullUserRecord,
    r#"
    WITH upserted_user AS (
        INSERT INTO users (firebase_uid, display_name, avatar_url)
        VALUES ($1, $2, $3)
        ON CONFLICT (firebase_uid) DO UPDATE
        SET firebase_uid = EXCLUDED.firebase_uid
        RETURNING *
    )
    SELECT
        u.id,
        u.firebase_uid,
        u.display_name,
        un.username AS "username?", -- The ? tells SQLx this field might be null
        u.bio,
        u.avatar_url,
        u.gender AS "gender: Gender",
        u.dob,
        u.embedding_dirty,
        u.created_at,
        u.updated_at,
        u.deleted_at
    FROM upserted_user u
    LEFT JOIN usernames un ON u.id = un.user_id
    "#,
    firebase_uid,
    token_claims.name,
    token_claims.picture
)
            .fetch_one(pool) // Using pool directly like in auth_page
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;


        // 5. Security Check: Block access if the user is soft-deleted
        if user_record.deleted_at.is_some() {
            return Err(StatusError::forbidden()
                .brief("This account has been deactivated.")
                .into());
        }
        // now check if firebase_uid is there or not if exists then just update some stuffs in users table else create new user record (first query ends)then let use users id then let we will this time tart a transation so that first we SELECT set_config('app.current_user_id', $1, true) and keeping that query open update ==>


        // B. Establish RLS Session for the current transaction
        // This allows the following 'auth_identities' queries to pass RLS checks
        // 5. Establish RLS Session
        let current_user_id = user_record.id.to_string();

        let current_user_id = user_record.id.to_string();
        sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(&current_user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // 6. Sync Identities with COALESCE and Trigger support
        for (slug, p_uid, v_at) in normalized_accounts {
            sqlx::query!(
                r#"
                INSERT INTO auth_identities (user_id, provider, provider_uid, verified_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (provider, provider_uid) DO UPDATE
                SET verified_at = COALESCE(auth_identities.verified_at, EXCLUDED.verified_at)
                "#,
                user_record.id,
                slug,
                p_uid,
                v_at
            )
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }


        // --- NEW: SESSION TRACKING ---
        let user_agent = idata.user_agent.clone().or_else(|| {
            req.headers()
                .get(salvo::http::header::USER_AGENT)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        }).unwrap_or_else(|| "unknown".to_string());

        // Salvo's remote_addr usually returns a reference to a SocketAddr directly
        // If it's not an Option, we just call to_string() on it.
        let ip_address = req.remote_addr().to_string();


        // Convert Firebase Unix i64 to Postgres-compatible Timestamp
        // Convert Firebase Unix u64 to i64, then to Postgres-compatible Timestamp
        let auth_exp = chrono::DateTime::from_timestamp(token_claims.exp as i64, 0)
            .ok_or_else(|| AppError::Internal("Invalid expiration timestamp".to_string()))?;
        // 8. Upsert Session
        let session_row = sqlx::query!(
            r#"
            INSERT INTO users_sessions (user_id, device_id, fcm_token, user_agent, ip_address, auth_exp)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id, device_id) DO UPDATE
            SET
                fcm_token = COALESCE(EXCLUDED.fcm_token, users_sessions.fcm_token),
                user_agent = EXCLUDED.user_agent,
                ip_address = EXCLUDED.ip_address,
                auth_exp = EXCLUDED.auth_exp,
                revoked_at = NULL,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id
            "#,
            user_record.id,
            idata.device_id,
            idata.fcm_token,
            user_agent,
            ip_address,
            auth_exp
        )
            .fetch_one(&mut *tx)
            .await?;
        tx.commit().await.map_err(|e| AppError::Internal(e.to_string()))?;
        let (jwt_token, exp) = jwt::generate_jwt_token(user_record.id, session_row.id)?;

        // // Profile Info

        // let email = token_claims.email; //<- not just email i want that provider to be verified
        // let is_email_verified = token_claims.email_verified.unwrap_or(false);
        // let conn = db::pool();
        // let Some(User {
        //              id,
        //             firebase_uid,
        //              display_name,
        //               bio,
        //               avatar_url,
        //              gender,
        //              dob,
        //              embedding_dirty,
        //
        //          }) = sqlx::query_as!(
        //     User,
        //     r#"
        //         SELECT id, username, password FROM users
        //         WHERE username = $1
        //         "#,
        //     idata.username
        // )
        // .fetch_optional(conn)
        // .await?
        // else {
        //     return Err(StatusError::unauthorized()
        //         .brief("User does not exist.")
        //         .into());
        // };

        // the flow will be like

        // if utils::verify_password(&idata.password, &password).is_err()
        // {
        //     return Err(StatusError::unauthorized()
        //         .brief("Account not exist or password is incorrect.")
        //         .into());
        // }

        // let (token, exp) = jwt::generate_jwt_token(id)?; //- this id is just that User { id,
        // let odata = FirebaseLoginOutData {
        //     id,
        //     username: None,
        //     display_name,
        //     bio,
        //     avatar_url,
        //     gender,
        //     dob,
        //     provider_id: provider_name,
        //     is_verified: false,
        //     token,
        //     exp,
        // };
        let cookie = Cookie::build(("jwt_token", jwt_token.clone()))
            .path("/")
            .http_only(true)
            // If is_secure_context() is true, browser only sends over HTTPS.
            // If false (local dev), browser allows plain HTTP.
            .secure(utils::is_secure_context())
            .build();
        res.add_cookie(cookie);
        // json_ok(odata)
        let is_verified = if current_provider_name == "password" {
            token_claims.email_verified.unwrap_or(false)
        } else {
            true // Social/Phone providers are implicitly verified by Firebase
        };

        let odata = FirebaseLoginOutData {
            id: user_record.id,
            firebase_uid: token_claims.sub,
            session_id: session_row.id,
            username: user_record.username, // Tell the frontend if setup is needed
            display_name:user_record.display_name,
            bio: user_record.bio,
            avatar_url: user_record.avatar_url,
            gender: user_record.gender,
            dob: user_record.dob,
            // Convert current_provider_name (&str) to String
            current_auth_provider_slug: current_provider_name.to_string(),
            // Map Option<&str> to Option<String>
            current_provider_internal_uid: current_provider_internal_uid.map(|s| s.to_string()),
            is_verified,
            token:jwt_token,
            exp,
        };
        json_ok(odata)
    }




