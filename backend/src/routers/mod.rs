use rust_embed::RustEmbed;
use salvo::prelude::*;
use salvo::serve_static::{static_embed, EmbeddedFileExt};

mod auth;
mod demo;
mod user;

use crate::{config, hoops};
use crate::firebase::firebase_sw_handler;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Assets;

pub fn root() -> Router {
    let favicon = Assets::get("favicon.ico")
        .expect("favicon not found")
        .into_handler();
    let router = Router::new()
        .hoop(Logger::new())
        .get(demo::hello)
        .push(
            Router::with_path("auth").get(auth::auth_page)// New landing page for the email link
            .push(Router::with_path("verify-link").get(auth::verify_link_page))
        )
        // .push(Router::with_path("setup-profile").get(user::setup_profile_page))
        // /aut is already here
    // geo/postal-lookup?code=${this.form.postal_code
        // todo: /auth/verify-link?apiKey=AIzaSyAUtirDdNPTmQz0Ze4lZ_r6du48HdpJIxQ&oobCode=bzT1-JzH-neckfVOzFVeGT9J_At3Yz8c0EfqcNWG4kIAAAGcZdU-qw&mode=signIn&lang=en
        // so that email link for firebase can work properly
        // .push(Router::with_path("users").get(user::list_page))
        .push(

            Router::with_path("api")
                .push(Router::with_path("authenticate").post(auth::post_authenticate))
                // .push(
                //     Router::with_path("users")
                //         .hoop(hoops::auth_hoop(&config::get().jwt))
                //         .get(user::list_users)
                //         .post(user::create_user)
                //         .push(
                //             Router::with_path("{user_id}")
                //                 .put(user::update_user)
                //                 .delete(user::delete_user),
                //         ),
                // ),
        )
        .push(Router::with_path("favicon.ico").get(favicon))
        // Dynamic Service Worker
        .push(Router::with_path("firebase-messaging-sw.js").get(firebase_sw_handler))
        .push(Router::with_path("assets/{**rest}").get(static_embed::<Assets>()));
    let doc = OpenApi::new("salvo web api", "0.0.1").merge_router(&router);
    router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(Scalar::new("/api-doc/openapi.json").into_router("scalar"))
}
