use crate::util::Config;
use axum::Router;
use axum::routing::get;
use sqlx::{Pool, Postgres};
use void_log::log_info;

mod orange;
mod system;
mod util;

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}

pub async fn run() {
    let config = Config::get().await;
    let server = config.get_server();
    let database = config.get_database();
    let app_state = AppState {
        pool: database.get().await,
    };
    let address = format!("{}:{}", &server.get_path(), &server.get_port());
    log_info!("启动参数: {}", &address);

    let mut app = Router::new()
        .with_state(app_state)
        .route("/", get(|| async { "Is run time!" }));
    app = system::router(app);
    app = orange::router(app);

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
