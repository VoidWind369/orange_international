use crate::util::Config;
use axum::routing::get;
use axum::{Router, ServiceExt};
use sqlx::{Pool, Postgres};
use tower_http::cors::CorsLayer;
use void_log::log_info;

mod api;
mod orange;
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
    log_info!("启动参数: {}", &address);

    let app = Router::new()
        .route("/", get(|| async { "Is run time!" }))
        .nest("/system", system::router())
        .nest("/orange", orange::router())
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
