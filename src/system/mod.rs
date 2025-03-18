use axum::Router;
use axum::routing::get;

mod user;

pub fn router(app: Router) -> Router {
    app.route("/", get(|| async { "Is system time!" }))
}