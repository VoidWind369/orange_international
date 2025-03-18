use axum::Router;
use axum::routing::get;

pub fn router(app: Router) -> Router {
    app.route("/", get(|| async { "Is orange time!" }))
}