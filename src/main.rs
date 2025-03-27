use crate::util::Config;
use axum::routing::get;
use axum::{Router, ServiceExt};
use r2d2::PooledConnection;
use sqlx::{Pool, Postgres};
use tower_http::cors::CorsLayer;
use void_log::log_info;

mod orange;
mod system;
mod util;
mod api;

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

    let mut app = Router::new().route("/", get(|| async { "Is run time!" }));
    app = system::router(app);
    app = orange::router(app);
    let app = app.with_state(app_state).layer(CorsLayer::permissive());

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
