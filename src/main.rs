use axum::Router;
use axum::routing::get;
use void_log::log_info;
use crate::util::Config;

mod system;
mod orange;
mod util;

pub async fn run() {
    let config = Config::get().await.get_server();
    let address = format!(
        "{}:{}",
        &config.get_path(),
        &config.get_port()
    );
    log_info!("启动参数: {}", &address);

    let mut app = Router::new().route("/", get(|| async { "Is run time!" }));
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
