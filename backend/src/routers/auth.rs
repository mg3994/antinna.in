use cookie::Cookie;
use askama::Template;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::hoops::jwt;
use crate::models::User;
use crate::{db, json_ok, utils, AppResult, JsonResult};

#[handler]
pub async fn login_page(req: &mut Request,res: &mut Response) -> AppResult<()> {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct LoginTemplate {}
    // 1. Extract token using your universal finders (Header, Query, Cookies)
    if let Some(jwt_token) = utils::extract_jwt_token_manually(req).await { // check even from headers adn query
        // 2. Validate the token (checks signature + expiration)
        if jwt::is_jwt_token_valid(&jwt_token) {
            res.render(Redirect::other("/users")); //
            return Ok(());
        }
    }
    let hello_tmpl = LoginTemplate {};
    res.render(Text::Html(hello_tmpl.render().unwrap()));
    Ok(())
}

#[derive(Deserialize, ToSchema, Default, Debug)]
pub struct LoginInData {
    pub username: String,
    pub password: String,
}
#[derive(Serialize, ToSchema, Default, Debug)]
pub struct LoginOutData {
    pub id: String,
    pub username: String,
    pub token: String,
    pub exp: i64,
}
#[endpoint(tags("auth"))]
pub async fn post_login(
    idata: JsonBody<LoginInData>,
    res: &mut Response,
) -> JsonResult<LoginOutData> {
    let idata = idata.into_inner();
    let conn = db::pool();
    let Some(User {
        id,
        username,
        password,
    }) = sqlx::query_as!(
        User,
        r#"
            SELECT id, username, password FROM users
            WHERE username = $1
            "#,
        idata.username
    )
    .fetch_optional(conn)
    .await?
    else {
        return Err(StatusError::unauthorized()
            .brief("User does not exist.")
            .into());
    };

    if utils::verify_password(&idata.password, &password).is_err()
    {
        return Err(StatusError::unauthorized()
            .brief("Account not exist or password is incorrect.")
            .into());
    }

    let (token, exp) = jwt::generate_jwt_token(&id)?;
    let odata = LoginOutData {
        id,
        username,
        token,
        exp,
    };
    let cookie = Cookie::build(("jwt_token", odata.token.clone()))
        .path("/")
        .http_only(true)
        // If is_secure_context() is true, browser only sends over HTTPS.
        // If false (local dev), browser allows plain HTTP.
        .secure(utils::is_secure_context())
        .build();
    res.add_cookie(cookie);
    json_ok(odata)
}



