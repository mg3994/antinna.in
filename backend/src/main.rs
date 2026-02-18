use salvo::catcher::Catcher;
use salvo::conn::rustls::{Keycert, RustlsConfig};
use salvo::prelude::*;
use salvo::server::ServerHandle;
use serde::Serialize;
use tokio::signal;
use tracing::info;
use crate::infrastructure::i18n::init_i18n;

// mod db;
mod hoops;
mod models;
mod routers;
mod utils;



mod firebase;



//
mod application;
mod core;
mod infrastructure;
mod interface;
//
use crate::core::errors::AppError;
use crate::infrastructure::config::LogConfig;


#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider().install_default().expect("Failed to install rustls crypto provider");
    crate::infrastructure::config::init();
    let config = crate::infrastructure::config::get();
    init_i18n(config.default_locale);
    crate::infrastructure::persistence::init(&config.db).await;

    // locales init


    // Firebase Admin
    if let Some(firebase_admin) = &config.firebase.admin {
        crate::firebase::init(firebase_admin).await;
    }
    // firebase::firebase_admin().auth()
    // firebase::firebase_admin().messaging()

    let _guard = crate::infrastructure::log::init(&config.log); // todo change to init
    tracing::info!("log level: {}", &config.log.filter_level);

    let service = Service::new(routers::root())
        .catcher(Catcher::default().hoop(hoops::error_404))
        .hoop(hoops::cors_hoop());
    println!("🔄 listen on {}", &config.listen_addr);
    println!("Debug: TLS config is {:?}", config.tls); // Add this
    //Acme support, automatically get TLS certificate from Let's Encrypt. For example, see https://github.com/salvo-rs/salvo/blob/main/examples/acme-http01-quinn/src/main.rs
    if let Some(tls) = &config.tls {
        let listen_addr = &config.listen_addr;
        println!(
            "📖 Open API Page (test Quinn): https://{}/scalar",
            listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Auth Page (test Quinn) : https://{}/auth",
            listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let config = RustlsConfig::new(
            Keycert::new().cert(
                std::fs::read(tls.cert.clone()).expect("cert file not found")
            ).key(
                std::fs::read(tls.key.clone()).expect("key file not found")
            ));
        let acceptor = QuinnListener::new(config.clone().build_quinn_config().unwrap(),listen_addr).join(TcpListener::new(listen_addr).rustls(config)).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    } else {
        println!(
            "📖 Open API Page: http://{}/scalar",
            config.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        println!(
            "🔑 Login Page: http://{}/login",
            config.listen_addr.replace("0.0.0.0", "127.0.0.1")
        );
        let acceptor =TcpListener::new(&config.listen_addr).bind().await;
        let server = Server::new(acceptor);
        tokio::spawn(shutdown_signal(server.handle()));
        server.serve(service).await;
    }
}

async fn shutdown_signal(handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("ctrl_c signal received"),
        _ = terminate => info!("terminate signal received"),
    }
    handle.stop_graceful(std::time::Duration::from_secs(60));
}

#[cfg(test)]
mod tests {
    use salvo::prelude::*;
    use salvo::test::{ResponseExt, TestClient};

    use crate::infrastructure::config;

    #[tokio::test]
    async fn test_hello_world() {
        config::init();

        let service = Service::new(crate::routers::root());

        let content = TestClient::get(format!(
            "http://{}",
            config::get().listen_addr.replace("0.0.0.0", "127.0.0.1")
        ))
        .send(&service)
        .await
        .take_string()
        .await
        .unwrap();
        assert_eq!(content, "Hello World from salvo");
    }
}
