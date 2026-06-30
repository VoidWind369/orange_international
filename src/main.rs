use crate::util::Config;
use axum::{Router, routing::get};
use axum_server::tls_rustls::RustlsConfig;
use sqlx::{Pool, Postgres};
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};
use void_log::log_info;

mod api;
mod core;
mod mail;
mod middle;
mod orange;
mod safety;
mod system;
mod util;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

pub async fn run() {
    let config = Config::get().await;
    let server = config.get_server();

    // Database Link
    let database = config.get_database();
    let app_state = AppState {
        pool: database.get().await,
    };

    let address = format!("{}:{}", &server.get_path(), &server.get_port());
    let address = SocketAddr::from_str(&address).unwrap();
    log_info!("启动参数: {}", &address);

    let app = Router::new()
        .route("/", get(|| async { "Is run time!" }))
        .nest("/system", system::router())
        .nest("/orange", orange::router())
        .nest("/middle", middle::router())
        .nest("/safety", safety::router())
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let serve_dir = ServeDir::new("public").not_found_service(ServeFile::new("public/index.html"));

    let app = Router::new().nest("/api", app).fallback_service(serve_dir);

    if let Some(pem) = server.get_pem_path() {
        log_info!("TLS is on");
        let tls_config =
            RustlsConfig::from_pem_file(PathBuf::from(pem.cert), PathBuf::from(pem.key))
                .await
                .unwrap();
        axum_server::bind_rustls(address, tls_config)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    } else {
        log_info!("TLS is off");
        axum_server::bind(address)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    let mut banner = String::from("--------------");
    banner.push_str("\n   ██████╗ ██████╗  █████╗ ███╗   ██╗ ██████╗ ███████╗  ");
    banner.push_str("\n  ██╔═══██╗██╔══██╗██╔══██╗████╗  ██║██╔════╝ ██╔════╝  ");
    banner.push_str("\n  ██║   ██║██████╔╝███████║██╔██╗ ██║██║  ███╗█████╗    ");
    banner.push_str("\n  ██║   ██║██╔══██╗██╔══██║██║╚██╗██║██║   ██║██╔══╝    ");
    banner.push_str("\n  ╚██████╔╝██║  ██║██║  ██║██║ ╚████║╚██████╔╝███████╗  ");
    banner.push_str("\n   ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝  ");
    banner.push_str("\n--------------------------------------------------------");
    log_info!("{}", banner);

    run().await;
}
